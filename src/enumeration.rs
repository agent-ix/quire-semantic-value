// SPDX-License-Identifier: AGPL-3.0-or-later
//! QSpec FR-141 enumerations at run time: an admitted
//! declaration's structure ([`EnumDeclaration`]), an admitted member as an
//! [`EnumValue`], the checked `VariantId` -> [`EnumValue`] index
//! ([`EnumMemberIndex`]), and the QSpec FR-141 comparison schedule
//! ([`compare_enum`]). Node identity (preimages, digests, owner join and
//! stale-key refusal) stays with the caller.

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::semantic_node::{InvalidSemanticGraph, SemanticGraphCause};
use crate::stop::{outcome_from_stop, Stop};
use quire_exact::{
    is_identifier, Charge, ChargePoint, ComparisonOperator, IllTyped, IllTypedCause, LimitKind,
    Meter, NodeKey, Outcome, VariantId,
};

/// Whether `members` is a schema member list: nonempty, distinct
/// identifiers. The one statement of the rule, shared by
/// [`EnumDeclaration::new`] and a caller's preimage readers.
pub fn is_member_list(members: &[String]) -> bool {
    let distinct: BTreeSet<_> = members.iter().collect();
    !members.is_empty()
        && distinct.len() == members.len()
        && members.iter().all(|case| is_identifier(case))
}

/// An admitted enum declaration node: its key, its ordering and its member
/// cases. This type checks the structural rules a declaration is compared
/// under. It does not mint or verify the key: this type takes the declaration key
/// the caller admitted as given (only the checking stage mints and
/// verifies node keys).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumDeclaration {
    key: NodeKey,
    ordered: bool,
    members: Vec<String>,
}

impl EnumDeclaration {
    /// The declaration node `key` with its member cases, in declaration
    /// order for an `ordered enum` and sorted by case name otherwise
    /// (QSpec FR-141).
    ///
    /// An empty, repeated or non-identifier member list is
    /// `NonCanonicalPreimage`; an unordered declaration whose cases are not
    /// sorted is `UnsortedUnorderedMembers`. `key` is taken as given: it is
    /// the caller's admitted declaration key, not checked against the
    /// members.
    pub fn new(
        key: NodeKey,
        ordered: bool,
        members: Vec<String>,
    ) -> Result<Self, InvalidSemanticGraph> {
        if !is_member_list(&members) {
            return Err(InvalidSemanticGraph {
                cause: SemanticGraphCause::NonCanonicalPreimage,
            });
        }
        Self::sorted_if_unordered(key, ordered, members)
    }

    /// The declaration node `key` of a model-owned variant type (QSpec
    /// FR-141, FR-144): an unordered enumeration whose case names are any
    /// non-empty strings, kept verbatim. The caller supplies the cases in
    /// ascending UTF-8 byte order of their names.
    ///
    /// An empty list, an empty case name or a repeated case is
    /// `NonCanonicalPreimage`; cases not in ascending byte order are
    /// `UnsortedUnorderedMembers`. Names are never normalized, so names that
    /// differ in any byte are distinct cases. `key` is taken as given, as in
    /// [`Self::new`]. Source-declared enums use [`Self::new`], which still
    /// requires identifier cases.
    pub fn model_owned(key: NodeKey, members: Vec<String>) -> Result<Self, InvalidSemanticGraph> {
        let distinct: BTreeSet<_> = members.iter().collect();
        if members.is_empty()
            || distinct.len() != members.len()
            || members.iter().any(String::is_empty)
        {
            return Err(InvalidSemanticGraph {
                cause: SemanticGraphCause::NonCanonicalPreimage,
            });
        }
        Self::sorted_if_unordered(key, false, members)
    }

    fn sorted_if_unordered(
        key: NodeKey,
        ordered: bool,
        members: Vec<String>,
    ) -> Result<Self, InvalidSemanticGraph> {
        if !ordered && !members.is_sorted() {
            return Err(InvalidSemanticGraph {
                cause: SemanticGraphCause::UnsortedUnorderedMembers,
            });
        }
        Ok(Self {
            key,
            ordered,
            members,
        })
    }

    /// The declaration node key.
    pub fn key(&self) -> NodeKey {
        self.key.clone()
    }

    /// Whether the declaration selects ordered semantics.
    pub fn is_ordered(&self) -> bool {
        self.ordered
    }

    /// Declaration-ordered (or case-sorted, if unordered) member cases.
    pub fn members(&self) -> &[String] {
        &self.members
    }

    /// The value of member `case`, whose admitted member node key is
    /// `member`. A case outside the declaration is `UndeclaredCase`.
    ///
    /// The position and the ordered flag come from this declaration. The
    /// member key is taken as given: it is the caller's admitted
    /// `quire.enum-member-node/v1` key for `case`, and nothing here checks
    /// it against the case (this crate mints and verifies no key). A wrong key gives
    /// a value whose `=` (member keys) and ordering (positions) disagree.
    pub fn member(&self, case: &str, member: NodeKey) -> Result<EnumValue, InvalidSemanticGraph> {
        let position = self
            .members
            .iter()
            .position(|declared| declared == case)
            .ok_or(InvalidSemanticGraph {
                cause: SemanticGraphCause::UndeclaredCase,
            })?;
        Ok(EnumValue {
            declaration: self.key.clone(),
            member,
            ordered: self.ordered,
            position,
            case: case.into(),
        })
    }
}

/// An enumeration value: (declaration identity, member identity).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumValue {
    declaration: NodeKey,
    member: NodeKey,
    ordered: bool,
    position: usize,
    case: Box<str>,
}

impl EnumValue {
    /// Declaration node key.
    pub fn declaration(&self) -> NodeKey {
        self.declaration.clone()
    }

    /// Member node key.
    pub fn member(&self) -> NodeKey {
        self.member.clone()
    }

    /// Zero-based QSpec FR-141 canonical-list position of the member: the same
    /// value `quire_exact::EnumMember::rank` carries once a caller
    /// pairs it with this member's [`Self::variant`]. A caller building an
    /// `EnumShape`/`Value::Enum` from an admitted [`EnumValue`] needs it.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Whether the declaration is an `ordered enum`.
    pub fn is_ordered(&self) -> bool {
        self.ordered
    }

    /// The case identifier.
    pub fn case(&self) -> &str {
        &self.case
    }

    /// This member's kernel `VariantId`: its admitted declaration node and
    /// exact case name, with no digest or fresh node identity computation.
    /// The caller checks that the member node belongs to that declaration.
    pub fn variant(&self) -> VariantId {
        VariantId::new(self.declaration.clone(), self.case.to_string())
            .expect("admitted enumeration cases are nonempty")
    }
}

/// The checked `VariantId` -> [`EnumValue`]
/// index, built once as the checking stage admits each enum member and consulted
/// wherever an evaluated kernel `Value::Enum` (a bare `VariantId` and rank)
/// needs its declaration, ordered flag, position or case name back. The
/// kernel is a leaf and carries none of this (`quire_exact::value`'s own
/// module doc); the QSpec FR-141 enum-specific `=`/ordering schedule
/// ([`compare_enum`], `declaration::CheckedEquality`'s `Enum`
/// schedule, and an evaluator's ordered-enum arm)
/// resolves a `VariantId` back to its full [`EnumValue`] through this index
/// rather than the kernel ever holding one.
///
/// The table is shared behind an `Arc`: a clone is a reference
/// count, so every checked equality over one enum can hold that enum's
/// table without copying it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnumMemberIndex(Arc<BTreeMap<VariantId, EnumValue>>);

impl EnumMemberIndex {
    /// Record one admitted member, keyed by its own `VariantId`
    /// ([`EnumValue::variant`], its member key retyped). The last write for
    /// a given `VariantId` wins. When member keys are the verified
    /// content-addressed identities, two structurally
    /// identical members (same declaration, same case) share one
    /// `VariantId`, so recording either is equivalent; with keys taken as
    /// given, the index is keyed by whatever key the caller admitted.
    pub fn record(&mut self, member: EnumValue) {
        Arc::make_mut(&mut self.0).insert(member.variant(), member);
    }

    /// The full checked member `variant` names, or `None` when it was never
    /// recorded -- never re-derived by any other means (R-05).
    pub fn resolve(&self, variant: VariantId) -> Option<&EnumValue> {
        self.0.get(&variant)
    }

    /// The sub-index holding only the entries named by `variants`, silently
    /// skipping any this index never recorded. `declaration::
    /// CheckedEquality` uses this to retain, inside a checked
    /// `Enum`-scheduled equality node, only the compared operands' own enum
    /// declaration -- an `EnumShape::variants()` iterator -- rather than a
    /// clone of the whole package's enum-member index.
    pub fn filtered(&self, variants: impl IntoIterator<Item = VariantId>) -> Self {
        let mut filtered = Self::default();
        for variant in variants {
            if let Some(member) = self.resolve(variant) {
                filtered.record(member.clone());
            }
        }
        filtered
    }
}

/// Compare two enum values. Members of different declarations, and ordering
/// requests over an unordered declaration, are ill-typed and consume nothing.
pub fn compare_enum(
    operator: ComparisonOperator,
    left: &EnumValue,
    right: &EnumValue,
    meter: &mut Meter,
) -> Result<Outcome<bool>, IllTyped> {
    if left.declaration != right.declaration {
        return Err(IllTyped {
            cause: IllTypedCause::DistinctEnumDeclarations,
        });
    }
    if operator.is_ordering() && !left.ordered {
        return Err(IllTyped {
            cause: IllTypedCause::UnorderedEnumOrdering,
        });
    }
    Ok(outcome_from_stop(compare(operator, left, right, meter)))
}

fn compare(
    operator: ComparisonOperator,
    left: &EnumValue,
    right: &EnumValue,
    meter: &mut Meter,
) -> Result<bool, Stop> {
    for read in 1..=2 {
        meter.charge(
            Charge::new(ChargePoint::EnumIdentityRead).size(LimitKind::ValueOccurrences, read),
        )?;
    }
    let ordering = if operator.is_ordering() {
        left.position.cmp(&right.position)
    } else {
        left.member.cmp(&right.member)
    };
    meter.charge(Charge::new(ChargePoint::EnumResultRetain).results(1))?;
    Ok(operator.holds(ordering))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use ix_trace_rs::trace;
    use quire_exact::ScalarLimits;

    #[allow(
        clippy::disallowed_methods,
        reason = "a test fixture needs a NodeKey; production code mints none"
    )]
    fn key(byte: u8) -> NodeKey {
        crate::test_identity::node(u64::from(byte))
    }

    fn cases(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| String::from(*name)).collect()
    }

    fn refused(cause: SemanticGraphCause) -> Result<EnumDeclaration, InvalidSemanticGraph> {
        Err(InvalidSemanticGraph { cause })
    }

    #[test]
    fn a_declaration_refuses_a_malformed_member_list_before_an_unsorted_one() {
        for members in [
            vec![],
            cases(&["Ready", "Ready"]),
            cases(&["Ready", "9done"]),
            cases(&["b", "a", "a"]),
        ] {
            assert_eq!(
                EnumDeclaration::new(key(1), false, members),
                refused(SemanticGraphCause::NonCanonicalPreimage)
            );
        }
        assert_eq!(
            EnumDeclaration::new(key(1), false, cases(&["Ready", "Done"])),
            refused(SemanticGraphCause::UnsortedUnorderedMembers)
        );
        let ordered = EnumDeclaration::new(key(1), true, cases(&["Ready", "Done"])).unwrap();
        assert!(ordered.is_ordered());
        assert_eq!(ordered.key(), key(1));
        assert_eq!(ordered.members(), cases(&["Ready", "Done"]).as_slice());
    }

    #[test]
    fn a_member_carries_its_declaration_position_and_case() {
        let status = EnumDeclaration::new(key(1), true, cases(&["Ready", "Done"])).unwrap();
        let done = status.member("Done", key(3)).unwrap();
        assert_eq!(done.declaration(), key(1));
        assert_eq!(done.member(), key(3));
        assert_eq!(done.position(), 1);
        assert!(done.is_ordered());
        assert_eq!(done.case(), "Done");
        assert_eq!(
            status.member("Closed", key(4)).unwrap_err().cause,
            SemanticGraphCause::UndeclaredCase
        );
    }

    #[trace("TC-916", "FR-116-AC-1", "FR-116-AC-2", "FR-116-AC-3")]
    #[test]
    fn a_model_owned_declaration_accepts_any_nonempty_case_name_verbatim() {
        let names = ["  padded ", "2x", "Gold Plus", "Zürich", "日本"];
        let mut sorted = cases(&names);
        sorted.sort();
        let declaration = EnumDeclaration::model_owned(key(1), sorted.clone()).unwrap();
        assert!(!declaration.is_ordered());
        assert_eq!(declaration.key(), key(1));
        assert_eq!(declaration.members(), sorted.as_slice());
        for (rank, case) in sorted.iter().enumerate() {
            let value = declaration.member(case, key(10 + rank as u8)).unwrap();
            assert_eq!(value.case(), case.as_str());
            assert_eq!(value.position(), rank);
            assert_eq!(value.declaration(), key(1));
            assert!(!value.is_ordered());
        }
        assert_eq!(
            declaration.member("gold plus", key(9)).unwrap_err().cause,
            SemanticGraphCause::UndeclaredCase
        );
    }

    #[trace("TC-916", "FR-116-AC-4")]
    #[test]
    fn a_model_owned_declaration_refuses_empty_duplicate_and_unsorted_cases() {
        for members in [
            vec![],
            cases(&[""]),
            cases(&["", "a"]),
            cases(&["Gold Plus", "Gold Plus"]),
            // Malformed and unsorted: the malformed cause wins.
            cases(&["b", "a", "a"]),
        ] {
            assert_eq!(
                EnumDeclaration::model_owned(key(1), members),
                refused(SemanticGraphCause::NonCanonicalPreimage)
            );
        }
        assert_eq!(
            EnumDeclaration::model_owned(key(1), cases(&["b b", "a a"])),
            refused(SemanticGraphCause::UnsortedUnorderedMembers)
        );
        // Names are distinct byte-wise, never normalized: a trailing space
        // and a precomposed versus decomposed letter are different cases.
        let decomposed =
            EnumDeclaration::model_owned(key(1), cases(&["a", "a ", "e\u{301}", "\u{e9}"]))
                .unwrap();
        assert_eq!(decomposed.members().len(), 4);
    }

    #[trace("TC-916", "FR-116-AC-5")]
    #[test]
    fn two_model_owned_declarations_of_one_shape_agree_on_equality_and_variants() {
        let names = cases(&["2x", "Gold Plus", "Silver"]);
        let first = EnumDeclaration::model_owned(key(1), names.clone()).unwrap();
        let second = EnumDeclaration::model_owned(key(1), names).unwrap();
        assert_eq!(first, second);
        let left = first.member("Gold Plus", key(5)).unwrap();
        let right = second.member("Gold Plus", key(5)).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.variant(), right.variant());
        let other = first.member("Silver", key(6)).unwrap();
        assert_ne!(left, other);
        let mut index = EnumMemberIndex::default();
        index.record(left.clone());
        index.record(other.clone());
        assert_eq!(index.resolve(left.variant()), Some(&left));
        assert_eq!(index.resolve(other.variant()).unwrap().case(), "Silver");
        let mut meter = Meter::new(ScalarLimits {
            integer_bits: 1024,
            decimal_digits: 1024,
            scale_expansion: 1024,
            text_input_bytes: 1024,
            text_scalars: 1024,
            normalized_scalars: 1024,
            unit_edges: 1024,
            value_occurrences: 1024,
            work_units: 1024,
            result_units: 1024,
        });
        assert_eq!(
            compare_enum(ComparisonOperator::Equal, &left, &right, &mut meter),
            Ok(Outcome::Completed(true))
        );
        assert_eq!(
            compare_enum(ComparisonOperator::Less, &left, &other, &mut meter),
            Err(IllTyped {
                cause: IllTypedCause::UnorderedEnumOrdering
            })
        );
    }

    #[trace("TC-916", "FR-116-AC-6")]
    #[test]
    fn the_identifier_path_still_refuses_non_identifier_cases() {
        for members in [cases(&["Gold Plus"]), cases(&["2x"]), cases(&[""])] {
            assert_eq!(
                EnumDeclaration::new(key(1), false, members),
                refused(SemanticGraphCause::NonCanonicalPreimage)
            );
        }
        assert!(!is_member_list(&cases(&["Gold Plus"])));
    }
}
