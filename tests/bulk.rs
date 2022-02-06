//! test bulk processing

use csv::ReaderBuilder;
use std::{
    fs::File,
    io::{BufReader, Read},
};
use transactions::*;

/// process an input file and compare it to an output file
fn harness(infile: &str, outfile: &str) -> bool {
    let file = File::open(infile).unwrap();
    let reader = BufReader::new(file);

    let mut rdr = ReaderBuilder::new().flexible(true).from_reader(reader);
    let deserialized_stream = rdr.deserialize::<Transaction>();

    let mut state: State = Default::default();

    for record in deserialized_stream {
        let record: Transaction = match record {
            Ok(o) => o,
            Err(_) => continue,
        };
        state.transact(record);
    }

    let mut expected_output = String::new();
    File::open(outfile)
        .unwrap()
        .read_to_string(&mut expected_output)
        .unwrap();

    let serialized_output = state.serialize_to_csv().unwrap();

    serialized_output == expected_output
}

#[test]
fn bulk_1() {
    assert!(harness(
        "data/bulk_1.csv",
        "data/bulk_1_expected_output.csv",
    ));
}

#[test]
fn bulk_2() {
    assert!(harness(
        "data/bulk_2.csv",
        "data/bulk_2_expected_output.csv",
    ));
}
