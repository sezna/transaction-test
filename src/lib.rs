//#![deny(missing_docs)]




mod transaction;
pub use transaction::{ClientId, Transaction, TxId};

mod state;
pub use state::State;

// TODO: command line interface for input file
// docstrings
// tests
// replace unwrap w/ proper handling
// db state
// make processedtxn type so that the type system represents that properly
// clippy
// list assumptions made
// add to funds when withdrawal is charged back
//
