# Transaction Processor
This is a toy example of a streaming application written in Rust.

## Assumptions made 
1. A chargeback on a withdrawal results in an addition of funds to an account. 
1. A dispute on a withdrawal results in no funds frozen, as a dispute on a withdrawal would be conveying that the withdraw is disputed and the client is therefore _owed_ money.
1. Balances _can_ go negative, since overdrafting is a real thing.
1. If the client ID specified in a dispute, chargeback, or resolution do not match the client ID in the transaction they reference; the dispute, chargeback, or resolution is invalid.
1. Any invalid transactions should simply be omitted without being reported.
