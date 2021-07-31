use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use payments::{Account, Ledger, Transaction};

// bootstrap a ledger with one client that has one deposit tx
pub fn make_ledger(client: u16, tx: u32, amount: Decimal) -> Ledger {
    let mut ledger: Ledger = Default::default();

    assert_eq!(ledger.accounts.len(), 0);
    
    let deposit = Transaction {
        tx_type: "deposit".to_string(),
        client: client,
        tx: tx,
        amount: amount.to_string(),
    };

    ledger.process(&deposit).expect("Failed to process transaction");

    assert_eq!(ledger.accounts.len(), 1);
    
    let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client");

    assert_eq!(account.available, amount);
    assert_eq!(account.total, amount);

    ledger
}

// bootstrap a ledger with one client that has one deposit tx
pub fn make_disputed_ledger(client: u16, tx: u32, amount: Decimal) -> Ledger {
    let mut ledger = make_ledger(client, tx, amount);

    let dispute = Transaction {
        tx_type: "dispute".to_string(),
        client: client,
        tx: tx,
        amount: "".to_string(),
    };

    ledger.process(&dispute).expect("Failed to process dispute");

    assert_eq!(ledger.accounts.len(), 1);

    let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client");
    assert_eq!(account.available, dec!(0.0));
    assert_eq!(account.held, amount);
    assert_eq!(account.total, amount);
    
    ledger
}
