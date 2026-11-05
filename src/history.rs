use crate::transaction::Transaction;

/// Maintains an ordered log of committed transactions with undo support.
#[derive(Debug, Default)]
pub struct TransactionHistory {
    entries: Vec<Transaction>,
}

impl TransactionHistory {
    /// Creates a new empty [`TransactionHistory`].
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Appends a transaction to the history.
    pub fn push(&mut self, tx: Transaction) {
        self.entries.push(tx);
    }

    /// Returns the last `n` transactions (most-recent last).
    pub fn last(&self, n: usize) -> &[Transaction] {
        let len = self.entries.len();
        if n >= len {
            &self.entries
        } else {
            &self.entries[len - n..]
        }
    }

    /// Removes and returns the most-recent transaction, or `None` if empty.
    pub fn undo_last(&mut self) -> Option<Transaction> {
        self.entries.pop()
    }

    /// Total number of transactions recorded.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no transactions have been recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{Op, Transaction};

    fn make_tx(ops: Vec<Op>) -> Transaction {
        Transaction::new(ops)
    }

    // AAA: Arrange history, Act push + undo_last, Assert returned tx matches.
    #[test]
    fn test_undo_last_returns_most_recent() {
        // Arrange
        let mut history = TransactionHistory::new();
        let tx1 = make_tx(vec![Op::Install("vim".into())]);
        let tx2 = make_tx(vec![Op::Remove("nano".into())]);
        history.push(tx1);
        history.push(tx2.clone());
        // Act
        let undone = history.undo_last().unwrap();
        // Assert
        assert_eq!(undone.ops, tx2.ops);
        assert_eq!(history.len(), 1);
    }

    // AAA: Arrange empty history, Act undo_last, Assert None.
    #[test]
    fn test_undo_last_on_empty_returns_none() {
        // Arrange
        let mut history = TransactionHistory::new();
        // Act
        let result = history.undo_last();
        // Assert
        assert!(result.is_none());
    }

    // AAA: Arrange 5 entries, Act last(3), Assert slice length 3.
    #[test]
    fn test_last_n_returns_correct_slice() {
        // Arrange
        let mut history = TransactionHistory::new();
        for i in 0..5u32 {
            history.push(make_tx(vec![Op::Install(format!("pkg{}", i))]));
        }
        // Act
        let slice = history.last(3);
        // Assert
        assert_eq!(slice.len(), 3);
    }

    // AAA: Arrange history with 2 entries, Act last(10), Assert returns all.
    #[test]
    fn test_last_n_clamps_to_available() {
        // Arrange
        let mut history = TransactionHistory::new();
        history.push(make_tx(vec![Op::Install("a".into())]));
        history.push(make_tx(vec![Op::Install("b".into())]));
        // Act
        let slice = history.last(100);
        // Assert
        assert_eq!(slice.len(), 2);
    }
}
