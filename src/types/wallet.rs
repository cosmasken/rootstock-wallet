use crate::types::contacts::Contact;
use aes::Aes256;
use anyhow::Result;
use anyhow::{Error, anyhow};
use base64::engine::general_purpose::STANDARD;
use base64::{self, Engine as _};
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};
use chrono::Utc;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, U256};
use generic_array::GenericArray;
use rand::RngCore;
use scrypt::{Params, scrypt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use zeroize::Zeroize;

use crate::security::redacted_debug::RedactedDebug;
use crate::security::{SecureString, SecurePassword};

#[derive(Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: Address,
    pub balance: U256,
    pub network: String,
    pub name: String,
    encrypted_private_key: SecureString,
    salt: SecureString,
    iv: SecureString,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct WalletData {
    pub current_wallet: String,
    pub wallets: HashMap<String, Wallet>,
    pub contacts: Vec<Contact>,
    api_key: Option<SecureString>,
}

impl Wallet {
    pub fn address(&self) -> Address {
        self.address
    }

    pub fn new(wallet: LocalWallet, name: &str, password: &SecurePassword) -> Result<Self, Error> {
        let (encrypted_key, iv, salt) =
            Self::encrypt_private_key(wallet.signer().to_bytes().as_ref(), password)?;
        Ok(Self {
            address: wallet.address(),
            balance: U256::zero(),
            network: String::new(),
            name: name.to_string(),
            encrypted_private_key: SecureString::new(STANDARD.encode(&encrypted_key)),
            salt: SecureString::new(STANDARD.encode(&salt)),
            iv: SecureString::new(STANDARD.encode(&iv)),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    pub fn encrypt_private_key(
        private_key: &[u8],
        password: &SecurePassword,
    ) -> anyhow::Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);
        let params = Params::recommended();
        let mut key = [0u8; 32];
        scrypt(password.expose_bytes(), &salt, &params, &mut key)?;
        let mut buffer = private_key.to_vec();
        let pos = buffer.len();
        let pad_len = 16 - (pos % 16);
        buffer.extend(std::iter::repeat_n(pad_len as u8, pad_len));
        let encryptor = Encryptor::<Aes256>::new(&key.into(), &iv.into());
        let _ = encryptor.encrypt_padded_mut::<Pkcs7>(&mut buffer, pos);
        
        // Clear the derived key from memory
        key.zeroize();
        
        Ok((buffer, iv.to_vec(), salt.to_vec()))
    }

    pub fn decrypt_private_key(&self, password: &SecurePassword) -> Result<String, anyhow::Error> {
        // Decode Base64-encoded salt, IV, and encrypted key
        let salt = STANDARD
            .decode(self.salt.expose().map_err(|e| anyhow!("Invalid UTF-8 in salt: {}", e))?)
            .map_err(|e| anyhow!("Failed to decode salt: {}", e))?;
        let iv = STANDARD
            .decode(self.iv.expose().map_err(|e| anyhow!("Invalid UTF-8 in IV: {}", e))?)
            .map_err(|e| anyhow!("Failed to decode IV: {}", e))?;
        let encrypted_key = STANDARD
            .decode(self.encrypted_private_key.expose().map_err(|e| anyhow!("Invalid UTF-8 in encrypted key: {}", e))?)
            .map_err(|e| anyhow!("Failed to decode encrypted private key: {}", e))?;

        // Validate lengths
        if salt.len() != 16 {
            return Err(anyhow!("Salt must be 16 bytes, got {} bytes", salt.len()));
        }
        if iv.len() != 16 {
            return Err(anyhow!("IV must be 16 bytes, got {} bytes", iv.len()));
        }
        if encrypted_key.len() % 16 != 0 {
            return Err(anyhow!(
                "Encrypted key length ({}) is not a multiple of 16",
                encrypted_key.len()
            ));
        }

        // Derive the key using scrypt with parameters matching encryption
        let mut key = [0u8; 32];
        let params = Params::recommended(); // Ensure this matches your encryption params
        scrypt(password.expose_bytes(), &salt, &params, &mut key)
            .map_err(|e| anyhow!("Key derivation failed: {}", e))?;

        // Convert key and IV to GenericArray for the cipher
        let key_array = GenericArray::from_slice(&key[..]); // returns &GenericArray<u8, U32>
        let iv_array = GenericArray::from_slice(&iv[..]); // returns &GenericArray<u8, U16>
        // Set up AES-256-CBC decryptor
        type Aes256CbcDec = Decryptor<Aes256>;
        let cipher = Aes256CbcDec::new(key_array, iv_array);

        // Create a mutable buffer for decryption
        let mut buffer = encrypted_key.clone(); // Clone to make it mutable
        let decrypted = cipher
            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        // Ensure the decrypted key is exactly 32 bytes
        if decrypted.len() != 32 {
            return Err(anyhow!(
                "Decrypted private key has invalid length: {} bytes (expected 32)",
                decrypted.len()
            ));
        }

        // Clear the derived key from memory
        key.zeroize();
        
        // Return the decrypted private key as a 0x-prefixed hex string
        Ok(format!("0x{}", hex::encode(decrypted)))
    }
}

impl RedactedDebug for Wallet {
    fn redacted_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address)
            .field("balance", &self.balance)
            .field("network", &self.network)
            .field("name", &self.name)
            .field("encrypted_private_key", &"[REDACTED]")
            .field("salt", &"[REDACTED]")
            .field("iv", &"[REDACTED]")
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl fmt::Debug for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted_fmt(f)
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Name: {}\nAddress: {}\nNetwork: {}",
            self.name, self.address, self.network
        )
    }
}

impl Zeroize for Wallet {
    fn zeroize(&mut self) {
        // Zeroize sensitive fields
        self.encrypted_private_key.zeroize();
        self.salt.zeroize();
        self.iv.zeroize();
        // Note: We don't zeroize public fields like address, balance, network, name, created_at
        // as they are not considered sensitive for security purposes
    }
}

impl Drop for Wallet {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl WalletData {
    /// Creates a new, empty wallet data structure.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the API key
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(SecureString::new(api_key));
    }

    /// Get the API key (returns None if not set)
    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_ref().and_then(|key| key.expose().ok())
    }

    /// Clear the API key
    pub fn clear_api_key(&mut self) {
        self.api_key = None;
    }

    pub fn add_wallet(&mut self, wallet: Wallet) -> anyhow::Result<()> {
        let address = format!("0x{:x}", wallet.address);
        if self.wallets.contains_key(&address) {
            return Err(anyhow!("Wallet with address {} already exists", address));
        }
        self.wallets.insert(address.clone(), wallet);
        self.current_wallet = address;
        Ok(())
    }

    pub fn get_current_wallet(&self) -> Option<&Wallet> {
        self.wallets.get(&self.current_wallet)
    }

    pub fn switch_wallet(&mut self, address: &str) -> anyhow::Result<()> {
        if !self.wallets.contains_key(address) {
            return Err(anyhow!("Wallet with address {} not found", address));
        }
        self.current_wallet = address.to_string();
        Ok(())
    }

    pub fn get_wallet_by_name(&self, name: &str) -> Option<&Wallet> {
        self.wallets.values().find(|w| w.name == name)
    }

    pub fn remove_wallet(&mut self, address: &str) -> anyhow::Result<()> {
        if !self.wallets.contains_key(address) {
            return Err(anyhow!("Wallet with address {} not found", address));
        }
        if self.current_wallet == address {
            self.current_wallet = String::new();
        }
        self.wallets.remove(address);
        Ok(())
    }

    pub fn rename_wallet(&mut self, wallet: &Wallet, new_name: &str) -> anyhow::Result<()> {
        let address = format!("0x{:x}", wallet.address);
        if !self.wallets.contains_key(&address) {
            return Err(anyhow!("Wallet with address {} not found", address));
        }
        if let Some(w) = self.wallets.get_mut(&address) {
            w.name = new_name.to_string();
            Ok(())
        } else {
            Err(anyhow!("Failed to rename wallet {}", address))
        }
    }

    pub fn list_wallets(&self) -> Vec<&Wallet> {
        self.wallets.values().collect()
    }

    pub fn add_contact(&mut self, contact: Contact) -> anyhow::Result<()> {
        if self
            .contacts
            .iter()
            .any(|c| c.name == contact.name || c.address == contact.address)
        {
            return Err(anyhow!("Contact with name or address already exists"));
        }
        self.contacts.push(contact);
        Ok(())
    }

    pub fn remove_contact(&mut self, identifier: &str) -> anyhow::Result<()> {
        let index = self
            .contacts
            .iter()
            .position(|c| c.name == identifier || c.address.to_string() == identifier)
            .ok_or_else(|| anyhow!("Contact not found"))?;
        self.contacts.remove(index);
        Ok(())
    }

    pub fn update_contact(&mut self, identifier: &str, contact: Contact) -> anyhow::Result<()> {
        let index = self
            .contacts
            .iter()
            .position(|c| c.name == identifier || c.address.to_string() == identifier)
            .ok_or_else(|| anyhow!("Contact not found"))?;
        self.contacts[index] = contact;
        Ok(())
    }

    pub fn get_contact(&self, identifier: &str) -> Option<&Contact> {
        self.contacts
            .iter()
            .find(|c| c.name == identifier || c.address.to_string() == identifier)
    }

    pub fn search_contacts(&self, query: &str) -> Vec<&Contact> {
        self.contacts
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&query.to_lowercase())
                    || c.address.to_string().contains(query)
                    || c.notes.as_ref().is_some_and(|n| n.contains(query))
                    || c.tags.iter().any(|t| t.contains(query))
            })
            .collect()
    }
}

impl RedactedDebug for WalletData {
    fn redacted_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalletData")
            .field("current_wallet", &self.current_wallet)
            .field("wallets_count", &self.wallets.len())
            .field("contacts_count", &self.contacts.len())
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

impl fmt::Debug for WalletData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.redacted_fmt(f)
    }
}

impl Zeroize for WalletData {
    fn zeroize(&mut self) {
        // Zeroize sensitive fields
        if let Some(ref mut api_key) = self.api_key {
            api_key.zeroize();
        }
        // Zeroize all wallets
        for wallet in self.wallets.values_mut() {
            wallet.zeroize();
        }
        // Note: We don't zeroize current_wallet and contacts as they contain
        // non-sensitive metadata
    }
}

impl Drop for WalletData {
    fn drop(&mut self) {
        self.zeroize();
    }
}
