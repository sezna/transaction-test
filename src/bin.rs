//#![deny(missing_docs)]
use csv::ReaderBuilder;
use std::{env, fs::File, io::BufReader};

mod transaction;
pub use transaction::{ClientId, Transaction, TxId};

mod state;
pub use state::State;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let filename = match args.get(1) {
        Some(name) => name,
        None => {
            return Err(
                "Input file name must be provided as the first argument to this program.".into(),
            )
        }
    };
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut rdr = ReaderBuilder::new().flexible(true).from_reader(reader);
    let deserialized_stream = rdr.deserialize::<Transaction>();

    let mut state: State = Default::default();

    for record in deserialized_stream {
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
        state.transact(record);
    }
    println!("{}", state.serialize_to_csv()?);
    Ok(())
}
