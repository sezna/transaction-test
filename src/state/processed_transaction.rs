use crate::ClientId;
/// A processed transaction is a processed deposit or withdrawal. These deserve a different data
/// representation from Transactions for the following reasons:
/// 1. [Transaction]s are limited in their representation due to the fact that they are a direct
///    model of the incoming user data. If we were to alter that data type, we would be directly
///    impacting our representation of user input. Having two types, one for the internal
///    transaction representation and one for the user input, gives us more flexibility and
///    prevents a leaky abstraction.
/// 2. Processed transactions cannot be disputes, resolutions, or chargebacks. To avoid having an
///    enum with invalid potential states, we need a new type. It would not be ideal to have a type
///    layout that does not accurately represent the data, i.e. an enum with five variants but only
///    two of which are ever constructed.
/// 3. Processed transactions can be disputed and therefore need another flag for that.
#[derive(Debug, Clone)]
pub struct ProcessedTransaction {
    r#type: ProcessedTransactionType,
    client: ClientId,
    amount: f64,
    disputed: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum ProcessedTransactionType {
    Deposit,
    Withdrawal,
}

impl ProcessedTransaction {
    pub fn new_deposit(client: ClientId, amount: f64) -> Self {
        ProcessedTransaction {
            r#type: ProcessedTransactionType::Deposit,
            amount,
            client,
            disputed: false,
        }
    }
    pub fn new_withdrawal(client: ClientId, amount: f64) -> Self {
        ProcessedTransaction {
            r#type: ProcessedTransactionType::Withdrawal,
            amount,
            client,
            disputed: false,
        }
    }

    pub fn is_deposit(&self) -> bool {
        self.r#type == ProcessedTransactionType::Deposit
    }

    pub fn set_disputed(&mut self, val: bool) {
        self.disputed = val;
    }

    pub fn client_id(&self) -> ClientId {
        self.client
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }
}
