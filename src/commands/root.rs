use crate::commands::contacts::ContactsCommand;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Commands {
    /// Manage contacts
    Contacts(ContactsCommand),
}
