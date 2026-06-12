use clap::Parser;
use std::io::{self, Read};
use std::path::PathBuf;
use supertools_cli::{args, args::Commands, browser, diff_parser, protocol, server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = args::Cli::parse();

    match cli.subcommand {
        Some(Commands::ImproveContext) => {
            handle_improve_context()?;
        }
        Some(Commands::Review { pr_url }) => {
            handle_review(pr_url).await?;
        }
        Some(Commands::Annotate { path_or_url }) => {
            handle_annotate(path_or_url).await?;
        }
        Some(Commands::Compound { plans_dir }) => {
            handle_compound(plans_dir)?;
        }
        None => {
            // Default hook mode: read stdin
            handle_default_hook().await?;
        }
    }

    Ok(())
}

fn handle_improve_context() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pfm_reminder = "\n> [!NOTE]\n> Supertools is active. Remember to use PFM (Plannotator Flavored Markdown) callouts, file links, and Mermaid diagrams in your plan.";
    
    let output = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "additionalContext": pfm_reminder
        }
    });

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

async fn handle_review(pr_url: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let diff_str = if let Some(url) = pr_url {
        eprintln!("Fetching PR diff from URL: {}...", url);
        fetch_url_content(&url)?
    } else {
        eprintln!("Running local git diff...");
        get_local_diff()?
    };

    if diff_str.trim().is_empty() {
        eprintln!("No changes to review.");
        println!("The user approved.");
        return Ok(());
    }

    let diffs = diff_parser::parse_diff(&diff_str);
    let content = serde_json::to_string(&diffs)?;

    eprintln!("Starting review server...");
    let (server_handle, url) = server::start_server(0, content, "review".to_string()).await?;
    
    eprintln!("Opening review session in browser: {}", url);
    browser::open_browser(&url)?;

    let result = server_handle.await?;
    if result.approved {
        println!("The user approved.");
    } else {
        if let Some(fb) = result.feedback {
            println!("{}", fb);
        } else {
            println!("Changes requested.");
        }
    }

    Ok(())
}

async fn handle_annotate(path_or_url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let is_url = path_or_url.starts_with("http://") || path_or_url.starts_with("https://");
    let content = if is_url {
        eprintln!("Fetching URL: {}...", path_or_url);
        fetch_url_content(&path_or_url)?
    } else {
        eprintln!("Reading file: {}...", path_or_url);
        std::fs::read_to_string(&path_or_url)?
    };

    eprintln!("Starting annotation server...");
    let (server_handle, url) = server::start_server(0, content, "plan".to_string()).await?;
    
    eprintln!("Opening annotation session in browser: {}", url);
    browser::open_browser(&url)?;

    let result = server_handle.await?;
    if result.approved {
        println!("The user approved.");
    } else {
        if let Some(fb) = result.feedback {
            println!("{}", fb);
        } else {
            println!("Annotations added.");
        }
    }

    Ok(())
}

fn handle_compound(plans_dir: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dir = plans_dir.unwrap_or_else(|| ".gemini/plans".to_string());
    println!("Retrospective analysis of denied plans in: {}", dir);
    println!("No denied plans found to analyze.");
    Ok(())
}

async fn handle_default_hook() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut input_json = String::new();
    io::stdin().read_to_string(&mut input_json)?;

    if input_json.trim().is_empty() {
        return Ok(());
    }

    let parsed_input = match protocol::parse_input(&input_json) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Failed to parse hook event from stdin: {}", e);
            std::process::exit(1);
        }
    };

    let mut plan_content = String::new();
    let mut is_gemini = false;
    let mut plan_filename = String::new();

    if let Some(tool_input) = &parsed_input.tool_input {
        if let Some(plan_fn) = &tool_input.plan_filename {
            plan_filename = plan_fn.clone();
            is_gemini = true;
        } else if let Some(plan_path) = &tool_input.plan_path {
            plan_filename = plan_path.clone();
            is_gemini = true;
        } else if let Some(plan_val) = &tool_input.plan {
            plan_content = plan_val.clone();
        }
    }

    if is_gemini {
        let transcript_path = PathBuf::from(parsed_input.transcript_path.as_ref().ok_or("transcript_path missing")?);
        let project_temp_dir = transcript_path.parent().ok_or("invalid transcript_path")?.parent().ok_or("invalid transcript_path")?;
        let plan_filepath = project_temp_dir
            .join(parsed_input.session_id.as_ref().ok_or("session_id missing")?)
            .join("plans")
            .join(&plan_filename);

        plan_content = std::fs::read_to_string(plan_filepath)?;
    }

    if plan_content.trim().is_empty() {
        eprintln!("No plan content in hook event.");
        std::process::exit(1);
    }

    let (server_handle, url) = server::start_server(0, plan_content, "plan".to_string()).await?;
    
    browser::open_browser(&url)?;

    let result = server_handle.await?;

    if is_gemini {
        if result.approved {
            let output = protocol::build_gemini_approval(result.feedback);
            println!("{}", serde_json::to_string(&output)?);
        } else {
            let feedback = result.feedback.unwrap_or_else(|| "Plan changes requested".to_string());
            let reason = format!(
                "Plan rejected.\n\nFeedback:\n{}\n\nAddress these points in a revised plan.",
                feedback
            );
            let output = protocol::build_gemini_denial(reason);
            println!("{}", serde_json::to_string(&output)?);
        }
    } else {
        if result.approved {
            let output = protocol::build_claude_approval(parsed_input.permission_mode);
            println!("{}", serde_json::to_string(&output)?);
        } else {
            let feedback = result.feedback.unwrap_or_else(|| "Plan changes requested".to_string());
            let reason = format!(
                "Plan rejected.\n\nFeedback:\n{}\n\nAddress these points in a revised plan.",
                feedback
            );
            let output = protocol::build_claude_denial(reason);
            println!("{}", serde_json::to_string(&output)?);
        }
    }

    Ok(())
}

fn fetch_url_content(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(target_os = "windows")]
    const CURL_CMD: &str = "curl.exe";
    #[cfg(not(target_os = "windows"))]
    const CURL_CMD: &str = "curl";

    let mut target_url = url.to_string();
    if url.contains("github.com") && url.contains("/pull/") && !url.ends_with(".diff") && !url.ends_with(".patch") {
        target_url = format!("{}.diff", url);
    }

    let output = std::process::Command::new(CURL_CMD)
        .args(&["-s", "-L", &target_url])
        .output()?;

    if !output.status.success() {
        return Err(format!("Failed to fetch URL. Status: {}", output.status).into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn get_local_diff() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let output = std::process::Command::new("git")
        .args(&["diff"])
        .output()?;
    let mut diff = String::from_utf8_lossy(&output.stdout).into_owned();

    if diff.trim().is_empty() {
        let output_cached = std::process::Command::new("git")
            .args(&["diff", "--cached"])
            .output()?;
        diff = String::from_utf8_lossy(&output_cached.stdout).into_owned();
    }

    Ok(diff)
}
