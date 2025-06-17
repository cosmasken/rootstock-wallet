use bip32::{DerivationPath, XPrv};
use bip39::{Language, Mnemonic};
use eth_keystore::{self, KeystoreError};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Bytes, transaction::eip2718::TypedTransaction};
use k256::ecdsa::SigningKey;
use log::{error, info};
use rand::rngs::OsRng;
use secp256k1::{Secp256k1, rand};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use std::fs::{self};
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;
use zeroize::Zeroize;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Invalid private key")]
    InvalidPrivateKey,
    #[error("Invalid mnemonic")]
    InvalidMnemonic,
    #[error("Invalid derivation path")]
    InvalidDerivationPath,
    #[error("Keystore error: {0}")]
    KeystoreError(#[from] KeystoreError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Wallet not found")]
    WalletNotFound,
    #[error("Invalid address")]
    InvalidAddress,
    // #[error("Signing error: {0}")]
    // SigningError(ethers::signers::WalletError),
    #[error("Signing error: {0}")]
    SigningError(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub address: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub private_key: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct WalletManager {
    wallets: HashMap<String, Wallet>,
    current_wallet: Option<String>,
}

impl WalletManager {
    pub fn load_from_file(path: &str) -> Result<Self, WalletError> {
        if Path::new(path).exists() {
            let data = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(WalletManager::default())
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), WalletError> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn add_wallet(&mut self, wallet: Wallet) {
        let address = wallet.address.clone();
        self.wallets.insert(address.clone(), wallet);
        if self.current_wallet.is_none() {
            self.current_wallet = Some(address);
        }
    }

    pub fn get_current_wallet(&self) -> Option<&Wallet> {
        self.current_wallet
            .as_ref()
            .and_then(|addr| self.wallets.get(addr))
    }

    pub fn set_current_wallet(&mut self, address: &str) -> Result<(), WalletError> {
        if self.wallets.contains_key(address) {
            self.current_wallet = Some(address.to_string());
            Ok(())
        } else {
            Err(WalletError::WalletNotFound)
        }
    }

    pub fn list_wallets(&self) -> Vec<(&String, &Wallet)> {
        self.wallets.iter().collect()
    }
}

impl Wallet {
    pub fn generate() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let public_key_bytes = public_key.serialize_uncompressed();
        let address = Self::public_key_to_address(&public_key_bytes[1..]);
        let private_key = hex::encode(secret_key.secret_bytes());

        info!("Generated new wallet with address: {}", address);

        Wallet {
            address,
            public_key: hex::encode(&public_key_bytes[1..]),
            private_key,
        }
    }

    pub fn from_private_key(private_key: &str) -> Result<Self, WalletError> {
        let private_key_bytes =
            hex::decode(private_key).map_err(|_| WalletError::InvalidPrivateKey)?;
        let secret_key = secp256k1::SecretKey::from_slice(&private_key_bytes)
            .map_err(|_| WalletError::InvalidPrivateKey)?;
        let secp = Secp256k1::new();
        let public_key = secret_key.public_key(&secp);
        let public_key_bytes = public_key.serialize_uncompressed();
        let address = Self::public_key_to_address(&public_key_bytes[1..]);

        Ok(Wallet {
            address,
            public_key: hex::encode(&public_key_bytes[1..]),
            private_key: private_key.to_string(),
        })
    }

    pub fn from_mnemonic(mnemonic: &str, derivation_path: &str) -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic)
            .map_err(|_| WalletError::InvalidMnemonic)?;
        let seed = mnemonic.to_seed_normalized("");
        let derivation_path = DerivationPath::from_str(derivation_path)
            .map_err(|_| WalletError::InvalidDerivationPath)?;
        let xprv = XPrv::derive_from_path(&seed, &derivation_path)
            .map_err(|_| WalletError::InvalidDerivationPath)?;
        let private_key_bytes = xprv.private_key().to_bytes();
        let secret_key = secp256k1::SecretKey::from_slice(&private_key_bytes)
            .map_err(|_| WalletError::InvalidPrivateKey)?;
        let secp = Secp256k1::new();
        let public_key = secret_key.public_key(&secp);
        let public_key_bytes = public_key.serialize_uncompressed();
        let address = Self::public_key_to_address(&public_key_bytes[1..]);

        Ok(Wallet {
            address,
            public_key: hex::encode(&public_key_bytes[1..]),
            private_key: hex::encode(private_key_bytes),
        })
    }

    fn public_key_to_address(public_key: &[u8]) -> String {
        let mut hasher = Keccak256::new();
        hasher.update(public_key);
        let result = hasher.finalize();
        let address_bytes = &result[12..];
        format!("0x{}", hex::encode(address_bytes))
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), WalletError> {
        let mut manager = WalletManager::load_from_file(path)?;
        manager.add_wallet(self.clone());
        manager.save_to_file(path)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Option<Self> {
        let manager = WalletManager::load_from_file(path).ok()?;
        manager.get_current_wallet().cloned()
    }

    pub async fn sign_transaction(&self, tx: &TypedTransaction) -> Result<Bytes, WalletError> {
        let private_key =
            hex::decode(&self.private_key).map_err(|_| WalletError::InvalidPrivateKey)?;
        let wallet = LocalWallet::from(
            SigningKey::from_slice(&private_key).map_err(|_| WalletError::InvalidPrivateKey)?,
        );
        let signature = wallet
            .sign_transaction(tx)
            .await
            .map_err(|e| WalletError::SigningError(e.to_string()))?;
        // .map_err(|e| WalletError::SigningError(e))?;
        let signed_tx = tx.rlp_signed(&signature);
        Ok(signed_tx)
    }

    pub fn encrypt(&self, password: &str) -> Result<String, WalletError> {
        let private_key =
            hex::decode(&self.private_key).map_err(|_| WalletError::InvalidPrivateKey)?;
        let dir = tempfile::TempDir::new()?;
        let keystore =
            eth_keystore::encrypt_key(dir.path(), &mut OsRng, &private_key, password, None)?;
        let keystore_json = serde_json::to_string(&keystore)?;
        Ok(keystore_json)
    }

    pub fn decrypt(keystore_path: &str, password: &str) -> Result<Self, WalletError> {
        let private_key = eth_keystore::decrypt_key(keystore_path, password)?;
        Self::from_private_key(&hex::encode(private_key))
    }
}

impl Drop for Wallet {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_wallet() {
        let wallet = Wallet::generate();
        assert_eq!(wallet.address.len(), 42);
        assert_eq!(wallet.public_key.len(), 128);
        assert_eq!(wallet.private_key.len(), 64);
    }

    #[test]
    fn test_from_private_key() {
        let private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let wallet = Wallet::from_private_key(private_key).unwrap();
        assert_eq!(wallet.private_key, private_key);
        assert_eq!(wallet.address.len(), 42);
    }

    #[test]
    fn test_from_mnemonic() {
        let mnemonic = "test test test test test test test test test test test junk";
        let wallet = Wallet::from_mnemonic(mnemonic, "m/44'/137'/0'/0/0").unwrap();
        assert_eq!(wallet.address.len(), 42);
        assert_eq!(wallet.private_key.len(), 64);
    }

    #[test]
    fn test_wallet_manager() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        let wallet1 = Wallet::generate();
        let wallet2 = Wallet::generate();

        let mut manager = WalletManager::load_from_file(path).unwrap();
        manager.add_wallet(wallet1.clone());
        manager.add_wallet(wallet2.clone());
        manager.save_to_file(path).unwrap();

        let loaded_manager = WalletManager::load_from_file(path).unwrap();
        assert_eq!(loaded_manager.wallets.len(), 2);
        assert_eq!(
            loaded_manager.get_current_wallet().unwrap().address,
            wallet1.address
        );

        manager.set_current_wallet(&wallet2.address).unwrap();
        assert_eq!(
            manager.get_current_wallet().unwrap().address,
            wallet2.address
        );
    }

    #[test]
    fn test_encrypt_decrypt() {
        let wallet = Wallet::generate();
        let password = "testpassword";
        let keystore = wallet.encrypt(password).unwrap();

        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();
        fs::write(path, keystore).unwrap();

        let decrypted_wallet = Wallet::decrypt(path, password).unwrap();
        assert_eq!(decrypted_wallet.address, wallet.address);
        assert_eq!(decrypted_wallet.private_key, wallet.private_key);
    }
}
