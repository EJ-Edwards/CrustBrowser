// --- network.rs ---
// This module handles all the network stuff — fetching web pages, making requests, etc.

// Import reqwest's blocking Client for making HTTP requests
use reqwest::blocking::Client;

// Fetches a web page and returns the raw HTML as a string
// This is the main function used by "go <url>" in the browser
// Returns Ok(html) on success, or an error if the request fails
pub fn get(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::blocking::get(url)?; // Send the GET request
    let body = response.text()?;                 // Read the response body as text
    Ok(body)
}

// Sends a POST request with a body (for forms, APIs, etc.)
// Not used yet in the browser, but available for future features
pub fn post(url: &str, body: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let response = client.post(url).body(body.to_string()).send()?;
    let response_body = response.text()?;
    Ok(response_body)
}