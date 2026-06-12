use std::env;

/// Opens the given URL in the user's browser.
/// Respects SUPERTOOLS_REMOTE (prints URL and returns Ok(()) without launching a browser)
/// and SUPERTOOLS_BROWSER (launches the specified browser).
pub fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let remote = env::var("SUPERTOOLS_REMOTE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if remote {
        println!("Please open this URL in your browser: {}", url);
        return Ok(());
    }

    if let Ok(browser_name) = env::var("SUPERTOOLS_BROWSER") {
        open::with(url, browser_name)?;
    } else {
        open::that(url)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_mode() {
        unsafe {
            std::env::set_var("SUPERTOOLS_REMOTE", "true");
        }
        let res = open_browser("http://127.0.0.1:8080");
        unsafe {
            std::env::remove_var("SUPERTOOLS_REMOTE");
        }
        assert!(
            res.is_ok(),
            "open_browser should succeed in remote mode by printing the URL"
        );
    }

    #[test]
    fn test_remote_mode_alternative() {
        unsafe {
            std::env::set_var("SUPERTOOLS_REMOTE", "1");
        }
        let res = open_browser("http://127.0.0.1:8080");
        unsafe {
            std::env::remove_var("SUPERTOOLS_REMOTE");
        }
        assert!(
            res.is_ok(),
            "open_browser should succeed in remote mode when variable is '1'"
        );
    }
}
