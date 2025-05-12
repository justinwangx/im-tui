mod cli;
mod config;
mod db;
mod error;
mod formatter;
mod sender;
mod tui;

use crate::cli::{Cli, Commands, ConfigCommands, ContactCommands};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::formatter::{format_display_number, format_phone_number};
use clap::Parser;
use std::process;

/// Application name used for configuration files.
pub const APP_NAME: &str = "im";

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
        println!("im v{}", APP_VERSION);
    }

    let mut config = Config::load()?;

    match args.command {
        Some(Commands::Message {
            contact_name,
            contact,
        }) => {
            // Try to get contact info, if it fails with NoContact, run the setup TUI
            match get_contact_info(&contact_name, &contact, &config, verbose) {
                Ok((contact, display_name)) => {
                    // Run the TUI with the contact
                    tui::run_chat_tui(contact, display_name)
                }
                Err(Error::NoContact) => {
                    if verbose {
                        println!("No contact configured. Launching setup TUI.");
                    }

                    let new_config = tui::run_setup_tui()?;

                    // Save the new configuration
                    let config = new_config;
                    config.save()?;

                    if let Some(contact) = config.default_contact() {
                        let display_name = match config.default_display_name() {
                            Some(name) => name.clone(),
                            None => format_display_number(&contact),
                        };

                        tui::run_chat_tui(contact, display_name)
                    } else {
                        // User canceled setup
                        Err(Error::NoContact)
                    }
                }
                Err(e) => Err(e),
            }
        }
        Some(Commands::Config { command }) => {
            match command {
                ConfigCommands::SetContact { contact } => {
                    let formatted_contact = format_phone_number(&contact);
                    config.set_default_contact(formatted_contact.clone());
                    println!("Saved default contact: {}", formatted_contact);
                    config.save()?;
                }
                ConfigCommands::SetName { name } => {
                    config.set_default_display_name(name.clone());
                    println!("Saved default display name: {}", name);
                    config.save()?;
                }
                ConfigCommands::Show => {
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
        Some(Commands::Contacts { command }) => {
            match command {
                ContactCommands::Add {
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
                }
                ContactCommands::Remove { name } => {
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
                ContactCommands::Contacts => {
                    tui::run_contacts_tui(config.clone())?;
                }
            }
            Ok(())
        }
        Some(Commands::Add {
            name,
            identifier,
            display_name,
        }) => {
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
            Ok(())
        }
        Some(Commands::Remove { name }) => {
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
            Ok(())
        }
        Some(Commands::ContactsList) => {
            tui::run_contacts_tui(config.clone())?;
            Ok(())
        }
        None => {
            // If a contact name was provided as a positional argument, use it
            if let Some(contact_name) = args.contact_name {
                match get_contact_info(&Some(contact_name), &None, &config, verbose) {
                    Ok((contact, display_name)) => tui::run_chat_tui(contact, display_name),
                    Err(e) => Err(e),
                }
            } else {
                // No command or contact name specified, default to messaging with default contact
                match get_contact_info(&None, &None, &config, verbose) {
                    Ok((contact, display_name)) => tui::run_chat_tui(contact, display_name),
                    Err(Error::NoContact) => {
                        if verbose {
                            println!("No contact configured. Launching setup TUI.");
                        }

                        let new_config = tui::run_setup_tui()?;
                        new_config.save()?;

                        if let Some(contact) = new_config.default_contact() {
                            let display_name = match new_config.default_display_name() {
                                Some(name) => name.clone(),
                                None => format_display_number(&contact),
                            };

                            tui::run_chat_tui(contact, display_name)
                        } else {
                            Err(Error::NoContact)
                        }
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}

/// Get contact information based on command-line arguments and configuration
fn get_contact_info(
    contact_name: &Option<String>,
    contact: &Option<String>,
    config: &Config,
    verbose: bool,
) -> Result<(String, String)> {
    // Priority:
    // 1. --contact CLI flag
    // 2. Positional contact_name argument (named contact)
    // 3. Default contact from config

    if let Some(cli_contact) = contact {
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

    if let Some(contact_name) = contact_name {
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
