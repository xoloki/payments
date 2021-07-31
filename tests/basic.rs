use rust_decimal_macros::dec;

use payments::{Account, Ledger, PaymentError, Transaction};

mod helper;

use helper::make_ledger;

#[test]
fn deposit() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let _ledger = make_ledger(client, tx, dec!(100.0));
}

#[test]
fn withdrawal() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_ledger(client, tx, dec!(100.0));
    
    let withdrawal = Transaction {
        tx_type: "withdrawal".to_string(),
        client: client,
        tx: tx+1,
        amount: "100.00".to_string(),
    };

    ledger.process(&withdrawal).expect("Failed to process withdrawal");

    assert_eq!(ledger.accounts.len(), 1);

    let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client");
    assert_eq!(account.available, dec!(0.0));
}

#[test]
fn insufficient_funds() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_ledger(client, tx, dec!(100.0));
    
    let withdrawal = Transaction {
        tx_type: "withdrawal".to_string(),
        client: client,
        tx: tx+1,
        amount: "200.00".to_string(),
    };
    
    match ledger.process(&withdrawal) {
        Ok(()) => assert!(false, "Overdraft withdrawal succeeded"),
        Err(err) => match err {
            PaymentError::InsufficientFunds => (),
            _ => assert!(false, "Overdraft withdrawal failed with wrong error"),
        }
    }

    {
        let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client 0");
        assert_eq!(account.available, dec!(100.0));
    }
}

#[test]
fn unknown_tx_type() {
    let mut ledger: Ledger = Default::default();

    assert_eq!(ledger.accounts.len(), 0);
    
    let depoosit = Transaction {
        tx_type: "depoosit".to_string(),
        client: 0,
        tx: 0,
        amount: "100.00".to_string(),
    };

    match ledger.process(&depoosit) {
        Ok(()) => assert!(false, "Unknown tx type succeeded"),
        Err(err) => match err {
            PaymentError::UnknownTxType => (),
            _ => assert!(false, "Unknown tx type failed with wrong error"),
        }
    }
}

#[test]
fn bad_decimal() {
    let mut ledger: Ledger = Default::default();

    assert_eq!(ledger.accounts.len(), 0);
    
    let deposit = Transaction {
        tx_type: "deposit".to_string(),
        client: 0,
        tx: 0,
        amount: "ABCDE".to_string(),
    };

    match ledger.process(&deposit) {
        Ok(()) => assert!(false, "Bad amount tx succeeded"),
        Err(err) => match err {
            PaymentError::BadDecimal => (),
            _ => assert!(false, "Bad amount tx failed with wrong error"),
        }
    }
}

#[test]
fn duplicate_tx() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_ledger(client, tx, dec!(100.0));

    let withdrawal = Transaction {
        tx_type: "withdrawal".to_string(),
        client: client,
        tx: tx,
        amount: "2.00".to_string(),
    };
    
    match ledger.process(&withdrawal) {
        Ok(()) => assert!(false, "Duplicate transaction succeeded"),
        Err(err) => match err {
            PaymentError::DuplicateTransaction => (),
            _ => assert!(false, "Duplicate transaction failed with wrong error"),
        }
    }
}

#[test]
fn account_locked() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_ledger(client, tx, dec!(100.0));

    {
        let account: &mut Account = ledger.accounts.get_mut(&client).expect("Failed to get account for client");
        account.locked = true;
    }

    let withdrawal = Transaction {
        tx_type: "withdrawal".to_string(),
        client: 0,
        tx: 0,
        amount: "2.00".to_string(),
    };
    
    match ledger.process(&withdrawal) {
        Ok(()) => assert!(false, "Account locked but tx succeeded"),
        Err(err) => match err {
            PaymentError::AccountLocked => (),
            _ => assert!(false, "Account locked but tx failed with wrong error"),
        }
    }
}
