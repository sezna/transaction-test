use crate::{ClientId, Transaction, TxId};
use std::collections::HashMap;

mod processed_transaction;
use processed_transaction::ProcessedTransaction;

/// The state of all accounts in the system.
#[derive(Default, Debug)]
pub struct State {
    client_accounts: HashMap<ClientId, ClientAccount>,
    // in the case of disputes, we need to find a transaction by ID. Therefore, we want to
    // prioritize quick lookups and identifications of transactions. We know transaction ids are
    // unique, so we can use them as the index in a vector for quick lookups and relatively quick
    // pushes.
    //
    // I don't know how often disputes/chargebacks/resolutions occur, but I am assuming it is
    // at least an order of magnitude less frequent than pushing processed deposits/withdrawals.
    // Therefore, of the standard collections Rust offers, I'm using a vector for maximal speed
    // when processing a deposit or withdrawal transaction, but trying not to sacrifice dispute
    // speed. A bunch of extremely random and disparate transaction ids would indeed slow this
    // down, but given the data, that seems unlikely. If that is the case, then it would be better
    // to use a different data structure.
    //
    // Given more time, it would make sense to build a custom data structure for faster dispute
    // processing. One that takes advantage of the fact that transactions are _likely_ sequential
    // but not necessarily, like a vector accompanied by a Trie that manages TxId -> Vector Index
    // mapping.
    //
    // In real life this would be a database with an index on the primary key, anyway.
    processed_txns: Vec<Option<ProcessedTransaction>>,
    active_disputes: Vec<TxId>,
    resolved_disputes: Vec<TxId>,
}

/// Represents the state of a specific account for a given client.
#[derive(Default, Debug)]
struct ClientAccount {
    held: f64,
    total: f64,
    locked: bool,
}

impl State {
    /// Given a transaction, effect it upon the state. Write it to the database when done.
    // As there is no desire to report invalid transactions back to the user, this function is
    // infallible.
    // TODO: DB
    pub fn transact(&mut self, transaction: Transaction) {
        use Transaction::*;
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
            Transaction::Unrecognized(_) => return (),
        };
    }

    pub fn withdraw(&mut self, client: ClientId, amount: f64) {
        let client = self.get_client(client);
        client.total -= amount;
    }

    pub fn deposit(&mut self, client: ClientId, amount: f64) {
        let client = self.get_client(client);
        client.total += amount;
    }

    fn get_transaction_slot_mut(&mut self, id: TxId) -> Option<&mut Option<ProcessedTransaction>> {
        self.processed_txns.get_mut(id as usize)
    }

    pub fn dispute(&mut self, client_id: ClientId, tx: TxId) {
        // TODO: if they want a db, use sqlite here to find the amount in the tx db
        match self.get_transaction_slot_mut(tx) {
            Some(Some(ref mut processed_txn)) => {
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
            }
            _ => (),
        };
    }

    fn get_client(&mut self, id: ClientId) -> &mut ClientAccount {
        self.client_accounts
            .entry(id)
            .or_insert_with(Default::default)
    }

    pub fn resolve(&mut self, client_id: ClientId, tx: TxId) {
        let mut tx = self.get_transaction_slot_mut(tx);
        if let Some(Some(ref mut tx)) = tx {
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
        let client = self.get_client(client);
        // TODO: if they want a db, use sqlite here to find the amount in the tx db
        //        client.held -= todo!("Find amount in tx db");
        //        client.amount -= todo!("Find amount in tx db");
        client.locked = true;
        todo!("handle funds");
    }

    /// Ensures there are enough available slots in this vector to hold this index.
    fn ensure_tx_space(&mut self, tx_id: TxId) {
        let tx_id = tx_id as usize;
        if self.processed_txns.len() < tx_id {
            self.processed_txns
                .append(&mut vec![None; tx_id - self.processed_txns.len()]);
        }
    }

    fn insert_processed_deposit(&mut self, client: ClientId, amount: f64, tx_id: TxId) {
        self.ensure_tx_space(tx_id);
        let mut slot = self.get_transaction_slot_mut(tx_id);
        match slot {
            Some(ref mut x) => **x = Some(ProcessedTransaction::new_deposit(client, amount)),
            None => (),
        }
    }
    fn insert_processed_withdrawal(&mut self, client: ClientId, amount: f64, tx_id: TxId) {
        self.ensure_tx_space(tx_id);
        let mut slot = self.get_transaction_slot_mut(tx_id);
        match slot {
            Some(ref mut x) => **x = Some(ProcessedTransaction::new_withdrawal(client, amount)),
            None => (),
        }
    }
}
