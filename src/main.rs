// --- Module declarations ---
// Each of these links to a separate .rs file in the src/ folder
mod network;  // Handles fetching web pages (HTTP requests)
mod parsar;   // Parses raw HTML into usable data (title, links, text)
mod search;   // Builds search engine URLs (Google, Bing, etc.)
mod utilts;   // Commands, welcome message, and help menu
mod render;   // Handles displaying pages nicely in the terminal

// Standard library imports
use std::io::{self, Write};
use std::fs;
use std::path::PathBuf;

// External crates
use colored::Colorize;
use serde::{Serialize, Deserialize};

// Pull in the stuff we need from utilts.rs
use utilts::{welcome_message, parse_command, show_help, Command};
// Pull in ParsedPage so we can store the last viewed page
use parsar::ParsedPage;

// --- Persistent data ---
// This struct gets saved to a JSON file so bookmarks survive between sessions
#[derive(Serialize, Deserialize, Default)]
struct BrowserData {
    bookmarks: Vec<(String, String)>, // (name, url)
}

// Figures out where to save the config file
// On Windows: C:\Users\<name>\AppData\Roaming\crust-browser\data.json
// On Mac: ~/Library/Application Support/crust-browser/data.json
// On Linux: ~/.config/crust-browser/data.json
fn data_file_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("crust-browser");
    fs::create_dir_all(&dir).ok();
    dir.join("data.json")
}

// Load saved bookmarks from the config file
fn load_data() -> BrowserData {
    let path = data_file_path();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(data) = serde_json::from_str(&contents) {
                return data;
            }
        }
    }
    BrowserData::default()
}

// Save bookmarks to the config file
fn save_data(data: &BrowserData) {
    let path = data_file_path();
    if let Ok(json) = serde_json::to_string_pretty(data) {
        fs::write(&path, json).ok();
    }
}

// Helper: fetches a URL, parses the HTML, renders it, and returns the parsed page + raw HTML
fn navigate(url: &str) -> Option<(ParsedPage, String)> {
    match network::get(url) {
        Ok(html) => {
            let page = parsar::parse_html(&html);
            render::render_page(&page);
            Some((page, html))
        }
        Err(e) => {
            println!("  {} {}", "Error:".red().bold(), e);
            None
        }
    }
}

fn main() {
    // Show the welcome message when the browser starts
    welcome_message();

    // --- Browser state ---
    let mut history: Vec<String> = Vec::new();      // All URLs visited (in order)
    let mut history_index: isize = -1;               // Where we are in history (-1 = nowhere)
    let mut last_page: Option<ParsedPage> = None;    // The last page we viewed (for links/refresh)
    let mut last_html: Option<String> = None;        // Raw HTML of the last page (for source/save)
    let mut current_url: Option<String> = None;      // The URL we're currently on
    let mut data = load_data();                       // Load saved bookmarks from disk

    // Main loop — keeps running until the user types "quit"
    loop {
        // Print the prompt — shows current URL if we're on a page
        match &current_url {
            Some(url) => print!("{} {} ", url.dimmed(), ">".cyan().bold()),
            None => print!("{} ", "crust>".cyan().bold()),
        }
        io::stdout().flush().unwrap();

        // Read whatever the user types
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("  {} Failed to read input.", "Error:".red().bold());
            continue;
        }

        // Skip empty input
        if input.trim().is_empty() {
            continue;
        }

        // Parse the input into a Command and handle it
        match parse_command(&input) {
            // User typed "go <url>" — fetch and display the web page
            Command::Go(raw_url) => {
                let url = network::normalize_url(&raw_url);
                println!("  {} {}", "Navigating to:".cyan(), url);
                if let Some((page, html)) = navigate(&url) {
                    // Add to history and update state
                    history.truncate((history_index + 1) as usize);
                    history.push(url.clone());
                    history_index = (history.len() as isize) - 1;
                    current_url = Some(url);
                    last_page = Some(page);
                    last_html = Some(html);
                }
            }

            // User typed "search <query>" — show search URLs and auto-navigate to Google
            Command::Search(query) => {
                println!("  {} {}", "Searching for:".cyan(), query);
                if let Some(search_url) = search::search_all_engines(&query) {
                    println!("  {}", "Auto-navigating to DuckDuckGo results...".dimmed());
                    if let Some((page, html)) = navigate(&search_url) {
                        history.truncate((history_index + 1) as usize);
                        history.push(search_url.clone());
                        history_index = (history.len() as isize) - 1;
                        current_url = Some(search_url);
                        last_page = Some(page);
                        last_html = Some(html);
                    }
                }
            }

            // User typed "click <number>" — navigate to a numbered link from the current page
            Command::Click(num) => {
                if let Some(ref page) = last_page {
                    if num == 0 || num > page.links.len() {
                        println!("  {} Link #{} doesn't exist. Page has {} links.",
                            "Error:".red().bold(), num, page.links.len());
                    } else {
                        let (_, ref href) = page.links[num - 1];
                        // Handle relative URLs by combining with current URL
                        let full_url = if href.starts_with("http://") || href.starts_with("https://") {
                            href.clone()
                        } else if let Some(ref base) = current_url {
                            if href.starts_with('/') {
                                // Absolute path — combine with domain
                                if let Ok(base_url) = url::Url::parse(base) {
                                    format!("{}://{}{}", base_url.scheme(), base_url.host_str().unwrap_or(""), href)
                                } else {
                                    href.clone()
                                }
                            } else {
                                format!("{}/{}", base.trim_end_matches('/'), href)
                            }
                        } else {
                            network::normalize_url(href)
                        };
                        println!("  {} {}", "Navigating to:".cyan(), full_url);
                        if let Some((page, html)) = navigate(&full_url) {
                            history.truncate((history_index + 1) as usize);
                            history.push(full_url.clone());
                            history_index = (history.len() as isize) - 1;
                            current_url = Some(full_url);
                            last_page = Some(page);
                            last_html = Some(html);
                        }
                    }
                } else {
                    println!("  {} Navigate to a page first.", "No page loaded.".yellow());
                }
            }

            // Go back to the previous page in history
            Command::Back => {
                if history_index > 0 {
                    history_index -= 1;
                    let url = history[history_index as usize].clone();
                    println!("  {} {}", "Going back to:".cyan(), url);
                    if let Some((page, html)) = navigate(&url) {
                        current_url = Some(url);
                        last_page = Some(page);
                        last_html = Some(html);
                    }
                } else {
                    println!("  {}", "Nothing to go back to.".yellow());
                }
            }

            // Go forward in history (if you went back before)
            Command::Forward => {
                if (history_index + 1) < history.len() as isize {
                    history_index += 1;
                    let url = history[history_index as usize].clone();
                    println!("  {} {}", "Going forward to:".cyan(), url);
                    if let Some((page, html)) = navigate(&url) {
                        current_url = Some(url);
                        last_page = Some(page);
                        last_html = Some(html);
                    }
                } else {
                    println!("  {}", "Nothing to go forward to.".yellow());
                }
            }

            // Reload the current page
            Command::Refresh => {
                if let Some(ref url) = current_url {
                    let url = url.clone();
                    println!("  {} {}", "Refreshing:".cyan(), url);
                    if let Some((page, html)) = navigate(&url) {
                        last_page = Some(page);
                        last_html = Some(html);
                    }
                } else {
                    println!("  {}", "No page to refresh.".yellow());
                }
            }

            // Show all links on the current page
            Command::Links => {
                if let Some(ref page) = last_page {
                    if page.links.is_empty() {
                        println!("  {}", "No links on this page.".yellow());
                    } else {
                        println!("\n  {}", "Links on this page:".yellow().bold());
                        println!("  {}", "───────────────────────────────────────".dimmed());
                        for (i, (text, url)) in page.links.iter().enumerate() {
                            let display = if text.is_empty() { url } else { text };
                            println!("    {} {}  {}", format!("[{}]", i + 1).green().bold(), display, url.dimmed());
                        }
                        println!();
                    }
                } else {
                    println!("  {}", "Navigate to a page first.".yellow());
                }
            }

            // Show only headings on the current page
            Command::Headings => {
                if let Some(ref page) = last_page {
                    if page.headings.is_empty() {
                        println!("  {}", "No headings on this page.".yellow());
                    } else {
                        println!("\n  {}", "Headings:".yellow().bold());
                        println!("  {}", "───────────────────────────────────────".dimmed());
                        for heading in &page.headings {
                            println!("    {} {}", "▸".cyan(), heading);
                        }
                        println!();
                    }
                } else {
                    println!("  {}", "Navigate to a page first.".yellow());
                }
            }

            // Show only the text content of the page
            Command::Text => {
                if let Some(ref page) = last_page {
                    if page.text.is_empty() {
                        println!("  {}", "No text content on this page.".yellow());
                    } else {
                        println!("\n  {}", "Page Text:".yellow().bold());
                        println!("  {}", "───────────────────────────────────────".dimmed());
                        for paragraph in &page.text {
                            println!("    {}", paragraph);
                        }
                        println!();
                    }
                } else {
                    println!("  {}", "Navigate to a page first.".yellow());
                }
            }

            // Show the browsing history
            Command::History => {
                if history.is_empty() {
                    println!("  {}", "No history yet.".yellow());
                } else {
                    println!("\n  {}", "Browsing History:".yellow().bold());
                    println!("  {}", "───────────────────────────────────────".dimmed());
                    for (i, url) in history.iter().enumerate() {
                        let marker = if i as isize == history_index { "→".green().bold() } else { " ".normal() };
                        println!("    {} {} {}", marker, format!("[{}]", i + 1).dimmed(), url);
                    }
                    println!();
                }
            }

            // Show all saved bookmarks
            Command::Bookmarks => {
                if data.bookmarks.is_empty() {
                    println!("  {}", "No bookmarks saved yet.".yellow());
                } else {
                    println!("\n  {}", "Bookmarks:".yellow().bold());
                    println!("  {}", "───────────────────────────────────────".dimmed());
                    for (i, (name, url)) in data.bookmarks.iter().enumerate() {
                        println!("    {} {}  {}", format!("[{}]", i + 1).green().bold(), name.white().bold(), url.dimmed());
                    }
                    println!("  {}", "Use 'go <url>' to visit a bookmark.".dimmed());
                    println!();
                }
            }

            // Save the current page as a bookmark
            Command::Bookmark(name) => {
                if let Some(ref url) = current_url {
                    data.bookmarks.push((name.clone(), url.clone()));
                    save_data(&data);
                    println!("  {} Saved '{}' → {}", "Bookmarked!".green().bold(), name, url);
                } else {
                    println!("  {}", "Navigate to a page first before bookmarking.".yellow());
                }
            }

            // Delete a bookmark by number
            Command::DelBookmark(num) => {
                if num == 0 || num > data.bookmarks.len() {
                    println!("  {} Bookmark #{} doesn't exist. You have {} bookmarks.",
                        "Error:".red().bold(), num, data.bookmarks.len());
                } else {
                    let (name, url) = data.bookmarks.remove(num - 1);
                    save_data(&data);
                    println!("  {} Removed '{}' ({})", "Deleted!".red().bold(), name, url);
                }
            }

            // View raw HTML source of the current page
            Command::Source => {
                if let Some(ref html) = last_html {
                    println!("\n  {}", "Page Source:".yellow().bold());
                    println!("  {}", "───────────────────────────────────────".dimmed());
                    // Show first 200 lines to avoid flooding the terminal
                    for (i, line) in html.lines().take(200).enumerate() {
                        println!("  {} {}", format!("{:>4}", i + 1).dimmed(), line);
                    }
                    let total = html.lines().count();
                    if total > 200 {
                        println!("  {}", format!("... ({} more lines, use 'save' to export full source)", total - 200).dimmed());
                    }
                    println!();
                } else {
                    println!("  {}", "Navigate to a page first.".yellow());
                }
            }

            // Save page content to a file
            Command::Save(filename) => {
                if let Some(ref page) = last_page {
                    let mut content = String::new();
                    content.push_str(&format!("Title: {}\n\n", page.title));
                    if !page.headings.is_empty() {
                        content.push_str("Headings:\n");
                        for h in &page.headings {
                            content.push_str(&format!("  - {}\n", h));
                        }
                        content.push('\n');
                    }
                    if !page.links.is_empty() {
                        content.push_str("Links:\n");
                        for (text, url) in &page.links {
                            let display = if text.is_empty() { url } else { text };
                            content.push_str(&format!("  {} → {}\n", display, url));
                        }
                        content.push('\n');
                    }
                    if !page.text.is_empty() {
                        content.push_str("Content:\n");
                        for p in &page.text {
                            content.push_str(&format!("  {}\n", p));
                        }
                    }
                    match fs::write(&filename, &content) {
                        Ok(_) => println!("  {} Saved to '{}'", "Done!".green().bold(), filename),
                        Err(e) => println!("  {} {}", "Error:".red().bold(), e),
                    }
                } else {
                    println!("  {}", "Navigate to a page first.".yellow());
                }
            }

            // Show the current URL
            Command::Url => {
                if let Some(ref url) = current_url {
                    println!("  {} {}", "Current URL:".cyan(), url);
                } else {
                    println!("  {}", "No page loaded.".yellow());
                }
            }

            // Navigate to homepage
            Command::Home => {
                let home_url = "https://html.duckduckgo.com/";
                println!("  {} {}", "Going home:".cyan(), home_url);
                if let Some((page, html)) = navigate(home_url) {
                    history.truncate((history_index + 1) as usize);
                    history.push(home_url.to_string());
                    history_index = (history.len() as isize) - 1;
                    current_url = Some(home_url.to_string());
                    last_page = Some(page);
                    last_html = Some(html);
                }
            }

            // Clear the terminal screen
            Command::Clear => {
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush().ok();
            }

            // Show the help menu
            Command::Help => show_help(),

            // Exit the browser
            Command::Quit => {
                save_data(&data); // Save bookmarks before quitting
                println!("  {}", "Goodbye! Thanks for using Crust Browser.".cyan());
                break;
            }

            // Anything we don't recognize
            Command::Unknown(msg) => {
                println!("  {} {}. Type {} for commands.", "Unknown:".red().bold(), msg, "help".yellow());
            }
        }
    }
}
