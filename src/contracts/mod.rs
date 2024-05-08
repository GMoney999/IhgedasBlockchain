/***************************************************************************************************
    smart_contract.rs

        This module implements smart contract functionalities using the transaction module.
        Smart contracts automatically execute and enforce conditions or rules within transactions.

****************************************************************************************************/

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RateLimitContract {
    pub last_transaction_times: HashMap<String, u64>, // Maps wallet addresses to the last transaction UNIX timestamp
    pub minimum_interval_seconds: u64, // Minimum number of seconds required between transactions
}

impl RateLimitContract {
    pub fn new(minimum_interval_seconds: u64) -> Self {
        RateLimitContract {
            last_transaction_times: HashMap::new(),
            minimum_interval_seconds,
        }
    }

    pub fn execute(&mut self, wallet_address: &str) -> Result<(), String> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)
            .expect("The space time continuum is broken.")
            .as_secs();

        if let Some(last_time) = self.last_transaction_times.get(wallet_address) {
            if current_time - last_time < self.minimum_interval_seconds {
                return Err(format!("Please wait {} more seconds before making another transaction.",
                                   self.minimum_interval_seconds - (current_time - last_time)));
            }
        }

        // Update the last transaction time to the current time
        self.last_transaction_times.insert(wallet_address.to_string(), current_time);

        Ok(())
    }
}
