use rust_decimal_macros::dec;

use payments::{Account, PaymentError, Transaction, WITHDRAWAL, DISPUTE, RESOLVE, CHARGEBACK};

mod helpers;

use helpers::{make_ledger, make_disputed_ledger};

#[test]
fn dispute_deposit() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let _ledger = make_disputed_ledger(client, tx, dec!(100.0));
}
    
#[test]
fn resolve_deposit() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_disputed_ledger(client, tx, dec!(100.0));
    
    let resolve = Transaction {
        tx_type: RESOLVE.to_string(),
        client: client,
        tx: tx,
        amount: "".to_string(),
    };

    ledger.process(&resolve).expect("Failed to process resolve");

    assert_eq!(ledger.accounts.len(), 1);

    let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client");
    assert_eq!(account.available, dec!(100.0));
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.total, dec!(100.0));
}

#[test]
fn chargeback_deposit() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_disputed_ledger(client, tx, dec!(100.0));
    
    let chargeback = Transaction {
        tx_type: CHARGEBACK.to_string(),
        client: client,
        tx: tx,
        amount: "".to_string(),
    };

    ledger.process(&chargeback).expect("Failed to process chargeback");

    assert_eq!(ledger.accounts.len(), 1);

    let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client");
    assert_eq!(account.available, dec!(0.0));
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.total, dec!(0.0));
}

#[test]
fn already_disputed() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_disputed_ledger(client, tx, dec!(100.0));
    
    let dispute = Transaction {
        tx_type: DISPUTE.to_string(),
        client: client,
        tx: tx,
        amount: "".to_string(),
    };

    match ledger.process(&dispute) {
        Ok(()) => assert!(false, "Double dispute succeeded"),
        Err(err) => match err {
            PaymentError::AlreadyDisputed => (),
            _ => assert!(false, "Double dispute failed with wrong error"),
        }
    }

    {
        let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client 0");
        assert_eq!(account.available, dec!(0.0));
    }
}

#[test]
fn not_disputed() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_disputed_ledger(client, tx, dec!(100.0));
    
    let resolve = Transaction {
        tx_type: RESOLVE.to_string(),
        client: client,
        tx: tx,
        amount: "".to_string(),
    };

    ledger.process(&resolve).expect("Failed to process resolve");

    assert_eq!(ledger.accounts.len(), 1);

    let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client");
    assert_eq!(account.available, dec!(100.0));
    assert_eq!(account.held, dec!(0.0));

    match ledger.process(&resolve) {
        Ok(()) => assert!(false, "Not disputed succeeded"),
        Err(err) => match err {
            PaymentError::NotDisputed => (),
            _ => assert!(false, "Not disputed failed with wrong error"),
        }
    }
    
}

#[test]
fn disputed_wrong_client() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_ledger(client, tx, dec!(100.0));
    
    let deposit = Transaction {
        tx_type: "deposit".to_string(),
        client: client+1,
        tx: tx+1,
        amount: "100.00".to_string(),
    };

    ledger.process(&deposit).expect("Failed to process deposit");

    assert_eq!(ledger.accounts.len(), 2);

    let dispute = Transaction {
        tx_type: DISPUTE.to_string(),
        client: client+1,
        tx: tx,
        amount: "".to_string(),
    };

    match ledger.process(&dispute) {
        Ok(()) => assert!(false, "Disputed wrong client succeeded"),
        Err(err) => match err {
            PaymentError::DisputedWrongClient => (),
            _ => assert!(false, "Disputed wrong client failed with wrong error"),
        }
    }
}

#[test]
fn disputed_tx_not_found() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_ledger(client, tx, dec!(100.0));
    
    let dispute = Transaction {
        tx_type: DISPUTE.to_string(),
        client: client,
        tx: tx+1,
        amount: "".to_string(),
    };

    match ledger.process(&dispute) {
        Ok(()) => assert!(false, "Disputed tx not found succeeded"),
        Err(err) => match err {
            PaymentError::DisputedTxNotFound => (),
            _ => assert!(false, "Disputed tx not found failed with wrong error"),
        }
    }
}

#[test]
fn dispute_withdrawal() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_ledger(client, tx, dec!(100.0));
    
    let withdrawal = Transaction {
        tx_type: WITHDRAWAL.to_string(),
        client: client,
        tx: tx+1,
        amount: "50.00".to_string(),
    };

    ledger.process(&withdrawal).expect("Failed to process withdrawal");

    let dispute = Transaction {
        tx_type: DISPUTE.to_string(),
        client: client,
        tx: tx+1,
        amount: "".to_string(),
    };

    ledger.process(&dispute).expect("Failed to dispute withdrawal");

    {
        let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client");
        assert_eq!(account.available, dec!(50.0));
        assert_eq!(account.held, dec!(0.0));
        assert_eq!(account.total, dec!(50.0));
        assert_eq!(account.locked, true);
    }
}

#[test]
fn resolve_withdrawal() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_ledger(client, tx, dec!(100.0));
    
    let withdrawal = Transaction {
        tx_type: WITHDRAWAL.to_string(),
        client: client,
        tx: tx+1,
        amount: "50.00".to_string(),
    };

    ledger.process(&withdrawal).expect("Failed to process withdrawal");

    let dispute = Transaction {
        tx_type: DISPUTE.to_string(),
        client: client,
        tx: tx+1,
        amount: "".to_string(),
    };

    ledger.process(&dispute).expect("Failed to dispute withdrawal");

    let resolve = Transaction {
        tx_type: RESOLVE.to_string(),
        client: client,
        tx: tx+1,
        amount: "".to_string(),
    };

    ledger.process(&resolve).expect("Failed to resolve withdrawal");

    {
        let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client");
        assert_eq!(account.available, dec!(50.0));
        assert_eq!(account.held, dec!(0.0));
        assert_eq!(account.total, dec!(50.0));
        assert_eq!(account.locked, false);
    }
}

#[test]
fn chargeback_withdrawal() {
    let client: u16 = 0;
    let tx: u32 = 0;
    let mut ledger = make_ledger(client, tx, dec!(100.0));
    
    let withdrawal = Transaction {
        tx_type: WITHDRAWAL.to_string(),
        client: client,
        tx: tx+1,
        amount: "50.00".to_string(),
    };

    ledger.process(&withdrawal).expect("Failed to process withdrawal");

    let dispute = Transaction {
        tx_type: DISPUTE.to_string(),
        client: client,
        tx: tx+1,
        amount: "".to_string(),
    };

    ledger.process(&dispute).expect("Failed to dispute withdrawal");

    let chargeback = Transaction {
        tx_type: CHARGEBACK.to_string(),
        client: client,
        tx: tx+1,
        amount: "".to_string(),
    };

    ledger.process(&chargeback).expect("Failed to chargeback withdrawal");

    {
        let account: &Account = ledger.accounts.get(&client).expect("Failed to get account for client");
        assert_eq!(account.available, dec!(50.0));
        assert_eq!(account.held, dec!(0.0));
        assert_eq!(account.total, dec!(50.0));
        assert_eq!(account.locked, true);
    }
}

