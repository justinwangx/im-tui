use crate::error::{Error, Result};
use crate::APP_NAME;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for the application.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// The default contact identifier (e.g., phone number or email).
    default_contact: Option<String>,
    /// The display name for the default contact.
    default_display_name: Option<String>,
    /// Map of named contacts to their identifiers.
    #[serde(default)]
    contacts: HashMap<String, ContactEntry>,
}

/// A contact entry in the contacts map.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactEntry {
    /// The contact identifier (e.g., phone number or email).
    pub identifier: String,
    /// Optional display name for the contact.
    pub display_name: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_contact: None,
            default_display_name: None,
            contacts: HashMap::new(),
        }
    }
}

impl Config {
    /// Load configuration from disk.
    pub fn load() -> Result<Self> {
        match confy::load(APP_NAME, None) {
            Ok(config) => Ok(config),
            Err(e) => {
                // Get the config path for error reporting
                let path = confy::get_configuration_file_path(APP_NAME, None)
                    .unwrap_or_else(|_| PathBuf::from("unknown path"));

                // Try to read the raw file contents
                let contents = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| "Could not read file".to_string());

                Err(Error::Generic(format!(
                    "Failed to load config from {}: {}\nFile contents:\n{}",
                    path.display(),
                    e,
                    contents
                )))
            }
        }
    }

    /// Save configuration to disk.
    pub fn save(&self) -> Result<()> {
        Ok(confy::store(APP_NAME, None, self)?)
    }

    /// Get the path to the configuration file.
    pub fn config_path() -> Option<PathBuf> {
        confy::get_configuration_file_path(APP_NAME, None).ok()
    }

    /// Get the default contact identifier.
    pub fn default_contact(&self) -> Option<String> {
        self.default_contact.clone()
    }

    /// Set the default contact identifier.
    pub fn set_default_contact(&mut self, contact: String) {
        self.default_contact = Some(contact);
    }

    /// Get the default display name.
    pub fn default_display_name(&self) -> Option<&String> {
        self.default_display_name.as_ref()
    }

    /// Set the default display name.
    pub fn set_default_display_name(&mut self, name: String) {
        self.default_display_name = Some(name);
    }

    /// Add or update a named contact.
    pub fn add_contact(&mut self, name: String, identifier: String, display_name: Option<String>) {
        self.contacts.insert(
            name,
            ContactEntry {
                identifier,
                display_name,
            },
        );
    }

    /// Remove a named contact.
    pub fn remove_contact(&mut self, name: &str) -> bool {
        self.contacts.remove(name).is_some()
    }

    /// Get a contact by name (case-sensitive).
    pub fn get_contact(&self, name: &str) -> Option<&ContactEntry> {
        self.contacts.get(name)
    }

    /// Get a contact by name (case-insensitive).
    pub fn get_contact_case_insensitive(&self, name: &str) -> Option<(&String, &ContactEntry)> {
        let lowercase_name = name.to_lowercase();
        self.contacts
            .iter()
            .find(|(k, _)| k.to_lowercase() == lowercase_name)
    }

    /// List all contacts in the configuration.
    pub fn list_contacts(&self) -> Vec<(&String, &ContactEntry)> {
        self.contacts.iter().collect()
    }

    /// Get the number of configured contacts.
    pub fn contact_count(&self) -> usize {
        self.contacts.len()
    }
}
