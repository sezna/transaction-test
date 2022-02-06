use transactions::{State, Transaction};

/// Given an input list of transactions, run it through the state machine and assess the output.
fn harness(input: &str, expected_output: &str) -> bool {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(input.as_bytes());

    let mut state: State = Default::default();

    let _records = reader
        .deserialize()
        .collect::<Result<Vec<Transaction>, _>>()
        .unwrap()
        .into_iter()
        .for_each(|tx| state.transact(tx));

    let output = match state.serialize_to_csv() {
        Ok(o) => o,
        Err(_) => return false,
    };
    println!("EXPECTED\n{}", expected_output);
    println!("\nRECEIVED\n{}", output);
    output == expected_output
}

#[test]
fn client_should_lock() {
    assert!(harness(
        r#"
type,client,tx,amount
deposit,1,1,1.0
deposit,1,2,200
dispute,1,1
chargeback,1,1"#,
        r#"client,available,held,total,locked
1,200,0,200,true
"#
    ));
}

/// A chargeback should be a noop if a dispute was not first filed.
#[test]
fn chargeback_without_dispute() {
    assert!(harness(
        r#"
type,client,tx,amount
deposit,1,1,1.0
deposit,1,2,200
chargeback,1,1"#,
        r#"client,available,held,total,locked
1,201,0,201,false
"#
    ))
}

/// A chargeback cannot follow a resolution of the same dispute.
#[test]
fn chargeback_after_resolution() {
    assert!(harness(
        r#"
type,client,tx,amount
deposit,1,1,1.0
deposit,1,2,200
dispute,1,1
resolve,1,1
chargeback,1,1"#,
        r#"client,available,held,total,locked
1,201,0,201,false
"#
    ));
}

/// If a dispute references a client id that is not part of the transaction it refers to, it should
/// be a noop
#[test]
fn dispute_wrong_client_id() {
    assert!(harness(
        r#"
type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,1
dispute,2,1
chargeback,2,1"#,
        r#"client,available,held,total,locked
1,1,0,1,false
2,1,0,1,false
"#
    ));
}

/// A chargeback'd withdrawal should return money to the client.
#[test]
fn chargeback_withdrawal() {
    assert!(harness(
        r#"
type,client,tx,amount
deposit ,  1,1,  1.0
withdrawal,  1,2,1
dispute, 1,2
chargeback,1,2"#,
        r#"client,available,held,total,locked
1,1,0,1,true
"#
    ));
}
