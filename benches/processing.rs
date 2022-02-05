use csv::ReaderBuilder;
use std::{env, fs::File, io::BufReader};
use transactions::*;

const DATA: &'static str = "data/large_dataset.csv";

#[bench]
fn iterative_fibonacci(b: &mut Bencher) {
    let file = File::open(DATA).unwrap();
    let reader = BufReader::new(file);

    b.iter(|| {
        let mut state: State = Default::default();
        let mut rdr = ReaderBuilder::new().flexible(true).from_reader(reader);
        let deserialized_stream = rdr.deserialize::<Transaction>();
        for record in deserialized_stream {
            state.transact(record);
        }
    })
}
