/***************************************************************************************************
    transaction.rs

        This module focuses on the creation, signing, verification, and manipulation of transactions.

        This module defines the structure and operations for transactions within a blockchain context,
        analogous to financial transactions in traditional banking,
        but extended to include data and other blockchain-specific features.

            1) handles the creation of new transactions,
            2) handles standard and coinbase (reward) transactions
            3) ensures transactions are correctly signed for security
            4) verifies their integrity before acceptance into the blockchain.

****************************************************************************************************/

// Transactions are comprised of inputs (TXInput) and outputs (TXOutput),
// where each input references a previous transaction's output,
// and each output specifies how many coins are being transferred and who can claim them.

use std::collections::HashMap;
use crate::error::{Result};
use crate::tx::{TXInput, TXOutput};
use crypto::sha2::{Sha256};
use crypto::digest::{Digest};
use crypto::{ed25519};
use failure::{format_err};
use serde::{Serialize, Deserialize};
use log::{error};
use crate::utxoset::UTXOSet;
use crate::wallet::{hash_pub_key, Wallets};


/***************************************************************************************************
    "Transaction" struct

        Defines the Transaction struct with fields for the
            transaction ID (id) - uniquely identifies each transaction
            inputs (vin) - specifies where the value is coming from
            outputs (vout) - specifies where the value is going to

****************************************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String, // Transaction hash
    pub vin: Vec<TXInput>, // list of transaction inputs
    pub vout: Vec<TXOutput>, // list of transaction outputs
}

impl Transaction {
    /***********************************************************************************************

        new_coinbase() creates a new coinbase transaction

            Special transactions that generate new currency as a reward for mining a new block.

            They have no inputs and a single output directing the reward to the miner's address.

            A default message or custom data can be included.

    ***********************************************************************************************/
    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        // If no data is provided to the function, a default message is constructed using the recipient's address.
        // This data field often includes arbitrary data or messages, but here it's used to indicate the reward's recipient.
        if data.is_empty() {
            data += &format!("Reward to '{}'", to);
        }

        // Initialize a new Transaction struct
        let mut tx = Transaction {
            id: String::new(), // An empty string for the transaction ID, to be calculated
            vin: vec![ // The vector of transaction inputs. For a coinbase transaction, this has a special form:
                       TXInput {
                           txid: String::new(), // An empty string as TXID, since there's no previous transaction to reference
                           vout: -1, // A special index value (-1) indicating this is a coinbase transaction
                           signature: Vec::new(), // An empty signature, as there's no need to sign a coinbase transaction
                           pub_key: Vec::from(data.as_bytes()), // Use the provided data (or the default message) as the "public key".
                       }
            ],
            vout: vec![TXOutput::new(100, to)?], // A single transaction output creating 100 units of currency, awarded to the 'to' address
        };

        // Calculate and set the transaction's ID based on its contents, including its inputs and outputs.
        tx.id = tx.hash()?;

        // Return the constructed coinbase transaction, ready to be added to a block and processed by the blockchain.
        Ok(tx)
    }

    /***********************************************************************************************

        new_utxo() creates a new standard transaction

            Transfers currency from one address to another.

            It requires identifying spendable outputs (UTXOs) from previous transactions
            that the sender can use as inputs and creating outputs for the recipient(s).

    ***********************************************************************************************/
    pub fn new_utxo(to: &str, from: &str, amount: i32, bc: &UTXOSet) -> Result<Transaction> {
        // Initialize a vector to hold the transaction inputs.
        let mut vin = Vec::new();

        // Initialize the wallets and retrieve it
        let wallets = Wallets::new()?;

        // Retrieve the sender's wallet from the wallet system.
        // If not found, return an error.
        let wallet = match wallets.get_wallet(from) {
            Some(w) => w,
            None => return Err(format_err!("source wallet not found")),
        };

        // Check if the recipient's wallet address exists in the wallet system.
        // If not, returns an error.
        if let None = wallets.get_wallet(&to) {
            return Err(format_err!("destination wallet not found"));
        }

        // Prepare the sender's public key hash for use in finding spendable outputs.
        let mut pub_key_hash = wallet.public_key.clone();
        hash_pub_key(&mut pub_key_hash);

        // Find spendable outputs (UTXOs) for the sender's wallet that can cover the 'amount'.
        let acc_v = bc.find_spendable_outputs(&pub_key_hash, amount)?;

        // Check if sufficient funds are available.
        // If not, return an error indicating insufficient funds.
        if acc_v.0 < amount {
            error!("Insufficient funds");
            return Err(format_err!("Insufficient funds! current balance: {}", acc_v.0));
        }

        // For each spendable output found, create a transaction input referencing it.
        for tx in acc_v.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: tx.0.clone(), // The ID of the transaction the output is from
                    vout: out, // The index of the output in the transaction
                    signature: Vec::new(), // Initially empty; to be filled in during the signing process
                    pub_key: wallet.public_key.clone(), // The public key of the sender (for verifying the signature)
                };
                vin.push(input);
            }
        }

        // Prepare the transaction output(s)
        let mut vout = vec![TXOutput::new(amount, to.to_string())?];

        // If there's change (the total spendable amount exceeds the transfer amount),
        // create an additional output sending the change back to the sender.
        if acc_v.0 > amount {
            vout.push(TXOutput::new(acc_v.0 - amount, from.to_string())?)
        }

        // Construct the new transaction with the prepared inputs and outputs
        let mut tx = Self {
            id: String::new(), // Initially empty; to be generated based on the transaction's content.
            vin,
            vout,
        };

        // Generate a unique ID for the transaction based on its contents
        tx.id = tx.hash()?;

        // Sign the transaction with the sender's private key, authorizing the inputs for spending.
        bc.blockchain.sign_transaction(&mut tx, &wallet.secret_key)?;

        // Return the successfully created and signed transaction
        Ok(tx)
    }





    /***********************************************************************************************

        sign() signs a transaction using a private key.

            This is crucial for authenticating the transaction's origin and preventing unauthorized
            spending of funds.

            The signing process involves creating a trimmed copy of the transaction
            to avoid circular dependencies, then generating and attaching a digital
            signature to each input.

    ***********************************************************************************************/
    pub fn sign(
        &mut self, // The transaction to sign, mutable to allow modification (adding the signature)
        private_key: &[u8], // The private key used to sign the transaction.
        prev_txs: HashMap<String, Self>, // A map of previous transactions that are referenced by the inputs of this transaction.
    ) -> Result<()> {
        // If the transaction is a coinbase transaction (a transaction with no inputs, creating new coins),
        // it doesn't need to be signed, so the function returns successfully immediately.
        if self.is_coinbase() {
            return Ok(());
        }

        // Iterate through each input of the transaction to check the validity of referenced previous transactions.
        for vin in &self.vin {
            // Retrieve the previous transaction referenced by this input. If it's missing or incorrect, return an error.
            if prev_txs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: Previous transaction is not correct."));
            }
        }

        // Create a trimmed copy of the transaction to sign.
        // This copy excludes the signatures from the inputs to avoid circular dependencies when hashing.
        let mut tx_copy = self.trim_copy();

        // Iterates over each input in the trimmed copy of the transaction
        for input_id in 0..tx_copy.vin.len() {
            // Retrieve the previous transaction referenced by the current input
            let prev_tx = prev_txs.get(&tx_copy.vin[input_id].txid).unwrap();

            // Clear any existing signature in the input, preparing it for a new signature
            tx_copy.vin[input_id].signature.clear();

            // Temporarily sets the public key in the input to the hash of the public key from the referenced output.
            // This is necessary for correctly hashing the transaction during the signing process.
            tx_copy.vin[input_id].pub_key = prev_tx.vout[tx_copy.vin[input_id].vout as usize]
                .pub_key_hash
                .clone();

            // Hash the modified transaction to produce a unique identifier for signing
            tx_copy.id = tx_copy.hash()?;

            // Clear the public key in the input after hashing, as it is no longer needed
            tx_copy.vin[input_id].pub_key = Vec::new();

            // Generate a digital signature using the transaction's hash and the provided private key
            let signature = ed25519::signature(tx_copy.id.as_bytes(), private_key);

            // Assign the generated signature to the corresponding input in the original transaction
            self.vin[input_id].signature = signature.to_vec();
        }

        // Return successfully after signing all inputs
        Ok(())
    }






    /***********************************************************************************************

        verify() ensures a transaction's authenticity and integrity.

            This function checks that each input is properly signed and authorized
            by the rightful owners.

            This involves verifying digital signatures against the transaction data
            and the public keys associated with each input.

    ***********************************************************************************************/
    #[allow(dead_code)]
    pub fn verify(&mut self, prev_txs: HashMap<String, Self>) -> Result<bool> {
        // Coinbase Transactions are always considered valid as they introduce new coins and have no inputs to verify.
        if self.is_coinbase() {
            return Ok(true);
        }

        // Iterate through each input of the transaction to check the validity of referenced previous transactions.
        for vin in &self.vin {
            // Retrieve the previous transaction referenced by this input. If it's missing or incorrect, return an error.
            if prev_txs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: Previous transaction is not correct."));
            }
        }

        // Create a trimmed copy of the transaction to prepare for signature verification.
        // This involves removing potentially mutable parts, like signatures, to ensure a consistent data structure for hashing.
        let mut tx_copy = self.trim_copy();

        // Iterate over each input of the transaction again, this time for signature verification.
        for input_id in 0..self.vin.len() {
            // Retrieve the corresponding previous transaction for the current input.
            let prev_tx = prev_txs.get(&self.vin[input_id].txid).unwrap();

            // Clear the signature of the current input in the trimmed copy to ensure the hash is consistent.
            tx_copy.vin[input_id].signature.clear();

            // Temporarily set the public key of the input to the hash of the public key found in the referenced output.
            // This is necessary for computing the transaction hash for verification.
            tx_copy.vin[input_id].pub_key = prev_tx.vout[self.vin[input_id].vout as usize]
                .pub_key_hash
                .clone();

            // Hash the modified transaction copy to get a consistent identifier for signature verification.
            tx_copy.id = tx_copy.hash()?;

            // Clear the public key again to prevent its misuse after verification.
            tx_copy.vin[input_id].pub_key = Vec::new();

            // Verify the signature of the current input against the hash of the transaction copy.
            // If any signature fails to verify, return false indicating the transaction is invalid.
            if !ed25519::verify(
                &tx_copy.id.as_bytes(),
                &self.vin[input_id].pub_key,
                &self.vin[input_id].signature,
            ) {
                return Ok(false);
            }
        }

        // If all inputs are successfully verified, return true indicating the transaction is valid.
        Ok(true)
    }




    /***********************************************************************************************

         trim_copy() creates a trimmed copy of the transaction.

            Trimming here refers to removing or clearing certain fields that should not be included
            during the hashing or signature verification process.

                (e.g. signatures and public keys in inputs.).


           This trimmed transaction copy is typically used in scenarios where the transaction
           needs to be hashed (for creating a new signature or for verification purposes).

           By removing mutable fields like signatures, the resulting hash can serve as
           a stable identifier or be securely signed without risk of circular dependencies.

    ***********************************************************************************************/
    fn trim_copy(&self) -> Self {
        // Initialize empty vectors to hold the trimmed versions of the transaction inputs (vin) and outputs (vout).
        let mut vin = Vec::new();
        let mut vout = Vec::new();

        // Iterate over each input in the original transaction.
        for v in &self.vin {
            // Create a trimmed copy of each input. The transaction ID (txid) and output index (vout) are preserved,
            // but the signature and public key fields are cleared.
            // This is because the signature should not be part of the data that the signature itself is signing (to avoid circular dependency),
            // and the public key is not needed for the purpose this trimmed transaction will serve (e.g., generating a transaction hash for signing).
            vin.push(TXInput {
                txid: v.txid.clone(), // Clone the txid of the input to preserve the reference to the previous transaction.
                vout: v.vout.clone(), // Clone the output index directly as it's a simple numerical value.
                signature: Vec::new(), // Clear the signature field
                pub_key: Vec::new(), // Clear the public key field
            });
        }

        // Iterate over each output in the original transaction.
        for v in &self.vout {
            // Create a copy of each output. Unlike inputs, outputs are copied as is,
            // except that the structure may be trimmed or altered based on the implementation.
            // In this case, outputs are copied without modification,
            // indicating that all fields in an output are relevant for the transaction's integrity and its verification process.
            vout.push(TXOutput {
                value: v.value, // Copy the value of the output, which indicates the amount of cryptocurrency being transferred.
                pub_key_hash: v.pub_key_hash.clone(), // Clone the public key hash, which identifies the recipient of the output.
            });
        }

        // Return a new instance of the transaction (Self) constructed with the trimmed inputs and outputs.
        // The transaction ID (id) is preserved as is, which is important for identifying the transaction uniquely.
        Self {
            id: self.id.clone(), // Clone the transaction ID
            vin, // Set the trimmed inputs
            vout, // Set the trimmed (in this case, unchanged) outputs
        }
    }








    /***********************************************************************************************

        hash() generates a unique hash for a transaction.

            This hash serves as the transaction's ID and is used in the verification process.

    ***********************************************************************************************/
    pub fn hash(&mut self) -> Result<String> {
        // Create a string to hold the hash
        self.id = String::new();

        // Serialize the transaction data
        let data = bincode::serialize(self)?;

        // Create a hasher
        let mut hasher = Sha256::new();

        // Input the serialized data into the hasher
        hasher.input(&data[..]);

        // Return the hashed string
        Ok(hasher.result_str())
    }





    /***********************************************************************************************

        is_coinbase() determines if a transaction is a coinbase transaction.

            Coinbase Transactions have specific characteristics:

                1) A single input with no previous transaction ID

                2) An output index of -1.

    ***********************************************************************************************/
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }
}
