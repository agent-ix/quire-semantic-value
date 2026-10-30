// SPDX-License-Identifier: AGPL-3.0-or-later
//! The nominal semantic-node refusal vocabulary: the strict reader's
//! `invalid_semantic_graph` refusal, its typed causes, and the structural term
//! check every dimension and compound unit applies.
//!
//! The checked-package admission of structural node identities stays with the
//! caller; this module holds only what a backend handed admitted keys raises.

use alloc::collections::BTreeSet;
use quire_canonical::Limits;
use quire_exact::Integer;

/// The strict reader's `refused { code: invalid_semantic_graph }`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[error("invalid_semantic_graph: {cause:?}")]
pub struct InvalidSemanticGraph {
    /// The typed reason.
    pub cause: SemanticGraphCause,
}

impl InvalidSemanticGraph {
    /// Stable refusal code.
    pub const CODE: &'static str = "invalid_semantic_graph";
}

/// Why a nominal semantic node or node graph was refused.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SemanticGraphCause {
    /// The preimage does not satisfy `node-identity-preimage.schema.json`, or a
    /// model-owned enum declaration's case list is empty, has an empty case
    /// name or repeats a case name.
    NonCanonicalPreimage,
    /// An unordered declaration's members are not sorted by case name.
    UnsortedUnorderedMembers,
    /// The owner does not join the lock selection.
    OwnerNotSelected,
    /// The retained structural key does not match the admitted content.
    StaleKey,
    /// A member node references a different declaration node.
    ForeignDeclaration,
    /// A member node's case is not a member of its declaration.
    UndeclaredCase,
    /// A dimension term has exponent zero.
    ZeroExponent,
    /// A dimension term names the same base dimension twice.
    DuplicateTerm,
    /// Dimension terms are not strictly ascending by node key.
    UnsortedTerms,
    /// A unit scale or offset is not a reduced rational.
    UnreducedRational,
    /// A unit scale is zero.
    ZeroScale,
    /// A targetless unit does not have scale one and offset zero.
    NonIdentityRoot,
    /// Two admitted nodes have the same key.
    DuplicateNode,
    /// A dimension term or unit names a dimension node that is not admitted.
    UnknownDimension,
    /// A dimension term names a derived (non-base) dimension.
    NonBaseDimensionTerm,
    /// A unit target is not an admitted unit.
    UnknownTarget,
    /// A unit target belongs to a different dimension node.
    CrossDimensionTarget,
    /// A dimension's unit graph has no targetless canonical root.
    MissingRoot,
    /// A dimension's unit graph has more than one targetless root.
    DuplicateRoot,
    /// Unit targets form a cycle.
    TargetCycle,
}

/// Refuse terms that are zero, repeated or not strictly ascending, in that
/// order.
pub fn check_terms<K: Ord>(terms: &[(K, Integer)]) -> Result<(), SemanticGraphCause> {
    if terms.iter().any(|(_, exponent)| exponent.is_zero()) {
        return Err(SemanticGraphCause::ZeroExponent);
    }
    let distinct: BTreeSet<_> = terms.iter().map(|(key, _)| key).collect();
    if distinct.len() != terms.len() {
        return Err(SemanticGraphCause::DuplicateTerm);
    }
    if !terms.is_sorted_by(|(left, _), (right, _)| left < right) {
        return Err(SemanticGraphCause::UnsortedTerms);
    }
    Ok(())
}

/// The default byte limit for measuring a compound unit's structural terms.
///
/// The canonical encoder checks the ordered FR-201 `terms` arm under this
/// limit, but its output is never retained or hashed as an identity. A caller
/// with a tighter byte budget can pass its own [`Limits`] instead.
pub const IDENTITY_LIMITS: Limits = Limits::new(16_777_216);

/// Why a structural identity's bounded encoding was refused.
///
/// QSL FR-259 Behavior 4: a byte limit reached while encoding is the
/// calling stage's input-bytes limit (QSL's `identity.input_bytes`), never a
/// malformed value. Behavior 6: a failed heap reservation is its own
/// outcome, naming no bound.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, thiserror::Error)]
pub enum IdentityRefusal {
    /// The structural terms' canonical bytes would exceed the byte limit they were
    /// encoded under.
    #[error("identity encoding needs {required} bytes, over the bound of {bound}")]
    InputBytes {
        /// The byte limit the preimage was encoded under.
        bound: u64,
        /// The bytes it needed when the encoder refused it; always more
        /// than `bound`.
        required: u64,
    },
    /// A heap reservation while carrying or encoding the structural identity failed.
    #[error("allocation of {requested} bytes for a structural identity failed")]
    Allocation {
        /// The additional bytes requested by the failed reservation.
        requested: usize,
    },
    /// The structural terms have no RFC 8785 encoding.
    #[error("the identity has no RFC 8785 encoding")]
    NonCanonical,
}

impl From<quire_canonical::Error> for IdentityRefusal {
    fn from(error: quire_canonical::Error) -> Self {
        match error {
            quire_canonical::Error::Limit(limit) => Self::InputBytes {
                bound: limit.bound,
                required: limit.required,
            },
            quire_canonical::Error::Allocation { requested } => Self::Allocation { requested },
            _ => Self::NonCanonical,
        }
    }
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::*;

    /// QSL FR-259 Behavior 3: the identity encoder's default byte bound is
    /// 16 MiB.
    #[trace("TC-728", "FR-259-AC-2")]
    #[test]
    fn the_identity_limits_default_is_sixteen_mebibytes() {
        assert_eq!(IDENTITY_LIMITS.max_bytes(), 16_777_216);
    }

    fn term(key: u8, exponent: i64) -> (u8, Integer) {
        (key, Integer::from(exponent))
    }

    #[test]
    fn check_terms_refuses_zero_then_repeated_then_unsorted() {
        assert_eq!(check_terms::<u8>(&[]), Ok(()));
        assert_eq!(check_terms(&[term(1, 2), term(2, -1)]), Ok(()));
        assert_eq!(
            check_terms(&[term(2, 1), term(1, 0), term(1, 1)]),
            Err(SemanticGraphCause::ZeroExponent)
        );
        assert_eq!(
            check_terms(&[term(2, 1), term(1, 1), term(1, 1)]),
            Err(SemanticGraphCause::DuplicateTerm)
        );
        assert_eq!(
            check_terms(&[term(2, 1), term(1, 1)]),
            Err(SemanticGraphCause::UnsortedTerms)
        );
    }
}
