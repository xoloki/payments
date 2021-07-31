use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

// the set of errors which can happen during
#[derive(Debug)]
pub enum PaymentError {
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

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PaymentError {
    fn description(&self) -> &str {
        match self {
            PaymentError::AccountLocked => "AccountLocked",
            PaymentError::BadDecimal => "BadDecimal",
            PaymentError::UnknownTxType => "UnknownTxType",
            PaymentError::InsufficientFunds => "InsufficientFunds",
            PaymentError::AlreadyDisputed => "AlreadyDisputed",
            PaymentError::NotDisputed => "NotDisputed",
            PaymentError::DuplicateTransaction => "DuplicateTransaction",
            PaymentError::DisputedWrongClient => "DisputedWrongClient",
            PaymentError::DisputedTxNotFound => "DisputedTxNotFound",
            PaymentError::InvalidDisputedTxType => "InvalidDisputedTxType"
        }
    }
}

// i tried to use string constants but rust got all rusty on me
/*
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref WITHDRAWAL: String = "withdrawal".to_string();
    static ref DEPOSIT: String = "deposit".to_string();
}
*/

// a client transaction, deserialized from input
#[derive(Clone, Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: String,
    pub client: u16,
    pub tx: u32,
    #[serde(default)]
    pub amount: String,
}

// metadata about all client transactions, used in disputes
#[derive(Debug, Default)]
pub struct Metadata {
    txs: HashMap<u32, Transaction>,
    disputes: HashSet<u32>,
}

// current state of a client account, will be serialized as output
#[derive(Debug, Serialize)]
pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    // ctor
    pub fn new(id: u16) -> Account {
        Account {
            client: id,
            available: dec!(0.0),
            held: dec!(0.0),
            total: dec!(0.0),
            locked: false,
        }
    }

    // process the passed transaction for this account
    pub fn process(&mut self, tx: &Transaction, meta: &mut Metadata) -> Result<(), PaymentError> {
        if tx.tx_type == "withdrawal" || tx.tx_type == "deposit" {
            if self.locked {
                return Err(PaymentError::AccountLocked);
            }

            let amount = match Decimal::from_str(&tx.amount) {
                Ok(amt) => amt,
                Err(_) => return Err(PaymentError::BadDecimal)
            };

            if tx.tx_type == "withdrawal" && self.available < amount {
                return Err(PaymentError::InsufficientFunds);
            }

            if meta.txs.contains_key(&tx.tx) {
                return Err(PaymentError::DuplicateTransaction);
            }

            meta.txs.insert(tx.tx, tx.clone());

            if tx.tx_type == "withdrawal" {
                self.available -= amount;
            } else {
                self.available += amount;
            }
            
            return Ok(());

        } else if tx.tx_type == "dispute" {
            if meta.disputes.contains(&tx.tx) {
                return Err(PaymentError::AlreadyDisputed);
            }

            let disputed_tx = match meta.txs.get(&tx.tx) {
                Some(dtx) => dtx,
                None => return Err(PaymentError::DisputedTxNotFound)
            };

            if disputed_tx.client != tx.client {
                return Err(PaymentError::DisputedWrongClient);
            }
            
            if disputed_tx.tx_type != "deposit" {
                return Err(PaymentError::InvalidDisputedTxType);
            }
            
            meta.disputes.insert(tx.tx);
            
            let amount = match Decimal::from_str(&disputed_tx.amount) {
                Ok(amt) => amt,
                Err(_) => return Err(PaymentError::BadDecimal)
            };

            self.available -= amount;
            self.held += amount;
            
            return Ok(());

        } else if tx.tx_type == "resolve" || tx.tx_type == "chargeback" {
            if !meta.disputes.contains(&tx.tx) {
                return Err(PaymentError::NotDisputed);
            }

            let disputed_tx = match meta.txs.get(&tx.tx) {
                Some(dtx) => dtx,
                None => return Err(PaymentError::DisputedTxNotFound)
            };

            if disputed_tx.client != tx.client {
                return Err(PaymentError::DisputedWrongClient);
            }
            
            if disputed_tx.tx_type != "deposit" {
                return Err(PaymentError::InvalidDisputedTxType);
            }
            
            meta.disputes.remove(&tx.tx);

            let amount = match Decimal::from_str(&disputed_tx.amount) {
                Ok(amt) => amt,
                Err(_) => return Err(PaymentError::BadDecimal)
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
            return Err(PaymentError::UnknownTxType);
        }
    }

    // rescale all the decimal vars for uniform output
    pub fn rescale(&mut self, scale: u32) {
        self.available.rescale(scale);
        self.held.rescale(scale);
        self.total.rescale(scale);
    }
}
