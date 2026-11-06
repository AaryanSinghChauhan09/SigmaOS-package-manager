#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState { Pending, InProgress, Committed, RolledBack, Failed(String) }
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionOp { Install(String), Remove(String), Upgrade(String, String) }

pub struct PackageTransaction {
    pub id: u64,
    pub operations: Vec<TransactionOp>,
    pub state: TransactionState,
}
impl PackageTransaction {
    pub fn new(id: u64) -> Self { Self { id, operations: Vec::new(), state: TransactionState::Pending } }
    pub fn add_install(&mut self, pkg: &str) { self.operations.push(TransactionOp::Install(pkg.to_string())); }
    pub fn add_remove(&mut self, pkg: &str) { self.operations.push(TransactionOp::Remove(pkg.to_string())); }
    pub fn commit(&mut self) -> Result<(), String> {
        if self.operations.is_empty() { return Err("no operations".to_string()); }
        self.state = TransactionState::Committed; Ok(())
    }
    pub fn rollback(&mut self) { self.state = TransactionState::RolledBack; }
    pub fn op_count(&self) -> usize { self.operations.len() }
}
pub struct PackageManagerService { transactions: Vec<PackageTransaction>, next_id: u64 }
impl PackageManagerService {
    pub fn new() -> Self { Self { transactions: Vec::new(), next_id: 1 } }
    pub fn begin_transaction(&mut self) -> &mut PackageTransaction {
        let tx = PackageTransaction::new(self.next_id);
        self.next_id += 1;
        self.transactions.push(tx);
        self.transactions.last_mut().unwrap()
    }
    pub fn transaction_count(&self) -> usize { self.transactions.len() }
}
impl Default for PackageManagerService { fn default() -> Self { Self::new() } }

/// A single package operation recorded in a transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Install(String),
    Remove(String),
    Upgrade(String, String),
}

/// A recorded transaction used by the history log.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub ops: Vec<Op>,
}

impl Transaction {
    pub fn new(ops: Vec<Op>) -> Self {
        Self { ops }
    }
}
