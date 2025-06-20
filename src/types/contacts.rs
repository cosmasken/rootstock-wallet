use colored::Colorize;
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub address: Address,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Local>,
}

impl Contact {
    pub fn new(name: String, address: Address, notes: Option<String>, tags: Vec<String>) -> Self {
        Self {
            name,
            address,
            notes,
            tags,
            created_at: chrono::Local::now(),
        }
    }

    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Contact name cannot be empty"));
        }
        if self.address == Address::zero() {
            return Err(anyhow::anyhow!("Contact address cannot be zero"));
        }
        if self.notes.as_ref().map_or(false, |n| n.is_empty()) {
            return Err(anyhow::anyhow!("Notes cannot be empty if provided"));
        }
        if self.tags.iter().any(|tag| tag.is_empty()) {
            return Err(anyhow::anyhow!("Tags cannot be empty"));
        }
        if self.tags.len() > 5 {
            return Err(anyhow::anyhow!("A contact can have a maximum of 5 tags"));
        }
        if self.created_at.timestamp() > chrono::Local::now().timestamp() {
            return Err(anyhow::anyhow!(
                "Created at timestamp cannot be in the future"
            ));
        }
        if self.created_at.timestamp() < 0 {
            return Err(anyhow::anyhow!("Created at timestamp cannot be negative"));
        }
        if self.created_at.timestamp() < 1_000_000_000 {
            return Err(anyhow::anyhow!("Created at timestamp is too old"));
        }
        if self.created_at.timestamp() > chrono::Local::now().timestamp() + 60 * 60 * 24 * 365 {
            return Err(anyhow::anyhow!(
                "Created at timestamp is too far in the future"
            ));
        }
        if self.created_at.timestamp() < chrono::Local::now().timestamp() - 60 * 60 * 24 * 365 {
            return Err(anyhow::anyhow!(
                "Created at timestamp is too far in the past"
            ));
        }
        Ok(())
    }
}

impl fmt::Display for Contact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.name,
            format!(
                "{}{}",
                "0x".on_green(),
                self.address.to_string()[2..].on_green()
            )
        )?;

        if let Some(ref notes) = self.notes {
            write!(f, "\nNotes: {}", notes)?;
        }

        if !self.tags.is_empty() {
            write!(f, "\nTags: {}", self.tags.join(", "))?;
        }

        Ok(())
    }
}
