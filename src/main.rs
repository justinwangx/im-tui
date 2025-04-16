use chrono::{DateTime, Local, TimeZone};
use clap::Parser;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

fn format_phone_number(number: &str) -> String {
    // If it's a digit-only string without country code, add +1
    if number.chars().all(|c| c.is_digit(10)) {
        format!("+1{}", number)
    } else if !number.contains('+')
        && number
            .trim_start_matches('1')
            .chars()
            .all(|c| c.is_digit(10))
    {
        // Handle numbers with country code digit but missing "+" (e.g., "13015057171" → "+13015057171")
        format!("+{}", number)
    } else {
        // Already has a country code or isn't a phone number
        number.to_string()
    }
}

fn format_display_number(number: &str) -> String {
    if number.starts_with("+1") && number.len() > 2 {
        number[2..].to_string()
    } else if number.starts_with("1") && number.chars().skip(1).all(|c| c.is_digit(10)) {
        number[1..].to_string()
    } else {
        number.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    /// The contact identifier (e.g., phone number or email).
    contact: Option<String>,
    /// The display name for the contact.
    display_name: Option<String>,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            contact: None,
            display_name: None,
        }
    }
}

/// Command line options using clap.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set the contact identifier (e.g., phone number or email) and save it to configuration.
    #[arg(short, long)]
    set: Option<String>,

    /// Set the display name for the contact.
    #[arg(short, long)]
    name: Option<String>,

    /// Optionally override the saved contact identifier for this run.
    #[arg(short, long)]
    contact: Option<String>,
}

fn format_relative_time(dt: DateTime<Local>) -> String {
    let now = Local::now();
    let today = now.date_naive();
    let message_date = dt.date_naive();

    if message_date == today {
        format!(
            "today at {}",
            dt.format("%l:%M%p").to_string().to_lowercase().trim()
        )
    } else if message_date == today.pred_opt().unwrap() {
        format!(
            "yesterday at {}",
            dt.format("%l:%M%p").to_string().to_lowercase().trim()
        )
    } else {
        let days = (today - message_date).num_days();
        format!(
            "{} days ago at {}",
            days,
            dt.format("%l:%M%p").to_string().to_lowercase().trim()
        )
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut cfg: Config = confy::load("gf", None).unwrap();

    let set_flag = args.set.is_some();
    let name_flag = args.name.is_some();

    // If the --set flag is provided, update the stored configuration.
    if let Some(set_contact) = &args.set {
        // Format phone number before saving
        let formatted_contact = format_phone_number(set_contact);
        cfg.contact = Some(formatted_contact.clone());
        println!("Saved contact: {}", formatted_contact);
    }

    // If the --name flag is provided, update the stored configuration.
    if let Some(name) = &args.name {
        cfg.display_name = Some(name.clone());
        println!("Saved display name: {}", name);
    }

    // Save config if either --set or --name was provided
    if set_flag || name_flag {
        confy::store("gf", None, cfg).unwrap();
        return Ok(());
    }

    // Determine which contact to use: command-line override has precedence over saved configuration.
    let using_cli_contact = args.contact.is_some();
    let contact = if let Some(cli_contact) = args.contact {
        // Format phone number from command line
        format_phone_number(&cli_contact)
    } else if let Some(cfg_contact) = cfg.contact {
        // Config contact is already formatted when saved
        cfg_contact
    } else {
        eprintln!("No contact configured. Please set one using: gf --set <contact>");
        return Ok(());
    };

    // Build the path to the Messages database.
    let home_dir = env::var("HOME").expect("Could not determine HOME directory");
    let mut db_path = PathBuf::from(home_dir);
    db_path.push("Library/Messages/chat.db");

    // Open the SQLite connection.
    let conn = Connection::open(db_path)?;

    // SQL query to select the last message for the specified contact.
    // Get the raw timestamp for formatting in Rust
    let query = r#"
        SELECT text,
               date / 1000000000 + strftime('%s','2001-01-01') as unix_timestamp
        FROM message
        JOIN handle ON message.handle_id = handle.ROWID
        WHERE handle.id = ?
        ORDER BY date DESC
        LIMIT 1;
    "#;

    let mut stmt = conn.prepare(query)?;
    let mut rows = stmt.query(params![contact])?;

    if let Some(row) = rows.next()? {
        // Retrieve the text and timestamp for the latest message.
        let text: Option<String> = row.get(0)?;
        let timestamp: i64 = row.get(1)?;

        // Convert Unix timestamp to DateTime<Local>
        let dt = Local.timestamp_opt(timestamp, 0).unwrap();

        // Get display name or fall back to contact identifier - only use display name if using saved contact
        let contact_for_display = format_display_number(&contact);
        let display_name = if !using_cli_contact {
            cfg.display_name.as_ref().unwrap_or(&contact_for_display)
        } else {
            &contact_for_display
        };

        println!(
            "Last message from {} received {}",
            display_name,
            format_relative_time(dt)
        );
        println!("{}", text.unwrap_or_else(|| "<empty message>".into()));
    } else {
        let contact_for_display = format_display_number(&contact);
        println!("No messages found for contact: {}", contact_for_display);
    }

    Ok(())
}
