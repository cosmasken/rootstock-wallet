use aes::Aes256;
use anyhow::Error;
use base64;
use cbc::Encryptor;
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use chrono::Utc;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, U256};
use rand::RngCore;
use scrypt::{Params, scrypt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: Address,
    pub balance: U256,
    pub network: String,
    pub name: String,
    pub encrypted_private_key: String,
    pub salt: String,
    pub iv: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub current_wallet: String,
    pub wallets: HashMap<String, Wallet>,
}

impl Wallet {
    pub fn address(&self) -> Address {
        self.address
    }

    // new function
    pub fn new(wallet: LocalWallet, name: &str, password: &str) -> Result<Self, Error> {
        let (encrypted_key, iv, salt) =
            Self::encrypt_private_key(wallet.signer().to_bytes().as_ref(), password)?;
        Ok(Self {
            address: wallet.address(),
            balance: U256::zero(),
            network: String::new(),
            name: name.to_string(),
            encrypted_private_key: base64::encode(&encrypted_key),
            salt: base64::encode(&salt),
            iv: base64::encode(&iv),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    /// Encrypts a private key using AES-256-CBC with a password-derived key.
    pub fn encrypt_private_key(
        private_key: &[u8],
        password: &str,
    ) -> anyhow::Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        // Generate random salt and IV
        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);

        // Derive key using scrypt
        let params = Params::recommended();
        let mut key = [0u8; 32];
        scrypt(password.as_bytes(), &salt, &params, &mut key)?;

        // Encrypt using AES-256-CBC (with PKCS7 padding)
        let mut buffer = private_key.to_vec();
        let pos = buffer.len();
        // Pad buffer to next multiple of block size (16)
        let pad_len = 16 - (pos % 16);
        buffer.extend(std::iter::repeat(pad_len as u8).take(pad_len));

        let mut encryptor = Encryptor::<Aes256>::new(&key.into(), &iv.into());
        let _ = encryptor.encrypt_padded_mut::<Pkcs7>(&mut buffer, pos);

        Ok((buffer, iv.to_vec(), salt.to_vec()))
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Name: {}\nAddress: {}\nNetwork: {}",
            self.name,
            self.address, // Simplified; assuming `.green()` was from a color crate not in scope
            self.network
        )
    }
}

impl WalletData {
    pub fn new() -> Self {
        Self {
            current_wallet: String::new(),
            wallets: HashMap::new(),
        }
    }

    pub fn add_wallet(&mut self, wallet: Wallet) -> Result<(), String> {
        let address = format!("0x{:x}", wallet.address);
        if self.wallets.contains_key(&address) {
            return Err(format!("Wallet with address {} already exists", address));
        }
        self.wallets.insert(address.clone(), wallet);
        self.current_wallet = address;
        Ok(())
    }

    pub fn get_current_wallet(&self) -> Option<&Wallet> {
        self.wallets.get(&self.current_wallet)
    }

    pub fn switch_wallet(&mut self, address: &str) -> Result<(), String> {
        if !self.wallets.contains_key(address) {
            return Err(format!("Wallet with address {} not found", address));
        }
        self.current_wallet = address.to_string();
        Ok(())
    }

    pub fn get_wallet_by_name(&self, name: &str) -> Option<&Wallet> {
        self.wallets.values().find(|w| w.name == name)
    }

    pub fn remove_wallet(&mut self, address: &str) -> Result<(), String> {
        if !self.wallets.contains_key(address) {
            return Err(format!("Wallet with address {} not found", address));
        }
        if self.current_wallet == address {
            self.current_wallet = String::new();
        }
        self.wallets.remove(address);
        Ok(())
    }

    pub fn rename_wallet(&mut self, wallet: &Wallet, new_name: &str) -> Result<(), String> {
        let address = format!("0x{:x}", wallet.address);
        if !self.wallets.contains_key(&address) {
            return Err(format!("Wallet with address {} not found", address));
        }
        if let Some(w) = self.wallets.get_mut(&address) {
            w.name = new_name.to_string();
            Ok(())
        } else {
            Err(format!("Failed to rename wallet {}", address))
        }
    }

    pub fn list_wallets(&self) -> Vec<&Wallet> {
        self.wallets.values().collect()
    }
}
