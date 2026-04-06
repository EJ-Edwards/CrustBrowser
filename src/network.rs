// --- network.rs ---
// This module handles all the network stuff — fetching web pages, making requests, etc.

// Import reqwest's blocking Client for making HTTP requests
use reqwest::blocking::Client;

// Makes sure a URL starts with http:// or https://
// If someone types "example.com", this turns it into "https://example.com"
pub fn normalize_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{}", trimmed)
    }
}

// Fetches a web page and returns the raw HTML as a string
// This is the main function used by "go <url>" in the browser
// Returns Ok(html) on success, or an error message if the request fails
pub fn get(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .user_agent("CrustBrowser/0.1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client.get(url).send()
        .map_err(|e| format!("Could not reach '{}': {}", url, e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Server returned error {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")));
    }

    response.text()
        .map_err(|e| format!("Failed to read page content: {}", e))
}

