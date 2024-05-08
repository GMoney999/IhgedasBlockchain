use crate::transaction::{Transaction};
use crate::error::{Result};
use std::time::{SystemTime, UNIX_EPOCH};
use crypto::sha2::{Sha256};
use crypto::digest::{Digest};
use merkle_cbt::merkle_tree::{CBMT, Merge};
use serde::{Serialize, Deserialize};
use log::{info};


// Difficulty of Proof-Of-Work algorithm
const TARGET_HEXT: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    timestamp: u128,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    height: i32,
    nonce: i32,
} impl Block {
    pub fn new_genesis_block(coinbase: Transaction) -> Block {
        Block::new(vec![coinbase], String::new(), 0).unwrap()
    }
    pub fn new(data: Vec<Transaction>, prev_block_hash: String, height: i32) -> Result<Block> {
        let timestamp = get_timestamp()?;

        let mut block = Block {
            timestamp,
            transactions: data,
            prev_block_hash,
            hash: String::new(),
            height,
            nonce: 0
        };

        block.run_proof_of_work()?;
        Ok(block)
    }
    pub fn validate(&self) -> Result<bool> {
        let hash = self.generate_hash()?;
        // Generate a string of zeros for comparison
        let target = "0".repeat(TARGET_HEXT);
        // Compare the first TARGET_HEXT characters of the hex result with the target string of zeros
        Ok(hash.starts_with(&target))
    }
    pub fn run_proof_of_work(&mut self) -> Result<()> {
        info!("Mining the block...");
        // While the hash does not start with 4 leading zeroes, increment nonce and try again
        while !self.validate()? {
            self.nonce += 1;
        }
        // Generate the hash for the block
        let hash = self.generate_hash()?;
        // Set the hash valid hash to the hash of the block
        self.hash = hash;

        Ok(())
    }
    pub fn generate_hash(&self) -> Result<String> {
        // Get an array of bytes to represent our hash
        let data = self.prepare_hash_data()?;
        // Create a hasher
        let mut hasher = Sha256::new();
        // Enter our data into the hashing algorithm
        hasher.input(&data[..]);
        // Get the result of entering the data into the hashing algorithm
        let result = hasher.result_str();

        Ok(result)
    }
    pub fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        // Format the content to serialize based on the contents of the block
        let content = (
            self.prev_block_hash.clone(),
            self.hash_transactions()?,
            self.timestamp,
            TARGET_HEXT,
            self.nonce,
        );

        let bytes = bincode::serialize(&content)?;
        Ok(bytes)
    }

    // returns a hash of the transactions in a block
    fn hash_transactions(&self) -> Result<Vec<u8>> {
        let mut transactions = Vec::new();
        for tx in &self.transactions {
            let mut new_tx = tx.clone();
            transactions.push(new_tx.hash()?.as_bytes().to_owned());
        }

        let tree = CBMT::<Vec<u8>, MergeTX>::build_merkle_tree(&*transactions);

        Ok(tree.root())
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }
    pub fn get_previous_hash(&self) -> String {
        self.prev_block_hash.clone()
    }
    #[allow(dead_code)]
    pub fn get_height(&self) -> i32 {
        self.height.clone()
    }
    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }
}

struct MergeTX {}
impl Merge for MergeTX {
    type Item = Vec<u8>;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut hasher = Sha256::new();
        let mut data: Vec<u8> = left.clone();
        data.append(&mut right.clone());
        hasher.input(&data);
        let mut re: [u8; 32] = [0; 32];
        hasher.result(&mut re);
        re.to_vec()
    }
}


pub fn get_timestamp() -> Result<u128> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis();
    Ok(timestamp)
}