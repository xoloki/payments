use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, stdout};
use std::str::FromStr;

/*
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref WITHDRAWAL: String = "withdrawal".to_string();
    static ref DEPOSIT: String = "deposit".to_string();
}
*/
#[derive(Debug)]
pub enum UpdateError {
    AccountLocked,
    BadDecimal,
    UnknownTxType,
    InsufficientFunds,
    DuplicateTransaction,
    AlreadyDisputed,
    NotDisputed,
    DisputedWrongClient,
    DisputedTxNotFound,
    InvalidDisputedTxType,
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for UpdateError {
    fn description(&self) -> &str {
        match self {
            UpdateError::AccountLocked => "AccountLocked",
            UpdateError::BadDecimal => "BadDecimal",
            UpdateError::UnknownTxType => "UnknownTxType",
            UpdateError::InsufficientFunds => "InsufficientFunds",
            UpdateError::AlreadyDisputed => "AlreadyDisputed",
            UpdateError::NotDisputed => "NotDisputed",
            UpdateError::DuplicateTransaction => "DuplicateTransaction",
            UpdateError::DisputedWrongClient => "DisputedWrongClient",
            UpdateError::DisputedTxNotFound => "DisputedTxNotFound",
            UpdateError::InvalidDisputedTxType => "InvalidDisputedTxType"
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    tx_type: String,
    client: u16,
    tx: u32,
    #[serde(default)]
    amount: String,
}

#[derive(Debug, Serialize)]
struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new(id: u16) -> Account {
        Account {
            client: id,
            available: dec!(0.0),
            held: dec!(0.0),
            total: dec!(0.0),
            locked: false,
        }
    }

    pub fn rescale(&mut self, scale: u32) {
        self.available.rescale(scale);
        self.held.rescale(scale);
        self.total.rescale(scale);
    }

    pub fn update(&mut self, tx: &Transaction, txs: &mut HashMap<u32, Transaction>, disputes: &mut HashSet<u32>) -> Result<(), UpdateError> {
        if tx.tx_type == "withdrawal" {
            if self.locked {
                return Err(UpdateError::AccountLocked);
            }

            let amount = match Decimal::from_str(&tx.amount) {
                Ok(amt) => amt,
                Err(_) => return Err(UpdateError::BadDecimal)
            };

            if self.available < amount {
                return Err(UpdateError::InsufficientFunds);
            }

            if txs.contains_key(&tx.tx) {
                return Err(UpdateError::DuplicateTransaction);
            }
            
            txs.insert(tx.tx, tx.clone());
            self.available -= amount;
            
            return Ok(());

        } else if tx.tx_type == "deposit" {
            if self.locked {
                return Err(UpdateError::AccountLocked);
            }

            let amount = match Decimal::from_str(&tx.amount) {
                Ok(amt) => amt,
                Err(_) => return Err(UpdateError::BadDecimal)
            };

            if txs.contains_key(&tx.tx) {
                return Err(UpdateError::DuplicateTransaction);
            }
            
            txs.insert(tx.tx, tx.clone());
            self.available += amount;
            
            return Ok(());

        } else if tx.tx_type == "dispute" {
            if disputes.contains(&tx.tx) {
                return Err(UpdateError::AlreadyDisputed);
            }

            let disputed_tx = match txs.get(&tx.tx) {
                Some(dtx) => dtx,
                None => return Err(UpdateError::DisputedTxNotFound)
            };

            if disputed_tx.client != tx.client {
                return Err(UpdateError::DisputedWrongClient);
            }
            
            if disputed_tx.tx_type != "deposit" {
                return Err(UpdateError::InvalidDisputedTxType);
            }
            
            disputes.insert(tx.tx);
            
            let amount = match Decimal::from_str(&disputed_tx.amount) {
                Ok(amt) => amt,
                Err(_) => return Err(UpdateError::BadDecimal)
            };

            self.available -= amount;
            self.held += amount;
            
            return Ok(());

        } else if tx.tx_type == "resolve" || tx.tx_type == "chargeback" {
            if !disputes.contains(&tx.tx) {
                return Err(UpdateError::NotDisputed);
            }

            let disputed_tx = match txs.get(&tx.tx) {
                Some(dtx) => dtx,
                None => return Err(UpdateError::DisputedTxNotFound)
            };

            if disputed_tx.client != tx.client {
                return Err(UpdateError::DisputedWrongClient);
            }
            
            if disputed_tx.tx_type != "deposit" {
                return Err(UpdateError::InvalidDisputedTxType);
            }
            
            disputes.remove(&tx.tx);

            let amount = match Decimal::from_str(&disputed_tx.amount) {
                Ok(amt) => amt,
                Err(_) => return Err(UpdateError::BadDecimal)
            };

            if tx.tx_type == "resolve" {
                self.available += amount;
                self.held -= amount;
            } else {
                self.held -= amount;
                self.locked = true;
            }
            
            return Ok(());

        } else {
            return Err(UpdateError::UnknownTxType);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let files = &args[1..]; // first arg is exe
    let mut accounts = HashMap::new();
    let mut txs = HashMap::new();
    let mut disputes = HashSet::new();
    
    for file in files {
        match update_accounts(file, &mut accounts, &mut txs, &mut disputes) {
            Err(err) => eprintln!("Error reading records from {}: {}", file, err),
            Ok(()) => ()
        }
    }

    let mut wtr = csv::Writer::from_writer(stdout());

    for account in accounts.values_mut() {
        account.rescale(4);
        if let Err(err) = wtr.serialize(&account) {
            eprintln!("Error writing account {}: {}", account.client, err);
        }
    }
}

fn update_accounts(path: &String, accounts: &mut HashMap<u16, Account>, txs: &mut HashMap<u32, Transaction>, disputes: &mut HashSet<u32>) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);

    let mut csv_reader = csv::Reader::from_reader(buf_reader);
    for result in csv_reader.deserialize() {
        let tx: Transaction = result?;
        let account = accounts.entry(tx.client).or_insert(Account::new(tx.client));

        if let Err(err) = account.update(&tx, txs, disputes) {
            eprintln!("Error updating account {} with tx {}: {}", account.client, tx.tx, err); 

        }
    }

    Ok(())
}
