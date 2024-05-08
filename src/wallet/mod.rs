// Algorithm - ECDSA (Elliptic Curve Digital Signature Algorithm)
use bitcoincash_addr::{Address, HashType, Scheme};
use crypto::{ed25519};
use crypto::digest::{Digest};
use crypto::sha2::{Sha256};
use crypto::ripemd160::{Ripemd160};
use rand::{RngCore};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use log::{info};
use std::collections::{HashMap};
use crate::error::{Result};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Wallet {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
} impl Wallet {
    // Generate a new cryptographic wallet
    fn new() -> Self {
        // Create an array of bytes to hold the wallet key
        let mut key: [u8; 32] = [0; 32];
        // Use the operating systems random number generator to fill the key with cryptographically secure random bytes
        OsRng.fill_bytes(&mut key);
        // Generate a key pair with the Ed25519 algorithm given a key
        let (secret_key, public_key) = ed25519::keypair(&key);
        // Convert the keys into vectors
        let secret_key = secret_key.to_vec();
        let public_key = public_key.to_vec();
        // Create and return the new wallet with the generated key pair
        Wallet {
            secret_key,
            public_key,
        }
    }

    fn get_address(&self) -> String {
        let mut pub_hash = self.public_key.clone();
        hash_pub_key(&mut pub_hash);

        let address = Address {
            body: pub_hash,
            scheme: Scheme::Base58, // Removes '0, O, 1, I' for ease of use
            hash_type: HashType::Script,
            ..Default::default()
        };

        address.encode().unwrap()
    }
}

// Util
pub fn hash_pub_key(pub_key: &mut Vec<u8>) {
    let mut hasher1 = Sha256::new();
    hasher1.input(pub_key);
    hasher1.result(pub_key);
    let mut hasher2 = Ripemd160::new();
    hasher2.input(pub_key);
    pub_key.resize(20, 0);
    hasher2.result(pub_key);
}



pub struct Wallets {
    wallets: HashMap<String, Wallet> // Key: address ; Value: Wallet
} impl Wallets {
    // Creates a new set of wallets
    pub fn new() -> Result<Wallets> {
        // Create a HashMap to store set of wallets
        let mut wlts = Wallets {
            wallets: HashMap::<String, Wallet>::new(),
        };
        // Open the wallets section of the database
        let db = sled::open("data/wallets")?;
        // Iterate over each wallet in the database
        for item in db.into_iter() {
            // Extract the current item as a tuple
            // Items are key-value pairs (address, wallet contents) from the hashmap
            let i = item?;
            // Extract the key (wallet address) of the current item
            let address = String::from_utf8(i.0.to_vec())?;
            // Extract the value (wallet contents) of the current value
            let wallet = bincode::deserialize(&i.1.to_vec())?;
            // Insert the wallet address and wallet contents into my set of wallets
            wlts.wallets.insert(address, wallet);
        }

        drop(db);
        Ok(wlts)
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();

        self.wallets.insert(address.clone(), wallet);

        info!("Created wallet: {}", address);

        address
    }

    pub fn get_all_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();

        for (address, _) in &self.wallets {
            addresses.push(address.clone());
        }

        addresses
    }

    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    // Save all current wallets into the database
    pub fn save_all(&self) -> Result<()> {
        // Open the wallets section the database
        let db = sled::open("data/wallets")?;
        // Iterate over the current list of wallets
        for (address, wallet) in &self.wallets {
            // Serialize the wallet contents
            let data = bincode::serialize(wallet)?;
            // Add the wallet to the database
            db.insert(address, data)?;
        }
        db.flush()?;
        drop(db);
        Ok(())
    }
}
