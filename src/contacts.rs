use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Contact {
    pub name: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ContactsBook {
    pub contacts: HashMap<String, Contact>,
}

impl ContactsBook {
    pub fn load(path: &str) -> Self {
        if Path::new(path).exists() {
            let data = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            ContactsBook::default()
        }
    }

    pub fn save(&self, path: &str) {
        let data = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, data).unwrap();
    }

    pub fn add_contact(&mut self, name: String, address: String) {
        let contact = Contact {
            name: name.clone(),
            address,
        };
        self.contacts.insert(name, contact);
    }

    pub fn get_contact(&self, name: &str) -> Option<&Contact> {
        self.contacts.get(name)
    }

    pub fn list_contacts(&self) -> Vec<&Contact> {
        self.contacts.values().collect()
    }
}


pub async fn handle_transfer_to_contact(
    name: &str,
    amount: &str,
    wallet: &Wallet,
    contacts_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!(
        "Attempting transfer to contact '{}' for amount {}",
        name,
        amount
    );
    let book = ContactsBook::load(contacts_file);
    match book.get_contact(name) {
        Some(contact) => {
            log::info!("Resolved contact '{}' to address {}", name, contact.address);
            match handle_transfer(&contact.address, amount, wallet).await {
                Ok(_) => {
                    log::info!(
                        "Transfer to contact '{}' ({}) succeeded.",
                        name,
                        contact.address
                    );
                    println!(
                        "Transfer to contact '{}' ({}) succeeded.",
                        name, contact.address
                    );
                    Ok(())
                }
                Err(e) => {
                    log::error!(
                        "Transfer to contact '{}' ({}) failed: {}",
                        name,
                        contact.address,
                        e
                    );
                    println!(
                        "Transfer to contact '{}' ({}) failed: {}",
                        name, contact.address, e
                    );
                    if e.to_string().contains("nonce too low") {
                        println!(
                            "Hint: The transaction nonce is too low. You may have pending transactions or need to increment the nonce."
                        );
                    }
                    Err(e)
                }
            }
        }
        None => {
            log::error!("Contact '{}' not found.", name);
            println!("Contact '{}' not found.", name);
            Err("Contact not found".into())
        }
    }
}