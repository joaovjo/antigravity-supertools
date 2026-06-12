use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(name = "supertools", about = "Unified engineering toolkit")]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Commands {
    /// Interactive code review for git diff or a PR URL
    Review {
        #[arg(index = 1)]
        pr_url: Option<String>,
    },
    /// Interactive annotation session for a file or URL
    Annotate {
        #[arg(index = 1)]
        path_or_url: String,
    },
    /// Pre-plan hook to improve context before plan generation
    ImproveContext,
    /// Run retrospective analysis on denied plans
    Compound {
        #[arg(index = 1)]
        plans_dir: Option<String>,
    },
}

/// Parse arguments from an iterator of strings
pub fn parse_from<I, T>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: IntoValue,
{
    Cli::try_parse_from(args.into_iter().map(|a| a.into_value()))
}

pub trait IntoValue {
    fn into_value(self) -> std::ffi::OsString;
}

impl IntoValue for &str {
    fn into_value(self) -> std::ffi::OsString {
        std::ffi::OsString::from(self)
    }
}

impl IntoValue for String {
    fn into_value(self) -> std::ffi::OsString {
        std::ffi::OsString::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let args = vec!["supertools"];
        let cli = parse_from(args).unwrap();
        assert_eq!(cli.subcommand, None);
    }

    #[test]
    fn test_parse_review_no_url() {
        let args = vec!["supertools", "review"];
        let cli = parse_from(args).unwrap();
        assert_eq!(cli.subcommand, Some(Commands::Review { pr_url: None }));
    }

    #[test]
    fn test_parse_review_with_url() {
        let args = vec![
            "supertools",
            "review",
            "https://github.com/test/repo/pull/1",
        ];
        let cli = parse_from(args).unwrap();
        assert_eq!(
            cli.subcommand,
            Some(Commands::Review {
                pr_url: Some("https://github.com/test/repo/pull/1".to_string())
            })
        );
    }

    #[test]
    fn test_parse_annotate() {
        let args = vec!["supertools", "annotate", "file.md"];
        let cli = parse_from(args).unwrap();
        assert_eq!(
            cli.subcommand,
            Some(Commands::Annotate {
                path_or_url: "file.md".to_string()
            })
        );
    }

    #[test]
    fn test_parse_improve_context() {
        let args = vec!["supertools", "improve-context"];
        let cli = parse_from(args).unwrap();
        assert_eq!(cli.subcommand, Some(Commands::ImproveContext));
    }

    #[test]
    fn test_parse_compound() {
        let args = vec!["supertools", "compound"];
        let cli = parse_from(args).unwrap();
        assert_eq!(cli.subcommand, Some(Commands::Compound { plans_dir: None }));
    }
}
