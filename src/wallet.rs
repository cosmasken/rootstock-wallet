use crate::transactions::{TransactionService, TransferError};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::signers::{Signer, Wallet as EthersWallet};
use ethers_providers::Http;
use ethers_providers::Provider;
use rand::{TryRngCore, rngs::OsRng};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::fs;
use std::str::FromStr;
use zeroize::Zeroize;
use zeroize::Zeroizing;
use bip32::DerivationPath;

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
}

impl Wallet {
    pub fn generate() -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let mut secret_key_bytes = [0u8; 32];
        let _ = rng.try_fill_bytes(&mut secret_key_bytes);
        let secret_key =
            SecretKey::from_slice(&secret_key_bytes).expect("Failed to create secret key");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let private_key_hex = hex::encode(secret_key.secret_bytes());
        let public_key_bytes = public_key.serialize_uncompressed();
        let public_key_hex = hex::encode(public_key_bytes);

        let mut hasher = Keccak256::new();
        hasher.update(&public_key_bytes[1..]);
        let address = &hasher.finalize()[12..]; // Take the last 20 bytes
        let address_hex = format!("0x{}", hex::encode(address));

        Wallet {
            private_key: private_key_hex,
            public_key: public_key_hex,
            address: address_hex,
        }
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let data = serde_json::to_string_pretty(&self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Option<Self> {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(wallet) = serde_json::from_str(&data) {
                return Some(wallet);
            }
        }
        None
    }

    pub fn transaction_service(
        &self,
        provider: Provider<Http>,
    ) -> Result<TransactionService, TransferError> {
        TransactionService::new(provider, Zeroizing::new(self.private_key.clone()))
    }

    /// Signs a transaction using the wallet's private key
    pub async fn sign_transaction(
        &self,
        tx: &TypedTransaction,
    ) -> Result<ethers::types::Bytes, Box<dyn std::error::Error>> {
        // Parse the private key into an ethers Wallet
        let ethers_wallet: EthersWallet<k256::ecdsa::SigningKey> = self.private_key.parse()?;

        // Sign the transaction
        let signature = ethers_wallet.sign_transaction(tx).await?;

        // Serialize the signed transaction
        let signed_tx = tx.rlp_signed(&signature);
        Ok(signed_tx)
    }

    pub fn from_mnemonic(mnemonic: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse the mnemonic phrase into a BIP-39 mnemonic
        let mnemonic = bip39::Mnemonic::parse(mnemonic)?;
    
        // Generate the seed from the mnemonic
        let seed = mnemonic.to_seed("");
    
        // Define the HD path for Rootstock (m/44'/137'/0'/0/0)
        let hd_path = hdpath::StandardHDPath::new(hdpath::Purpose::Pubkey, 137, 0, 0, 0);
    
        // Convert the HD path to a string and parse it into a derivation path
        let derivation_path = DerivationPath::from_str(&hd_path.to_string())?;
    
        // Derive the extended private key (XPrv) from the seed and derivation path
        let xprv = bip32::XPrv::derive_from_path(&seed, &derivation_path)?;
    
        // Extract the raw private key bytes from the XPrv
        let secret_key = secp256k1::SecretKey::from_slice(&xprv.private_key().to_bytes())
            .map_err(|_| "Failed to convert private key")?;
    
        // Create a Wallet instance from the derived secret key
        Ok(Self::from_secret_key(secret_key))
    }

    pub fn from_private_key(private_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let secret_key = secp256k1::SecretKey::from_str(private_key)?;
        Ok(Self::from_secret_key(secret_key))
    }

    fn from_secret_key(secret_key: secp256k1::SecretKey) -> Self {
        let secp = secp256k1::Secp256k1::new();
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
        let private_key_hex = hex::encode(secret_key.secret_bytes());
        let public_key_hex = hex::encode(public_key.serialize_uncompressed());
        let address = Wallet::derive_address(&public_key);

        Wallet {
            private_key: private_key_hex,
            public_key: public_key_hex,
            address,
        }
    }

    fn derive_address(public_key: &secp256k1::PublicKey) -> String {
        let mut hasher = sha3::Keccak256::new();
        hasher.update(&public_key.serialize_uncompressed()[1..]);
        let address = &hasher.finalize()[12..];
        format!("0x{}", hex::encode(address))
    }

    // pub fn to_keystore(private_key: &str, password: &str) -> Result<String, Box<dyn std::error::Error>> {
    
    //     // Convert the private key from hex to bytes
    //     let private_key_bytes = hex::decode(private_key)?;
    
    //     // Encrypt the private key with the provided password
    //     let keystore = Keystore::new(&private_key_bytes, password)?;
    
    //     // Serialize the keystore to JSON
    //     let keystore_json = serde_json::to_string_pretty(&keystore)?;
    //     Ok(keystore_json)
    // }
}
impl Drop for Wallet {
    fn drop(&mut self) {
        self.private_key.zeroize();
        self.public_key.zeroize();
        self.address.zeroize();
    }
}
