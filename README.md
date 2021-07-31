# payments
A simple toy payments engine that reads a series of transactions from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the state of clients accounts as a CSV.

## testing
You can test with the provided input.csv to verify that basic operations complete, and also trigger all errors:

```
cargo run -- input.csv > output.csv
```

To test a larger dataset, first create it, then run:

```
echo "type,client,tx,amount" > input_large.csv; for (( i=0; i<1000000; i++ )); do echo "deposit,1,$i,1.0" >> input_large.csv; done
cargo run -- input_large.csv > output_large.csv
```

You can also run unit tests:

```
cargo test
```
