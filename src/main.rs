//! Cosmic Notebook - A lightweight Markdown editor for the COSMIC desktop
//!
//! Entry point for the application. Handles CLI argument parsing,
//! logging initialization, and application bootstrap.

mod app;
mod config;
mod error;
mod message;
mod state;

// Module stubs for future phases
mod editor;
mod file_handler;
mod markdown;
mod search;
mod ui;
mod utils;

// Icon cache for bundled SVG icons
mod icon_cache;

// Menu and keyboard shortcuts
mod menu;

// Internationalization
mod i18n;

use app::{CosmicNotebook, Flags};
use std::path::PathBuf;

/// Application name for logging
const APP_NAME: &str = "cosmic-notebook";

fn main() -> cosmic::iced::Result {
    // Initialize logging
    init_logging();

    log::info!("Starting Cosmic Notebook");

    // Parse command line arguments
    let flags = parse_args();

    // Initialize and run the Cosmic application
    // Note: Don't use .size() with cosmic apps - it can cause Wayland protocol errors
    // The window size is managed by the compositor
    cosmic::app::run::<CosmicNotebook>(
        cosmic::app::Settings::default()
            .size_limits(cosmic::iced::Limits::NONE.min_width(400.0).min_height(300.0)),
        flags,
    )
}

/// Initialize the logging system
fn init_logging() {
    // Set default log level if not specified
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,cosmic_notebook=debug");
    }

    env_logger::Builder::from_default_env()
        .format_timestamp_millis()
        .init();
}

/// Parse command line arguments
fn parse_args() -> Flags {
    let args: Vec<String> = std::env::args().collect();
    let mut flags = Flags::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-v" | "--version" => {
                print_version();
                std::process::exit(0);
            }
            "-d" | "--directory" => {
                if i + 1 < args.len() {
                    flags.working_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                } else {
                    eprintln!("Error: --directory requires a path argument");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
            _ => {
                // Treat as a file path
                let path = PathBuf::from(&args[i]);
                if path.exists() {
                    flags.files.push(path);
                } else {
                    // Could be a new file - add anyway
                    flags.files.push(path);
                }
            }
        }
        i += 1;
    }

    // If no working directory specified but files given, use first file's parent
    if flags.working_dir.is_none() && !flags.files.is_empty() {
        if let Some(parent) = flags.files[0].parent() {
            flags.working_dir = Some(parent.to_path_buf());
        }
    }

    // Default to current directory
    if flags.working_dir.is_none() {
        flags.working_dir = std::env::current_dir().ok();
    }

    flags
}

/// Print help message
fn print_help() {
    println!(
        r#"Cosmic Notebook - A lightweight Markdown editor

USAGE:
    cosmic-notebook [OPTIONS] [FILES...]

OPTIONS:
    -h, --help          Show this help message
    -v, --version       Show version information
    -d, --directory     Set working directory for file browser

EXAMPLES:
    cosmic-notebook                     Open with empty document
    cosmic-notebook README.md           Open a specific file
    cosmic-notebook *.md                Open multiple files
    cosmic-notebook -d ~/Documents      Open with specific working directory

KEYBOARD SHORTCUTS:
    Ctrl+N              New file
    Ctrl+O              Open file
    Ctrl+S              Save file
    Ctrl+Shift+S        Save as
    Ctrl+W              Close tab
    Ctrl+Q              Quit
    Ctrl+Z              Undo
    Ctrl+Y              Redo
    Ctrl+F              Find
    Ctrl+H              Find and replace
    Ctrl+Shift+P        Command palette
    Ctrl+B              Toggle sidebar
"#
    );
}

/// Print version information
fn print_version() {
    println!(
        "{} {}",
        APP_NAME,
        env!("CARGO_PKG_VERSION")
    );
}
