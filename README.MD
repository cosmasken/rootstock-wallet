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
  - Update Contact names and/or address
  - List all saved contacts
  - Remove contacts
  - Quick send to Contacts

- **Network**:
  - Automatic network detection from RPC URL
  - Support for both mainnet and testnet

## Installation

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/cosmasken/rootstock-wallet
cd rootstock-wallet
cargo build
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

## Configuration

The application connects to the Rootstock network using the public JSON-RPC endpoint:

- **Mainnet**: `https://public-node.rsk.co`
- **Testnet**: `https://public-node.testnet.rsk.co`


## Contribution

Contributions are welcome! Please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Submit a pull request with a detailed description of your changes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project was developed as part of the Rootstock Hacktivator program to contribute to the Rootstock ecosystem. Special thanks to the Rootstock team for their support and resources.