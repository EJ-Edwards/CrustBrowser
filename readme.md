# CrustBrowser

A simple text-based web browser written in Rust with a Node.js CLI frontend.

CrustBrowser lets you navigate the web, search across multiple engines, and view page content — all from your terminal.

## Features

- Browse any URL and view page titles, headings, links, and text
- Search the web across Google, Bing, DuckDuckGo, and Yahoo
- Bookmark pages and track browsing history
- Navigate back and forward through your session
- Node.js CLI with welcome screen and Terms of Service prompt

## Project Structure

```
CrustBrowser/
├── Cargo.toml              # Rust dependencies and project config
├── install.bat             # Windows install script
├── install.sh              # Mac/Linux install script
├── update.bat              # Windows update script
├── CLI/
│   ├── index.js            # Node.js CLI entry point (welcome, TOS, launches Rust binary)
│   └── package.json        # CLI dependencies and version
└── src/
    ├── main.rs             # Entry point — REPL loop that reads commands and dispatches them
    ├── network.rs          # HTTP requests — fetches web pages using reqwest (blocking)
    ├── parsar.rs           # HTML parser — extracts title, headings, links, and text using scraper
    ├── search.rs           # Search engine URL builder — supports Google, Bing, DuckDuckGo, Yahoo
    ├── utilts.rs           # Command system — defines all commands, parses input, help menu
    └── render.rs           # Display renderer — formats parsed pages for terminal output
```

## Installation

### Quick Install (clone + run script)

```bash
git clone https://github.com/EJ-Edwards/CrustBrowser.git
cd CrustBrowser
```

**Windows:**
```
install.bat
```

**Mac/Linux:**
```bash
chmod +x install.sh
./install.sh
```

The install script checks for Rust and Node.js, builds the binary, and installs CLI dependencies automatically.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- [Node.js](https://nodejs.org/) (18+) — for the CLI frontend
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Windows only — select "Desktop development with C++")

### Manual Setup

```bash
cargo build --release
cd CLI
npm install
```

## Updating

CrustBrowser checks for updates automatically when you launch the CLI. If a new version is available, you'll see a notification.

**To update manually:**

Windows:
```
update.bat
```

Mac/Linux:
```bash
git pull origin main
cargo build --release
cd CLI && npm install
```

## Usage

### Run via the CLI (recommended)

```bash
node CLI/index.js
```

This shows the welcome screen and Terms of Service. After accepting, it launches the Rust browser.

### Run the Rust binary directly

```bash
cargo run
```

## Commands

| Command | Description |
|---|---|
| `go <url>` | Navigate to a URL |
| `search <query>` | Search across all engines |
| `back` | Go to previous page |
| `forward` | Go to next page |
| `refresh` | Reload current page |
| `links` | List all links on the page |
| `history` | Show browsing history |
| `bookmarks` | Show saved bookmarks |
| `bookmark <name>` | Save current page as a bookmark |
| `help` | Show available commands |
| `quit` / `exit` | Close the browser |

## Dependencies

| Crate | Purpose |
|---|---|
| `reqwest` | HTTP client (blocking mode with rustls-tls) |
| `scraper` | HTML parsing and CSS selector queries |
| `url` | URL parsing and query string building |

## How It Works

1. **CLI** (`index.js`) — Shows welcome message and Terms of Service, then spawns the Rust binary
2. **REPL** (`main.rs`) — Reads user input in a loop and matches it against known commands
3. **Network** (`network.rs`) — Fetches the raw HTML from a URL
4. **Parser** (`parsar.rs`) — Parses the HTML into structured data (title, headings, links, text)
5. **Search** (`search.rs`) — Builds search URLs with query parameters for multiple engines
6. **Commands** (`utilts.rs`) — Parses raw input into a `Command` enum for the REPL to handle
7. **Render** (`render.rs`) — Formats and displays parsed pages in the terminal

## License

MIT