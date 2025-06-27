use crate::{
    config::ConfigManager,
    types::network::Network,
};
use anyhow::{Result,anyhow};
use console::style;
use dialoguer::Confirm;
use ethers::types::U256;

/// Helper function to convert wei to RBTC
fn convert_wei_to_rbtc(wei: U256) -> f64 {
    // 1 RBTC = 10^18 wei
    let wei_f64 = wei.as_u128() as f64;
    wei_f64 / 1_000_000_000_000_000_000.0
}

/// Displays transaction details and asks for confirmation
pub async fn show_transaction_preview(
    to: &str,
    amount: &str,
    network: Network,
) -> Result<bool> {
    println!("\n{}", style("Transaction Preview").bold().underlined());
    println!("• To: {}", style(to).cyan());
    
    // Parse amount
    let amount_wei = U256::from_dec_str(amount).map_err(|e| {
        anyhow::anyhow!("Invalid amount format: {}", e)
    })?;
    
    // Convert to RBTC for display
    let amount_rbtc = convert_wei_to_rbtc(amount_wei);
    println!("• Amount: {} RBTC ({} wei)", 
        style(amount_rbtc).green(), 
        style(amount_wei).dim()
    );
    
    // Get current gas price and estimate gas
    let _config = ConfigManager::new()?.load()?;
    
    // For the preview, we'll use a default gas price and gas limit
    // In a real implementation, you might want to fetch these from the network
    let gas_price = U256::from(20_000_000_000u64); // 20 Gwei as default
    let estimated_gas = U256::from(21_000); // Default gas limit for simple transfers
    
    // Note: In a real implementation, you would uncomment the following:
    // let provider = config.get_provider()?;
    // let gas_price = provider.get_gas_price().await?;
    let gas_cost = gas_price.checked_mul(estimated_gas).unwrap_or_default();
    let gas_cost_rbtc = convert_wei_to_rbtc(gas_cost);
    
    println!("• Network: {}", style(network).cyan());
    println!("• Gas Price: {} Gwei", style(convert_wei_to_gwei(gas_price)).yellow());
    println!("• Estimated Gas: {}", style(estimated_gas).yellow());
    println!("• Estimated Fee: {} RBTC", style(gas_cost_rbtc).red());
    
    let total_amount = amount_wei.checked_add(gas_cost).unwrap_or(amount_wei);
    let total_rbtc = convert_wei_to_rbtc(total_amount);
    println!("• Total (Amount + Fee): {} RBTC", style(total_rbtc).green().bold());
    
    // Ask for confirmation
    let confirm = Confirm::new()
        .with_prompt("\nDo you want to send this transaction?")
        .default(false)
        .interact()?;
    
    Ok(confirm)
}

/// Helper function to convert wei to Gwei
fn convert_wei_to_gwei(wei: U256) -> f64 {
    let gwei = wei.as_u128() as f64 / 1_000_000_000.0;
    (gwei * 100.0).round() / 100.0 // Round to 2 decimal places
}
