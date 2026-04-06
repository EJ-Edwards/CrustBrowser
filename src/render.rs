// Import ParsedPage from parsar.rs so we can display its data
use crate::parsar::ParsedPage;

// Takes a parsed page and prints it nicely in the terminal
// This is what the user sees after navigating to a URL
pub fn render_page(page: &ParsedPage) {
    // Show the page title in a banner
    println!("=== {} ===\n", page.title);
    
    // Show headings if the page has any
    if !page.headings.is_empty() {
        println!("Headings:");
        for heading in &page.headings {
            println!("  - {}", heading);
        }
        println!();
    }
    
    // Show all clickable links (text + URL)
    if !page.links.is_empty() {
        println!("Links:");
        for (text, url) in &page.links {
            println!("  - {} → {}", text, url);
        }
        println!();
    }
    
    // Show the main readable text from the page
    if !page.text.is_empty() {
        println!("Content:");
        for paragraph in &page.text {
            println!("  {}", paragraph);
        }
    }
}