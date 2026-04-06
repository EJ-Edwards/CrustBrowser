// Import necessary modules for CLI functionality

import chalk from 'chalk';
import readline from 'readline/promises';
import { spawn, execSync } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';

// Figure out where this file lives so we can find the Rust binary
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, '..');

// On Windows the binary ends in .exe, on Mac/Linux it doesn't
const binaryName = process.platform === 'win32' ? 'crust-browser.exe' : 'crust-browser';
const RUST_BINARY = path.resolve(ROOT_DIR, 'target', 'release', binaryName);

// Read the current version from package.json
const pkg = JSON.parse(fs.readFileSync(path.resolve(__dirname, 'package.json'), 'utf-8'));
const LOCAL_VERSION = pkg.version;

// Checks GitHub for a newer version by comparing local and remote package.json
async function checkForUpdates() {
    try {
        const res = await fetch('https://raw.githubusercontent.com/EJ-Edwards/CrustBrowser/main/CLI/package.json');
        if (!res.ok) return; // silently skip if we can't reach GitHub
        const remote = await res.json();
        if (remote.version !== LOCAL_VERSION) {
            console.log(chalk.yellow.bold(`\n  Update available! ${LOCAL_VERSION} → ${remote.version}`));
            console.log(chalk.yellow(`  Run update.bat or: git pull && cargo build --release\n`));
        }
    } catch {
        // No internet or GitHub down — just skip the check
    }
}

function welcome() {
    console.log(chalk.green.bold("\n  Welcome to Crust-Browser CLI!\n"));
}


// Displays the Terms and Services of Crust Browser
function showTOS() {
    const tosText = `
${chalk.yellow.bold("Terms of Service")}
1. You agree to use this tool responsibly.
2. No illegal or harmful activities.
3. We are not liable for any damages caused by misuse.
4. Continued use means acceptance of these terms.
    `;
    console.log(tosText);
}

// Prompts the user to accept the Terms of Service and returns their response

async function askTOSAcceptance() {
    const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
    const answer = await rl.question(chalk.cyan('Do you accept the Terms of Service? (yes/no): '));
    rl.close();
    return answer.trim().toLowerCase();
}
// Launches the Rust browser binary
// Uses 'inherit' so the user can interact with it directly (type commands, see output)
function launchBrowser(args) {
    console.log(chalk.green.bold("Launching Crust-Browser...\n"));
    const child = spawn(RUST_BINARY, args, { stdio: 'inherit' });

    child.on('error', (error) => {
        console.error(chalk.red(`\nError: ${error.message}`));
        console.error(chalk.yellow('Make sure you built the Rust binary first: cargo build --release'));
        process.exit(1);
    });

    child.on('close', (code) => {
        console.log(chalk.gray(`\nCrust-Browser exited with code ${code}`));
    });
}

// Main function to run the CLI application
async function main() {
    welcome();
    await checkForUpdates(); // Check GitHub for a newer version
    showTOS();

    const answer = await askTOSAcceptance();
    if (answer === 'yes' || answer === 'y') {
        console.log(chalk.green.bold("\nThank you for accepting the Terms of Service!"));
        launchBrowser(process.argv.slice(2));
    } else {
        console.log(chalk.red.bold("\nYou must accept the Terms of Service to use this tool."));
        process.exit(1);
    }
}

main();