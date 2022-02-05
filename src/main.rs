//#![deny(missing_docs)]
use csv::ReaderBuilder;
use serde::{de::Deserializer, Deserialize, Serialize};
use std::{fs::File, io::BufReader};

mod transaction;
pub use transaction::{ClientId, Transaction, TxId};

mod state;

// TODO: command line interface for input file
// docstrings
// tests
// replace unwrap w/ proper handling
// db state
/// The [TransactionProcessor] handles one incoming transaction at a time. It processes the
/// transaction and applies it to the current state.
/// It also writes the transaction to a database ledger.
struct TransactionProcessor;

/// The [DataStream] holds a streaming reader that reads the input CSV.
struct DataStream;

fn main() {
    let file = File::open("data/transactions.csv").unwrap();
    let reader = BufReader::new(file);

    let mut rdr = ReaderBuilder::new().flexible(true).from_reader(reader);
    let mut deserialized_stream = rdr.deserialize::<Transaction>();
    while let Some(record) = deserialized_stream.next() {
        // You could make this return a result, but I believe `Result` should represent an internal
        // error in the execution of the program that must be handled. Because transactions are user
        // data, we don't want to do an excess of work on malformed user data, which is likely to
        // occur. In fact, it should be considered a _valid_ execution of the function to ignore or
        // omit invalid transactions.
        let record: Transaction = match record {
            Ok(o) => o,
            // continuing here because invalid transactions should be ignored as stated above
            Err(_) => continue,
        };

        println!("{:?}", record);
    }
}
