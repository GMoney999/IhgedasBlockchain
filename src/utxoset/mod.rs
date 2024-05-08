use std::collections::HashMap;
use log::{info};
use crate::models::block::{Block};
use crate::models::blockchain::{Blockchain};
use crate::error::{Result};
use crate::tx::TXOutputs;

// Unspent Transaction Output Set
// Persistent layer for UTXOS
// This allows us to access database that is connected to our blockchain,
// and then we can create a new layer inside the database where we just have UTXOs.
pub struct UTXOSet {
    pub blockchain: Blockchain,
}

impl UTXOSet {
    // rebuilds the UTXO set
    pub fn reindex(&self) -> Result<()> {
        if let Err(_) = std::fs::remove_dir_all("data/utxos") {
            info!("There are no utxos to delete.")
        }

        let db = sled::open("data/utxos")?;

        let utxos = self.blockchain.find_utxo();

        for (txid, outs) in utxos {
            db.insert(txid.as_bytes(), bincode::serialize(&outs)?)?;
        }

        Ok(())
    }

    // updates the UTXO set with transactions from a block
    // The block is considered to be the tip of the blockchain
    pub fn update(&self, block: &Block) -> Result<()> {
        let db = sled::open("data/utxos")?;

        for tx in block.get_transactions() {
            if !tx.is_coinbase() {
                for vin in &tx.vin {
                    let mut update_outputs = TXOutputs { outputs: Vec::new() };
                    let outs: TXOutputs = bincode::deserialize(&db.get(&vin.txid)?.unwrap().to_vec())?;

                    for out_idx in 0..outs.outputs.len() {
                        if out_idx != vin.vout as usize {
                            update_outputs.outputs.push(outs.outputs[out_idx].clone());
                        }
                    }

                    if update_outputs.outputs.is_empty() {
                        db.remove(&vin.txid)?;
                    } else {
                        db.insert(vin.txid.as_bytes(), bincode::serialize(&update_outputs)?)?;
                    }
                }
            }

            let mut new_outputs = TXOutputs { outputs: Vec::new() };

            for out in &tx.vout {
                new_outputs.outputs.push(out.clone());
            }

            db.insert(tx.id.as_bytes(), bincode::serialize(&new_outputs)?)?;
        }

        Ok(())
    }

    //// find_spendable_outputs() identifies unspent outputs (UTXOs) that can be unlocked (spent)
    //// using the given address, and aggregates them until the requested amount is reached or surpassed.
    // Returns the total accumulated value and a map of transactions to the indices of their outputs that can be spent.
    // Returns a list of transactions containing unspent outputs
    pub fn find_spendable_outputs(
        &self,
        address: &[u8], // The address used to find spendable outputs for
        amount: i32, // The total amount needed for those outputs
    ) -> Result<(i32, HashMap<String, Vec<i32>>)> {
        // Create a hashmap to store the transaction IDs and the indices of their spendable outputs.
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();

        // Create an accumulator for the total value of the found spendable outputs.
        let mut accumulated: i32 = 0;

        let db = sled::open("data/utxos")?;

        for kv in db.iter() {
            let (k, v) = kv?;
            let txid = String::from_utf8(k.to_vec())?;
            let outs: TXOutputs = bincode::deserialize(&v.to_vec())?;

            for out_idx in 0..outs.outputs.len() {
                if outs.outputs[out_idx].is_locked_with_key(address) && accumulated < amount {
                    accumulated += outs.outputs[out_idx].value;

                    match unspent_outputs.get_mut(&txid) {
                        Some(v) => v.push(out_idx as i32),
                        None => {
                            unspent_outputs.insert(txid.clone(), vec![out_idx as i32]);
                        }
                    }
                }
            }
        }

        Ok((accumulated, unspent_outputs))
    }

    // finds UTXO for a public key hash
    pub fn find_utxos(&self, pub_key_hash: &[u8]) -> Result<TXOutputs> {
        let mut utxos = TXOutputs {
            outputs: Vec::new(),
        };

        let db = sled::open("data/utxos")?;

        for kv in db.iter() {
            let (_, v) = kv?;

            let outs: TXOutputs = bincode::deserialize(&v.to_vec())?;

            for out in outs.outputs {
                if out.can_be_unlocked_with(pub_key_hash) {
                    utxos.outputs.push(out.clone())
                }
            }
        }

        Ok(utxos)
    }


    // returns the number of transactions in the UTXO set
    pub fn count_transactions(&self) -> Result<i32> {
        let mut counter: i32 = 0;
        let db = sled::open("data/utxos")?;
        for kv in db.iter() {
            kv?;
            counter+=1;
        }

        Ok(counter)
    }
}