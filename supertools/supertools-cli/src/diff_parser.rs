use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum LineType {
    Context,
    Addition,
    Deletion,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub line_type: LineType,
    pub content: String,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub status: FileStatus,
    pub hunks: Vec<Hunk>,
}

fn extract_path(line: &str, prefix: &str) -> Option<Option<String>> {
    let mut path = line[prefix.len()..].trim();
    if let Some(idx) = path.find('\t') {
        path = &path[..idx];
    }
    let path = if (path.starts_with("a/") || path.starts_with("b/")) && path.len() > 2 {
        &path[2..]
    } else {
        path
    };

    if path == "/dev/null" {
        Some(None)
    } else {
        Some(Some(path.to_string()))
    }
}

fn parse_hunk_header(line: &str) -> Option<(usize, usize, String)> {
    if line.starts_with("@@") {
        if let Some(end_idx) = line[2..].find("@@") {
            let header = line[..end_idx + 4].to_string();
            let ranges = line[2..end_idx + 2].trim();
            let parts: Vec<&str> = ranges.split_whitespace().collect();
            let mut old_start = 0;
            let mut new_start = 0;
            for part in parts {
                if part.starts_with('-') {
                    let num_part = &part[1..];
                    let val = num_part.split(',').next().unwrap_or(num_part);
                    old_start = val.parse::<usize>().unwrap_or(0);
                } else if part.starts_with('+') {
                    let num_part = &part[1..];
                    let val = num_part.split(',').next().unwrap_or(num_part);
                    new_start = val.parse::<usize>().unwrap_or(0);
                }
            }
            return Some((old_start, new_start, header));
        }
    }
    None
}

/// Parse a unified git diff string into structured FileDiffs
pub fn parse_diff(diff_str: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();

    let mut current_old_path: Option<String> = None;
    let mut current_new_path: Option<String> = None;
    let mut current_hunks: Vec<Hunk> = Vec::new();

    let mut current_hunk: Option<Hunk> = None;
    let mut old_curr = 0;
    let mut new_curr = 0;

    let mut in_file = false;

    let mut commit_file = |old: &mut Option<String>,
                           new: &mut Option<String>,
                           hunks: &mut Vec<Hunk>,
                           hunk: &mut Option<Hunk>| {
        if let Some(h) = hunk.take() {
            hunks.push(h);
        }
        let status = if old.is_none() {
            FileStatus::Added
        } else if new.is_none() {
            FileStatus::Deleted
        } else {
            FileStatus::Modified
        };
        files.push(FileDiff {
            old_path: old.take(),
            new_path: new.take(),
            status,
            hunks: std::mem::take(hunks),
        });
    };

    for line in diff_str.lines() {
        if line.starts_with("diff --git") {
            if in_file {
                commit_file(
                    &mut current_old_path,
                    &mut current_new_path,
                    &mut current_hunks,
                    &mut current_hunk,
                );
            }
            in_file = true;
            current_old_path = None;
            current_new_path = None;
            current_hunks = Vec::new();
            current_hunk = None;
            continue;
        }

        if line.starts_with("--- ") {
            if let Some(path) = extract_path(line, "--- ") {
                current_old_path = path;
            }
            continue;
        }
        if line.starts_with("+++ ") {
            if let Some(path) = extract_path(line, "+++ ") {
                current_new_path = path;
            }
            continue;
        }

        if line.starts_with("@@") {
            if let Some(h) = current_hunk.take() {
                current_hunks.push(h);
            }

            if let Some((old_start, new_start, header)) = parse_hunk_header(line) {
                old_curr = old_start;
                new_curr = new_start;
                current_hunk = Some(Hunk {
                    header,
                    lines: Vec::new(),
                });
            }
            continue;
        }

        if current_hunk.is_some() {
            if line.starts_with('\\') {
                continue;
            }

            if line.starts_with('-') {
                let hunk = current_hunk.as_mut().unwrap();
                hunk.lines.push(DiffLine {
                    line_type: LineType::Deletion,
                    content: line.to_string(),
                    old_line_num: Some(old_curr),
                    new_line_num: None,
                });
                old_curr += 1;
            } else if line.starts_with('+') {
                let hunk = current_hunk.as_mut().unwrap();
                hunk.lines.push(DiffLine {
                    line_type: LineType::Addition,
                    content: line.to_string(),
                    old_line_num: None,
                    new_line_num: Some(new_curr),
                });
                new_curr += 1;
            } else if line.starts_with(' ') || line.is_empty() {
                let hunk = current_hunk.as_mut().unwrap();
                hunk.lines.push(DiffLine {
                    line_type: LineType::Context,
                    content: line.to_string(),
                    old_line_num: Some(old_curr),
                    new_line_num: Some(new_curr),
                });
                old_curr += 1;
                new_curr += 1;
            } else {
                if let Some(h) = current_hunk.take() {
                    current_hunks.push(h);
                }
            }
        }
    }

    if in_file {
        commit_file(
            &mut current_old_path,
            &mut current_new_path,
            &mut current_hunks,
            &mut current_hunk,
        );
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modified_file() {
        let diff = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdef0 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,4 +1,4 @@
 fn main() {
-    println!("hello");
+    println!("hello world");
 }
"#;
        let files = parse_diff(diff);
        assert_eq!(files.len(), 1);
        let f = &files[0];
        assert_eq!(f.old_path.as_deref(), Some("src/main.rs"));
        assert_eq!(f.new_path.as_deref(), Some("src/main.rs"));
        assert_eq!(f.status, FileStatus::Modified);
        assert_eq!(f.hunks.len(), 1);
        let h = &f.hunks[0];
        assert_eq!(h.header, "@@ -1,4 +1,4 @@");
        assert_eq!(h.lines.len(), 4);
        assert_eq!(h.lines[0].line_type, LineType::Context);
        assert_eq!(h.lines[0].old_line_num, Some(1));
        assert_eq!(h.lines[0].new_line_num, Some(1));

        assert_eq!(h.lines[1].line_type, LineType::Deletion);
        assert_eq!(h.lines[1].old_line_num, Some(2));
        assert_eq!(h.lines[1].new_line_num, None);

        assert_eq!(h.lines[2].line_type, LineType::Addition);
        assert_eq!(h.lines[2].old_line_num, None);
        assert_eq!(h.lines[2].new_line_num, Some(2));

        assert_eq!(h.lines[3].line_type, LineType::Context);
        assert_eq!(h.lines[3].old_line_num, Some(3));
        assert_eq!(h.lines[3].new_line_num, Some(3));
    }

    #[test]
    fn test_parse_added_file() {
        let diff = r#"diff --git a/new_file.rs b/new_file.rs
new file mode 100644
index 0000000..1234567
--- /dev/null
+++ b/new_file.rs
@@ -0,0 +1,3 @@
+fn new_function() {
+    println!("I am new!");
+}
"#;
        let files = parse_diff(diff);
        assert_eq!(files.len(), 1);
        let f = &files[0];
        assert_eq!(f.old_path, None);
        assert_eq!(f.new_path.as_deref(), Some("new_file.rs"));
        assert_eq!(f.status, FileStatus::Added);
        assert_eq!(f.hunks.len(), 1);
        let h = &f.hunks[0];
        assert_eq!(h.header, "@@ -0,0 +1,3 @@");
        assert_eq!(h.lines.len(), 3);
        assert_eq!(h.lines[0].line_type, LineType::Addition);
        assert_eq!(h.lines[0].old_line_num, None);
        assert_eq!(h.lines[0].new_line_num, Some(1));
    }

    #[test]
    fn test_parse_deleted_file() {
        let diff = r#"diff --git a/old_file.rs b/old_file.rs
deleted file mode 100644
index 1234567..0000000
--- a/old_file.rs
+++ /dev/null
@@ -1,3 +0,0 @@
-fn old_function() {
-    println!("Goodbye!");
-}
"#;
        let files = parse_diff(diff);
        assert_eq!(files.len(), 1);
        let f = &files[0];
        assert_eq!(f.old_path.as_deref(), Some("old_file.rs"));
        assert_eq!(f.new_path, None);
        assert_eq!(f.status, FileStatus::Deleted);
        assert_eq!(f.hunks.len(), 1);
        let h = &f.hunks[0];
        assert_eq!(h.header, "@@ -1,3 +0,0 @@");
        assert_eq!(h.lines.len(), 3);
        assert_eq!(h.lines[0].line_type, LineType::Deletion);
        assert_eq!(h.lines[0].old_line_num, Some(1));
        assert_eq!(h.lines[0].new_line_num, None);
    }

    #[test]
    fn test_parse_multiple_files() {
        let diff = r#"diff --git a/file1.rs b/file1.rs
index 1111111..2222222 100644
--- a/file1.rs
+++ b/file1.rs
@@ -1,2 +1,2 @@
-hello
+world
diff --git a/file2.rs b/file2.rs
new file mode 100644
index 0000000..3333333
--- /dev/null
+++ b/file2.rs
@@ -0,0 +1 @@
+new file content
"#;
        let files = parse_diff(diff);
        assert_eq!(files.len(), 2);

        let f1 = &files[0];
        assert_eq!(f1.old_path.as_deref(), Some("file1.rs"));
        assert_eq!(f1.new_path.as_deref(), Some("file1.rs"));
        assert_eq!(f1.status, FileStatus::Modified);

        let f2 = &files[1];
        assert_eq!(f2.old_path, None);
        assert_eq!(f2.new_path.as_deref(), Some("file2.rs"));
        assert_eq!(f2.status, FileStatus::Added);
    }
}
