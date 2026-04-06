// This module provides functions to build search URLs for various search engines
use url::Url;


// This module handles building search URLs for different search engines
const GOOGLE: &str = "https://www.google.com/search";
const BING: &str = "https://www.bing.com/search";
const DUCKDUCKGO: &str = "https://www.duckduckgo.com/";
const YAHOO: &str = "https://www.yahoo.com/search";

// Builds a search URL by adding the query as a "q" parameter to the base URL
pub fn build_search_url(base_url: &str, query: &str) -> Result<String, url::ParseError> {
    let mut url = Url::parse(base_url)?;
    url.query_pairs_mut().append_pair("q", query);
    Ok(url.to_string())
}

// Convenience function to get a search URL for Google
pub fn search(query: &str) -> String {
    match build_search_url(GOOGLE, query) {
        Ok(search_url) => search_url,
        Err(e) => format!("Error building search URL: {}", e),
    }
}

// Shows search URLs for all supported engines
pub fn search_all_engines(query: &str) {
    let engines = [
        ("Google", GOOGLE),
        ("Bing", BING),
        ("DuckDuckGo", DUCKDUCKGO),
        ("Yahoo", YAHOO),
    ];

    // Build and print the search URL for each engine
    for (name, base_url) in &engines {
        match build_search_url(base_url, query) {
            Ok(search_url) => println!("  {} → {}", name, search_url),
            Err(e) => println!("  {} → Error: {}", name, e),
        }
    }
}