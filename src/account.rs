use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

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

/*
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref WITHDRAWAL: String = "withdrawal".to_string();
    static ref DEPOSIT: String = "deposit".to_string();
}
*/

#[derive(Clone, Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: String,
    pub client: u16,
    pub tx: u32,
    #[serde(default)]
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct Account {
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
        if tx.tx_type == "withdrawal" || tx.tx_type == "deposit" {
            if self.locked {
                return Err(UpdateError::AccountLocked);
            }

            let amount = match Decimal::from_str(&tx.amount) {
                Ok(amt) => amt,
                Err(_) => return Err(UpdateError::BadDecimal)
            };

            if tx.tx_type == "withdrawal" && self.available < amount {
                return Err(UpdateError::InsufficientFunds);
            }

            if txs.contains_key(&tx.tx) {
                return Err(UpdateError::DuplicateTransaction);
            }

            txs.insert(tx.tx, tx.clone());

            if tx.tx_type == "withdrawal" {
                self.available -= amount;
            } else {
                self.available += amount;
            }
            
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

