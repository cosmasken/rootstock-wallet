# Rootstock Wallet CLI

> A secure, feature-rich Rust-based CLI wallet for the Rootstock blockchain

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cosmasken/rootstock-wallet/blob/main/LICENSE)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange)](https://www.rust-lang.org/)

## 🚀 Features

### Wallet Management

- Create new wallets with custom names
- Import existing wallets using private keys
- Switch between multiple wallets
- Backup and restore wallet data
- List all available wallets with balances
- Rename or delete wallets

### Asset Management

- Check RBTC balance for any address
- View token balances (supports all ERC-20 tokens)
- Add custom tokens to your wallet
- Remove unwanted tokens from your token list
- View detailed token information

### Transactions

- Send RBTC to any address
- Transfer ERC-20 tokens
- View transaction history with filtering options
- Check transaction status
- Bulk transfer functionality

### Contact Management

- Save frequently used addresses as contacts
- Add notes and tags to contacts
- Search through your contacts
- Update or remove existing contacts
- Import/export contact lists

### Network & Configuration

- Switch between Mainnet and Testnet
- Configure custom RPC endpoints
- Manage API keys for services
- View network status and connection details

## 🛠️ Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Git](https://git-scm.com/)
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)

### Building from Source

1. Clone the repository:

   ```bash
   git clone https://github.com/cosmasken/rootstock-wallet.git
   cd rootstock-wallet
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

3. Run the application:

   ```bash
   # For development
   cargo run
   
   # For production
   ./target/release/rootstock-wallet
   ```


## ⚙️ Configuration

The wallet automatically handles configuration through an interactive setup wizard when you first run it. You can also manage your configuration at any time using the interactive menu:

## 🎮 Usage

### Interactive Mode 

The wallet features an intuitive interactive interface. Simply run:

```bash
cargo run
```

## Demo
[![asciicast](https://asciinema.org/a/Xoe9AIZi400i0us2PLXGs3lYx.svg)](https://asciinema.org/a/Xoe9AIZi400i0us2PLXGs3lYx)
## 🔍 Troubleshooting

### Common Issues

#### Transaction Failures

- Ensure you have enough RBTC for gas fees
- Verify recipient address format (0x...)
- Check network compatibility (mainnet/testnet)

#### Connection Issues

- Verify your internet connection

#### Wallet Issues

- Confirm wallet file exists at `~/.local/share/rootstock-wallet/`
- Check file permissions if access is denied
- Ensure you're using the correct network (mainnet/testnet)

## 🤝 Contributing

We welcome contributions from the community! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please read our [Contributing Guidelines](CONTRIBUTING.md) for more details on our development process and code of conduct.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built as part of the Rootstock Hacktivator program
- Special thanks to the Rootstock team for their support
- Inspired by the vibrant Rootstock community

## 📚 Resources

- [Rootstock Documentation](https://developers.rsk.co/)
- [RSK Smart Contracts](https://developers.rsk.co/develop/)
- [RSK Developer Portal](https://developers.rsk.co/)
- [GitHub Repository](https://github.com/cosmasken/rootstock-wallet)
- [Issue Tracker](https://github.com/cosmasken/rootstock-wallet/issues)

---

Made with ❤️ for the Rootstock ecosystem