// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::fs;
// use std::path::Pa

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Contact {
//     pub name: String,
//     pub address: String,
// }

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct ContactsBook {
//     pub contacts: HashMap<String, Contact>,
// }

// impl ContactsBook {
//     pub fn load(path: &str) -> Self {
//         if Path::new(path).exists() {
//             let data = fs::read_to_string(path).unwrap_or_default();
//             serde_json::from_str(&data).unwrap_or_default()
//         } else {
//             ContactsBook::default()
//         }
//     }

//     pub fn save(&self, path: &str) {
//         let data = serde_json::to_string_pretty(self).unwrap();
//         fs::write(path, data).unwrap();
//     }

//     pub fn add_contact(&mut self, name: String, address: String) {
//         let contact = Contact {
//             name: name.clone(),
//             address,
//         };
//         self.contacts.insert(name, contact);
//     }

//     pub fn get_contact(&self, name: &str) -> Option<&Contact> {
//         self.contacts.get(name)
//     }

//     pub fn list_contacts(&self) -> Vec<&Contact> {
//         self.contacts.values().collect()
//     }
// }

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

    pub fn delete_contact(&mut self, name: &str) -> Option<Contact> {
        self.contacts.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_contacts_book() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        let mut book = ContactsBook::load(path);
        assert_eq!(book.list_contacts().len(), 0);

        book.add_contact("Alice".to_string(), "0x1234".to_string());
        book.save(path);

        let loaded_book = ContactsBook::load(path);
        assert_eq!(loaded_book.list_contacts().len(), 1);
        assert_eq!(loaded_book.get_contact("Alice").unwrap().address, "0x1234");

        book.delete_contact("Alice");
        assert_eq!(book.list_contacts().len(), 0);
    }
}
