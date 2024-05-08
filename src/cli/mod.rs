use clap::{Command, arg};
use crate::models::blockchain::{Blockchain};
use crate::transaction::Transaction;
use crate::error::{Result};
use bitcoincash_addr::Address;
use failure::format_err;
use crate::utxoset::UTXOSet;
use crate::wallet::Wallets;
use crate::contracts::{RateLimitContract};
use std::sync::{Mutex};
use lazy_static::{lazy_static};

lazy_static! {
    static ref RATE_LIMIT_CONTRACT: Mutex<RateLimitContract> = Mutex::new(RateLimitContract::new(300)); // 300 seconds interval
}

pub struct Cli {}

impl Cli {
    pub fn new() -> Result<Cli> {
        Ok(Cli {})
    }
    pub fn run(&mut self) -> Result<()> {
        let matches = Command::new("Ihgedas-Blockchain demo")
            .version("0.1")
            .author("Gerami.Sadeghi@gmail.com")
            .about("A rudimentary blockchain")
            .subcommand(
                Command::new("printchain")
                    .about("Print all blocks in the blockchain")
            )
            .subcommand(
                Command::new("getbalance")
                    .about("get balance in the blockchain")
                    .arg(arg!(<ADDRESS>"'The address it gets balance for'"))
            )
            .subcommand(
                Command::new("create")
                    .about("create new blockchain")
                    .arg(arg!(<ADDRESS>"'The address to send the genesis block reward to'"))
            )
            .subcommand(
                Command::new("send")
                    .about("send in the blockchain")
                    .arg(arg!(<FROM>" 'Source wallet address'"))
                    .arg(arg!(<TO>" 'Destination wallet address'"))
                    .arg(arg!(<AMOUNT>" 'Number of tokens'"))
            )
            .subcommand(
                Command::new("createwallet")
                    .about("create a wallet")
            )
            .subcommand(
                Command::new("listaddresses")
                    .about("list all addresses")
            )
            .subcommand(
                Command::new("reindex")
                    .about("reindex UTXO set")
            )
            .get_matches();

        if let Some(ref matches) = matches.subcommand_matches("create") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let address = String::from(address);

                let bc = Blockchain::create_blockchain(address.clone())?;
                let utxo_set = UTXOSet { blockchain: bc };
                utxo_set.reindex()?;

                println!("created blockchain!");
            }
        }

        if let Some(ref matches) = matches.subcommand_matches("getbalance") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let pub_key_hash = Address::decode(address).unwrap().body;
                let bc = Blockchain::new()?;
                let utxo_set = UTXOSet { blockchain: bc };
                let utxos = utxo_set.find_utxos(&pub_key_hash)?;

                let mut balance: i32 = 0;
                for out in utxos.outputs {
                    balance += out.value;
                }
                println!("Balance of '{}': {}", address, balance);
            }
        }

        if let Some(ref matches) = matches.subcommand_matches("send") {
            let from = matches.get_one::<String>("FROM").expect("FROM address required");
            let to = matches.get_one::<String>("TO").expect("TO address required");
            let amount: i32 = matches.get_one::<String>("AMOUNT").expect("Amount required").parse().expect("Invalid amount");

            // Check the rate limit for the 'from' wallet
            let mut contract = RATE_LIMIT_CONTRACT.lock().unwrap();
            match contract.execute(from) {
                Ok(_) => {
                    let bc = Blockchain::new()?;
                    let mut utxo_set = UTXOSet { blockchain: bc };
                    let tx = Transaction::new_utxo(from, to, amount, &utxo_set)?;
                    let cbtx = Transaction::new_coinbase(from.to_string(), String::from("Reward!"))?;
                    let new_block = utxo_set.blockchain.add_block(vec![cbtx, tx])?;
                    utxo_set.update(&new_block)?;
                    println!("Success!");
                },
                Err(e) => {
                    return Err(format_err!("Not enough time has elapsed: {}", e)); // Stop processing if the rate limit is violated
                }
            }
        }

        if let Some(_) = matches.subcommand_matches("reindex") {
            let bc = Blockchain::new()?;
            let utxo_set = UTXOSet { blockchain: bc };
            utxo_set.reindex()?;
            let count = utxo_set.count_transactions()?;
            println!("Done! There are {} transactions in the UTXO set.", count);
        }

        if let Some(_) = matches.subcommand_matches("createwallet") {
            let mut ws = Wallets::new()?;
            let address = ws.create_wallet();
            ws.save_all()?;
            println!("Success! address: {}", address);
        }

        if let Some(_) = matches.subcommand_matches("listaddresses") {
            let ws = Wallets::new()?;
            let addresses = ws.get_all_addresses();

            println!("addresses:");
            for ad in addresses {
                println!("{}", ad);
            }
        }

        #[allow(unused_variables)]
        if let Some(ref matches) = matches.subcommand_matches("printchain") {
            cmd_print_chain()?;
        }

        Ok(())
    }
}

fn cmd_print_chain() -> Result<()> {
    let bc = Blockchain::new()?;

    for block in bc.iter() {
        println!("{:#?}", block);
    }

    Ok(())
}



