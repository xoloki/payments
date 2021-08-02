# Payments
A simple toy payments engine that reads a series of transactions from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the state of clients accounts as a CSV.

## Notes

### Withdrawal Transaction Type
The spec doc I was working from used both ```"withdraw"``` and ```"withdrawal"``` to refer to that tx type, the latter in the example data and the former when detailing the fields of the tx types.  The code provides a ```pub const WITHDRAWAL: &str = "..."``` at the top of ```src/payments.rs```.  If you wish to change the string just change it there, and everything should work as expected.

### Disputes
The spec doc was also ambiguous in regards to disputes.  The description of how a dispute should be handled in terms of the ```Account``` balances (subtract the disputed amount from ```available```, and put it in ```held```; resolve moves back to ```available```, while chargebacks take the ```held``` amount) seemed to only apply to deposits, not withdrawals.  So I originally only implemented disputes for deposit transactions.

But certainly there are disputes on withdrawals, so I thought for a long time about how those should work.  Here's the scenario that cemented my thinking.  It involves a client depositing, then withdrawing, then reversing the deposit.  At this point the client should have lost money equal to the amount fraudulently withdrawn.

Client deposits ```100.00```, then withdraws ```50.00```.  Client then disputes the deposit, ultimately resulting in a chargeback, leaving the client with ```-50.00``` both ```available``` and ```total``` (and also locked).  The processor then disputes the withdrawal, leaving the client ```Account``` with ```-100.00``` ```available```, ```50.00``` ```held```, ```-50.00``` ```total```.  Processor then charges back the withdrawal, removing the ```held``` amount, and decreasing ```total``` accordingly, such that it and ```available``` are now ```-100.00```.

This seems wrong.  The client should only need to deposit ```50.00``` to get back to zero (i.e. give back the withdrawn amount).  But the deposit dispute algorithm instead puts the client double that negative.

So my withdrawal dispute algorithm is as follows: upon dispute, simply lock the account.  A resolve clears the lock, while a chargeback leaves it in place.  At no time do the balances change, and were the client to simply return the withdrawal the account would no longer have any net change.

If the withdrawal chargeback results in a return of funds externally, then these funds can be added back to the account.  But that's external to this API.

### Amounts
Most places in the code use ```rust_decimal::Decimal``` to represent amounts.  But for ```struct Transaction``` I used ```String```.  This was because the various dispute transaction types have an empty string for the amount, and ```Decimal``` really didn't want to parse it.

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
