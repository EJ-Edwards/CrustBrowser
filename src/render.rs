// Import ParsedPage from parsar.rs so we can display its data
use crate::parsar::ParsedPage;
use colored::Colorize;

// Takes a parsed page and prints it nicely in the terminal with colors
// This is what the user sees after navigating to a URL
pub fn render_page(page: &ParsedPage) {
    // Show the page title in a colored banner
    println!("\n  {}", "═══════════════════════════════════════".cyan());
    println!("  {}", page.title.white().bold());
    println!("  {}\n", "═══════════════════════════════════════".cyan());

    // Show headings if the page has any
    if !page.headings.is_empty() {
        println!("  {}", "Headings:".yellow().bold());
        for heading in &page.headings {
            println!("    {} {}", "▸".cyan(), heading);
        }
        println!();
    }

    // Show all clickable links with numbers so the user can "click <number>"
    if !page.links.is_empty() {
        println!("  {}", "Links:".yellow().bold());
        for (i, (text, url)) in page.links.iter().enumerate() {
            let display = if text.is_empty() { url } else { text };
            println!("    {} {}  {}", format!("[{}]", i + 1).green().bold(), display, url.dimmed());
        }
        println!();
    }

    // Show the main readable text from the page
    if !page.text.is_empty() {
        println!("  {}", "Content:".yellow().bold());
        for paragraph in &page.text {
            // Wrap long paragraphs so they look nice in the terminal
            println!("    {}", paragraph);
        }
    }
    println!();
}