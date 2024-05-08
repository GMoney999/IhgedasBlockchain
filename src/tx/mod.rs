use bitcoincash_addr::{Address};
use serde::{Deserialize, Serialize};
use log::{debug};
use crate::error::{Result};
use crate::wallet::hash_pub_key;


// TXInput represents an input of a transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    // 'txid' - represents the transactional ID from which the input is coming.
    // txid serves as a reference to a previous transaction that is being used as an input for a new transaction
    pub txid: String,
    // 'vout' - represents the index of the specific output in the transaction
    // referred to by 'txid' that the input is using.
    // Since a single transaction can have multiple outputs, 'vout' specifies which of those outputs
    // this input is claiming or spending.
    // 'vout' is an integer that serves as an index into the list of outputs of the transaction
    // identified by 'txid'.
    pub vout: i32,
    // 'signature' - represents a digital signature produced by the sender.
    // The signature is used to prove that the owner of the inputs has authorized the creation of this transaction.
    // The signature is created by signing the new transaction's details with the private key of the sender.
    // This ensures the authenticity and integrity of the transaction, as it proves the sender
    // had the right to use the outputs being spent.
    pub signature: Vec<u8>,
    // 'pub_key' - represents the public key corresponding to the private key used to sign the transaction.
    // 'pub_key' is used to verify the signature attached to the transaction input. It also serves
    // to identify the sender of the funds.
    pub pub_key: Vec<u8>,
}

impl TXInput {
    #[allow(dead_code)]
    // Checks whether the address initiated the transaction
    pub fn can_unlock_output_with(&self, unlocking_data: &[u8]) -> bool {
        let mut pub_key_hash = self.pub_key.clone();
        hash_pub_key(&mut pub_key_hash);
        pub_key_hash == unlocking_data
    }
}

// TXOutput represents a transactional output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32, // The amount of cryptocurrency being transferred
    pub pub_key_hash: Vec<u8>,
}

impl TXOutput {
    pub fn new(value: i32, addr: String) -> Result<Self> {
        let mut txo = TXOutput {
            value,
            pub_key_hash: Vec::new(),
        };
        txo.lock(&addr)?;

        Ok(txo)
    }

    // Signs the output
    fn lock(&mut self, addr: &str) -> Result<()> {
        let pub_key_hash = Address::decode(addr).unwrap().body;
        debug!("lock: {}", addr);
        self.pub_key_hash = pub_key_hash;
        Ok(())
    }

    // Checks if the output can be unlocked with the given unlocking data
    pub fn can_be_unlocked_with(&self, unlocking_data: &[u8]) -> bool {
        self.pub_key_hash == unlocking_data
    }

    // checks if the output can be used by the owner of the public key
    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash == pub_key_hash
    }
}

// collects TXOutputs
// We can use this to identify our transaction output and then sort them by unspent output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutputs {
    pub outputs: Vec<TXOutput>,
}