//! Transaction journal: records operations and their inverses so a failed
//! commit can be rolled back deterministically.

use crate::transaction::{Op, Transaction};

/// One journaled step: the operation performed and how to undo it.
#[derive(Debug, Clone, PartialEq)]
pub struct JournalEntry {
    /// Operation that was applied.
    pub applied: Op,
    /// Inverse operation that restores prior state, if one exists.
    pub inverse: Option<Op>,
    /// Whether the operation completed successfully.
    pub succeeded: bool,
}

/// Invert an operation where a meaningful inverse exists.
///
/// - Install(p)   -> Remove(p)
/// - Remove(p)    -> Install(p)   (caller must restore version separately)
/// - Upgrade(a,b) -> Upgrade(b,a) (swap direction; backend validates versions)
pub fn invert(op: &Op) -> Option<Op> {
    match op {
        Op::Install(p) => Some(Op::Remove(p.clone())),
        Op::Remove(p) => Some(Op::Install(p.clone())),
        Op::Upgrade(a, b) => Some(Op::Upgrade(b.clone(), a.clone())),
    }
}

/// Ordered record of a transaction's execution for audit and rollback.
#[derive(Debug, Default)]
pub struct Journal {
    entries: Vec<JournalEntry>,
}

impl Journal {
    /// Empty journal.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an applied operation.
    pub fn record(&mut self, applied: Op, succeeded: bool) {
        self.entries.push(JournalEntry {
            inverse: invert(&applied),
            applied,
            succeeded,
        });
    }

    /// Inverse operations for all succeeded entries, in reverse order.
    pub fn rollback_ops(&self) -> Vec<Op> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.succeeded)
            .filter_map(|e| e.inverse.clone())
            .collect()
    }

    /// Operations that succeeded.
    pub fn succeeded_ops(&self) -> Vec<Op> {
        self.entries
            .iter()
            .filter(|e| e.succeeded)
            .map(|e| e.applied.clone())
            .collect()
    }

    /// Number of recorded entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the journal is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Convert a completed journal into a [`Transaction`] for history.
pub fn journal_to_transaction(journal: &Journal) -> Transaction {
    Transaction::new(journal.succeeded_ops())
}

#[cfg(test)]
mod tests {
    use super::*;

    // AAA: install+remove journal -> rollback removes then reinstalls, reversed.
    #[test]
    fn rollback_order_reversed() {
        let mut j = Journal::new();
        j.record(Op::Install("a".into()), true);
        j.record(Op::Remove("b".into()), true);
        j.record(Op::Install("c".into()), false); // failed op has no inverse applied
        let ops = j.rollback_ops();
        assert_eq!(
            ops,
            vec![
                Op::Install("b".into()),
                Op::Remove("a".into())
            ]
        );
    }

    // AAA: invert mapping sanity.
    #[test]
    fn invert_maps_correctly() {
        assert_eq!(
            invert(&Op::Install("x".into())),
            Some(Op::Remove("x".into()))
        );
        assert_eq!(
            invert(&Op::Upgrade("a".into(), "b".into())),
            Some(Op::Upgrade("b".into(), "a".into()))
        );
    }
}
