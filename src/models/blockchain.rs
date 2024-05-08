use std::collections::{HashMap};
use failure::format_err;
use crate::error::{Result};
use crate::models::block::{Block};
use log::{info};
use crate::transaction::{Transaction};
use crate::tx::{TXOutputs};

#[allow(dead_code)]

const GENESIS_COINBASE_DATA: &str = "This is the Genesis Block";

#[derive(Debug, Clone)]
pub struct Blockchain {
    current_hash: String,
    db: sled::Db,
} impl Blockchain {
    // new() opens the blockchain at "data/blocks"
    // Returns a Blockchain instance
    pub fn new() -> Result<Self> {
        info!("Opening blockchain...");
        // Open the database
        let db = sled::open("data/blocks")?;
        // Get the last block in the chain
        let hash = db
            .get("LAST")?
            .expect("Must create a new block database first");
        info!("Found block database");
        // Set the current hash of the database to the hash of the last block
        let last_hash = String::from_utf8(hash.to_vec())?;
        // Return a new blockchain instance with the database and the hash of the last block
        Ok(Self {
            current_hash: last_hash.clone(),
            db,
        })
    }

    //// create_blockchain() creates a new blockchain instance
    // Takes an address for a transaction
    // Returns a blockchain instance
    pub fn create_blockchain(address: String) -> Result<Self> {
        info!("Creating new blockchain...");
        if let Err(_) = std::fs::remove_dir_all("data/blocks") {
            info!("There are no blocks to delete.")
        }
        // Open the database
        let db = sled::open("data/blocks")?;
        info!("Creating new block database...");
        // Create a transaction for the genesis block
        let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
        // Create a genesis block
        let genesis = Block::new_genesis_block(cbtx);
        // Insert the genesis block into the blockchain
        db.insert(genesis.get_hash(), bincode::serialize(&genesis)?)?;
        // Set the last block in the blockchain to the block just created
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        // Create an instance of the blockchain and set the current hash to the hash of the new block
        let bc = Self {
            current_hash: genesis.get_hash(),
            db
        };
        // Flush the database
        bc.db.flush()?;
        // Return the Blockchain
        Ok(bc)
    }

    //// add_block() adds a new block to the blockchain
    // Takes a list of transactions contained in the block
    // Returns nothing
    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<Block> {
        // Get the hash of the last block in the blockchain
        let last_hash = self.db.get("LAST")?.unwrap();

        // Create a new block with the transaction list and the hash of the previous block
        let new_block = Block::new(transactions, String::from_utf8(last_hash.to_vec())?, self.get_best_height().unwrap())?;

        // Insert the new block into the blockchain
        self.db.insert(new_block.get_hash(), bincode::serialize(&new_block)?)?;

        // Set the hash of the last block to the new block since it is now the last block
        self.db.insert("LAST", new_block.get_hash().as_bytes())?;

        // Set the current hash of the blockchain to the hash of the new block
        self.current_hash = new_block.get_hash();

        Ok(new_block)
    }

    //// find_unspent_transactions() finds all transactions in the blockchain that contain outputs which are unspent and can be unlocked (i.e., spent) using the given address.
    // Each output specifies how many coins are being transferred and who can claim them.
    // This function ensures that only legitimate, unspent outputs are used in new transactions
    // Returns a list of transactions containing unspent output
    #[allow(dead_code)]
    pub fn find_unspent_transactions(&self, address: &[u8]) -> Vec<Transaction> {
        // A hashmap to track outputs that have been spent.
        // Key: Transaction ID
        // Value: List of output indices in that transaction.
        let mut spent_txos: HashMap<String, Vec<i32>> = HashMap::new();
        // A collection of transactions that have at least one output that hasn't been spent.
        let mut unspent_txs: Vec<Transaction> = Vec::new();

        // For each block in the blockchain...
        for block in self.iter() {
            // For each transaction within the current block...
            for tx in block.get_transactions() {
                // Examine each output (vout) within the current transaction
                for index in 0..tx.vout.len() {
                    // If there are recorded spent outputs for the current transaction...
                    if let Some(ids) = spent_txos.get(&tx.id) {
                        // and if the current output index is in the list of spent outputs, skip this output.
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }
                    // If the current output has not been marked as spent and can be unlocked with the given address...
                    if tx.vout[index].can_be_unlocked_with(address) {
                        // Add the transaction to the list of unspent transactions.
                        unspent_txs.push(tx.to_owned())
                    }
                }
                // For transactions that are not coinbase transactions (i.e., regular transactions with inputs)...
                if !tx.is_coinbase() {
                    // Examine each input (vin) in the transaction.
                    for i in &tx.vin {
                        // If the input unlocks an output using the given address...
                        if i.can_unlock_output_with(address) {
                            // Then mark the output it references as 'spent' by adding it to the spent_txos hashmap.
                            match spent_txos.get_mut(&i.txid) {
                                // If there's already an entry for this transaction, add the output index to the list.
                                Some(v) => {
                                    v.push(i.vout);
                                },
                                // If not, create a new entry with the output index.
                                None => {
                                    spent_txos.insert(i.txid.clone(), vec![i.vout]);
                                },
                            }
                        }
                    }
                }
            }
        }

        // Return the list of transactions that contain unspent outputs unlockable by the given address.
        unspent_txs
    }

    // Finds and returns all unspent transaction outputs
    /*
        This function is like looking through the entire wallet (and previous wallets)
        to see which bills have never been spent.
        It makes a list of all these bills so that the blockchain knows what is available
        to be spent in future transactions.
        It also keeps track of which bills have been spent to avoid reusing them.
    */
    pub fn find_utxo(&self) -> HashMap<String, TXOutputs> {
        // Initialize a HashMap to store unspent transaction outputs
        let mut utxos: HashMap<String, TXOutputs> = HashMap::new();
        // Initialize a HashMap to keep track of spent transaction output references
        let mut spent_txos: HashMap<String, Vec<i32>> = HashMap::new();

        // Iterate over each block in the blockchain
        for block in self.iter() {
            // iterate over each transaction in the current block
            for tx in block.get_transactions() {
                // Iterate over each output in the transaction
                for index in 0..tx.vout.len() {
                    // Check if the current transaction's outputs are already marked as spent
                    if let Some(ids) = spent_txos.get(&tx.id) {
                        // If the current output index is in the spent list, skip it
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }
                    // Try to find the transaction ID in the unspent outputs map
                    match utxos.get_mut(&tx.id) {
                        // If found, add the current output to the existing list
                        Some(v) => v.outputs.push(tx.vout[index].clone()),
                        // If not found, create a new entry with the current output
                        None => {
                            utxos.insert(
                                tx.id.clone(),
                                TXOutputs {
                                    outputs: vec![tx.vout[index].clone()]
                                },
                            );
                        }
                    }
                }
                // If the transaction is not a coinbase transaction
                if !tx.is_coinbase() {
                    // Iterate over each input in the transaction
                    for i in &tx.vin {
                        // Try to find the input transaction ID in the spent outputs map
                        match spent_txos.get_mut(&i.txid) {
                            // If found, add the output index referred by the input to the spent list
                            Some(v) => {
                                v.push(i.vout)
                            },
                            // If not found, create a new entry with the output index
                            None => {
                                spent_txos.insert(i.txid.clone(), vec![i.vout]);
                            },
                        }
                    }
                }
            }
        }
        // Return the map containing all unspent transaction outputs
        utxos
    }

    // Finds a transaction by its ID
    pub fn find_transaction(&self, id: &str) -> Result<Transaction> {
        // For each block in the blockchain...
        for b in self.iter() {
            // For each transaction in a block...
            for tx in b.get_transactions() {
                // Check if the transaction ID is equal to the ID we are searching for
                if tx.id == id {
                    // If the IDs match, we have found the transaction
                    return Ok(tx.clone());
                }
            }
        }

        // If we loop through every transaction for every block and still cannot find the ID,
        // then the transaction does not exist.
        Err(format_err!("Transaction not found."))
    }

    //// sign_transaction() signs inputs of a transaction given a private key
    pub fn sign_transaction(&self, tx: &mut Transaction, private_key: &[u8]) -> Result<()> {
        // Retrieve all previous transactions referenced by the inputs (TXInputs) of the transaction to be signed.
        // These previous transactions are needed because they contain the outputs that the transaction inputs are spending,
        // and information from these outputs is required for signing.
        let prev_txs = self.get_prev_txs(tx)?;

        // Sign the transaction. This involves:
        // 1. Creating a simplified copy of the transaction to be signed (excluding the input signatures to avoid circular dependency).
        // 2. For each input in the transaction, signing the transaction copy with the private key and saving the signature in the input.
        // This effectively signs the transaction, authorizing the spending of outputs referenced by the transaction's inputs.
        tx.sign(private_key, prev_txs)?;

        // Return Ok to indicate success.
        // If any part of the signing process fails (e.g., if a referenced previous transaction cannot be found,
        // or the signing operation itself fails), an error will be returned instead,
        // and execution will not reach this point.
        Ok(())
    }

    //// get_prev_txs() retrieves all previous transactions referenced by the inputs of the given transaction.
    // It's essential for validating and signing transactions,
    // as it provides the context needed to verify inputs are valid and can be spent.
    fn get_prev_txs(&self, tx: &Transaction) -> Result<HashMap<String, Transaction>> {
        // Initialize an empty HashMap to store the previous transactions.
        // Key: the transaction ID
        // Value: the transaction itself
        let mut prev_txs = HashMap::new();

        // Iterate over each input (TXInput) in the transaction
        for vin in &tx.vin {
            // Attempt to find the transaction referenced by the input's txid in the blockchain.
            // This requires searching through the blockchain data to find the transaction
            // that has an ID matching the input's txid.
            let prev_tx = self.find_transaction(&vin.txid)?;
            // If the previous transaction is found, insert it into the HashMap.
            // This effectively maps each input of the current transaction to its corresponding previous transaction,
            // providing the necessary context for further processing (like validation or signing).
            prev_txs.insert(prev_tx.id.clone(), prev_tx);
        }

        // Return the populated HashMap as the result.
        // This map now contains all transactions referenced by the inputs of the given transaction.
        Ok(prev_txs)
    }

    //// verify_transaction() verifies the validity of a given transaction.
    // It checks if the transaction's inputs are valid and correctly signed,
    // ensuring the integrity and authenticity of the transaction.
    #[allow(dead_code)]
    pub fn verify_transaction(&self, tx: &mut Transaction) -> Result<bool> {
        // First, retrieve all previous transactions that are referenced by the inputs of the transaction to be verified.
        // These previous transactions are needed because they contain the outputs that the current transaction's inputs are attempting to spend.
        // This step is important for verifying that the inputs are authorized to spend the outputs they claim to.
        let prev_txs = self.get_prev_txs(tx)?;

        // Next, call the `verify` method on the transaction, passing in the previous transactions as context.
        // The `verify` method will use this context to check various conditions:
        // - That each input is authorized to spend the output it references, typically by checking digital signatures.
        // - That the transaction has not been tampered with.
        // - That any additional rules specific to the blockchain implementation are followed.
        tx.verify(prev_txs)
    }

    // Retrieves all blocks from the blockchain.
    #[allow(dead_code)]
    pub fn get_blocks(&self) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();
        for block in self.iter() {
            blocks.push(block);
        }
        Ok(blocks)
    }

    pub fn get_best_height(&self) -> Result<i32> {
        let last_hash = if let Ok(Some(h)) = self.db.get("LAST") {
            h
        } else {
            return Ok(-1);
        };

        let last_data = self.db.get(last_hash)?.unwrap();
        let last_block: Block = bincode::deserialize(&last_data.to_vec())?;
        Ok(last_block.get_height())
    }

    #[allow(dead_code)]
    pub fn get_block_hashes(&self) -> Vec<String> {
        let mut list = Vec::new();
        for b in self.iter() {
            list.push(b.get_hash());
        }
        list
    }

    pub fn iter(&self) -> BlockchainIter {
        BlockchainIter {
            current_hash: self.current_hash.clone(),
            blockchain: &self
        }
    }
}

pub struct BlockchainIter<'a> {
    current_hash: String,
    blockchain: &'a Blockchain
} impl<'a> Iterator for BlockchainIter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        if let Ok(encode_block) = self.blockchain.db.get(&self.current_hash) {
            return match encode_block {
                Some(b) => {
                    if let Ok(block) = bincode::deserialize::<Block>(&b) {
                        self.current_hash = block.get_previous_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}