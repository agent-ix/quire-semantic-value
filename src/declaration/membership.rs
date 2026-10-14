// SPDX-License-Identifier: AGPL-3.0-or-later
//! The typed-event membership walk behind [`TypeEnvironment::admits`].
//!
//! The walk is an explicit worklist. The caller's `on_event` callback
//! receives one typed event and its rooted locus immediately before each
//! logical action (a value visit, a type-link comparison, a child
//! scheduling or a descriptor query), and an `Err` from it stops the walk
//! unchanged before the action is performed. A membership failure is a
//! typed [`SuppliedAdmissionError::Invalid`] at the failing locus, separate
//! from the callback's own stops, and a refused worklist reservation is
//! [`SuppliedAdmissionError::Allocation`] or
//! [`SuppliedAdmissionError::Capacity`] with the exact native request.

use alloc::vec::Vec;

use quire_exact::{Value, ValueType};

use super::{reserve_elements, ReservationPolicy, StorageFailure, TypeEnvironment};
use crate::supplied_membership::{
    InvalidMembershipCause, MembershipEvent, MembershipLocus, MembershipPart, MembershipPathArena,
    PathAppendError, PathStep, PathToken, SuppliedAdmissionError,
};

/// A logical event request or a response boundary after its indivisible
/// helper. Only the former spends a work unit.
pub(super) enum MembershipBoundary {
    /// Charge one logical event before its action.
    Event(MembershipEvent, MembershipLocus),
    /// Poll cancellation after an indivisible helper without another charge.
    AfterHelper(MembershipEvent, MembershipLocus),
}

type Callback<'f, Cause> =
    dyn FnMut(MembershipBoundary) -> Result<(), SuppliedAdmissionError<Cause>> + 'f;

/// A pending occurrence: its expected type, the value, and its rooted path.
type Pending<'a> = (&'a ValueType, &'a Value, PathToken);

/// The worklist and path arena of one walk, whose every growth is checked.
struct Frontier<'w, 'a, R> {
    arena: &'w mut MembershipPathArena,
    pending: Vec<Pending<'a>>,
    reservation: &'w R,
}

impl<'a, R: ReservationPolicy> Frontier<'_, 'a, R> {
    /// Reserve room for one node's whole batch of children, before the batch's
    /// first scheduling event. `first` is the child that would be scheduled
    /// first, which names the locus of a failure.
    fn reserve<Cause>(
        &mut self,
        additional: usize,
        parent: PathToken,
        first: PathStep,
        on_boundary: &mut Callback<'_, Cause>,
    ) -> Result<(), SuppliedAdmissionError<Cause>> {
        let locus = MembershipLocus {
            path: parent,
            part: MembershipPart::Child(first),
        };
        reserve_elements(&mut self.pending, additional, self.reservation)
            .map_err(|failure| storage(failure, locus))?;
        on_boundary(MembershipBoundary::AfterHelper(
            MembershipEvent::ChildScheduling,
            locus,
        ))
    }

    /// Emit the scheduling event for one actual child, then retain its path
    /// node and queue it into the already reserved worklist.
    fn schedule<Cause: From<InvalidMembershipCause>>(
        &mut self,
        parent: PathToken,
        step: PathStep,
        child: (&'a ValueType, &'a Value),
        on_boundary: &mut Callback<'_, Cause>,
    ) -> Result<(), SuppliedAdmissionError<Cause>> {
        let at = MembershipLocus {
            path: parent,
            part: MembershipPart::Child(step),
        };
        on_boundary(MembershipBoundary::Event(
            MembershipEvent::ChildScheduling,
            at,
        ))?;
        let reservation = self.reservation;
        let path = self
            .arena
            .append_child::<Cause>(parent, step, &mut |request| reservation.permit(request))
            .map_err(|failure| match failure {
                PathAppendError::ForeignParent => invalid(InvalidMembershipCause::ForeignPath, at),
                PathAppendError::Admission(stop) => stop,
            })?;
        on_boundary(MembershipBoundary::AfterHelper(
            MembershipEvent::ChildScheduling,
            at,
        ))?;
        self.pending.push((child.0, child.1, path));
        Ok(())
    }
}

/// The cause of a failed kernel leaf admission, from what the kernel compared:
/// a value of another kind, a value outside the declared domain of its own
/// kind, a retained type that differs, or an enum rank the shape does not hold.
fn leaf_cause(expected: &ValueType, value: &Value) -> InvalidMembershipCause {
    match (expected, value) {
        (ValueType::Int(_), Value::Integer(_))
        | (ValueType::Rational(_), Value::Rational(_))
        | (ValueType::Decimal(_), Value::Decimal(_)) => InvalidMembershipCause::Payload,
        (ValueType::Float(_), Value::Float(_))
        | (ValueType::Quantity(_), Value::Quantity(_))
        | (ValueType::Text(_), Value::Text(_)) => InvalidMembershipCause::Type,
        (ValueType::Enum(_), Value::Enum(_)) => InvalidMembershipCause::Member,
        _ => InvalidMembershipCause::Kind,
    }
}

fn invalid<Cause: From<InvalidMembershipCause>>(
    cause: InvalidMembershipCause,
    locus: MembershipLocus,
) -> SuppliedAdmissionError<Cause> {
    SuppliedAdmissionError::Invalid {
        cause: cause.into(),
        locus,
    }
}

/// Compare two type chains one link pair at a time, emitting one event per
/// pair, the initial one included. Ordinals count from `first_ordinal`.
fn same_type_walk<Cause: From<InvalidMembershipCause>>(
    left: &ValueType,
    right: &ValueType,
    path: PathToken,
    first_ordinal: u64,
    on_boundary: &mut Callback<'_, Cause>,
) -> Result<(), SuppliedAdmissionError<Cause>> {
    enum TypeStep<'a> {
        Continue(&'a ValueType, &'a ValueType),
        Equal,
        Mismatch,
    }
    let (mut left, mut right) = (left, right);
    let mut ordinal = first_ordinal;
    loop {
        let at = MembershipLocus {
            path,
            part: MembershipPart::TypeLink(ordinal),
        };
        on_boundary(MembershipBoundary::Event(
            MembershipEvent::TypeLinkComparison,
            at,
        ))?;
        let step = match (left, right) {
            (ValueType::Option(a), ValueType::Option(b)) => TypeStep::Continue(a, b),
            (ValueType::Collection(a), ValueType::Collection(b)) => {
                if a.kind() != b.kind() || a.bound() != b.bound() {
                    TypeStep::Mismatch
                } else {
                    TypeStep::Continue(a.element(), b.element())
                }
            }
            (left, right) if left == right => TypeStep::Equal,
            _ => TypeStep::Mismatch,
        };
        on_boundary(MembershipBoundary::AfterHelper(
            MembershipEvent::TypeLinkComparison,
            at,
        ))?;
        (left, right) = match step {
            TypeStep::Continue(left, right) => (left, right),
            TypeStep::Equal => return Ok(()),
            TypeStep::Mismatch => return Err(invalid(InvalidMembershipCause::Type, at)),
        };
        ordinal = ordinal.saturating_add(1);
    }
}

impl TypeEnvironment {
    /// Walk `value` against `expected` under this environment's conformance,
    /// as [`Self::admits`] decides it, emitting a typed event and rooted
    /// locus to `on_event` immediately before each logical action. The
    /// caller roots the walk (`at`) in `arena`.
    pub(super) fn admits_walk<'a, Cause: From<InvalidMembershipCause>>(
        &self,
        expected: &'a ValueType,
        value: &'a Value,
        arena: &mut MembershipPathArena,
        at: MembershipLocus,
        on_event: &mut impl FnMut(
            MembershipEvent,
            MembershipLocus,
        ) -> Result<(), SuppliedAdmissionError<Cause>>,
        reservation: &impl ReservationPolicy,
    ) -> Result<(), SuppliedAdmissionError<Cause>> {
        self.admits_walk_observed(
            expected,
            value,
            arena,
            at,
            &mut |boundary| match boundary {
                MembershipBoundary::Event(event, locus) => on_event(event, locus),
                MembershipBoundary::AfterHelper(..) => Ok(()),
            },
            reservation,
        )
    }

    /// Internal walk retaining both event requests and post-helper response
    /// boundaries for the caller-owned cancellation handle.
    pub(super) fn admits_walk_observed<'a, Cause: From<InvalidMembershipCause>>(
        &self,
        expected: &'a ValueType,
        value: &'a Value,
        arena: &mut MembershipPathArena,
        at: MembershipLocus,
        on_boundary: &mut impl FnMut(MembershipBoundary) -> Result<(), SuppliedAdmissionError<Cause>>,
        reservation: &impl ReservationPolicy,
    ) -> Result<(), SuppliedAdmissionError<Cause>> {
        if arena.resolve(at).is_none() {
            return Err(invalid(InvalidMembershipCause::ForeignPath, at));
        }
        let mut frontier = Frontier {
            arena,
            pending: Vec::new(),
            reservation,
        };
        let mut next = Some((expected, value, at.path));
        while let Some((expected, value, path)) = next.take().or_else(|| frontier.pending.pop()) {
            let here = MembershipLocus {
                path,
                part: MembershipPart::Value,
            };
            on_boundary(MembershipBoundary::Event(MembershipEvent::ValueVisit, here))?;
            self.admit_occurrence(expected, value, path, &mut frontier, on_boundary)?;
        }
        Ok(())
    }

    fn admit_occurrence<'a, Cause: From<InvalidMembershipCause>>(
        &self,
        expected: &'a ValueType,
        value: &'a Value,
        path: PathToken,
        frontier: &mut Frontier<'_, 'a, impl ReservationPolicy>,
        on_boundary: &mut Callback<'_, Cause>,
    ) -> Result<(), SuppliedAdmissionError<Cause>> {
        let here = MembershipLocus {
            path,
            part: MembershipPart::Value,
        };
        let query = MembershipLocus {
            path,
            part: MembershipPart::Descriptor(0),
        };
        match (expected, value) {
            (ValueType::Reference(declared), Value::Reference(reference)) => {
                on_boundary(MembershipBoundary::Event(
                    MembershipEvent::DescriptorQuery,
                    query,
                ))?;
                let conforms = self.conforms(reference.object_type(), *declared);
                on_boundary(MembershipBoundary::AfterHelper(
                    MembershipEvent::DescriptorQuery,
                    query,
                ))?;
                if !conforms {
                    return Err(invalid(InvalidMembershipCause::ReferenceConformance, query));
                }
            }
            (ValueType::Composite(declared), Value::Composite(composite)) => {
                on_boundary(MembershipBoundary::Event(
                    MembershipEvent::DescriptorQuery,
                    query,
                ))?;
                let conforms = self.composite_conforms(composite.declaration(), *declared);
                on_boundary(MembershipBoundary::AfterHelper(
                    MembershipEvent::DescriptorQuery,
                    query,
                ))?;
                if !conforms {
                    return Err(invalid(InvalidMembershipCause::CompositeConformance, query));
                }
            }
            (ValueType::Composite(_), Value::Union(_)) => {
                return Err(invalid(InvalidMembershipCause::UnsupportedUnion, here));
            }
            (ValueType::Option(payload), Value::Option(option)) => {
                same_type_walk(option.payload_type(), payload, path, 0, on_boundary)?;
                if let Some(present) = option.payload() {
                    frontier.reserve(1, path, PathStep::OptionPayload, on_boundary)?;
                    frontier.schedule(
                        path,
                        PathStep::OptionPayload,
                        (payload, present),
                        on_boundary,
                    )?;
                }
            }
            (ValueType::Collection(declared), Value::Collection(collection)) => {
                let actual = collection.collection_type();
                let pair = MembershipLocus {
                    path,
                    part: MembershipPart::TypeLink(0),
                };
                on_boundary(MembershipBoundary::Event(
                    MembershipEvent::TypeLinkComparison,
                    pair,
                ))?;
                let matching =
                    actual.kind() == declared.kind() && actual.bound() == declared.bound();
                on_boundary(MembershipBoundary::AfterHelper(
                    MembershipEvent::TypeLinkComparison,
                    pair,
                ))?;
                if !matching {
                    return Err(invalid(InvalidMembershipCause::Type, pair));
                }
                same_type_walk(actual.element(), declared.element(), path, 1, on_boundary)?;
                let elements = collection.elements();
                if let Some(last) = elements.len().checked_sub(1) {
                    frontier.reserve(
                        elements.len(),
                        path,
                        PathStep::CollectionElement(last),
                        on_boundary,
                    )?;
                }
                for (index, element) in elements.iter().enumerate().rev() {
                    frontier.schedule(
                        path,
                        PathStep::CollectionElement(index),
                        (declared.element(), element),
                        on_boundary,
                    )?;
                }
            }
            _ => {
                let admitted = expected.admits(value);
                on_boundary(MembershipBoundary::AfterHelper(
                    MembershipEvent::ValueVisit,
                    here,
                ))?;
                if !admitted {
                    return Err(invalid(leaf_cause(expected, value), here));
                }
            }
        }
        Ok(())
    }
}

fn storage<Cause>(
    failure: StorageFailure,
    locus: MembershipLocus,
) -> SuppliedAdmissionError<Cause> {
    match failure {
        StorageFailure::Allocation(request) => {
            SuppliedAdmissionError::Allocation { request, locus }
        }
        StorageFailure::Capacity(request) => SuppliedAdmissionError::Capacity { request, locus },
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;
    use core::cell::{Cell, RefCell};
    use std::time::Instant;

    use ix_trace_rs::trace;
    use quire_exact::{
        from_admitted, Cancel, CancelCause, CollectionKind, CollectionType, EffectiveId, Integer,
        ObjectId, ObjectReference, OptionValue, UniverseId,
    };

    use super::*;
    use crate::declaration::{
        CompositeDeclaration, GlobalReservations, ObjectTypeDeclaration, StorageRequest,
        StorageUnit,
    };
    use crate::supplied_membership::SuppliedMembershipBudget;

    type Stop = SuppliedAdmissionError<InvalidMembershipCause>;

    /// One recorded event with its resolved part and leaf-to-root steps.
    type Described = (MembershipEvent, MembershipPart, Vec<PathStep>);

    struct Run {
        result: Result<(), Stop>,
        raw: Vec<(MembershipEvent, MembershipLocus)>,
        arena: MembershipPathArena,
    }

    impl Run {
        fn describe(&self, locus: MembershipLocus) -> (MembershipPart, Vec<PathStep>) {
            let resolved = self
                .arena
                .resolve(locus)
                .expect("locus resolves in its arena");
            (resolved.part(), resolved.steps_from_leaf().collect())
        }

        fn seen(&self) -> Vec<Described> {
            self.raw
                .iter()
                .map(|&(event, locus)| {
                    let (part, steps) = self.describe(locus);
                    (event, part, steps)
                })
                .collect()
        }
    }

    fn environment() -> TypeEnvironment {
        TypeEnvironment::new(
            Vec::<CompositeDeclaration>::new(),
            Vec::<ObjectTypeDeclaration>::new(),
        )
        .expect("an empty environment admits")
    }

    /// Walk `value` against `expected`, denying the event at `stop_at` with a
    /// cancellation that carries the event's own position.
    fn run_with(
        expected: &ValueType,
        value: &Value,
        stop_at: Option<usize>,
        reservation: &impl ReservationPolicy,
    ) -> Run {
        let mut arena = MembershipPathArena::new().expect("a fresh arena id");
        let at = MembershipLocus {
            path: arena.root(0),
            part: MembershipPart::Value,
        };
        let mut raw = Vec::new();
        let result = environment().admits_walk::<InvalidMembershipCause>(
            expected,
            value,
            &mut arena,
            at,
            &mut |event, locus| {
                raw.push((event, locus));
                if stop_at == Some(raw.len() - 1) {
                    return Err(Stop::Cancelled {
                        cause: CancelCause::Requested,
                        spent: u64::try_from(raw.len() - 1).expect("small"),
                        event,
                        locus,
                    });
                }
                Ok(())
            },
            reservation,
        );
        Run { result, raw, arena }
    }

    fn run(expected: &ValueType, value: &Value) -> Run {
        run_with(expected, value, None, &GlobalReservations)
    }

    fn integer(n: i64) -> Value {
        Value::Integer(Integer::from(n))
    }

    fn sequence_of(element: ValueType) -> CollectionType {
        CollectionType::new(CollectionKind::Sequence, element, None)
    }

    /// `Some(Sequence[1, 2])` against `Option<Sequence<Integer>>`.
    fn nested() -> (ValueType, Value) {
        let sequence = sequence_of(ValueType::Integer);
        (
            ValueType::option(ValueType::collection(sequence.clone())),
            OptionValue::from_admitted(
                ValueType::collection(sequence.clone()),
                Some(from_admitted(sequence, vec![integer(1), integer(2)])),
            ),
        )
    }

    use MembershipEvent::{ChildScheduling, TypeLinkComparison, ValueVisit};
    use MembershipPart::{Child, TypeLink, Value as Occurrence};
    use PathStep::{CollectionElement, OptionPayload};

    /// The exact event stream of the nested value: each event fires once per
    /// actual step, the initial type pair has its own event, children are
    /// scheduled in push order with their true ordinal, and visits run forward.
    /// Trace: FR-109-AC-1
    #[trace("TC-908", "FR-109-AC-1")]
    #[test]
    fn event_stream_for_a_nested_value_is_exact() {
        let (expected, value) = nested();
        let walked = run(&expected, &value);
        assert_eq!(walked.result, Ok(()));
        assert_eq!(
            walked.seen(),
            vec![
                (ValueVisit, Occurrence, vec![]),
                (TypeLinkComparison, TypeLink(0), vec![]),
                (TypeLinkComparison, TypeLink(1), vec![]),
                (ChildScheduling, Child(OptionPayload), vec![]),
                (ValueVisit, Occurrence, vec![OptionPayload]),
                (TypeLinkComparison, TypeLink(0), vec![OptionPayload]),
                (TypeLinkComparison, TypeLink(1), vec![OptionPayload]),
                (
                    ChildScheduling,
                    Child(CollectionElement(1)),
                    vec![OptionPayload]
                ),
                (
                    ChildScheduling,
                    Child(CollectionElement(0)),
                    vec![OptionPayload]
                ),
                (
                    ValueVisit,
                    Occurrence,
                    vec![CollectionElement(0), OptionPayload]
                ),
                (
                    ValueVisit,
                    Occurrence,
                    vec![CollectionElement(1), OptionPayload]
                ),
            ]
        );
    }

    /// The initial pair is its own event: a two-link chain is two events and
    /// a one-link chain is one.
    /// Trace: FR-109-AC-1
    #[trace("TC-908", "FR-109-AC-1")]
    #[test]
    fn initial_type_pair_has_its_own_event() {
        let inner = ValueType::option(ValueType::Integer);
        let two = run(
            &ValueType::option(inner.clone()),
            &OptionValue::from_admitted(inner, None),
        );
        let one = run(
            &ValueType::option(ValueType::Integer),
            &OptionValue::from_admitted(ValueType::Integer, None),
        );
        let links = |walked: &Run| {
            walked
                .seen()
                .into_iter()
                .filter(|(event, ..)| *event == TypeLinkComparison)
                .map(|(_, part, _)| part)
                .collect::<Vec<_>>()
        };
        assert_eq!(links(&two), vec![TypeLink(0), TypeLink(1)]);
        assert_eq!(links(&one), vec![TypeLink(0)]);
    }

    /// A denied event at every position of the walk is the last event the
    /// callback sees, is returned unchanged, and nothing after it runs.
    /// Trace: FR-109-AC-1, FR-109-AC-3
    #[trace("TC-908", "FR-109-AC-1", "FR-109-AC-3")]
    #[test]
    fn denied_event_is_not_performed_and_its_error_is_unchanged() {
        let (expected, value) = nested();
        let full = run(&expected, &value).seen();
        assert_eq!(full.len(), 11);
        for index in 0..full.len() {
            let walked = run_with(&expected, &value, Some(index), &GlobalReservations);
            assert_eq!(walked.seen(), full[..=index], "events up to {index}");
            let Err(Stop::Cancelled {
                cause,
                spent,
                event,
                locus,
            }) = walked.result
            else {
                panic!("event {index} must stop with the callback's own error");
            };
            assert_eq!(cause, CancelCause::Requested);
            assert_eq!(spent, u64::try_from(index).expect("small"));
            assert_eq!(event, full[index].0);
            let (part, steps) = walked.describe(locus);
            assert_eq!((part, steps), (full[index].1, full[index].2.clone()));
        }
    }

    /// Removing the denial yields the walk's full outcome again.
    /// Trace: FR-109-AC-5
    #[trace("TC-908", "FR-109-AC-5")]
    #[test]
    fn retry_after_removing_the_denial_yields_the_same_outcome() {
        let (expected, value) = nested();
        let full = run(&expected, &value);
        let denied = run_with(&expected, &value, Some(8), &GlobalReservations);
        assert!(denied.result.is_err());
        let policy = DenyAll {
            seen: Cell::new(None),
        };
        let reservation_denied = run_with(&expected, &value, None, &policy);
        assert!(matches!(
            reservation_denied.result,
            Err(Stop::Allocation { .. })
        ));
        let retried = run(&expected, &value);
        assert_eq!(retried.result, Ok(()));
        assert_eq!(retried.result, full.result);
        assert_eq!(retried.seen(), full.seen());
    }

    /// A limit stop from the callback also passes through unchanged.
    /// Trace: FR-109-AC-1
    #[trace("TC-908", "FR-109-AC-1")]
    #[test]
    fn limit_stop_is_returned_unchanged() {
        let (expected, value) = nested();
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(3),
            part: MembershipPart::Value,
        };
        let mut calls = 0_u64;
        let result = environment().admits_walk::<InvalidMembershipCause>(
            &expected,
            &value,
            &mut arena,
            at,
            &mut |event, locus| {
                calls += 1;
                if calls == 4 {
                    return Err(Stop::Limit {
                        ceiling: 3,
                        spent: 3,
                        requested: 1,
                        event,
                        locus,
                    });
                }
                Ok(())
            },
            &GlobalReservations,
        );
        assert_eq!(
            result,
            Err(Stop::Limit {
                ceiling: 3,
                spent: 3,
                requested: 1,
                event: ChildScheduling,
                locus: MembershipLocus {
                    path: at.path,
                    part: Child(OptionPayload),
                },
            })
        );
        assert_eq!(calls, 4);
    }

    /// Children are scheduled one event each, never as one batch, and no
    /// event precedes the first value visit.
    /// Trace: FR-109-AC-1
    #[trace("TC-908", "FR-109-AC-1")]
    #[test]
    fn children_are_scheduled_one_event_each() {
        let sequence = sequence_of(ValueType::Integer);
        let value = from_admitted(sequence.clone(), (0..5).map(integer).collect());
        let expected = ValueType::collection(sequence);
        let walked = run(&expected, &value);
        let seen = walked.seen();
        assert_eq!(seen[0].0, ValueVisit);
        let scheduled: Vec<_> = seen
            .iter()
            .filter(|(event, ..)| *event == ChildScheduling)
            .map(|(_, part, _)| *part)
            .collect();
        assert_eq!(
            scheduled,
            (0..5)
                .rev()
                .map(|i| Child(CollectionElement(i)))
                .collect::<Vec<_>>()
        );
        // Accepting three and denying the fourth schedules exactly three.
        let denied = run_with(
            &expected,
            &value,
            Some(
                seen.iter()
                    .position(|e| e.0 == ChildScheduling)
                    .expect("first")
                    + 3,
            ),
            &GlobalReservations,
        );
        let after: Vec<_> = denied.seen();
        assert_eq!(after.iter().filter(|e| e.0 == ChildScheduling).count(), 4);
        assert_eq!(after.iter().filter(|e| e.0 == ValueVisit).count(), 1);
    }

    fn invalid(walked: &Run) -> (InvalidMembershipCause, MembershipPart, Vec<PathStep>) {
        let Err(Stop::Invalid { cause, locus }) = walked.result else {
            panic!("expected an invalid membership, got {:?}", walked.result);
        };
        let (part, steps) = walked.describe(locus);
        (cause, part, steps)
    }

    /// A leaf failure, a payload type failure and a collection-kind failure
    /// each carry their own cause and the actual locus.
    /// Trace: FR-109-AC-4
    #[trace("TC-908", "FR-109-AC-4")]
    #[test]
    fn invalid_membership_carries_distinct_cause_and_locus() {
        let sequence = sequence_of(ValueType::Integer);
        let leaf = run(
            &ValueType::collection(sequence.clone()),
            &from_admitted(sequence.clone(), vec![integer(1), Value::Boolean(true)]),
        );
        assert_eq!(
            invalid(&leaf),
            (
                InvalidMembershipCause::Kind,
                Occurrence,
                vec![CollectionElement(1)]
            )
        );
        let payload = run(
            &ValueType::option(ValueType::Integer),
            &OptionValue::from_admitted(ValueType::Boolean, None),
        );
        assert_eq!(
            invalid(&payload),
            (InvalidMembershipCause::Type, TypeLink(0), vec![])
        );
        let set = CollectionType::new(CollectionKind::Set, ValueType::Integer, None);
        let kind = run(
            &ValueType::collection(sequence),
            &from_admitted(set, vec![integer(1)]),
        );
        assert_eq!(
            invalid(&kind),
            (InvalidMembershipCause::Type, TypeLink(0), vec![])
        );
    }

    struct DenyAll {
        seen: Cell<Option<StorageRequest>>,
    }

    impl ReservationPolicy for DenyAll {
        fn permit(&self, request: StorageRequest) -> bool {
            self.seen.set(Some(request));
            false
        }
    }

    /// A refused worklist reservation reports its exact native request before
    /// any child event; arithmetic overflow keeps its own classification.
    /// Trace: FR-109-AC-5
    #[trace("TC-908", "FR-109-AC-5")]
    #[test]
    fn worklist_reservation_failures_are_distinct_and_precede_child_events() {
        let sequence = sequence_of(ValueType::Integer);
        let value = from_admitted(sequence.clone(), vec![integer(1), integer(2), integer(3)]);
        let policy = DenyAll {
            seen: Cell::new(None),
        };
        let walked = run_with(&ValueType::collection(sequence), &value, None, &policy);
        let request = StorageRequest::new(3, StorageUnit::AdditionalElements);
        let Err(Stop::Allocation {
            request: seen,
            locus,
        }) = walked.result
        else {
            panic!("expected an allocation failure, got {:?}", walked.result);
        };
        assert_eq!(seen, request);
        assert_eq!(policy.seen.get(), Some(request));
        assert_eq!(walked.describe(locus).0, Child(CollectionElement(2)));
        assert!(walked.seen().iter().all(|event| event.0 != ChildScheduling));
        let at = MembershipLocus {
            path: walked.arena.root(0),
            part: MembershipPart::Value,
        };
        assert_eq!(
            storage::<InvalidMembershipCause>(StorageFailure::Capacity(request), at),
            Stop::Capacity { request, locus: at }
        );
    }

    /// A starting locus from another arena is an invalid membership, never a
    /// panic, and no event fires.
    /// Trace: FR-109-AC-4
    #[trace("TC-908", "FR-109-AC-4")]
    #[test]
    fn foreign_arena_locus_is_refused_without_events() {
        let other = MembershipPathArena::new().expect("arena");
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: other.root(0),
            part: MembershipPart::Value,
        };
        let mut fired = 0;
        let result = environment().admits_walk::<InvalidMembershipCause>(
            &ValueType::Integer,
            &integer(1),
            &mut arena,
            at,
            &mut |_, _| {
                fired += 1;
                Ok(())
            },
            &GlobalReservations,
        );
        assert_eq!(
            result,
            Err(Stop::Invalid {
                cause: InvalidMembershipCause::ForeignPath,
                locus: at,
            })
        );
        assert_eq!(fired, 0);
    }

    /// A supported option/collection/reference fixture has an outer visit,
    /// two nested retained-type links, and a terminal descriptor query.
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies checked type identities without minting them in production"
    )]
    fn nested_reference() -> (TypeEnvironment, ValueType, Value) {
        let object_type = EffectiveId::from_digest([1; 32]);
        let universe = UniverseId::from_digest([2; 32]);
        let reference_type = ValueType::Reference(object_type);
        let collection = sequence_of(reference_type.clone());
        let expected = ValueType::option(ValueType::collection(collection.clone()));
        let reference = Value::Reference(ObjectReference::new(
            universe,
            object_type,
            ObjectId::new("item").expect("nonempty fixture id"),
        ));
        let value = OptionValue::from_admitted(
            ValueType::collection(collection.clone()),
            Some(from_admitted(collection, vec![reference])),
        );
        let environment = TypeEnvironment::new(
            Vec::<CompositeDeclaration>::new(),
            [ObjectTypeDeclaration::new(object_type, "Item", vec![])],
        )
        .expect("fixture declaration admits");
        (environment, expected, value)
    }

    /// The source-enumerated fixture has ten events: outer visit; two payload
    /// type links; payload scheduling and visit; two element type links;
    /// element scheduling and visit; descriptor query.
    /// Trace: FR-109-AC-1
    #[trace("TC-908", "FR-109-AC-1")]
    #[test]
    fn supplied_api_denies_zero_and_one_less_at_the_real_event() {
        let (environment, expected, value) = nested_reference();
        let cancel = Cancel::new();
        for ceiling in [0, 9, 10] {
            let mut arena = MembershipPathArena::new().expect("arena");
            let at = MembershipLocus {
                path: arena.root(0),
                part: Occurrence,
            };
            let mut budget = SuppliedMembershipBudget::new(ceiling);
            let result = environment.admit_supplied::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                at,
                &mut budget,
                &cancel,
            );
            if ceiling == 10 {
                assert_eq!(result, Ok(()));
                assert_eq!(budget.spent(), 10);
            } else {
                let Err(Stop::Limit {
                    ceiling: reported,
                    spent,
                    requested,
                    event,
                    locus,
                }) = result
                else {
                    panic!("expected the real denied event, got {result:?}");
                };
                assert_eq!((reported, spent, requested), (ceiling, ceiling, 1));
                let resolved = arena.resolve(locus).expect("denied path resolves");
                if ceiling == 0 {
                    assert_eq!((event, resolved.part()), (ValueVisit, Occurrence));
                    assert_eq!(resolved.steps_from_leaf().count(), 0);
                } else {
                    assert_eq!(
                        (event, resolved.part()),
                        (
                            MembershipEvent::DescriptorQuery,
                            MembershipPart::Descriptor(0)
                        )
                    );
                    assert_eq!(
                        resolved.steps_from_leaf().collect::<Vec<_>>(),
                        vec![CollectionElement(0), OptionPayload]
                    );
                }
                assert_eq!(budget.spent(), ceiling);
            }
        }
    }

    /// Two arguments reuse successful work, including the nested type walk;
    /// the second argument's final descriptor query is denied at total ten.
    /// Trace: FR-109-AC-2
    #[trace("TC-908", "FR-109-AC-2")]
    #[test]
    fn supplied_api_preserves_spend_across_arguments() {
        let (environment, expected, value) = nested_reference();
        let cancel = Cancel::new();
        let mut arena = MembershipPathArena::new().expect("arena");
        let mut budget = SuppliedMembershipBudget::new(10);
        let first = MembershipLocus {
            path: arena.root(0),
            part: Occurrence,
        };
        let second = MembershipLocus {
            path: arena.root(1),
            part: Occurrence,
        };
        assert_eq!(
            environment.admit_supplied::<InvalidMembershipCause>(
                &ValueType::Boolean,
                &Value::Boolean(true),
                &mut arena,
                first,
                &mut budget,
                &cancel
            ),
            Ok(())
        );
        assert_eq!(budget.spent(), 1);
        let Err(Stop::Limit {
            ceiling,
            spent,
            requested,
            event,
            locus,
        }) = environment.admit_supplied::<InvalidMembershipCause>(
            &expected,
            &value,
            &mut arena,
            second,
            &mut budget,
            &cancel,
        )
        else {
            panic!("second argument should reach its final event");
        };
        assert_eq!(
            (ceiling, spent, requested, event),
            (10, 10, 1, MembershipEvent::DescriptorQuery)
        );
        let resolved = arena.resolve(locus).expect("second path resolves");
        assert_eq!(resolved.argument(), 1);
        assert_eq!(resolved.part(), MembershipPart::Descriptor(0));
        assert_eq!(
            resolved.steps_from_leaf().collect::<Vec<_>>(),
            vec![CollectionElement(0), OptionPayload]
        );
        assert_eq!(budget.spent(), 10);
    }

    /// Invalid membership is a typed refusal after the real root visit,
    /// rather than a successful answer or a fabricated budget stop.
    /// Trace: FR-109-AC-4
    #[trace("TC-908", "FR-109-AC-4")]
    #[test]
    fn supplied_api_keeps_invalid_membership_distinct_from_a_stop() {
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(0),
            part: Occurrence,
        };
        let mut budget = SuppliedMembershipBudget::new(1);
        let result = environment().admit_supplied::<InvalidMembershipCause>(
            &ValueType::Boolean,
            &integer(1),
            &mut arena,
            at,
            &mut budget,
            &Cancel::new(),
        );
        assert_eq!(
            result,
            Err(Stop::Invalid {
                cause: InvalidMembershipCause::Kind,
                locus: at,
            })
        );
        assert_eq!(budget.spent(), 1);
    }

    struct CancelAtReservation<'a> {
        cancel: &'a Cancel,
    }

    impl ReservationPolicy for CancelAtReservation<'_> {
        fn permit(&self, _request: StorageRequest) -> bool {
            self.cancel.cancel(CancelCause::Requested);
            true
        }
    }

    /// Cancellation on the caller's handle during storage reservation is
    /// observed before the next child event; no replacement handle is used.
    /// Trace: FR-109-AC-3
    #[trace("TC-908", "FR-109-AC-3")]
    #[test]
    fn supplied_api_polls_original_handle_after_a_helper_boundary() {
        let (environment, expected, value) = nested_reference();
        let cancel = Cancel::new();
        let mut arena = MembershipPathArena::new().expect("arena");
        arena.probe_growth(1).expect("test observation reserves");
        let at = MembershipLocus {
            path: arena.root(0),
            part: Occurrence,
        };
        let mut budget = SuppliedMembershipBudget::new(10);
        let result = environment.admit_supplied_with::<InvalidMembershipCause>(
            &expected,
            &value,
            &mut arena,
            at,
            &mut budget,
            &cancel,
            &CancelAtReservation { cancel: &cancel },
        );
        assert_eq!(
            result,
            Err(Stop::Cancelled {
                cause: CancelCause::Requested,
                spent: 3,
                event: ChildScheduling,
                locus: MembershipLocus {
                    path: at.path,
                    part: Child(OptionPayload)
                },
            })
        );
        assert_eq!(budget.spent(), 3);
        assert_eq!(cancel.tripped(), Some(CancelCause::Requested));
        assert_eq!(arena.growth_observation().0, 0, "no child was scheduled");
    }

    /// The real pending-vector reservation is denied before any child event;
    /// restoring the reservation yields the same membership result.
    /// Trace: FR-109-AC-5
    #[trace("TC-908", "FR-109-AC-5")]
    #[test]
    fn supplied_api_reports_real_worklist_denial_and_recovers_on_retry() {
        let (environment, expected, value) = nested_reference();
        let cancel = Cancel::new();
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(0),
            part: Occurrence,
        };
        let mut budget = SuppliedMembershipBudget::new(10);
        let policy = DenyAll {
            seen: Cell::new(None),
        };
        let result = environment.admit_supplied_with::<InvalidMembershipCause>(
            &expected,
            &value,
            &mut arena,
            at,
            &mut budget,
            &cancel,
            &policy,
        );
        let request = StorageRequest::new(1, StorageUnit::AdditionalElements);
        assert_eq!(
            result,
            Err(Stop::Allocation {
                request,
                locus: MembershipLocus {
                    path: at.path,
                    part: Child(OptionPayload)
                },
            })
        );
        assert_eq!(policy.seen.get(), Some(request));
        assert_eq!(budget.spent(), 3);
        let mut retry_arena = MembershipPathArena::new().expect("arena");
        let retry_at = MembershipLocus {
            path: retry_arena.root(0),
            part: Occurrence,
        };
        let mut retry_budget = SuppliedMembershipBudget::new(10);
        assert_eq!(
            environment.admit_supplied::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut retry_arena,
                retry_at,
                &mut retry_budget,
                &cancel,
            ),
            Ok(())
        );
        assert_eq!(retry_budget.spent(), 10);
    }

    struct RecordReservations {
        requests: RefCell<Vec<StorageRequest>>,
        deny_call: Option<usize>,
    }

    impl RecordReservations {
        fn new(max_requests: usize, deny_call: Option<usize>) -> Self {
            let mut requests = Vec::new();
            requests
                .try_reserve_exact(max_requests)
                .expect("test observation reserves");
            Self {
                requests: RefCell::new(requests),
                deny_call,
            }
        }
    }

    impl ReservationPolicy for RecordReservations {
        fn permit(&self, request: StorageRequest) -> bool {
            let mut requests = self.requests.borrow_mut();
            let call = requests.len();
            requests.push(request);
            self.deny_call != Some(call)
        }
    }

    /// An isolated resource probe, outside the default unit-test lane. The
    /// 10k oracle is 1 outer visit + 2 type links + 10k child schedules +
    /// 10k element visits = 20,003, independent of the budget counter.
    /// Trace: FR-109-AC-1, FR-109-AC-5
    #[trace("TC-908", "FR-109-AC-1", "FR-109-AC-5")]
    #[test]
    #[ignore = "resource probe: run alone in the IR-714 performance lane"]
    fn supplied_path_arena_growth_probe_10k() {
        for count in [1_000_usize, 5_000, 10_000] {
            let sequence = sequence_of(ValueType::Integer);
            let value = from_admitted(sequence.clone(), (0..count as i64).map(integer).collect());
            let expected = ValueType::collection(sequence);
            let mut arena = MembershipPathArena::new().expect("arena");
            arena
                .probe_growth(count)
                .expect("test observation reserves");
            let at = MembershipLocus {
                path: arena.root(7),
                part: Occurrence,
            };
            let cancel = Cancel::new();
            let mut budget = SuppliedMembershipBudget::new(2 * count as u64 + 3);
            let policy = RecordReservations::new(count + 1, None);
            let env = environment();
            let started = Instant::now();
            let result = env.admit_supplied_with::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                at,
                &mut budget,
                &cancel,
                &policy,
            );
            let elapsed = started.elapsed();
            assert_eq!(result, Ok(()));
            assert_eq!(budget.spent(), 2 * count as u64 + 3);
            let (nodes, probe) = arena.growth_observation();
            assert_eq!(nodes, count);
            assert!(!probe.truncated);
            let requests = policy.requests.borrow();
            assert_eq!(
                requests.first().copied(),
                Some(StorageRequest::new(count, StorageUnit::AdditionalElements))
            );
            assert_eq!(requests.len(), probe.transitions.len() + 1);
            let copied_bound: u128 = probe
                .transitions
                .iter()
                .map(|(before, _)| {
                    (*before as u128) * (MembershipPathArena::path_node_size() as u128)
                })
                .sum();
            std::println!(
                "path_arena count={count} events={} nodes={nodes} path_reservations={} first_growth={:?} last_growth={:?} copied_bound_bytes={copied_bound} elapsed={elapsed:?}",
                budget.spent(),
                requests.len() - 1,
                probe.transitions.first(),
                probe.transitions.last(),
            );
            assert!(
                copied_bound <= (2 * count as u128 * MembershipPathArena::path_node_size() as u128),
                "path arena copying exceeds a linear bound"
            );
            if count == 10_000 {
                assert!(
                    requests.len() >= 3,
                    "probe needs a later actual path-growth boundary"
                );
                let denied_request = requests[2];
                drop(requests);
                let mut denied_arena = MembershipPathArena::new().expect("arena");
                denied_arena
                    .probe_growth(count)
                    .expect("test observation reserves");
                let denied_at = MembershipLocus {
                    path: denied_arena.root(7),
                    part: Occurrence,
                };
                let mut denied_budget = SuppliedMembershipBudget::new(2 * count as u64 + 3);
                let denied_policy = RecordReservations::new(count + 1, Some(2));
                let denied = environment().admit_supplied_with::<InvalidMembershipCause>(
                    &expected,
                    &value,
                    &mut denied_arena,
                    denied_at,
                    &mut denied_budget,
                    &cancel,
                    &denied_policy,
                );
                let Err(Stop::Allocation { request, locus }) = denied else {
                    panic!("expected a path-growth denial, got {denied:?}");
                };
                assert_eq!(request, denied_request);
                assert_eq!(
                    denied_policy.requests.borrow().get(2).copied(),
                    Some(request)
                );
                let (retained, _) = denied_arena.growth_observation();
                assert_eq!(locus.part, Child(CollectionElement(count - 1 - retained)));
                let resolved = denied_arena
                    .resolve(locus)
                    .expect("denied parent path resolves");
                assert_eq!(resolved.argument(), 7);
                assert_eq!(
                    resolved.part(),
                    Child(CollectionElement(count - 1 - retained))
                );
                assert_eq!(resolved.steps_from_leaf().count(), 0);
                assert_eq!(denied_budget.spent(), 3 + retained as u64 + 1);
                let mut retry_arena = MembershipPathArena::new().expect("arena");
                let retry_at = MembershipLocus {
                    path: retry_arena.root(7),
                    part: Occurrence,
                };
                let mut retry_budget = SuppliedMembershipBudget::new(20_003);
                assert_eq!(
                    environment().admit_supplied::<InvalidMembershipCause>(
                        &expected,
                        &value,
                        &mut retry_arena,
                        retry_at,
                        &mut retry_budget,
                        &cancel,
                    ),
                    Ok(())
                );
                assert_eq!(retry_budget.spent(), 20_003);
            }
        }
    }
}
