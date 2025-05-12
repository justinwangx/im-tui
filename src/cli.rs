use clap::{Parser, Subcommand};

/// im - a tool for sending and receiving iMessages in the terminal
#[derive(Parser)]
#[command(
    version,
    author = "Justin Wang",
    about = "send and receive iMessages in the terminal"
)]
pub struct Cli {
    /// Set the default contact identifier (e.g., phone number or email) and save it to configuration.
    #[arg(short, long)]
    pub set: Option<String>,

    /// Set the display name for the default contact.
    #[arg(short, long)]
    pub name: Option<String>,

    /// Optionally override the saved contact identifier for this run.
    #[arg(short, long)]
    pub contact: Option<String>,

    /// Show more detailed information.
    #[arg(short, long)]
    pub verbose: bool,

    /// Optional contact name to fetch messages from. Uses contacts from the configuration.
    #[arg(value_name = "CONTACT_NAME")]
    pub contact_name: Option<String>,

    /// Subcommands for managing contacts
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Subcommands for the CLI
#[derive(Subcommand)]
pub enum Commands {
    /// Add or update a contact in the configuration
    Add {
        /// Name for the contact (used to reference it)
        #[arg(value_name = "NAME")]
        name: String,

        /// Contact identifier (phone number or email)
        #[arg(value_name = "IDENTIFIER")]
        identifier: String,

        /// Optional display name for the contact
        #[arg(short, long)]
        display_name: Option<String>,
    },

    /// Remove a contact from the configuration
    Remove {
        /// Name of the contact to remove
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// List all configured contacts
    Contacts,

    /// Show the path to the configuration file
    Config,
}
