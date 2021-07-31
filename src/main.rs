use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, stdout};

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
    BadTxType,
    InsufficientFunds,
    DuplicateTransaction,
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for UpdateError {
    fn description(&self) -> &str {
        match self {
            UpdateError::BadTxType => "BadTxType",
            UpdateError::InsufficientFunds => "InsufficientFunds",
            UpdateError::DuplicateTransaction => "DuplicateTransaction"
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    tx_type: String,
    client: u16,
    tx: u32,
    amount: Decimal,
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

    pub fn update(&mut self, tx: &Transaction, txs: &mut HashMap<u32, Transaction>) -> Result<(), UpdateError> {
        if tx.tx_type == "withdrawal" {
            if self.available < tx.amount {
                return Err(UpdateError::InsufficientFunds);
            }

            if txs.contains_key(&tx.tx) {
                return Err(UpdateError::DuplicateTransaction);
            }
            
            txs.insert(tx.tx, tx.clone());
            self.available -= tx.amount;
            
            return Ok(());

        } else if tx.tx_type == "deposit" {
            if txs.contains_key(&tx.tx) {
                return Err(UpdateError::DuplicateTransaction);
            }
            
            txs.insert(tx.tx, tx.clone());
            self.available += tx.amount;
            
            return Ok(());

        } else {
            return Err(UpdateError::BadTxType);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let files = &args[1..]; // first arg is exe
    let mut accounts = HashMap::new();
    let mut txs = HashMap::new();
    
    for file in files {
        match update_accounts(file, &mut accounts, &mut txs) {
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

fn update_accounts(path: &String, accounts: &mut HashMap<u16, Account>, txs: &mut HashMap<u32, Transaction>) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);

    let mut csv_reader = csv::Reader::from_reader(buf_reader);
    for result in csv_reader.deserialize() {
        let tx: Transaction = result?;
        let account = accounts.entry(tx.client).or_insert(Account::new(tx.client));

        if let Err(err) = account.update(&tx, txs) {
            eprintln!("Error updating account {} with tx {}: {}", account.client, tx.tx, err); 

        }
    }

    Ok(())
}
