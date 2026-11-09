//! Transaction engine: executes [`Transaction`] operations through a
//! [`PackageBackend`], journaling each step and rolling back on failure.

use crate::backend::{BackendError, PackageBackend};
use crate::journal::{journal_to_transaction, Journal};
use crate::transaction::{Op, Transaction};

/// Outcome of executing a transaction.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecOutcome {
    /// All operations applied.
    Committed,
    /// A backend call failed; rollback was attempted.
    Failed {
        /// Operation that failed.
        op: Op,
        /// Backend error.
        error: BackendError,
        /// Whether rollback fully restored prior state.
        rollback_ok: bool,
    },
}

/// Executes package transactions against a backend with journaling.
pub struct TransactionEngine<'a, B: PackageBackend> {
    backend: &'a mut B,
}

impl<'a, B: PackageBackend> TransactionEngine<'a, B> {
    /// New engine borrowing a backend.
    pub fn new(backend: &'a mut B) -> Self {
        Self { backend }
    }

    /// Apply one op to the backend.
    fn apply(&mut self, op: &Op) -> Result<(), BackendError> {
        match op {
            Op::Install(p) => self.backend.install(p, None),
            Op::Remove(p) => self.backend.remove(p),
            Op::Upgrade(a, b) => {
                // old version is looked up lazily; backends that need it
                // receive the recorded "from" string.
                let old = self
                    .backend
                    .installed()
                    .get(a)
                    .cloned()
                    .unwrap_or_else(|| "unknown".into());
                self.backend.upgrade(a, &old, b)
            }
        }
    }

    /// Execute all ops in `tx` in order. On first failure, rolls back every
    /// succeeded op in reverse order and reports the outcome.
    pub fn execute(&mut self, tx: &mut Transaction) -> ExecOutcome {
        if tx.ops.is_empty() {
            return ExecOutcome::Committed;
        }
        let mut journal = Journal::new();
        for op in tx.ops.clone() {
            match self.apply(&op) {
                Ok(()) => journal.record(op, true),
                Err(error) => {
                    // Roll back succeeded ops in reverse.
                    let rollback_ok = self.rollback(&journal);
                    return ExecOutcome::Failed {
                        op,
                        error,
                        rollback_ok,
                    };
                }
            }
        }
        ExecOutcome::Committed
    }

    /// Apply inverse ops from the journal (reverse order). Inverse ops are
    /// ordinary ops, so they go through the same `apply` path — versions are
    /// resolved against current backend state.
    fn rollback(&mut self, journal: &Journal) -> bool {
        for inv in journal.rollback_ops() {
            if self.apply(&inv).is_err() {
                return false;
            }
        }
        true
    }

    /// Convert a journal to a history [`Transaction`] of applied ops.
    pub fn journal_transaction(journal: &Journal) -> crate::transaction::Transaction {
        journal_to_transaction(journal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MemoryBackend;

    // AAA: all installs succeed -> Committed, backend has packages.
    #[test]
    fn commit_installs_all() {
        let mut b = MemoryBackend::new();
        let mut tx = Transaction::new(vec![
            Op::Install("a".into()),
            Op::Install("b".into()),
        ]);
        let mut eng = TransactionEngine::new(&mut b);
        let out = eng.execute(&mut tx);
        assert_eq!(out, ExecOutcome::Committed);
        assert!(b.packages.contains_key("a"));
        assert!(b.packages.contains_key("b"));
    }

    // AAA: second install conflicts (already installed) -> Failed + rollback removes first.
    #[test]
    fn failure_rolls_back() {
        let mut b = MemoryBackend::new();
        b.seed("a", "1");
        // tx installs a (conflict) after installing c — order: install c, install a.
        let mut tx = Transaction::new(vec![
            Op::Install("c".into()),
            Op::Install("a".into()), // conflicts: a already installed
        ]);
        let mut eng = TransactionEngine::new(&mut b);
        let out = eng.execute(&mut tx);
        assert!(matches!(out, ExecOutcome::Failed { rollback_ok: true, .. }));
        // c was rolled back.
        assert!(!b.packages.contains_key("c"));
        assert!(b.packages.contains_key("a"));
    }

    // AAA: remove of absent package fails -> Failed.
    #[test]
    fn remove_missing_fails() {
        let mut b = MemoryBackend::new();
        let mut tx = Transaction::new(vec![Op::Remove("ghost".into())]);
        let mut eng = TransactionEngine::new(&mut b);
        let out = eng.execute(&mut tx);
        assert!(matches!(out, ExecOutcome::Failed { .. }));
    }
}
