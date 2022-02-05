use serde::{
    de::{self, Deserializer, MapAccess, SeqAccess, Visitor},
    Deserialize,
};
use std::fmt;

mod error;

pub type ClientId = u16;
pub type TxId = u32;

/// Represents one transaction from the input CSV.
// There are two transaction types that don't have an `amount`. The two options
// that I think are reasonable for representing this data type are the enum I have below, and
// something like:
// ```
// struct Transaction {
//  r#type: Type,
//  client: ClientId,
//  tx: TxId,
//  amount: Option<f64>
// }
// ```
//
// The disadvantage of the struct option is that there are multiple invalid states that can be
// represented within the type. A dispute could be deserialized with an amount, or worse, a deposit
// without an amount. By representing the transaction types with this enum, there are no inherently
// invalid states besides those that we would need runtime dependent typing to solve. I.e., a
// transaction id which refers to a transaction that does not exist.
//
// The downside is that now I have to manually implement deserialize, because this is too
// complicated to do with the macro automatically. But, that's a one time cost and not too hard.
// The benefits will be present throughout the system.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Transaction {
    Deposit {
        client: ClientId,
        tx: TxId,
        amount: f64,
    },
    Withdrawal {
        client: ClientId,
        tx: TxId,
        amount: f64,
    },
    Dispute {
        client: ClientId,
        tx: TxId,
    },
    Resolve {
        client: ClientId,
        tx: TxId,
    },
    Chargeback {
        client: ClientId,
        tx: TxId,
    },
    Unrecognized(String),
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransactionVisitor;

        impl<'de> Visitor<'de> for TransactionVisitor {
            type Value = Transaction;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Transaction")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let transaction_type = TransactionType::from_str(
                    seq.next_element()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?,
                );

                // for performance, bail early if transaction is unrecognized
                if let TransactionType::Unrecognized(s) = transaction_type {
                    return Ok(Transaction::Unrecognized(s));
                }

                let client: ClientId = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                let tx: TxId = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;

                // if this is a Dispute or a Resolve, then there is  no amount
                use TransactionType::*;
                Ok(match transaction_type {
                    Dispute => Transaction::Dispute { client, tx },
                    Resolve => Transaction::Resolve { client, tx },
                    Chargeback => Transaction::Chargeback { client, tx },
                    Deposit => {
                        let amount = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                        Transaction::Deposit { client, tx, amount }
                    }
                    Withdrawal => {
                        let amount = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                        Transaction::Withdrawal { client, tx, amount }
                    }
                    Unrecognized(_) => unreachable!("this was checked for earlier"),
                })
            }
        }
        deserializer.deserialize_seq(TransactionVisitor)
    }
}

#[derive(Debug)]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
    Unrecognized(String),
}

impl TransactionType {
    fn from_str(raw: &str) -> Self {
        use TransactionType::*;
        match raw {
            "deposit" => Deposit,
            "withdrawal" => Withdrawal,
            "dispute" => Dispute,
            "resolve" => Resolve,
            "chargeback" => Chargeback,
            otherwise => Unrecognized(otherwise.into()),
        }
    }
}

#[test]
fn test_transaction_deserialization() {
    let csv = r#"
type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0
dispute,1,6
resolve,1,7
foo
foo,1,2,4,34
chargeback,100,42"#;
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(csv.as_bytes());
    let records = reader
        .deserialize()
        .collect::<Result<Vec<Transaction>, _>>()
        .unwrap();

    assert_eq!(
        records,
        vec![
            Transaction::Deposit {
                client: 1,
                tx: 1,
                amount: 1.0
            },
            Transaction::Deposit {
                client: 2,
                tx: 2,
                amount: 2.0
            },
            Transaction::Deposit {
                client: 1,
                tx: 3,
                amount: 2.0
            },
            Transaction::Withdrawal {
                client: 1,
                tx: 4,
                amount: 1.5
            },
            Transaction::Withdrawal {
                client: 2,
                tx: 5,
                amount: 3.0
            },
            Transaction::Dispute { client: 1, tx: 6 },
            Transaction::Resolve { client: 1, tx: 7 },
            Transaction::Unrecognized("foo".into()),
            Transaction::Unrecognized("foo".into()),
            Transaction::Chargeback {
                client: 100,
                tx: 42
            }
        ]
    );
}
