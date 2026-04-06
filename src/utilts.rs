// --- utilts.rs ---
// This file handles commands, user input parsing, and the help menu.
// It's the brain behind understanding what the user types.

// Prints the welcome message when the browser first starts
pub fn welcome_message() {
    println!("Welcome to My Rust Browser!");
    println!("Type 'help' for a list of commands.\n");
}

// All the commands the browser understands
// Each variant represents a different action the user can take
pub enum Command {
    Go(String),          // go <url> — navigate to a URL
    Search(String),      // search <query> — web search
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
        "go" | "open" | "navigate" => {
            if arg.is_empty() {
                Command::Unknown("Usage: go <url>".to_string())
            } else {
                Command::Go(arg)
            }
        }
        "search" | "find" => {
            if arg.is_empty() {
                Command::Unknown("Usage: search <query>".to_string())
            } else {
                Command::Search(arg)
            }
        }
        "back"      => Command::Back,
        "forward"   => Command::Forward,
        "refresh"   => Command::Refresh,
        "links"     => Command::Links,
        "history"   => Command::History,
        "bookmarks" => Command::Bookmarks,
        "bookmark"  => {
            if arg.is_empty() {
                Command::Bookmark("Untitled".to_string())
            } else {
                Command::Bookmark(arg)
            }
        }
        "help"      => Command::Help,
        "quit" | "exit" | "q" => Command::Quit,
        _ => Command::Unknown(trimmed.to_string()),
    }
}

// Prints the full list of commands the user can type
pub fn show_help() {
    println!("\n  Available Commands:");
    println!("  ─────────────────────────────────────");
    println!("  go <url>          Navigate to a URL");
    println!("  search <query>    Search the web");
    println!("  back              Go to previous page");
    println!("  forward           Go to next page");
    println!("  refresh           Reload current page");
    println!("  links             List links on page");
    println!("  history           Show browsing history");
    println!("  bookmarks         Show saved bookmarks");
    println!("  bookmark <name>   Bookmark current page");
    println!("  help              Show this help menu");
    println!("  quit              Exit the browser");
    println!();
}