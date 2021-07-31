use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::process;
/*
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref WITHDRAWAL: String = "withdrawal".to_string();
    static ref DEPOSIT: String = "deposit".to_string();
}
*/
pub enum UpdateError {
    BadTxType,
    NoClientID,
    WrongClientID,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    tx_type: String,
    client: u16,
    tx: u16,
    amount: Decimal,
}

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

    pub fn update(tx: &Transaction) -> Result<(), UpdateError> {
        if tx.tx_type == "withdrawal" {
            return Ok(());
        } else if tx.tx_type == "deposit" {
            return Ok(());
        } else {
            return Err(UpdateError::BadTxType);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let files = &args[1..]; // first arg is exe
    let mut clients = HashMap::new();
    
    for file in files {
        match read_records(file, &mut clients) {
            Err(err) => println!("error reading records from {}: {}", file, err),
            Ok(()) => println!("success!")
        }
    }
}

fn read_records(path: &String, accounts: &mut HashMap<u16, Account>) -> Result<(), Box<dyn Error>> {
    println!("Read records from {:?}", path);

    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);

    let mut csv_reader = csv::Reader::from_reader(buf_reader);
    for result in csv_reader.deserialize() {
        let tx: Transaction = result?;
        let account = accounts.entry(tx.client).or_insert(Account::new(tx.client));
        println!("{:?}", tx);
    }

    Ok(())
}
