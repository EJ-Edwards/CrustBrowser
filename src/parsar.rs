// scraper is the library we use to read and search through HTML
use scraper::{Html, Selector};
use url::Url;

// This struct holds all the useful stuff we pull out of a web page
pub struct ParsedPage {
    pub title: String,                  // The page title (what shows in the browser tab)
    pub headings: Vec<String>,          // All the headings (h1, h2, h3) on the page
    pub links: Vec<(String, String)>,   // All links as (display text, URL)
    pub text: Vec<String>,              // The main readable text content
}

// Takes raw HTML (a big string of code) and pulls out the important parts
pub fn parse_html(html: &str) -> ParsedPage {
    // Turn the raw HTML string into something we can search through
    let document = Html::parse_document(html);

    // --- Get the page title ---
    // Look for the <title> tag (e.g. <title>Google</title>)
    let title = {
        let sel = Selector::parse("title").unwrap();
        document.select(&sel).next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "(no title)".to_string()) // fallback if no title found
    };

    // --- Get all headings ---
    // Look for <h1>, <h2>, <h3> tags (the big bold text on pages)
    let headings = {
        let sel = Selector::parse("h1, h2, h3").unwrap();
        document.select(&sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|t| !t.is_empty()) // skip empty ones
            .collect()
    };

    // --- Get all links ---
    // Look for <a href="..."> tags (clickable links)
    // We grab both the visible text and the URL it points to
    // Also resolves redirect URLs (e.g. Google's /url?q=...) to the real destination
    let links = {
        let sel = Selector::parse("a[href]").unwrap();
        document.select(&sel)
            .map(|el| {
                let text = el.text().collect::<String>().trim().to_string();
                let href = el.value().attr("href").unwrap_or("").to_string();
                let resolved = resolve_redirect(&href);
                (text, resolved)
            })
            .filter(|(_, href)| !href.is_empty()) // skip links with no URL
            .collect()
    };

    // --- Get the readable text ---
    // Look for paragraphs, list items, table cells, code blocks, and quotes
    let text = {
        let sel = Selector::parse("p, li, td, th, pre, blockquote").unwrap();
        document.select(&sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|t| !t.is_empty()) // skip empty ones
            .collect()
    };

    // Bundle everything up and return it
    ParsedPage { title, headings, links, text }
}

// Extracts the real URL from search engine redirect links
// e.g. "/url?q=https://example.com&sa=..." → "https://example.com"
// Also handles DuckDuckGo's "//duckduckgo.com/l/?uddg=https%3A..." redirects
fn resolve_redirect(href: &str) -> String {
    if let Ok(parsed) = Url::parse(&format!("https://placeholder{}", href)) {
        for (key, val) in parsed.query_pairs() {
            if (key == "q" || key == "uddg") && (val.starts_with("http://") || val.starts_with("https://")) {
                return val.to_string();
            }
        }
    }
    href.to_string()
}