use crate::{ClientId, Transaction, TxId};
use fnv::FnvHashMap;
use std::collections::HashMap;

mod processed_transaction;
use processed_transaction::ProcessedTransaction;

/// The state of all accounts in the system.
#[derive(Default, Debug)]
pub struct State {
    client_accounts: HashMap<ClientId, ClientAccount>,
    // in the case of disputes, we need to find a transaction by ID. Therefore, we want to
    // prioritize quick lookups and identifications of transactions. We know transaction ids are
    // unique, so we can use them as the index in a hash map for quick lookups and relatively quick
    // insertions. We know the keys are u32 values, so we can use a more optimal hashing algorithm.
    // In this case, I chose FNV which is good for small hashes.
    //
    // I don't know how often disputes/chargebacks/resolutions occur, but I am assuming it is
    // at least an order of magnitude less frequent than pushing processed deposits/withdrawals.
    // Therefore, of the standard collections Rust offers, I'm using a hashmap for speed
    // when processing a deposit or withdrawal transaction, but trying not to sacrifice dispute
    // speed. Indeed this will use more memory due to vacant slots than say, a vector.
    //
    // Given more time, it would make sense to build a custom data structure for faster dispute
    // processing. One that takes advantage of the fact that transactions are _likely_ sequential
    // but not necessarily, like a vector accompanied by a Trie that manages TxId -> Vector Index
    // mapping.
    //
    // In real life this would be a database with an index on the primary key, anyway.
    processed_txns: FnvHashMap<TxId, ProcessedTransaction>,
}

/// Represents the state of a specific account for a given client.
#[derive(Default, Debug)]
struct ClientAccount {
    /// total amount of funds currently held in dispute
    held: f64,
    /// total amount of funds in this account
    total: f64,
    /// whether or not the client account is frozen
    locked: bool,
}

impl State {
    /// Given a transaction, effect it upon the state. Write it to the database when done.
    // As there is no desire to report invalid transactions back to the user, this function is
    // infallible.
    // TODO: DB
    pub fn transact(&mut self, transaction: Transaction) {
        match transaction {
            Transaction::Deposit { client, tx, amount } => {
                self.deposit(client, amount);
                self.insert_processed_deposit(client, amount, tx);
            }
            Transaction::Withdrawal { client, tx, amount } => {
                self.withdraw(client, amount);
                self.insert_processed_withdrawal(client, amount, tx);
            }
            Transaction::Dispute { client, tx } => self.dispute(client, tx),
            Transaction::Resolve { client, tx } => self.resolve(client, tx),
            Transaction::Chargeback { client, tx } => self.chargeback(client, tx),
            Transaction::Unrecognized(_) => (),
        };
    }

    pub fn withdraw(&mut self, client: ClientId, amount: f64) {
        let client = self.get_client(client);

        if client.locked {
            return;
        }

        client.total -= amount;
    }

    pub fn deposit(&mut self, client: ClientId, amount: f64) {
        let client = self.get_client(client);

        if client.locked {
            return;
        }

        client.total += amount;
    }

    pub fn dispute(&mut self, client_id: ClientId, tx: TxId) {
        // TODO: if they want a db, use sqlite here to find the amount in the tx db
        if let Some(ref mut processed_txn) = self.processed_txns.get_mut(&tx) {
            // if the client ids don't match, the input is malformed.
            if processed_txn.client_id() != client_id {
                return;
            }

            processed_txn.set_disputed(true);

            // From how I understand the problem, we only want to hold funds if it is
            // a deposit? pending my email question
            let tx_amount = processed_txn.amount();
            if processed_txn.is_deposit() {
                let client = self.get_client(client_id);
                client.held += tx_amount;
            }
        };
    }

    fn get_client(&mut self, id: ClientId) -> &mut ClientAccount {
        self.client_accounts
            .entry(id)
            .or_insert_with(Default::default)
    }

    pub fn resolve(&mut self, client_id: ClientId, tx: TxId) {
        let _mutx = self.processed_txns.get_mut(&tx);
        if let Some(ref mut tx) = self.processed_txns.get_mut(&tx) {
            if tx.client_id() != client_id {
                return;
            }
            tx.set_disputed(false);

            let tx_amount = tx.amount();

            if tx.is_deposit() {
                let mut client = self.get_client(client_id);
                client.held -= tx_amount;
            }
        }

        // TODO: if they want a db, use sqlite here to find the amount in the tx db
    }
    pub fn chargeback(&mut self, client: ClientId, tx: TxId) {
        if let Some(tx) = self.processed_txns.get(&tx) {
            if tx.client_id() != client {
                return;
            }
            if !tx.is_disputed() {
                // disallow chargebacks on transactions that haven't been disputed
                return;
            }
            let tx_amount = tx.amount();
            let tx_is_deposit = tx.is_deposit();
            let client = self.get_client(client);
            client.locked = true;
            if tx_is_deposit {
                client.held -= tx_amount;
                client.total -= tx_amount;
            } else {
                client.total += tx_amount;
            }
        }
    }

    fn insert_processed_deposit(&mut self, client: ClientId, amount: f64, tx_id: TxId) {
        self.processed_txns
            .insert(tx_id, ProcessedTransaction::new_deposit(client, amount));
    }
    fn insert_processed_withdrawal(&mut self, client: ClientId, amount: f64, tx_id: TxId) {
        self.processed_txns
            .insert(tx_id, ProcessedTransaction::new_withdrawal(client, amount));
    }

    pub fn serialize_to_csv(self) -> Result<String, csv::Error> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record(&["client", "available", "held", "total", "locked"])?;
        // sort client accounts for testability
        let mut client_accounts = self
            .client_accounts
            .into_iter()
            .collect::<Vec<(ClientId, ClientAccount)>>();
        client_accounts.sort_by_key(|(id, _)| *id);
        for (
            id,
            ClientAccount {
                total,
                held,
                locked,
            },
        ) in client_accounts
        {
            wtr.write_record(&[
                id.to_string(),
                (total - held).to_string(),
                held.to_string(),
                total.to_string(),
                locked.to_string(),
            ])?;
        }
        Ok(String::from_utf8(wtr.into_inner().unwrap()).unwrap())
    }
}
