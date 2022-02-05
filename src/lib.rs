//#![deny(missing_docs)]
use csv::ReaderBuilder;
use serde::{de::Deserializer, Deserialize, Serialize};
use std::{fs::File, io::BufReader};

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
