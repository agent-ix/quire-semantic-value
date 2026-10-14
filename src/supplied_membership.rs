// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-owned accounting and typed stops for supplied-value membership.
//!
//! The traversal owns event ordering and storage reservations. This module
//! records only events it explicitly requests; it does not inspect values or
//! debit the evaluator's meter.

use alloc::vec::Vec;
use core::alloc::Layout;

use quire_exact::{Cancel, CancelCause};

use crate::declaration::{StorageRequest, StorageUnit};

/// The phase that owns every result from supplied-value membership.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppliedAdmissionPhase {
    /// Checking a supplied runtime value against its expected type.
    Membership,
}

/// One logical event requested by the supplied-membership traversal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MembershipEvent {
    /// Visit one actual value occurrence before inspecting its runtime kind.
    ValueVisit,
    /// Compare one expected and retained type-chain link pair.
    TypeLinkComparison,
    /// Schedule one actual child occurrence for a later membership visit.
    ChildScheduling,
    /// Query one admitted descriptor or ranked member shape.
    DescriptorQuery,
}

/// The part of an occurrence at which an event was requested.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MembershipPart {
    /// The occurrence itself.
    Value,
    /// A zero-based type-chain link pair.
    TypeLink(u64),
    /// A child being scheduled from the current occurrence.
    Child(PathStep),
    /// A zero-based descriptor lookup attached to the current occurrence.
    Descriptor(u64),
}

/// One exact edge from a parent occurrence to a required child occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PathStep {
    /// The present payload of an option.
    OptionPayload,
    /// A collection element in retained occurrence order.
    CollectionElement(usize),
    /// A present field of a record, by declaration position.
    CompositeField(usize),
    /// A tuple element, by declaration position.
    TuplePosition(usize),
    /// A union payload position, by declaration position.
    UnionPayload(usize),
}

/// A stable handle to an argument root or a scheduled child occurrence.
///
/// The handle belongs to the [`MembershipPathArena`] that created it. An
/// argument root needs no allocation; child handles remain valid while their
/// arena is retained, including after a traversal error.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PathToken {
    argument: usize,
    node: Option<usize>,
}

/// The rooted path and local part of an event boundary.
///
/// Repeated references to a shared value receive separate child tokens, so
/// siblings and repeated occurrences cannot collapse to the same locus.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MembershipLocus {
    /// The path to the current occurrence, rooted in one supplied argument.
    pub path: PathToken,
    /// The local part of that occurrence.
    pub part: MembershipPart,
}

#[derive(Clone, Copy, Debug)]
struct PathNode {
    parent: PathToken,
    step: PathStep,
}

/// Caller-owned storage for the rooted paths of one supplied invocation.
///
/// Store it with the result until the consumer resolves any error locus. One
/// node is stored per actually scheduled child; a denied scheduling event
/// still resolves through its parent token and [`MembershipPart::Child`].
#[derive(Debug, Default)]
pub struct MembershipPathArena {
    nodes: Vec<PathNode>,
}

impl MembershipPathArena {
    /// Create an empty invocation arena.
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Name one supplied argument without allocating path storage.
    pub const fn root(&self, argument: usize) -> PathToken {
        PathToken {
            argument,
            node: None,
        }
    }

    /// Retain one scheduled child after its logical event was permitted.
    ///
    /// `permit` sees the measured native reservation only when the vector
    /// must grow. A false response represents a real reservation denial,
    /// separately from checked capacity failure and allocator refusal.
    // The #12 traversal integration is the production caller for this staged carrier.
    #[allow(dead_code)]
    pub(crate) fn append_child(
        &mut self,
        parent: PathToken,
        step: PathStep,
        permit: &mut impl FnMut(StorageRequest) -> bool,
    ) -> Result<PathToken, SuppliedAdmissionError> {
        let locus = MembershipLocus {
            path: parent,
            part: MembershipPart::Child(step),
        };
        let request = StorageRequest::new(1, StorageUnit::AdditionalElements);
        let required = self
            .nodes
            .len()
            .checked_add(1)
            .ok_or(SuppliedAdmissionError::Capacity { request, locus })?;
        let layout = Layout::array::<PathNode>(required)
            .map_err(|_| SuppliedAdmissionError::Capacity { request, locus })?;
        if layout.size() > isize::MAX as usize {
            return Err(SuppliedAdmissionError::Capacity { request, locus });
        }
        if required > self.nodes.capacity() {
            if !permit(request) {
                return Err(SuppliedAdmissionError::Allocation { request, locus });
            }
            self.nodes
                .try_reserve_exact(1)
                .map_err(|_| SuppliedAdmissionError::Allocation { request, locus })?;
        }
        let token = PathToken {
            argument: parent.argument,
            node: Some(self.nodes.len()),
        };
        self.nodes.push(PathNode { parent, step });
        Ok(token)
    }

    /// Resolve a retained event locus without allocating or replaying work.
    ///
    /// Returns `None` only if the token does not name a node in this arena.
    /// Child steps are exposed from leaf to root in linear time; the argument
    /// root and final event part are separate typed fields.
    pub fn resolve(&self, locus: MembershipLocus) -> Option<ResolvedMembershipPath<'_>> {
        if let Some(index) = locus.path.node {
            self.nodes.get(index)?;
        }
        Some(ResolvedMembershipPath { arena: self, locus })
    }
}

/// An event path resolved against its original invocation arena.
pub struct ResolvedMembershipPath<'a> {
    arena: &'a MembershipPathArena,
    locus: MembershipLocus,
}

impl ResolvedMembershipPath<'_> {
    /// The zero-based supplied argument at the path root.
    pub const fn argument(&self) -> usize {
        self.locus.path.argument
    }

    /// The final event part after the child edges.
    pub const fn part(&self) -> MembershipPart {
        self.locus.part
    }

    /// Child edges from the current occurrence back to the argument root.
    pub fn steps_from_leaf(&self) -> PathAncestors<'_> {
        PathAncestors {
            arena: self.arena,
            next: self.locus.path.node,
        }
    }
}

/// An allocation-free, leaf-to-root iterator over retained child edges.
pub struct PathAncestors<'a> {
    arena: &'a MembershipPathArena,
    next: Option<usize>,
}

impl Iterator for PathAncestors<'_> {
    type Item = PathStep;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.arena.nodes.get(self.next?)?;
        self.next = node.parent.node;
        Some(node.step)
    }
}

/// Why a supplied value failed membership after permitted work.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InvalidMembershipCause {
    /// The value's runtime kind differs from the expected kind.
    Kind,
    /// The expected and retained types differ.
    Type,
    /// A ranked or declared member does not belong to its descriptor.
    Member,
    /// A required payload differs from its admitted shape or type.
    Payload,
}

/// A typed stop from the supplied-admission phase.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppliedAdmissionError {
    /// The input failed a membership rule at the given locus.
    Invalid {
        /// The failed membership rule.
        cause: InvalidMembershipCause,
        /// Where the rule failed.
        locus: MembershipLocus,
    },
    /// The next one-unit event would exceed the caller's ceiling.
    Limit {
        /// The configured supplied-membership ceiling.
        ceiling: u64,
        /// Successful cumulative work, excluding the denied event.
        spent: u64,
        /// The denied request, always one logical event unit.
        requested: u64,
        /// The event that was not performed.
        event: MembershipEvent,
        /// Where the event would have occurred.
        locus: MembershipLocus,
    },
    /// The caller's original cancellation handle stopped this boundary.
    Cancelled {
        /// The cause recorded on the original handle.
        cause: CancelCause,
        /// Successful cumulative work before cancellation.
        spent: u64,
        /// The event or helper boundary at which cancellation was observed.
        event: MembershipEvent,
        /// Where cancellation was observed.
        locus: MembershipLocus,
    },
    /// A real supplied-worklist reservation was denied.
    Allocation {
        /// The measured request at the reservation boundary.
        request: StorageRequest,
        /// Where the reservation was needed.
        locus: MembershipLocus,
    },
    /// Checked worklist size or representation arithmetic overflowed.
    Capacity {
        /// The measured request whose size could not be represented.
        request: StorageRequest,
        /// Where the reservation was needed.
        locus: MembershipLocus,
    },
}

impl SuppliedAdmissionError {
    /// The owning phase, independent of the failure classification.
    pub const fn phase(&self) -> SuppliedAdmissionPhase {
        SuppliedAdmissionPhase::Membership
    }
}

/// Cumulative logical work owned by one supplied-admission invocation.
///
/// Pass the same mutable budget through each argument and nested helper. A
/// denied event never changes `spent`; zero is a zero-work ceiling.
#[derive(Debug, Eq, PartialEq)]
pub struct SuppliedMembershipBudget {
    ceiling: u64,
    spent: u64,
}

impl SuppliedMembershipBudget {
    /// Create a budget with no successful events yet.
    pub const fn new(ceiling: u64) -> Self {
        Self { ceiling, spent: 0 }
    }

    /// The configured logical-work ceiling.
    pub const fn ceiling(&self) -> u64 {
        self.ceiling
    }

    /// The successful cumulative logical work.
    pub const fn spent(&self) -> u64 {
        self.spent
    }

    /// Request one event before performing it, polling the original handle.
    pub fn charge(
        &mut self,
        event: MembershipEvent,
        locus: MembershipLocus,
        cancel: &Cancel,
    ) -> Result<(), SuppliedAdmissionError> {
        self.check_cancel(event, locus, cancel)?;
        match self.spent.checked_add(1) {
            Some(next) if next <= self.ceiling => {
                self.spent = next;
                Ok(())
            }
            _ => Err(SuppliedAdmissionError::Limit {
                ceiling: self.ceiling,
                spent: self.spent,
                requested: 1,
                event,
                locus,
            }),
        }
    }

    /// Poll after an indivisible helper, without charging another event.
    pub fn check_cancel(
        &self,
        event: MembershipEvent,
        locus: MembershipLocus,
        cancel: &Cancel,
    ) -> Result<(), SuppliedAdmissionError> {
        let _ = cancel.poll();
        if let Some(cause) = cancel.cause() {
            return Err(SuppliedAdmissionError::Cancelled {
                cause,
                spent: self.spent,
                event,
                locus,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use quire_exact::{Cancel, CancelCause};

    use crate::declaration::{StorageRequest, StorageUnit};

    use super::{
        MembershipEvent, MembershipLocus, MembershipPart, MembershipPathArena, PathStep, PathToken,
        SuppliedAdmissionError, SuppliedAdmissionPhase, SuppliedMembershipBudget,
    };

    fn locus(path: PathToken, part: MembershipPart) -> MembershipLocus {
        MembershipLocus { path, part }
    }

    /// Trace: FR-109-AC-1, FR-109-AC-2
    #[ix_trace_rs::trace("TC-908", "FR-109-AC-1", "FR-109-AC-2")]
    #[test]
    fn exact_one_less_and_zero_share_cumulative_event_boundaries() {
        // The budget receives an independently enumerated event order across
        // two arguments, including a nested type and one scheduled child.
        let arena = MembershipPathArena::new();
        let first = arena.root(0);
        let second = arena.root(1);
        let events = [
            (
                MembershipEvent::ValueVisit,
                locus(first, MembershipPart::Value),
            ),
            (
                MembershipEvent::ValueVisit,
                locus(second, MembershipPart::Value),
            ),
            (
                MembershipEvent::TypeLinkComparison,
                locus(second, MembershipPart::TypeLink(0)),
            ),
            (
                MembershipEvent::TypeLinkComparison,
                locus(second, MembershipPart::TypeLink(1)),
            ),
            (
                MembershipEvent::ChildScheduling,
                locus(
                    second,
                    MembershipPart::Child(PathStep::CollectionElement(0)),
                ),
            ),
            (
                MembershipEvent::DescriptorQuery,
                locus(second, MembershipPart::Descriptor(0)),
            ),
        ];
        let cancel = Cancel::new();
        for ceiling in [0, 5, 6] {
            let mut budget = SuppliedMembershipBudget::new(ceiling);
            let result = events
                .iter()
                .copied()
                .try_for_each(|(event, at)| budget.charge(event, at, &cancel));
            if ceiling == 6 {
                assert_eq!(result, Ok(()));
                assert_eq!(budget.spent(), 6);
            } else {
                let (event, at) = events[ceiling as usize];
                assert_eq!(
                    result,
                    Err(SuppliedAdmissionError::Limit {
                        ceiling,
                        spent: ceiling,
                        requested: 1,
                        event,
                        locus: at,
                    })
                );
                assert_eq!(budget.spent(), ceiling);
                assert_eq!(
                    result.unwrap_err().phase(),
                    SuppliedAdmissionPhase::Membership
                );
            }
        }
    }

    /// Trace: FR-109-AC-3
    #[ix_trace_rs::trace("TC-908", "FR-109-AC-3")]
    #[test]
    fn original_handle_cancellation_stops_before_next_event() {
        let cancel = Cancel::new();
        let mut budget = SuppliedMembershipBudget::new(3);
        let arena = MembershipPathArena::new();
        let path = arena.root(0);
        let first = locus(path, MembershipPart::Value);
        let next = locus(path, MembershipPart::TypeLink(0));
        budget
            .charge(MembershipEvent::ValueVisit, first, &cancel)
            .unwrap();
        cancel.cancel(CancelCause::Requested);
        assert_eq!(
            budget.charge(MembershipEvent::TypeLinkComparison, next, &cancel),
            Err(SuppliedAdmissionError::Cancelled {
                cause: CancelCause::Requested,
                spent: 1,
                event: MembershipEvent::TypeLinkComparison,
                locus: next,
            })
        );
        assert_eq!(budget.spent(), 1);
        assert_eq!(cancel.tripped(), Some(CancelCause::Requested));
    }

    /// Trace: FR-109-AC-3
    #[ix_trace_rs::trace("TC-908", "FR-109-AC-3")]
    #[test]
    fn cancellation_after_bounded_helper_keeps_successful_event_spend() {
        let cancel = Cancel::new();
        let mut budget = SuppliedMembershipBudget::new(1);
        let arena = MembershipPathArena::new();
        let at = locus(arena.root(0), MembershipPart::Descriptor(0));
        budget
            .charge(MembershipEvent::DescriptorQuery, at, &cancel)
            .unwrap();
        cancel.cancel(CancelCause::Deadline);
        assert_eq!(
            budget.check_cancel(MembershipEvent::DescriptorQuery, at, &cancel),
            Err(SuppliedAdmissionError::Cancelled {
                cause: CancelCause::Deadline,
                spent: 1,
                event: MembershipEvent::DescriptorQuery,
                locus: at,
            })
        );
        assert_eq!(budget.spent(), 1);
    }

    /// Trace: FR-109-AC-1, FR-109-AC-2, FR-109-AC-4
    #[ix_trace_rs::trace("TC-908", "FR-109-AC-1", "FR-109-AC-2", "FR-109-AC-4")]
    #[test]
    fn sibling_payloads_and_repeated_occurrences_keep_distinct_rooted_paths() {
        let mut arena = MembershipPathArena::new();
        let root = arena.root(2);
        let mut permit = |_| true;
        let first = arena
            .append_child(root, PathStep::CollectionElement(0), &mut permit)
            .unwrap();
        let first_payload = arena
            .append_child(first, PathStep::OptionPayload, &mut permit)
            .unwrap();
        let second = arena
            .append_child(root, PathStep::CollectionElement(1), &mut permit)
            .unwrap();
        let second_payload = arena
            .append_child(second, PathStep::OptionPayload, &mut permit)
            .unwrap();
        let first_nested = arena
            .append_child(first_payload, PathStep::TuplePosition(0), &mut permit)
            .unwrap();
        let sibling_nested = arena
            .append_child(first_payload, PathStep::TuplePosition(1), &mut permit)
            .unwrap();
        let repeated = arena
            .append_child(root, PathStep::CollectionElement(2), &mut permit)
            .unwrap();

        let steps = |path| {
            arena
                .resolve(locus(path, MembershipPart::Value))
                .unwrap()
                .steps_from_leaf()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            steps(first_payload),
            vec![PathStep::OptionPayload, PathStep::CollectionElement(0)]
        );
        assert_eq!(
            steps(second_payload),
            vec![PathStep::OptionPayload, PathStep::CollectionElement(1)]
        );
        assert_eq!(
            steps(first_nested),
            vec![
                PathStep::TuplePosition(0),
                PathStep::OptionPayload,
                PathStep::CollectionElement(0),
            ]
        );
        assert_eq!(
            steps(sibling_nested),
            vec![
                PathStep::TuplePosition(1),
                PathStep::OptionPayload,
                PathStep::CollectionElement(0),
            ]
        );
        assert_eq!(steps(repeated), vec![PathStep::CollectionElement(2)]);
        assert_ne!(first_payload, second_payload);
        assert_ne!(first_nested, sibling_nested);
        assert_ne!(first, repeated);
    }

    /// Trace: FR-109-AC-1, FR-109-AC-5
    #[ix_trace_rs::trace("TC-908", "FR-109-AC-1", "FR-109-AC-5")]
    #[test]
    fn denied_child_event_and_reservation_keep_resolvable_parent_path() {
        let mut arena = MembershipPathArena::new();
        let root = arena.root(0);
        let mut permit = |_| true;
        let collection_child = arena
            .append_child(root, PathStep::CollectionElement(1), &mut permit)
            .unwrap();
        let payload = arena
            .append_child(collection_child, PathStep::OptionPayload, &mut permit)
            .unwrap();
        let denied_at = locus(payload, MembershipPart::Child(PathStep::TuplePosition(0)));
        let cancel = Cancel::new();
        let mut budget = SuppliedMembershipBudget::new(2);
        for (event, at) in [
            (
                MembershipEvent::ValueVisit,
                locus(root, MembershipPart::Value),
            ),
            (
                MembershipEvent::ChildScheduling,
                locus(root, MembershipPart::Child(PathStep::CollectionElement(1))),
            ),
        ] {
            budget.charge(event, at, &cancel).unwrap();
        }
        let denied = budget
            .charge(MembershipEvent::ChildScheduling, denied_at, &cancel)
            .unwrap_err();
        assert_eq!(
            denied,
            SuppliedAdmissionError::Limit {
                ceiling: 2,
                spent: 2,
                requested: 1,
                event: MembershipEvent::ChildScheduling,
                locus: denied_at,
            }
        );
        let route = arena.resolve(denied_at).unwrap();
        assert_eq!(route.argument(), 0);
        assert_eq!(
            route.part(),
            MembershipPart::Child(PathStep::TuplePosition(0))
        );
        assert_eq!(
            route.steps_from_leaf().collect::<Vec<_>>(),
            vec![PathStep::OptionPayload, PathStep::CollectionElement(1)]
        );

        let mut empty = MembershipPathArena::new();
        let other_root = empty.root(3);
        let request = StorageRequest::new(1, StorageUnit::AdditionalElements);
        let mut denied_request = None;
        let allocation = empty
            .append_child(other_root, PathStep::OptionPayload, &mut |seen| {
                denied_request = Some(seen);
                false
            })
            .unwrap_err();
        assert_eq!(denied_request, Some(request));
        assert_eq!(
            allocation,
            SuppliedAdmissionError::Allocation {
                request,
                locus: locus(other_root, MembershipPart::Child(PathStep::OptionPayload)),
            }
        );
        let unresolved_child = empty
            .resolve(locus(
                other_root,
                MembershipPart::Child(PathStep::OptionPayload),
            ))
            .unwrap();
        assert_eq!(unresolved_child.argument(), 3);
        assert!(unresolved_child.steps_from_leaf().next().is_none());
        let retry = empty
            .append_child(other_root, PathStep::OptionPayload, &mut permit)
            .unwrap();
        assert_eq!(
            empty
                .resolve(locus(retry, MembershipPart::Value))
                .unwrap()
                .steps_from_leaf()
                .collect::<Vec<_>>(),
            vec![PathStep::OptionPayload]
        );
    }
}
