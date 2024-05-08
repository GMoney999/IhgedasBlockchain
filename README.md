# Ihgedas-Blockchain

## Welcome to Ihgedas-Blockchain Demo
This is a rudimentary blockchain implementation using Rust. It allows you to interact with a blockchain system through a command line interface.

### Features
- **Blockchain Creation**: Start a new blockchain with a genesis reward sent to a specific address.
- **Wallet Management**: Create and manage wallets within the blockchain system.
- **Transactions**: Send tokens between addresses and check balances.
- **Blockchain Insights**: Print all blocks or reindex the UTXO set to verify and update transaction outputs.
- **Rate Limiting**: Implement rate limiting on transactions to prevent abuse.

---

### Installation
1. Install Rust and Cargo from the command line


      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh


2. Clone the repository:


      git clone [repo-name]

3. Build the project:


    cargo build --release


---

### Usage
Here are some common commands you might use:

#### Create a Wallet

        
    cargo run createwallet

- Wallet stores Public and Private key pair generate using Ed25519 algorithm
- Returns a 32-bit address encrypted using wallet's public key


#### Create a blockchain instance


    cargo run create [address]

- Creates genesis block, starting a new blockchain. 
- ascribes reward to [address].


#### Check the funds in a wallet


    cargo run getbalance [address]

- Returns number of tokens associated with an address


#### Send funds from one address to another

    
    cargo run send <TO_ADDRESS> <FROM_ADDRESS> <AMOUNT> 

- Creates a new transaction between two wallets
- New block containing transaction info is added to the ledger  



#### List all addresses 


    cargo run listaddresses

- Outputs a list of each address associated with the blockchain


#### Print blockchain ledger 


    cargo run printchain 

- Outputs entire blockchain ledger