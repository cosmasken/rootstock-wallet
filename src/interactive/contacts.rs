use crate::commands::transfer::TransferCommand;
use crate::types::transaction::RskTransaction;
use crate::{
    commands::contacts::{ContactsAction, ContactsCommand},
    utils::table::TableBuilder,
};
use anyhow::{Context, Result};
use console::style;
use ethers::types::{U64, U256};
use ethers::utils::{format_ether, format_units};
use inquire::{Confirm, Select, Text, validator::Validation};
/// Interacive contacts manage
pub async fn manage_contacts() -> Result<()> {
    loop {
        println!("\n{}", style("📇 Contact Management").bold());
        println!("{}", "=".repeat(30));

        let options = vec![
            "👥 List all contacts",
            "➕ Add new contact",
            "✏️  Update contact",
            "❌ Remove contact",
            "🔍 Search contacts",
            "💸 Quick send to contact",
            "📜 View contact transactions",
            "🏠 Back to main menu",
        ];

        let selection = inquire::Select::new("What would you like to do?", options).prompt()?;

        match selection {
            "👥 List all contacts" => list_contacts().await?,
            "➕ Add new contact" => add_contact().await?,
            "✏️  Update contact" => update_contact().await?,
            "❌ Remove contact" => remove_contact().await?,
            "🔍 Search contacts" => search_contacts().await?,
            "💸 Quick send to contact" => quick_send_to_contact().await?,
            "📜 View contact transactions" => view_contact_transactions().await?,
            "🏠 Back to main menu" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// List all contacts in a table
pub async fn list_contacts() -> Result<()> {
    let mut contacts = ContactsCommand {
        action: ContactsAction::List,
    }
    .load_contacts()?;

    // Sort contacts by most recently interacted with
    contacts.sort_by(|a, b| {
        let a_time = a
            .last_transaction_time()
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(0);
        let b_time = b
            .last_transaction_time()
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(0);
        b_time.cmp(&a_time)
    });

    if contacts.is_empty() {
        println!("No contacts found.");
        return Ok(());
    }

    let mut table = TableBuilder::new();
    table.add_header(&["Name", "Address", "Transactions", "Last Tx"]);

    for contact in contacts {
        let tx_info = if contact.has_transaction_history() {
            format!(
                "{} txs\n{} RBTC",
                contact.get_total_transactions(),
                // Format balance in RBTC (18 decimals)
                ethers::utils::format_units(contact.get_total_volume(), 18)
                    .unwrap_or_else(|_| "N/A".to_string())
            )
        } else {
            "No txs".to_string()
        };

        let last_tx = contact
            .last_transaction_time()
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "Never".to_string());

        table.add_row(&[
            &contact.name,
            &format!("0x{:x}", contact.address),
            &tx_info,
            &last_tx,
        ]);
    }

    table.print();
    Ok(())
}

/// Add a new contact interactively
pub async fn add_contact() -> Result<()> {
    println!("\n{}", style("➕ Add New Contact").bold());

    let name = Text::new("Contact name:")
        .with_help_message("Enter a name for this contact")
        .prompt()?;

    let address = Text::new("Ethereum address (0x...):")
        .with_help_message("Enter the contact's Ethereum address")
        .with_validator(|input: &str| {
            if input.starts_with("0x") && input.len() == 42 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    "Please enter a valid Ethereum address (0x...)".into(),
                ))
            }
        })
        .prompt()?;

    let notes = Text::new("Notes (optional):")
        .with_help_message("Add any notes about this contact")
        .prompt_skippable()?
        .filter(|s| !s.trim().is_empty());

    let tags = Text::new("Tags (comma-separated, optional):")
        .with_help_message("e.g., friend,team,client")
        .prompt_skippable()?
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let cmd = ContactsCommand {
        action: ContactsAction::Add {
            name,
            address,
            notes,
            tags,
        },
    };

    cmd.execute().await?;
    println!("✅ Contact added successfully!");
    Ok(())
}

/// Update an existing contact
pub async fn update_contact() -> Result<()> {
    let contacts = ContactsCommand {
        action: ContactsAction::List,
    }
    .load_contacts()?;

    if contacts.is_empty() {
        println!("No contacts found to update.");
        return Ok(());
    }

    let contact_names: Vec<String> = contacts
        .iter()
        .map(|c| format!("{} ({})", c.name, c.address))
        .collect();

    let selection = inquire::Select::new("Select contact to update:", contact_names).prompt()?;

    let contact_name = selection.split('(').next().unwrap_or("").trim();

    let new_name = Text::new("New name (press Enter to keep current):")
        .with_help_message("Enter new name or press Enter to skip")
        .prompt_skippable()?;

    let new_address = Text::new("New address (press Enter to keep current):")
        .with_help_message("Enter new address or press Enter to skip")
        .prompt_skippable()?;

    let new_notes = Text::new("New notes (press Enter to keep current):")
        .with_help_message("Enter new notes or press Enter to skip")
        .prompt_skippable()?;

    let new_tags = Text::new("New tags (comma-separated, press Enter to keep current):")
        .with_help_message("e.g., friend,team,client")
        .prompt_skippable()?;

    let cmd = ContactsCommand {
        action: ContactsAction::Update {
            identifier: contact_name.to_string(),
            name: new_name.filter(|s| !s.trim().is_empty()),
            address: new_address.filter(|s| !s.trim().is_empty()),
            notes: new_notes.filter(|s| !s.trim().is_empty()),
            tags: new_tags.map(|s| {
                s.split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect()
            }),
        },
    };

    cmd.execute().await?;
    println!("✅ Contact updated successfully!");
    Ok(())
}

/// Remove a contact
pub async fn remove_contact() -> Result<()> {
    let contacts = ContactsCommand {
        action: ContactsAction::List,
    }
    .load_contacts()?;

    if contacts.is_empty() {
        println!("No contacts found to remove.");
        return Ok(());
    }

    let contact_names: Vec<String> = contacts
        .iter()
        .map(|c| format!("{} ({})", c.name, c.address))
        .collect();

    let selection = inquire::Select::new("Select contact to remove:", contact_names).prompt()?;

    let contact_name = selection.split('(').next().unwrap_or("").trim();

    if Confirm::new(&format!(
        "Are you sure you want to remove '{}'?",
        contact_name
    ))
    .with_default(false)
    .prompt()?
    {
        let cmd = ContactsCommand {
            action: ContactsAction::Remove {
                identifier: contact_name.to_string(),
            },
        };

        cmd.execute().await?;
        println!("✅ Contact removed successfully!");
    } else {
        println!("Operation cancelled.");
    }

    Ok(())
}

/// Search contacts by name or address
pub async fn search_contacts() -> Result<()> {
    let query = Text::new("Search contacts (name or address):")
        .with_help_message("Enter search term")
        .prompt()?;

    let cmd = ContactsCommand {
        action: ContactsAction::Search {
            query: query.clone(),
        },
    };

    // First try to use the search command's execute
    if let Err(_e) = cmd.execute().await {
        // If execute fails (not implemented), fall back to manual search
        println!("Search not implemented, falling back to local search...");

        let contacts = cmd.load_contacts()?;
        let filtered: Vec<_> = contacts
            .into_iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&query.to_lowercase())
                    || c.address
                        .to_string()
                        .to_lowercase()
                        .contains(&query.to_lowercase())
            })
            .collect();

        if filtered.is_empty() {
            println!("No contacts found matching '{}'", query);
            return Ok(());
        }

        println!("\nFound {} contacts:", filtered.len());
        for contact in filtered {
            println!("• {} - {}", contact.name, contact.address);
        }
    }

    Ok(())
}

/// Quick send to a contact
pub async fn quick_send_to_contact() -> Result<()> {
    let cmd = ContactsCommand {
        action: ContactsAction::List,
    };
    let contacts = cmd.load_contacts()?;

    if contacts.is_empty() {
        println!("No contacts available. Please add a contact first.");
        return Ok(());
    }

    let contact_names: Vec<String> = contacts
        .iter()
        .map(|c| {
            format!(
                "{} (0x{:x}) - {} txs",
                c.name,
                c.address,
                c.get_total_transactions()
            )
        })
        .collect();

    let selection = Select::new("Select contact to send to:", contact_names)
        .prompt()
        .context("Failed to select contact")?;

    // Extract the address from the selection
    let selected_contact = contacts
        .iter()
        .find(|c| selection.starts_with(&c.name))
        .context("Selected contact not found")?;

    // Get amount to send
    let amount = Text::new("Amount to send (in RBTC):")
        .with_validator(|input: &str| {
            if input.parse::<f64>().is_ok() {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Please enter a valid number".into()))
            }
        })
        .prompt()
        .context("Failed to get amount")?;

    // Confirm the transaction
    let confirm = Confirm::new(&format!(
        "Send {} RBTC to {} (0x{:x})?",
        amount, selected_contact.name, selected_contact.address
    ))
    .with_default(false)
    .prompt()
    .context("Failed to get confirmation")?;

    if confirm {
        println!("Sending {} RBTC to {}...", amount, selected_contact.name);

        // Create and execute the transfer command
        let transfer_cmd = TransferCommand {
            address: format!("0x{:x}", selected_contact.address),
            value: amount.parse().unwrap_or(0.0),
            token: None, // Only RBTC for now
            network: "mainnet".to_string(),
        };

        match transfer_cmd.execute().await {
            Ok(transfer_result) => {
                println!(
                    "\n{} {}",
                    style("✅ Transaction sent successfully!").green(),
                    style(format!("(0x{:x})", transfer_result.tx_hash)).dim()
                );

                // Update contact's transaction history
                let mut contacts = cmd.load_contacts()?;
                if let Some(contact) = contacts
                    .iter_mut()
                    .find(|c| c.address == selected_contact.address)
                {
                    let tx = RskTransaction {
                        hash: transfer_result.tx_hash,
                        from: transfer_result.from,
                        to: Some(transfer_result.to),
                        value: transfer_result.value,
                        gas_price: transfer_result.gas_price,
                        gas: transfer_result.gas_used,
                        nonce: U256::zero(), // Not available in the receipt
                        input: None,
                        block_number: None, // Would need to be fetched separately
                        transaction_index: None, // Would need to be fetched separately
                        timestamp: std::time::SystemTime::now(),
                        status: if transfer_result.status == U64::from(1) {
                            crate::types::transaction::TransactionStatus::Success
                        } else {
                            crate::types::transaction::TransactionStatus::Failed
                        },
                        token_address: transfer_result.token_address,
                        confirms: Some(U64::from(1)), // Just confirmed
                        cumulative_gas_used: Some(transfer_result.gas_used),
                        logs: None, // Would need to be fetched separately
                    };

                    contact.update_transaction_stats(&tx, false);

                    // Save the updated contacts
                    cmd.save_contacts(&contacts)?;

                    // Show transaction details
                    println!("\n{}", style("Transaction Details:").bold());
                    println!("  • Hash: 0x{:x}", tx.hash);
                    println!("  • From: 0x{:x}", tx.from);
                    if let Some(to) = tx.to {
                        println!("  • To:   0x{:x}", to);
                    }
                    println!("  • Value: {} RBTC", format_ether(tx.value));
                    println!("  • Gas Used: {}", tx.gas);
                    println!(
                        "  • Gas Price: {} Gwei",
                        format_units(tx.gas_price, 9).unwrap_or_else(|_| "N/A".into())
                    );
                    println!("  • Status: {:?}", tx.status);
                }
            }
            Err(e) => {
                eprintln!(
                    "\n{}",
                    style(format!("❌ Error sending transaction: {}", e)).red()
                );
            }
        }
    } else {
        println!("\n{}", style("❌ Transaction cancelled").yellow());
    }

    Ok(())
}

/// View transaction history for a contact
pub async fn view_contact_transactions() -> Result<()> {
    let cmd = ContactsCommand {
        action: ContactsAction::List,
    };
    let contacts = cmd.load_contacts()?;

    if contacts.is_empty() {
        println!("No contacts available.");
        return Ok(());
    }

    let contact_names: Vec<String> = contacts
        .iter()
        .map(|c| {
            format!(
                "{} (0x{:x}) - {} txs",
                c.name,
                c.address,
                c.get_total_transactions()
            )
        })
        .collect();

    let selection = Select::new("Select contact to view transactions:", contact_names)
        .prompt()
        .context("Failed to select contact")?;

    let selected_contact = contacts
        .iter()
        .find(|c| selection.starts_with(&c.name))
        .context("Selected contact not found")?;

    // Load transactions (you'll need to implement this part)
    let all_transactions = Vec::new(); // Replace with actual transaction loading

    let contact_txs = selected_contact.get_recent_transactions(&all_transactions, None);

    if contact_txs.is_empty() {
        println!("No transactions found for this contact.");
        return Ok(());
    }

    let mut table = TableBuilder::new();
    table.add_header(&["Date", "Type", "Amount", "Status"]);

    for tx in contact_txs {
        let tx_type = if tx.from == selected_contact.address {
            "OUT"
        } else {
            "IN"
        };

        let amount =
            ethers::utils::format_units(tx.value, 18).unwrap_or_else(|_| "N/A".to_string());

        let date_str = tx
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| {
                chrono::DateTime::<chrono::Local>::from(std::time::UNIX_EPOCH + d)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_else(|_| "Unknown".to_string());

        let tx_type_str = tx_type.to_string();
        let amount_str = format!("{} RBTC", amount);
        let status_str = format!("{:?}", tx.status);

        table.add_row(&[&date_str, &tx_type_str, &amount_str, &status_str]);
    }

    println!(
        "\nTransaction history for {} (0x{:x}):",
        selected_contact.name, selected_contact.address
    );
    println!(
        "Total Volume: {} RBTC\n",
        ethers::utils::format_units(selected_contact.get_total_volume(), 18)
            .unwrap_or_else(|_| "N/A".to_string())
    );

    table.print();

    Ok(())
}
