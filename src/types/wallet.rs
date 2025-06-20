use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use chrono::Utc;
use anyhow::Error;
use aes::Aes256;
use cbc::{Encryptor, Decryptor};
use cbc::cipher::{KeyIvInit, BlockEncryptMut, BlockDecryptMut};
use cbc::cipher::block_padding::Pkcs7;
use rand::RngCore;
use scrypt::{scrypt,Params};
use std::fs::File;
use std::io::Write;
use base64;
use rand::thread_rng;
use rand::Rng;

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

impl Wallet {
    pub fn address(&self) -> Address {
        self.address
    }

    // pub fn new(wallet: LocalWallet, name: &str, password: &str) -> Result<Self, Error> {
    //     let (encrypted_key, iv, salt) = Self::encrypt_private_key(wallet.to_string().as_bytes(), password)?;
    //     Ok(Self {
    //         address: wallet.address(),
    //         balance: U256::zero(),
    //         network: String::new(),
    //         name: name.to_string(),
    //         encrypted_private_key: base64::encode(&encrypted_key),
    //         salt: base64::encode(&salt),
    //         iv: base64::encode(&iv),
    //         created_at: Utc::now().to_rfc3339(),
    //     })
    // }

    // new function
    pub fn new(wallet: LocalWallet, name: &str, password: &str) -> Result<Self, Error> {
        let (encrypted_key, iv, salt) = Self::encrypt_private_key(wallet.signer().to_bytes().as_ref(), password)?;
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
pub fn encrypt_private_key(private_key: &[u8], password: &str) -> anyhow::Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
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
    let _ =encryptor.encrypt_padded_mut::<Pkcs7>(&mut buffer, pos);

    Ok((buffer, iv.to_vec(), salt.to_vec()))
}

/// Decrypts the private key using AES-256-CBC with a password-derived key.
pub fn decrypt_private_key(
    ciphertext: &[u8],
    password: &str,
    iv: &[u8],
    salt: &[u8],
) -> anyhow::Result<Vec<u8>> {
    // Derive key using scrypt
    let params = Params::recommended();
    let mut key = [0u8; 32];
    scrypt(password.as_bytes(), salt, &params, &mut key)?;

    // Decrypt using AES-256-CBC (with PKCS7 padding)
    let mut buffer = ciphertext.to_vec();
    let mut decryptor = Decryptor::<Aes256>::new(&key.into(), iv.into());
    let decrypted = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| anyhow::anyhow!("Unpad error: {:?}", e))?;
    Ok(decrypted.to_vec())
}

    // pub fn to_local_wallet(&self, password: &str) -> Result<LocalWallet, Error> {
    //     let private_key = self.decrypt_private_key(password)?;
    //     Ok(LocalWallet::from_str(&private_key)?)
    // }

    pub fn update_balance(&mut self, new_balance: U256) {
        self.balance = new_balance;
    }

    pub fn update_network(&mut self, network: &str) {
        self.network = network.to_string();
    }

    pub fn to_string(&self) -> String {
        format!(
            "Address: {}\nBalance: {} RBTC\nNetwork: {}",
            self.address,
            ethers::utils::format_units(self.balance, 18).unwrap_or_default(),
            self.network
        )
    }

    pub fn backup(&self, backup_dir: &Path, _password: &str) -> Result<String, Error> {
        let timestamp = Utc::now().timestamp().to_string();
        let backup_path = backup_dir.join(format!("wallet_{}_{}.json", self.name, timestamp));
        
        // Corrected field names to match the struct
        let backup_wallet = Self {
            address: self.address,
            balance: self.balance,
            network: self.network.clone(),
            name: self.name.clone(), // Added missing 'name' field
            encrypted_private_key: self.encrypted_private_key.clone(), // Fixed 'encrypted_key'
            iv: self.iv.clone(),
            salt: self.salt.clone(),
            created_at: self.created_at.clone(), // Fixed 'creation_time'
            // 'is_hardware' omitted as it’s not in the struct
        };
        
        let contents = serde_json::to_string_pretty(&backup_wallet)?;
        
        let mut file = File::create(&backup_path)?;
        file.write_all(contents.as_bytes())?;
        
        Ok(backup_path.to_string_lossy().to_string())
    }

    // pub fn validate(&self, password: &str) -> Result<bool, Error> {
    //     self.decrypt_private_key(password).map(|_| true)
    // }

    pub fn update_name(&mut self, new_name: &str) -> Result<(), Error> {
        self.name = new_name.to_string();
        Ok(())
    }

    // pub fn delete(&self, _storage: &mut WalletStorage) -> Result<(), Error> {
    //     // Note: WalletStorage is undefined in the provided code.
    //     // Assuming it’s defined elsewhere, leaving as placeholder.
    //     // storage.remove_wallet(&self.name);
    //     Ok(())
    // }
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