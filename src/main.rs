mod cli;
mod config;
mod db;
mod error;
mod formatter;
mod sender;
mod tui;

use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::formatter::{format_display_number, format_phone_number};
use crate::tui::ContactsView;
use clap::Parser;
use std::process;

/// Application name used for configuration files.
pub const APP_NAME: &str = "gf";

/// Application version.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);

        // Try to print the config path even if there's an error
        if let Some(path) = Config::config_path() {
            eprintln!("Configuration file is located at: {}", path.display());
            eprintln!("You may need to delete this file to fix the 'Bad TOML data' error.");
        }

        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Cli::parse();
    let verbose = args.verbose;

    if verbose {
        println!("gf v{}", APP_VERSION);
    }

    // Handle configuration
    let mut config = Config::load()?;

    // Handle subcommands for contact management
    if let Some(cmd) = args.command {
        return handle_command(cmd, &mut config, verbose);
    }

    // Handle the set contact flag
    if let Some(set_contact) = &args.set {
        let formatted_contact = format_phone_number(set_contact);
        config.set_default_contact(formatted_contact.clone());
        println!("Saved default contact: {}", formatted_contact);

        if verbose {
            println!("Contact identifier normalized and saved to configuration.");
        }
    }

    // Handle the set name flag
    if let Some(name) = &args.name {
        config.set_default_display_name(name.clone());
        println!("Saved default display name: {}", name);

        if verbose {
            println!("Display name saved to configuration.");
        }
    }

    // Save config if either --set or --name was provided
    if args.set.is_some() || args.name.is_some() {
        config.save()?;
        return Ok(());
    }

    // Determine which contact to use
    let (contact, display_name) = get_contact_info(&args, &config, verbose)?;

    // Run the TUI
    tui::run_tui(contact, display_name)
}

/// Handle a CLI subcommand for contact management
fn handle_command(cmd: Commands, config: &mut Config, verbose: bool) -> Result<()> {
    match cmd {
        Commands::Add {
            name,
            identifier,
            display_name,
        } => {
            let formatted_id = format_phone_number(&identifier);
            config.add_contact(name.clone(), formatted_id.clone(), display_name.clone());
            config.save()?;

            println!(
                "Added contact '{}' with identifier '{}'",
                name, formatted_id
            );
            if let Some(display) = display_name {
                println!("Display name: {}", display);
            }

            if verbose {
                println!("Configuration updated successfully.");
            }
        }

        Commands::Remove { name } => {
            // Try to find a case-insensitive match first
            if let Some((actual_name, _)) = config.get_contact_case_insensitive(&name) {
                let actual_name = actual_name.clone(); // Clone to avoid borrow issues

                // Case-insensitive match found, now remove it
                if config.remove_contact(&actual_name) {
                    config.save()?;

                    if actual_name != name {
                        println!(
                            "Removed contact '{}' (matched '{}' case-insensitively)",
                            actual_name, name
                        );
                    } else {
                        println!("Removed contact '{}'", name);
                    }
                }
            } else if config.remove_contact(&name) {
                // Direct removal succeeded (unlikely to reach this after adding case-insensitive check)
                config.save()?;
                println!("Removed contact '{}'", name);
            } else {
                println!("Contact '{}' not found in configuration", name);
            }
        }

        Commands::Contacts => {
            let mut contacts_view = ContactsView::new(config.clone());
            contacts_view.run()?;
        }

        Commands::Config => {
            if let Some(path) = Config::config_path() {
                println!("Configuration file location:");
                println!("{}", path.display());
            } else {
                println!("Could not determine configuration file location.");
            }
        }
    }

    Ok(())
}

/// Get contact information based on command-line arguments and configuration
fn get_contact_info(args: &Cli, config: &Config, verbose: bool) -> Result<(String, String)> {
    // Priority:
    // 1. --contact CLI flag
    // 2. Positional contact_name argument (named contact)
    // 3. Default contact from config

    if let Some(cli_contact) = &args.contact {
        let formatted = format_phone_number(cli_contact);
        if verbose && formatted != *cli_contact {
            println!(
                "Note: Formatted contact identifier from '{}' to '{}'",
                cli_contact, formatted
            );
        }

        let display = format_display_number(&formatted);
        return Ok((formatted, display));
    }

    if let Some(contact_name) = &args.contact_name {
        // Try case-insensitive lookup first
        if let Some((actual_name, entry)) = config.get_contact_case_insensitive(contact_name) {
            let display = match &entry.display_name {
                Some(name) => name.clone(),
                None => format_display_number(&entry.identifier),
            };

            if verbose {
                if actual_name != contact_name {
                    println!(
                        "Using contact '{}' (matched '{}' case-insensitively)",
                        actual_name, contact_name
                    );
                } else {
                    println!("Using contact '{}'", actual_name);
                }
            }

            return Ok((entry.identifier.clone(), display));
        } else {
            // Fallback to case-sensitive lookup for backward compatibility
            if let Some(entry) = config.get_contact(contact_name) {
                let display = match &entry.display_name {
                    Some(name) => name.clone(),
                    None => format_display_number(&entry.identifier),
                };

                if verbose {
                    println!("Using contact '{}'", contact_name);
                }

                return Ok((entry.identifier.clone(), display));
            } else {
                return Err(Error::Generic(format!(
                    "Contact '{}' not found in configuration",
                    contact_name
                )));
            }
        }
    }

    if let Some(default_contact) = config.default_contact() {
        if verbose {
            println!("Using default contact: {}", default_contact);
        }

        let display = match config.default_display_name() {
            Some(name) => name.clone(),
            None => format_display_number(&default_contact),
        };

        return Ok((default_contact, display));
    }

    Err(Error::NoContact)
}
