// --- utilts.rs ---
// This file handles commands, user input parsing, and the help menu.
// It's the brain behind understanding what the user types.

use colored::Colorize;

// Prints the welcome message when the browser first starts
pub fn welcome_message() {
    println!("{}", "\n  ╔═══════════════════════════════════════╗".cyan());
    println!("{}", "  ║         🦀  Crust Browser  🦀         ║".cyan().bold());
    println!("{}", "  ║     A text-based web browser in Rust   ║".cyan());
    println!("{}", "  ╚═══════════════════════════════════════╝".cyan());
    println!("  Type {} for a list of commands.\n", "help".yellow().bold());
}

// All the commands the browser understands
// Each variant represents a different action the user can take
pub enum Command {
    Go(String),          // go <url> — navigate to a URL
    Search(String),      // search <query> — web search
    Click(usize),        // click <number> — click a numbered link on the page
    Back,                // back — go to previous page
    Forward,             // forward — go to next page
    Refresh,             // refresh — reload current page
    Links,               // links — list all links on the page
    History,             // history — show browsing history
    Bookmarks,           // bookmarks — show saved bookmarks
    Bookmark(String),    // bookmark <name> — save current page
    Help,                // help — show available commands
    Quit,                // quit/exit — close the browser
    Unknown(String),     // anything unrecognized
}

// Takes whatever the user typed and figures out which Command it is
// For example: "go https://example.com" becomes Command::Go("https://example.com")
pub fn parse_command(input: &str) -> Command {
    let trimmed = input.trim();
    let mut parts = trimmed.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("").to_lowercase();
    let arg = parts.next().unwrap_or("").trim().to_string();

    match cmd.as_str() {
        // Navigation — "go", "open", "navigate", or just "g"
        "go" | "open" | "navigate" | "g" => {
            if arg.is_empty() {
                Command::Unknown("Usage: go <url>".to_string())
            } else {
                Command::Go(arg)
            }
        }
        // Search — "search", "find", or just "s"
        "search" | "find" | "s" => {
            if arg.is_empty() {
                Command::Unknown("Usage: search <query>".to_string())
            } else {
                Command::Search(arg)
            }
        }
        // Click a numbered link from the page
        "click" | "c" => {
            if let Ok(num) = arg.parse::<usize>() {
                Command::Click(num)
            } else {
                Command::Unknown("Usage: click <number>".to_string())
            }
        }
        "back" | "b"       => Command::Back,
        "forward" | "f"    => Command::Forward,
        "refresh" | "r"    => Command::Refresh,
        "links" | "l"      => Command::Links,
        "history" | "h"    => Command::History,
        "bookmarks" | "bm" => Command::Bookmarks,
        "bookmark" => {
            if arg.is_empty() {
                Command::Bookmark("Untitled".to_string())
            } else {
                Command::Bookmark(arg)
            }
        }
        "help" | "?"       => Command::Help,
        "quit" | "exit" | "q" => Command::Quit,
        _ => Command::Unknown(trimmed.to_string()),
    }
}

// Prints the full list of commands the user can type
pub fn show_help() {
    println!("\n  {}", "Available Commands:".white().bold());
    println!("  {}", "═══════════════════════════════════════".dimmed());
    println!("  {}  {}    {}", "go".yellow().bold(), "<url>".dimmed(), "Navigate to a URL");
    println!("  {}  {}  {}", "search".yellow().bold(), "<query>".dimmed(), "Search the web");
    println!("  {}  {}    {}", "click".yellow().bold(), "<#>".dimmed(), "Click a numbered link");
    println!("  {}              {}", "back".yellow().bold(), "Go to previous page");
    println!("  {}           {}", "forward".yellow().bold(), "Go to next page");
    println!("  {}           {}", "refresh".yellow().bold(), "Reload current page");
    println!("  {}             {}", "links".yellow().bold(), "List links on page");
    println!("  {}           {}", "history".yellow().bold(), "Show browsing history");
    println!("  {}         {}", "bookmarks".yellow().bold(), "Show saved bookmarks");
    println!("  {}  {}  {}", "bookmark".yellow().bold(), "<name>".dimmed(), "Bookmark current page");
    println!("  {}              {}", "help".yellow().bold(), "Show this help menu");
    println!("  {}              {}", "quit".yellow().bold(), "Exit the browser");
    println!("\n  {}", "Shortcuts: g, s, c, b, f, r, l, h, bm, q, ?".dimmed());
    println!();
}