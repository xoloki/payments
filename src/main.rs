use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, stdout};

mod payments;

use payments::{Account, Metadata, Transaction};

fn main() {
    let args: Vec<String> = env::args().collect();
    let files = &args[1..]; // first arg is exe
    let mut accounts = HashMap::new();
    let mut metadata = Default::default();
    
    for file in files {
        match update_accounts(file, &mut accounts, &mut metadata) {
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

fn update_accounts(path: &String, accounts: &mut HashMap<u16, Account>, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);

    let mut csv_reader = csv::Reader::from_reader(buf_reader);
    for result in csv_reader.deserialize() {
        let tx: Transaction = result?;
        let account = accounts.entry(tx.client).or_insert(Account::new(tx.client));

        if let Err(err) = account.update(&tx, meta) {
            eprintln!("Error updating account {} with tx {}: {}", account.client, tx.tx, err); 
        }
    }

    Ok(())
}
