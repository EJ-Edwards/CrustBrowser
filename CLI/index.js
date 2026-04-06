// Import necessary modules for CLI functionality

import chalk from 'chalk';
import readline from 'readline/promises';
import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';
import os from 'os';

// Figure out where this file lives so we can find the Rust binary
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, '..');

// On Windows the binary ends in .exe, on Mac/Linux it doesn't
const binaryName = process.platform === 'win32' ? 'crust-browser.exe' : 'crust-browser';
const RUST_BINARY = path.resolve(ROOT_DIR, 'target', 'release', binaryName);

// Read the current version from package.json
const pkg = JSON.parse(fs.readFileSync(path.resolve(__dirname, 'package.json'), 'utf-8'));
const LOCAL_VERSION = pkg.version;

// Config file path — saves TOS acceptance so users aren't asked every time
const CONFIG_DIR = path.join(os.homedir(), '.crust-browser');
const CONFIG_FILE = path.join(CONFIG_DIR, 'config.json');

// Load saved config (TOS acceptance, etc.)
function loadConfig() {
    try {
        if (fs.existsSync(CONFIG_FILE)) {
            return JSON.parse(fs.readFileSync(CONFIG_FILE, 'utf-8'));
        }
    } catch { /* corrupted config — start fresh */ }
    return {};
}

// Save config to disk
function saveConfig(config) {
    fs.mkdirSync(CONFIG_DIR, { recursive: true });
    fs.writeFileSync(CONFIG_FILE, JSON.stringify(config, null, 2));
}

// Checks GitHub for a newer version by comparing local and remote package.json
async function checkForUpdates() {
    try {
        const res = await fetch('https://raw.githubusercontent.com/EJ-Edwards/CrustBrowser/main/CLI/package.json');
        if (!res.ok) return;
        const remote = await res.json();
        if (remote.version !== LOCAL_VERSION) {
            console.log(chalk.yellow.bold(`  Update available! ${LOCAL_VERSION} → ${remote.version}`));
            console.log(chalk.yellow(`  Run update.bat or: git pull && cargo build --release\n`));
        }
    } catch {
        // No internet or GitHub down — just skip the check
    }
}

function welcome() {
    console.log(chalk.cyan.bold("\n  ╔═══════════════════════════════════════╗"));
    console.log(chalk.cyan.bold("  ║         🦀  Crust Browser  🦀         ║"));
    console.log(chalk.cyan.bold("  ║     A text-based web browser in Rust   ║"));
    console.log(chalk.cyan.bold("  ╚═══════════════════════════════════════╝"));
    console.log(chalk.gray(`  v${LOCAL_VERSION}\n`));
}

// Displays the Terms and Services of Crust Browser
function showTOS() {
    console.log(chalk.yellow.bold("  Terms of Service"));
    console.log(chalk.gray("  ───────────────────────────────────────"));
    console.log(chalk.white("  1. You agree to use this tool responsibly."));
    console.log(chalk.white("  2. No illegal or harmful activities."));
    console.log(chalk.white("  3. We are not liable for any damages caused by misuse."));
    console.log(chalk.white("  4. Continued use means acceptance of these terms.\n"));
}

// Prompts the user to accept the Terms of Service and returns their response
async function askTOSAcceptance() {
    const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
    const answer = await rl.question(chalk.cyan('  Do you accept the Terms of Service? (yes/no): '));
    rl.close();
    return answer.trim().toLowerCase();
}

// Launches the Rust browser binary
// Uses 'inherit' so the user can interact with it directly (type commands, see output)
function launchBrowser(args) {
    // Check the binary exists before trying to launch it
    if (!fs.existsSync(RUST_BINARY)) {
        console.error(chalk.red.bold('\n  Binary not found!'));
        console.error(chalk.yellow('  Run: cargo build --release'));
        process.exit(1);
    }
    console.log(chalk.green.bold("  Launching Crust Browser...\n"));
    const child = spawn(RUST_BINARY, args, { stdio: 'inherit' });

    child.on('error', (error) => {
        console.error(chalk.red(`\n  Error: ${error.message}`));
        console.error(chalk.yellow('  Make sure you built the Rust binary first: cargo build --release'));
        process.exit(1);
    });

    child.on('close', (code) => {
        console.log(chalk.gray(`\n  Crust Browser exited with code ${code}`));
    });
}

// Main function to run the CLI application
async function main() {
    welcome();
    await checkForUpdates();

    const config = loadConfig();

    // If the user already accepted the TOS, skip the prompt
    if (config.tosAccepted) {
        console.log(chalk.gray("  Terms of Service previously accepted.\n"));
        launchBrowser(process.argv.slice(2));
        return;
    }

    showTOS();

    const answer = await askTOSAcceptance();
    if (answer === 'yes' || answer === 'y') {
        // Save acceptance so they won't be asked again
        config.tosAccepted = true;
        saveConfig(config);
        console.log(chalk.green.bold("\n  Thank you for accepting the Terms of Service!"));
        launchBrowser(process.argv.slice(2));
    } else {
        console.log(chalk.red.bold("\n  You must accept the Terms of Service to use this tool."));
        process.exit(1);
    }
}

main();