// --- Module declarations ---
// Each of these links to a separate .rs file in the src/ folder
mod network;  // Handles fetching web pages (HTTP requests)
mod parsar;   // Parses raw HTML into usable data (title, links, text)
mod search;   // Builds search engine URLs (Google, Bing, etc.)
mod utilts;   // Commands, welcome message, and help menu
mod render;   // Will handle displaying pages nicely in the terminal

// Standard library imports for reading user input
use std::io::{self, Write};
// Pull in the stuff we need from utilts.rs
use utilts::{welcome_message, parse_command, show_help, Command};

fn main() {
    // Show the welcome message when the browser starts
    welcome_message();

    // Main loop — keeps running until the user types "quit"
    loop {
        // Print the prompt and make sure it shows up before waiting for input
        print!("> ");
        io::stdout().flush().unwrap();

        // Read whatever the user types
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input.");
            continue; // Skip to the next loop iteration
        }

        // Parse the input into a Command and handle it
        match parse_command(&input) {
            // User typed "go <url>" — fetch and parse the web page
            Command::Go(url) => {
                println!("Navigating to: {}", url);
                match network::get(&url) {
                    Ok(html) => {
                        // HTML fetched successfully — parse it and display it
                        let page = parsar::parse_html(&html);
                        render::render_page(&page);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            // User typed "search <query>" — show search URLs for all engines
            Command::Search(query) => {
                println!("Searching for: {}", query);
                search::search_all_engines(&query);
            }

            // Navigation commands (not yet implemented)
            Command::Back => println!("Going back..."),
            Command::Forward => println!("Going forward..."),
            Command::Refresh => println!("Refreshing..."),

            // Page info commands (not yet implemented)
            Command::Links => println!("Listing links..."),
            Command::History => println!("Showing history..."),
            Command::Bookmarks => println!("Showing bookmarks..."),
            Command::Bookmark(name) => println!("Bookmarked as: {}", name),

            // Show the help menu
            Command::Help => show_help(),

            // Exit the browser
            Command::Quit => {
                println!("Goodbye!");
                break; // Break out of the loop to end the program
            }

            // Anything we don't recognize
            Command::Unknown(msg) => println!("Unknown command: {}", msg),
        }
    }
}
