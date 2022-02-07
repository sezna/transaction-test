# Transaction Processor
This is a toy example of a streaming application written in Rust.

## Assumptions made 
1. A chargeback on a withdrawal results in an addition of funds to an account. 
1. A dispute on a withdrawal results in no funds frozen, as a dispute on a withdrawal would be conveying that the withdraw is disputed and the client is therefore _owed_ money.
1. Balances _can_ go negative, since overdrafting is a real thing.
1. If the client ID specified in a dispute, chargeback, or resolution do not match the client ID in the transaction they reference; the dispute, chargeback, or resolution is invalid.
1. Any invalid transactions should simply be omitted without being reported.


## General Strategy
I tried to elevate as much of the domain as possible into the type system. Enums like `Transaction` and `ProcessedTransaction` exist to differentiate different states of the transaction. Using this method, it is impossible to represent certain classes of invalid transaction at the type-level. Read [here](https://github.com/sezna/transaction-test/blob/main/src/transaction.rs#L10) for more words on that.

For performance, a streaming CSV buffered reader is used. Since the performance requirements of parsing the input and executing the parsed transactions are distinct, it would be valid to push to a work queue from the parse and take from the work queue with some worker threads. I believe would be out of scope for this, though.

In general, any violation of the transaction format, be it valid or invalid, results in the transaction being omitted.

I did not track available funds in the client account since it could be derived from the other two fields and would therefore be a duplicate source of truth. With floating point math, that can be very problematic.

There are a few unit tests in the application code itself, then some edge case e2e testing and bulk-test-case-generator code in the `tests` directory.
