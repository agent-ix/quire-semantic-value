// SPDX-License-Identifier: AGPL-3.0-or-later
//! The per-step simulation record a refinement's `requirements` hook writes
//! ([`StepSimulationRecord`]) and the obligation it states ([`StepObligation`]).
//!
//! QSL's compiler writes the records and CG reads them, so the values live
//! here, below both (QSL ADR-018 LA-5). The crate interprets none of them
//! (FR-060): the refinement and hypothesis keys are minted by the caller, and
//! a [`StepRowRef`] only names a row by its position.

use alloc::vec::Vec;

use quire_canonical::{Encode, Error, Sink, Writer};
use quire_exact::NodeKey;

/// One step row of a refinement declaration, named by its zero-based position
/// in the refinement's step rows in declaration order (QSL FR-135 `steps`).
///
/// A refinement has exactly one row per concrete step class (QSL ADR-020
/// RM-5), so a refinement key and a row position name one row.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StepRowRef(u32);

impl StepRowRef {
    /// Name the step row at zero-based `index`.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// The zero-based position of the row.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl Encode for StepRowRef {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.integer(i128::from(self.0))
    }
}

/// What one per-step simulation record states (QSL ADR-020 RE-2).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StepObligation {
    /// RS-2: the abstract image of every concrete initial state is an abstract
    /// initial state.
    Initial,
    /// RS-3 for a `stutter` row, RS-4 for an abstract-operation row.
    Step {
        /// The step row the obligation is for.
        row: StepRowRef,
    },
}

impl Encode for StepObligation {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("kind")?;
        match self {
            Self::Initial => writer.string("initial")?,
            Self::Step { row } => {
                writer.string("step")?;
                writer.name("row")?;
                row.encode_into(writer)?;
            }
        }
        writer.end_object()
    }
}

/// One per-step simulation obligation of a refinement, with the concrete
/// package's invariant clauses it may assume.
///
/// Equality, hashing and order are field-wise. `hypotheses` keeps the order
/// supplied: QSV neither sorts nor deduplicates it.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StepSimulationRecord {
    /// The refinement node the record is for.
    pub refinement: NodeKey,
    /// Which obligation of the refinement it states.
    pub obligation: StepObligation,
    /// The concrete package's invariant clauses, each its own
    /// `operation-contract` record.
    pub hypotheses: Vec<NodeKey>,
}

impl StepSimulationRecord {
    /// Retain `refinement`, `obligation` and `hypotheses` verbatim.
    #[must_use]
    pub const fn new(
        refinement: NodeKey,
        obligation: StepObligation,
        hypotheses: Vec<NodeKey>,
    ) -> Self {
        Self {
            refinement,
            obligation,
            hypotheses,
        }
    }
}

impl Encode for StepSimulationRecord {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("hypotheses")?;
        writer.begin_array()?;
        for key in &self.hypotheses {
            writer.string(&alloc::string::ToString::to_string(key))?;
        }
        writer.end_array()?;
        writer.name("obligation")?;
        self.obligation.encode_into(writer)?;
        writer.name("refinement")?;
        writer.string(&alloc::string::ToString::to_string(&self.refinement))?;
        writer.end_object()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::string::String;
    use std::vec;
    use std::vec::Vec;

    use ix_trace_rs::trace;
    use quire_canonical::{to_vec, Error, Limits};
    use quire_exact::NodeKey;

    use super::{StepObligation, StepRowRef, StepSimulationRecord};

    #[allow(
        clippy::disallowed_methods,
        reason = "a test fixture supplies NodeKey; production code mints none"
    )]
    fn key(byte: u8) -> NodeKey {
        NodeKey::from_digest([byte; 32])
    }

    fn hex(byte: u8) -> String {
        std::format!("{byte:02x}").repeat(32)
    }

    fn step(index: u32) -> StepObligation {
        StepObligation::Step {
            row: StepRowRef::new(index),
        }
    }

    #[trace("TC-922", "FR-123-AC-1")]
    #[test]
    fn construction_retains_every_field_in_supplied_order() {
        let record = StepSimulationRecord::new(key(1), step(6), vec![key(9), key(3), key(9)]);
        assert_eq!(record.refinement, key(1));
        assert_eq!(record.obligation, step(6));
        assert_eq!(record.hypotheses, [key(9), key(3), key(9)]);
        assert_eq!(StepRowRef::new(6).index(), 6);
        let empty = StepSimulationRecord::new(key(1), StepObligation::Initial, Vec::new());
        assert_eq!(empty.obligation, StepObligation::Initial);
        assert!(empty.hypotheses.is_empty());
        let literal = StepSimulationRecord {
            refinement: key(1),
            obligation: StepObligation::Initial,
            hypotheses: Vec::new(),
        };
        assert_eq!(literal, empty);
    }

    #[trace("TC-922", "FR-123-AC-2")]
    #[test]
    fn equality_and_hash_distinguish_each_field() {
        let base = StepSimulationRecord::new(key(1), step(0), vec![key(2)]);
        let changed = [
            StepSimulationRecord::new(key(4), step(0), vec![key(2)]),
            StepSimulationRecord::new(key(1), StepObligation::Initial, vec![key(2)]),
            StepSimulationRecord::new(key(1), step(1), vec![key(2)]),
            StepSimulationRecord::new(key(1), step(0), Vec::new()),
            StepSimulationRecord::new(key(1), step(0), vec![key(5)]),
            StepSimulationRecord::new(key(1), step(0), vec![key(2), key(2)]),
        ];
        assert_eq!(base, base.clone());
        let mut set = HashSet::new();
        set.insert(base.clone());
        set.insert(base.clone());
        for value in changed {
            assert_ne!(base, value);
            set.insert(value);
        }
        assert_eq!(set.len(), 7);
        assert_ne!(StepRowRef::new(0), StepRowRef::new(1));
    }

    #[trace("TC-922", "FR-123-AC-3")]
    #[test]
    fn order_is_initial_before_steps_then_row_position() {
        let mut obligations = vec![step(2), StepObligation::Initial, step(0)];
        obligations.sort();
        assert_eq!(obligations, [StepObligation::Initial, step(0), step(2)]);
        let ordered = [
            StepSimulationRecord::new(key(1), StepObligation::Initial, vec![key(9)]),
            StepSimulationRecord::new(key(1), step(0), Vec::new()),
            StepSimulationRecord::new(key(1), step(0), vec![key(1)]),
            StepSimulationRecord::new(key(2), StepObligation::Initial, Vec::new()),
        ];
        let mut reversed = ordered.to_vec();
        reversed.reverse();
        reversed.sort();
        assert_eq!(reversed, ordered);
        for left in &ordered {
            for right in &ordered {
                assert_eq!(left.cmp(right) == std::cmp::Ordering::Equal, left == right);
                assert_eq!(left.partial_cmp(right), Some(left.cmp(right)));
            }
        }
    }

    #[trace("TC-922", "FR-123-AC-4")]
    #[test]
    fn canonical_bytes_are_exact_for_each_obligation() {
        let limits = Limits::new(4096);
        let initial = StepSimulationRecord::new(key(0), StepObligation::Initial, Vec::new());
        let expected = std::format!(
            "{{\"hypotheses\":[],\"obligation\":{{\"kind\":\"initial\"}},\"refinement\":\"{}\"}}",
            hex(0)
        );
        assert_eq!(
            to_vec(&initial, limits).expect("initial record"),
            expected.as_bytes()
        );
        let stepped = StepSimulationRecord::new(key(0xab), step(6), vec![key(1), key(0xcd)]);
        let expected = std::format!(
            "{{\"hypotheses\":[\"{}\",\"{}\"],\"obligation\":{{\"kind\":\"step\",\"row\":6}},\
             \"refinement\":\"{}\"}}",
            hex(1),
            hex(0xcd),
            hex(0xab)
        );
        assert_eq!(
            to_vec(&stepped, limits).expect("step record"),
            expected.as_bytes()
        );
        assert_eq!(
            to_vec(&StepObligation::Initial, limits).expect("obligation"),
            b"{\"kind\":\"initial\"}"
        );
        assert_eq!(
            to_vec(&StepRowRef::new(4_294_967_295), limits).expect("row"),
            b"4294967295"
        );
        for changed in [
            StepSimulationRecord::new(key(0xac), step(6), vec![key(1), key(0xcd)]),
            StepSimulationRecord::new(key(0xab), step(7), vec![key(1), key(0xcd)]),
            StepSimulationRecord::new(key(0xab), step(6), vec![key(0xcd), key(1)]),
            StepSimulationRecord::new(key(0xab), step(6), vec![key(1)]),
        ] {
            assert_ne!(
                to_vec(&changed, limits).expect("changed"),
                to_vec(&stepped, limits).expect("step record")
            );
        }
    }

    #[trace("TC-922", "FR-123-AC-5")]
    #[test]
    fn a_limit_one_byte_short_is_a_typed_error() {
        let record = StepSimulationRecord::new(key(1), step(3), vec![key(2)]);
        let complete = to_vec(&record, Limits::new(4096)).expect("record");
        let length = u64::try_from(complete.len()).expect("test output length");
        assert_eq!(
            to_vec(&record, Limits::new(length)).expect("exact limit"),
            complete
        );
        assert!(matches!(
            to_vec(&record, Limits::new(length - 1)),
            Err(Error::Limit(_))
        ));
    }
}
