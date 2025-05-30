# Rootstock Wallet CLI

A lightweight Rust-based CLI application for managing Rootstock wallets, transferring RBTC, estimating gas, and interacting with the Rootstock blockchain. Designed for developers building on the Rootstock ecosystem.

## Features

- **Wallet Management**:
  - Generate new wallets.
  - Load and save wallets securely.
  - Import wallets from mnemonic phrases or private keys.
  - Export private keys.
  - Export wallets in JSON keystore format.
- **RBTC Transfers**:
  - Send RBTC to any Rootstock address.
  - Locally sign transactions for enhanced security.
- **Balance Retrieval**:
  - Fetch and display the RBTC balance of any address.
- **Gas Estimation**:
  - Estimate gas costs for transactions.
- **Network Information**:
  - Retrieve current block number, gas price, and chain ID.
- **QR Code Generation**:
  - Generate QR codes for wallet addresses.
- **Multi-Signature Wallet Support**:
  - Create multi-signature wallets with specified owners and required approvals.
  - Propose transactions requiring multiple approvals.
  - Approve proposed transactions.

## Installation

Clone the repository and build the project using Cargo:

[//]: # (add screenshot of clone command)
![Installation](screenshots/clone.png)


```bash
cd rootstock-wallet
cargo build --release
```

## Usage

Run the CLI application:

```bash
./target/release/rootstock-wallet
```

![All Commands](screenshots/run-app.png)



### Commands

1. **Transfer RBTC**:
   Send RBTC to a recipient address.

   ```bash
   ./target/release/rootstock-wallet transfer --recipient <ADDRESS> --amount <AMOUNT>
   ```

   Example:

   ```bash
   ./target/release/rootstock-wallet transfer --recipient 0x09aB514B6974601967E7b379478EFf4073cceD06 --amount 0.0001
   ```

2. **Get Balance**:
   Fetch the RBTC balance of an address.

   ```bash
   ./target/release/rootstock-wallet get-balance --address <ADDRESS>
   ```

   Example:

![Get Balance](screenshots/get-balance.png)

3. **Show Wallet**:
   Display the wallet's address and public key.

![Show Wallet](screenshots/show-wallet.png)

4. **Generate QR Code**:
   Generate a QR code for the wallet address.

   ```bash
   ./target/release/rootstock-wallet generate-qr --output <FILENAME>
   ```

   Example:

   ![Installation](screenshots/generate-qr.png)

    This generates a QR Code image in the root of the project.
    
    ![Qr Code](wallet.png)

5. **Estimate Gas**:
   Estimate gas costs for a transaction.

   ```bash
   ./target/release/rootstock-wallet estimate-gas  <ADDRESS>  <AMOUNT>
   ```
   Example:

    ![Estimate Gas](screenshots/estimate-gas.png)



6. **Export Private Key**:
   Export the wallet's private key to a file.

   ```bash
   ./target/release/rootstock-wallet export-private-key --output <FILENAME>
   ```

   Example:

   ```bash
   ./target/release/rootstock-wallet export-private-key --output private.key
   ```
    ![Exported Key](screenshots/exported.png)

    This will generate a file with the private key.

    ![Exported Privte Key](screenshots/exported-private-key.png)

7. **Import Wallet**:
   Import a wallet from a mnemonic phrase or private key.

   ```bash
   ./target/release/rootstock-wallet import-wallet --mnemonic <MNEMONIC> --private-key <PRIVATE_KEY>
   ```

   Example (using a mnemonic):

   ```bash
   ./target/release/rootstock-wallet import-wallet --mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
   ```

   Example (using a private key):

   ```bash
   ./target/release/rootstock-wallet import-wallet --private-key 6cf034d3239788ab09169dac88edc3466c5b16b30c49674505190fa4c6d872fb
   ```

8. **Export Keystore**:
   Export the wallet in JSON keystore format.

   ```bash
   ./target/release/rootstock-wallet export-keystore --password <PASSWORD> --output <FILENAME>
   ```

   Example:

   ```bash
   ./target/release/rootstock-wallet export-keystore --password mypassword --output keystore.json
   ```

<!-- 9. **Create Multi-Signature Wallet**:
   Deploy a multi-signature wallet contract with specified owners and required approvals.

   ```bash
   ./target/release/rootstock-wallet create-multisig --owners <OWNER1> <OWNER2> ... --required <NUMBER>
   ```

   Example:

   ```bash
   ./target/release/rootstock-wallet create-multisig --owners 0xOwner1 0xOwner2 0xOwner3 --required 2
   ```

10. **Propose Transaction**:
    Propose a transaction requiring multiple approvals.

    ```bash
    ./target/release/rootstock-wallet propose-transaction --multisig <MULTISIG_ADDRESS> --to <RECIPIENT> --value <AMOUNT> --data <DATA>
    ```

    Example:

    ```bash
    ./target/release/rootstock-wallet propose-transaction --multisig 0xMultisigAddress --to 0xRecipientAddress --value 0.1 --data 0x
    ``` -->

<!-- 11. **Approve Transaction**:
    Approve a proposed transaction.

    ```bash
    ./target/release/rootstock-wallet approve-transaction --multisig <MULTISIG_ADDRESS> --tx-id <TRANSACTION_ID>
    ```

    Example:

    ```bash
    ./target/release/rootstock-wallet approve-transaction --multisig 0xMultisigAddress --tx-id 1
    ``` -->

9. **Network Info**:
    Display current network information (block number, gas price, chain ID).

    ```bash
    ./target/release/rootstock-wallet network-info
    ```
   ![Exported Key](screenshots/network-info.png)





10. **Transfer ERC-20 Tokens**:
    Transfer tokens like RIF, DOC, SOV, or rUSDT.

    ```bash
    ./target/release/rootstock-wallet transfer-token --token-address <TOKEN_ADDRESS> --recipient <RECIPIENT_ADDRESS> --amount <AMOUNT>
    ```

    Example:

    ```bash
    ./target/release/rootstock-wallet transfer-token --token-address 0xTokenContractAddress --recipient 0xRecipientAddress --amount 10
    ```

## Configuration

The application connects to the Rootstock network using the public JSON-RPC endpoint:

- **Mainnet**: `https://public-node.rsk.co`
- **Testnet**: `https://public-node.testnet.rsk.co`

Additionally, the `.env` file includes the following configurations:

- `CHAIN_ID="mainnet-or-testnet-id"`: Specifies the chain ID for the network (e.g., Mainnet or Testnet).
- `API_KEY="your-api-key-here"`: Your Rootstock API key for accessing the JSON-RPC endpoint.

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