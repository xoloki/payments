# Payments
A simple toy payments engine that reads a series of transactions from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the state of clients accounts as a CSV.

## Notes

### Withdrawal Transaction Type
The spec doc I was working from used both ```"withdraw"``` and ```"withdrawal"``` to refer to that tx type, the latter in the example data and the former when detailing the fields of the tx types.  The code provides a ```pub const WITHDRAWAL: &str = "..."``` at the top of ```src/payments.rs```.  If you wish to change the string just change it there, and everything should work as expected.

### Disputes
I originally only implemented disputes for deposit transactions.  Disputing withdrawals didn't seem to follow the same semantics.  But on further reflection, in the case where someone deposits, withdraws the deposit, then disputes the deposit, it will be necessary to then chargeback the withdrawal.  This should have the same semantics as a deposit dispute.

### Amounts
Most places in the code use ```rust_decimal::Decimal``` to represent amounts.  But for ```struct Transaction``` I used ```String```.  This was because the various dispute transaction types have an empty string for the amount, and ```rust_decimal::Decimal``` really didn't want to parse it.

Since I'm layering ```rust_decimal``` on top of ```serde``` on top of ```csv```, it didn't seem worth it to try getting ```Decimal``` to behave properly.

## Testing
I provided a complete set of unit tests, which can be run by invoking ```cargo test```.  This tests the library code, but not the main entrypoint.

You can test that with the provided input.csv to verify that basic operations complete, and also trigger all errors:

```
cargo run -- input.csv > output.csv
```

To test a larger dataset, first create it, then run:

```
echo "type,client,tx,amount" > input_large.csv
for (( i=0; i<1000000; i++ )); do
    echo "deposit,1,$i,1.0" >> input_large.csv;
done
cargo run -- input_large.csv > output_large.csv
```

On my machine, it takes about 8 seconds to create the dataset, and 11 seconds to process it.  Processing only needs a small amount of memory since the data is streamed.
