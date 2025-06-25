# Rootstock Wallet CLI

A lightweight Rust-based CLI application for managing Rootstock wallets, transferring RBTC and tokens, and interacting with the Rootstock blockchain. Designed for developers building on the Rootstock ecosystem.

## Features

- **Wallet Management**:
  - Generate new wallets
  - Import/export wallets
  - Manage multiple wallets
  - Switch between wallets

- **Transactions**:
  - Send RBTC to any address
  - Transfer ERC-20 tokens
  - View transaction history
  - Check balances (RBTC and tokens)

- **Token Management**:
  - Add custom tokens
  - List supported tokens
  - Remove tokens
  - View token balances

- **Contacts**:
  - Save frequently used addresses with names
  - List all saved contacts
  - Remove contacts

- **Network**:
  - Automatic network detection from RPC URL
  - Support for both mainnet and testnet
  - Set custom RPC API key

## Installation

Clone the repository and build the project using Cargo:

```bash
cd rootstock-wallet
cargo build --release
```

## Usage

### Interactive Mode (Default)

By default, the wallet runs in interactive mode with a text-based interface. Simply run:

```bash
cargo run
```

You'll see a menu like this:
```
Rootstock Wallet CLI

1. Wallet Management
2. Transfer RBTC/Tokens
3. Check Balance
4. View Transaction History
5. Manage Tokens
6. Manage Contacts
7. Network Settings
8. Exit

Select an option:
```

### Non-Interactive Mode

For scripting or automation, you can use the following commands directly:

#### Wallet Management

```bash
# Create a new wallet
cargo run -- wallet new

# List all wallets
cargo run -- wallet list

# Switch to a different wallet
cargo run -- wallet switch <WALLET_ADDRESS>

# Remove a wallet
cargo run -- wallet remove <WALLET_ADDRESS>
```

#### Balance and Transactions

```bash
# Check RBTC balance
cargo run -- balance --address <ADDRESS>

# Transfer RBTC
cargo run -- transfer --recipient <ADDRESS> --amount <AMOUNT>

# Transfer tokens (RIF, USDRIF, etc.)
cargo run -- transfer --recipient <ADDRESS> --amount <AMOUNT> --token <TOKEN_SYMBOL>

# View transaction history
cargo run -- history --address <ADDRESS> --limit <NUMBER>
```

#### Token Management

```bash
# Add a new token
cargo run -- token-add --symbol <SYMBOL> --address <ADDRESS> --decimals <DECIMALS>

# List all tokens
cargo run -- token-list

# Remove a token
cargo run -- token-remove --symbol <SYMBOL>
```

#### Contact Management

```bash
# Add a contact
cargo run -- contacts add --name <NAME> --address <ADDRESS>

# List all contacts
cargo run -- contacts list

# Remove a contact
cargo run -- contacts remove --name <NAME>
```

#### Network Configuration

```bash
# Set RPC API key
cargo run -- set-api-key <API_KEY>

# Force non-interactive mode (useful for scripts)
cargo run -- --no-interactive <command> [args...]
```

### Examples

1. **Interactive Mode** (default):
   ```bash
   cargo run
   # Then follow the on-screen menus
   ```

2. **Quick Balance Check** (non-interactive):
   ```bash
   cargo run -- balance --address 0x1234...
   ```

3. **Send RBTC** (non-interactive):
   ```bash
   cargo run -- transfer --recipient 0x1234... --amount 0.1
   ```

4. **Send Tokens** (non-interactive):
   ```bash
   cargo run -- transfer --recipient 0x1234... --amount 10 --token RIF
   ```

## Examples

1. **Create a new wallet**:
   ```bash
   cargo run -- wallet new
   ```

2. **Check RBTC balance**:
   ```bash
   cargo run -- balance --address 0x1234...
   ```

3. **Send 0.1 RBTC**:
   ```bash
   cargo run -- transfer --recipient 0x1234... --amount 0.1
   ```

4. **Send 10 RIF tokens**:
   ```bash
   cargo run -- transfer --recipient 0x1234... --amount 10 --token RIF
   ```

5. **View last 5 transactions**:
   ```bash
   cargo run -- history --address 0x1234... --limit 5
   ```

## Troubleshooting

### Common Issues

1. **Transaction Failures**:
   - Ensure you have enough RBTC for gas fees
   - Check that the recipient address is valid
   - Verify you're on the correct network (mainnet/testnet)

2. **Connection Issues**:
   - Check your internet connection
   - Verify your RPC API key is set correctly
   - Ensure the RPC endpoint is accessible

3. **Wallet Not Found**:
   - Make sure you've created a wallet first
   - Check that you're using the correct wallet address

   - **Show Contact Details**:

   ```bash
   cargo run show-contact --name <NAME>
   ```

   Example:

   ![Show Contact](screenshots/show-contact.png)

   - **Transfer to a Contact**:
   You can use a contact's name instead of an address in transfer commands:

   ```bash
   cargo run transfer --recipient <CONTACT_NAME> --amount <AMOUNT>
   ```

   Example:

   ```bash
   cargo run transfer --recipient alice --amount 0.01
   ```

## Adding Tokens

Add new tokens to the registry by adding them to the `tokens.json` file in the `src` directory.

```bash
   cargo run add-token --symbol ARCAD --address 0x0c917Ccc38203734aC963152059439cd33EA57fF --decimals 18 --network testnet
   ```

## Configuration

The application connects to the Rootstock network using the public JSON-RPC endpoint:

- **Mainnet**: `https://public-node.rsk.co`
- **Testnet**: `https://public-node.testnet.rsk.co`



Ensure these variables are properly set in your environment file before running the application.

You can modify the provider URL in the env files file if needed.

## Contribution

Contributions are welcome! Please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Submit a pull request with a detailed description of your changes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project was developed as part of the Rootstock Hacktivator program to contribute to the Rootstock ecosystem. Special thanks to the Rootstock team for their support and resources.