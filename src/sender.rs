use crate::error::{Error, Result};
use std::io::Write;

pub struct Sender {
    contact: String,
}

impl Sender {
    pub fn new(contact: String) -> Self {
        Self { contact }
    }

    pub fn send_message(&self, text: &str) -> Result<()> {
        // Create the AppleScript command
        let script = format!(
            r#"
            on run {{textBody}}
                tell application "Messages"
                    set targetService to first service whose service type = iMessage
                    set targetBuddy to buddy "{}" of targetService
                    send textBody to targetBuddy
                end tell
            end run
            "#,
            self.contact
        );

        // Execute the AppleScript
        let mut child = std::process::Command::new("osascript")
            .arg("-")
            .arg(text)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Write the script to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(script.as_bytes())?;
        }

        // Wait for the process to complete and check its output
        let output = child.wait_with_output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Generic(format!("Failed to send message: {}", error)));
        }

        Ok(())
    }
}
