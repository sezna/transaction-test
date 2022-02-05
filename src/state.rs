use crate::{ClientId, Transaction, TxId};
use std::collections::HashMap;

/// The state of all accounts in the system.
struct State {
    client_accounts: HashMap<ClientId, ClientAccount>,
}

/// Represents the state of a specific account for a given client.
#[derive(Default)]
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
            }
            Transaction::Withdrawal { client, tx, amount } => {
                self.withdraw(client, amount);
            }
            Transaction::Dispute { client, tx } => self.dispute(client, tx),
            Transaction::Resolve { client, tx } => self.resolve(client, tx),
            Transaction::Chargeback { client, tx } => self.resolve(client, tx),
            Transaction::Unrecognized(_) => (),
        }
    }

    pub fn withdraw(&mut self, client: ClientId, amount: f64) {
        let client = self
            .client_accounts
            .entry(client)
            .or_insert_with(Default::default);
        client.total -= amount;
    }

    pub fn deposit(&mut self, client: ClientId, amount: f64) {
        let client = self
            .client_accounts
            .entry(client)
            .or_insert_with(Default::default);
        client.total += amount;
    }

    pub fn dispute(&mut self, client: ClientId, tx: TxId) {
        let client = self
            .client_accounts
            .entry(client)
            .or_insert_with(Default::default);
        //        client.held += todo!("Find amount in tx db");
    }

    pub fn resolve(&mut self, client: ClientId, tx: TxId) {
        let client = self
            .client_accounts
            .entry(client)
            .or_insert_with(Default::default);
        //       client.held -= todo!("Find amount in tx db");
    }
    pub fn chargeback(&mut self, client: ClientId, tx: TxId) {
        let client = self
            .client_accounts
            .entry(client)
            .or_insert_with(Default::default);
        //        client.held -= todo!("Find amount in tx db");
        //        client.amount -= todo!("Find amount in tx db");
        client.locked = true;
    }
}
