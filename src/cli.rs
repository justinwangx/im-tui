use clap::{Parser, Subcommand};

/// im - a tool for sending and receiving iMessages in the terminal
#[derive(Parser)]
#[command(
    version,
    author = "Justin Wang",
    about = "send and receive iMessages in the terminal"
)]
pub struct Cli {
    /// Show more detailed information
    #[arg(short, long)]
    pub verbose: bool,

    /// Contact name to fetch messages from (uses contacts from the configuration)
    #[arg(value_name = "CONTACT_NAME")]
    pub contact_name: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Subcommands for the CLI
#[derive(Subcommand)]
pub enum Commands {
    /// Start messaging with a contact
    Message {
        /// Contact name to fetch messages from (uses contacts from the configuration)
        #[arg(value_name = "CONTACT_NAME")]
        contact_name: Option<String>,

        /// Override the saved contact identifier for this run
        #[arg(short, long)]
        contact: Option<String>,
    },

    /// Configure the application
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Manage contacts
    Contacts {
        #[command(subcommand)]
        command: ContactCommands,
    },

    /// Add or update a contact
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

    /// Remove a contact
    Remove {
        /// Name of the contact to remove
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Show contacts in an interactive TUI
    ContactsList,
}

/// Configuration subcommands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set the default contact identifier
    SetContact {
        /// Contact identifier (e.g., phone number or email)
        #[arg(value_name = "CONTACT")]
        contact: String,
    },

    /// Set the display name for the default contact
    SetName {
        /// Display name for the contact
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Show the path to the configuration file
    Show,
}

/// Contact management subcommands
#[derive(Subcommand)]
pub enum ContactCommands {
    /// Add or update a contact
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

    /// Remove a contact
    Remove {
        /// Name of the contact to remove
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Show contacts in an interactive TUI
    Contacts,
}
