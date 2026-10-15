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
use core::cell::Cell;

use quire_exact::{
    Cancel, DecimalMembershipFailure, NumericMembershipFailure, NumericTypeEqualityFailure, Value,
    ValueType,
};

use super::{
    reserve_elements, GlobalReservations, ReservationPolicy, StorageFailure, TypeEnvironment,
};
use crate::supplied_membership::{
    BoundedSuppliedAdmissionError, DecimalMembershipOperand, InvalidMembershipCause,
    MembershipEvent, MembershipLocus, MembershipPart, MembershipPathArena, PathAppendError,
    PathStep, PathToken, SuppliedAdmissionError, SuppliedMembershipBudget, UnsupportedHelperFamily,
};

/// A logical event request or a response boundary after its indivisible
/// helper. Only the former spends a work unit.
pub(super) enum MembershipBoundary {
    /// Charge one logical event before its action.
    Event(MembershipEvent, MembershipLocus),
    /// Poll cancellation after an indivisible helper without another charge.
    AfterHelper(MembershipEvent, MembershipLocus),
    /// Refuse an unqualified helper in supplied admission before calling it.
    RequiresQualification(UnsupportedHelperFamily, MembershipEvent, MembershipLocus),
}

type Callback<'f, Error> = dyn FnMut(MembershipBoundary) -> Result<(), Error> + 'f;

type ValueNumericCallback<'f, Error> =
    dyn FnMut(&ValueType, &Value, MembershipLocus) -> Result<Option<bool>, Error> + 'f;
type TypeNumericCallback<'f, Error> =
    dyn FnMut(&ValueType, &ValueType, MembershipLocus) -> Result<Option<bool>, Error> + 'f;

struct NumericChecks<'f, Error> {
    value: &'f mut ValueNumericCallback<'f, Error>,
    type_link: &'f mut TypeNumericCallback<'f, Error>,
}

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
    fn reserve<Cause, Error: From<SuppliedAdmissionError<Cause>>>(
        &mut self,
        additional: usize,
        parent: PathToken,
        first: PathStep,
        on_boundary: &mut Callback<'_, Error>,
    ) -> Result<(), Error> {
        let locus = MembershipLocus {
            path: parent,
            part: MembershipPart::Child(first),
        };
        reserve_elements(&mut self.pending, additional, self.reservation)
            .map_err(|failure| Error::from(storage(failure, locus)))?;
        on_boundary(MembershipBoundary::AfterHelper(
            MembershipEvent::ChildScheduling,
            locus,
        ))
    }

    /// Emit the scheduling event for one actual child, then retain its path
    /// node and queue it into the already reserved worklist.
    fn schedule<Cause: From<InvalidMembershipCause>, Error: From<SuppliedAdmissionError<Cause>>>(
        &mut self,
        parent: PathToken,
        step: PathStep,
        child: (&'a ValueType, &'a Value),
        on_boundary: &mut Callback<'_, Error>,
    ) -> Result<(), Error> {
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
            .map_err(|failure| {
                Error::from(match failure {
                    PathAppendError::ForeignParent => {
                        invalid(InvalidMembershipCause::ForeignPath, at)
                    }
                    PathAppendError::Admission(stop) => stop,
                })
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

/// Project the exact helper's completed membership or typed stop into the
/// supplied caller without admitting a result on a stopped helper.
fn decimal_value_result<Cause>(
    result: Result<bool, DecimalMembershipFailure>,
    locus: MembershipLocus,
    spent: u64,
) -> Result<Option<bool>, BoundedSuppliedAdmissionError<Cause>> {
    result.map(Some).map_err(|failure| match failure {
        DecimalMembershipFailure::Capacity { magnitude_bits } => {
            BoundedSuppliedAdmissionError::DecimalCapacity {
                magnitude_bits,
                locus,
            }
        }
        DecimalMembershipFailure::Allocation {
            additional_elements,
            element_width,
            layout_bytes,
        } => BoundedSuppliedAdmissionError::DecimalAllocation {
            additional_elements,
            element_width,
            layout_bytes,
            locus,
        },
        DecimalMembershipFailure::Cancelled(cause) => SuppliedAdmissionError::Cancelled {
            cause,
            spent,
            event: MembershipEvent::ValueVisit,
            locus,
        }
        .into(),
    })
}

/// Compare two type chains one link pair at a time, emitting one event per
/// pair, the initial one included. Ordinals count from `first_ordinal`.
fn same_type_walk<
    Cause: From<InvalidMembershipCause>,
    Error: From<SuppliedAdmissionError<Cause>>,
>(
    left: &ValueType,
    right: &ValueType,
    path: PathToken,
    first_ordinal: u64,
    on_boundary: &mut Callback<'_, Error>,
    on_numeric_type: &mut TypeNumericCallback<'_, Error>,
) -> Result<(), Error> {
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
        if matches!(
            (left, right),
            (ValueType::Decimal(_), ValueType::Decimal(_))
        ) {
            on_boundary(MembershipBoundary::RequiresQualification(
                UnsupportedHelperFamily::DecimalDomain,
                MembershipEvent::TypeLinkComparison,
                at,
            ))?;
        }
        let numeric_equal = on_numeric_type(right, left, at)?;
        if numeric_equal.is_none()
            && matches!(
                (left, right),
                (ValueType::Int(_), ValueType::Int(_))
                    | (ValueType::Rational(_), ValueType::Rational(_))
            )
        {
            on_boundary(MembershipBoundary::RequiresQualification(
                UnsupportedHelperFamily::NumericDomain,
                MembershipEvent::TypeLinkComparison,
                at,
            ))?;
        }
        let step = match (left, right) {
            (ValueType::Option(a), ValueType::Option(b)) => TypeStep::Continue(a, b),
            (ValueType::Collection(a), ValueType::Collection(b)) => {
                if a.kind() != b.kind() || a.bound() != b.bound() {
                    TypeStep::Mismatch
                } else {
                    TypeStep::Continue(a.element(), b.element())
                }
            }
            _ if numeric_equal == Some(true) => TypeStep::Equal,
            _ if numeric_equal == Some(false) => TypeStep::Mismatch,
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
            TypeStep::Mismatch => return Err(invalid(InvalidMembershipCause::Type, at).into()),
        };
        ordinal = ordinal.saturating_add(1);
    }
}

impl TypeEnvironment {
    /// Check a supplied value with the caller's authenticated maximum magnitude
    /// bit length for numeric VALUE membership. Reuse `budget` and `arena`
    /// across arguments to keep work cumulative and loci rooted. `magnitude_bits`
    /// is explicit; this method supplies no default. The same ceiling applies
    /// to numeric values and retained numeric type links. A Decimal's declared
    /// minimum scale bounds its alignment shift because the normalized scale
    /// is nonnegative; no independent scale allowance is selected here.
    pub fn admit_supplied_bounded_numeric<Cause: From<InvalidMembershipCause>>(
        &self,
        expected: &ValueType,
        value: &Value,
        arena: &mut MembershipPathArena,
        at: MembershipLocus,
        budget: &mut SuppliedMembershipBudget,
        cancel: &Cancel,
        magnitude_bits: u64,
    ) -> Result<(), BoundedSuppliedAdmissionError<Cause>> {
        let spent = Cell::new(budget.spent());
        self.admits_walk_core(
            expected,
            value,
            arena,
            at,
            &mut |boundary| match boundary {
                MembershipBoundary::Event(event, locus) => {
                    budget.charge(event, locus, cancel)?;
                    spent.set(budget.spent());
                    Ok(())
                }
                MembershipBoundary::AfterHelper(event, locus) => budget
                    .check_cancel(event, locus, cancel)
                    .map_err(Into::into),
                MembershipBoundary::RequiresQualification(family, event, locus) => {
                    budget.check_cancel(event, locus, cancel)?;
                    Err(SuppliedAdmissionError::UnsupportedHelper { family, locus }.into())
                }
            },
            &mut NumericChecks {
                value: &mut |expected, value, locus| {
                    if let (ValueType::Decimal(domain), Value::Decimal(decimal)) = (expected, value)
                    {
                        // S = min_scale bounds max(normalized_scale, min_scale)
                        // - normalized_scale. The kernel uses only the retained
                        // normalized representation and checks its actual layout.
                        for (operand, position) in [
                            (
                                decimal.normalized().coefficient(),
                                DecimalMembershipOperand::Coefficient,
                            ),
                            (domain.lower(), DecimalMembershipOperand::Lower),
                            (domain.upper(), DecimalMembershipOperand::Upper),
                        ] {
                            let measured_bits = operand.magnitude_bits();
                            if measured_bits > magnitude_bits {
                                return Err(BoundedSuppliedAdmissionError::DecimalSize {
                                    operand: position,
                                    measured_bits,
                                    limit_bits: magnitude_bits,
                                    locus,
                                });
                            }
                        }
                        return decimal_value_result::<Cause>(
                            domain.try_contains(decimal, cancel),
                            locus,
                            spent.get(),
                        );
                    }
                    let admitted = match (expected, value) {
                        (ValueType::Int(interval), Value::Integer(integer)) => {
                            interval.try_contains_bounded(integer, magnitude_bits, cancel)
                        }
                        (ValueType::Rational(domain), Value::Rational(rational)) => {
                            domain.try_contains_bounded(rational, magnitude_bits, cancel)
                        }
                        _ => return Ok(None),
                    };
                    admitted.map(Some).map_err(|failure| match failure {
                        NumericMembershipFailure::UnsupportedSize {
                            operand,
                            measured_bits,
                            limit_bits,
                        } => BoundedSuppliedAdmissionError::NumericSize {
                            operand,
                            measured_bits,
                            limit_bits,
                            locus,
                        },
                        NumericMembershipFailure::Cancelled(cause) => {
                            SuppliedAdmissionError::Cancelled {
                                cause,
                                spent: spent.get(),
                                event: MembershipEvent::ValueVisit,
                                locus,
                            }
                            .into()
                        }
                    })
                },
                type_link: &mut |expected, actual, locus| {
                    let equal = match (expected, actual) {
                        (ValueType::Int(expected), ValueType::Int(actual)) => {
                            expected.try_eq_bounded(actual, magnitude_bits, cancel)
                        }
                        (ValueType::Rational(expected), ValueType::Rational(actual)) => {
                            expected.try_eq_bounded(actual, magnitude_bits, cancel)
                        }
                        _ => return Ok(None),
                    };
                    equal.map(Some).map_err(|failure| match failure {
                        NumericTypeEqualityFailure::UnsupportedSize {
                            position,
                            measured_bits,
                            limit_bits,
                        } => BoundedSuppliedAdmissionError::NumericTypeSize {
                            position,
                            measured_bits,
                            limit_bits,
                            locus,
                        },
                        NumericTypeEqualityFailure::Cancelled(cause) => {
                            SuppliedAdmissionError::Cancelled {
                                cause,
                                spent: spent.get(),
                                event: MembershipEvent::TypeLinkComparison,
                                locus,
                            }
                            .into()
                        }
                    })
                },
            },
            &GlobalReservations,
        )
    }

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
                MembershipBoundary::AfterHelper(..)
                | MembershipBoundary::RequiresQualification(..) => Ok(()),
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
        self.admits_walk_core(
            expected,
            value,
            arena,
            at,
            on_boundary,
            &mut NumericChecks {
                value: &mut |_, _, _| Ok(None),
                type_link: &mut |_, _, _| Ok(None),
            },
            reservation,
        )
    }

    fn admits_walk_core<
        'a,
        Cause: From<InvalidMembershipCause>,
        Error: From<SuppliedAdmissionError<Cause>>,
    >(
        &self,
        expected: &'a ValueType,
        value: &'a Value,
        arena: &mut MembershipPathArena,
        at: MembershipLocus,
        on_boundary: &mut impl FnMut(MembershipBoundary) -> Result<(), Error>,
        numeric: &mut NumericChecks<'_, Error>,
        reservation: &impl ReservationPolicy,
    ) -> Result<(), Error> {
        if arena.resolve(at).is_none() {
            return Err(invalid(InvalidMembershipCause::ForeignPath, at).into());
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
            self.admit_occurrence::<Cause, Error>(
                expected,
                value,
                path,
                &mut frontier,
                on_boundary,
                numeric,
            )?;
        }
        Ok(())
    }

    fn admit_occurrence<
        'a,
        Cause: From<InvalidMembershipCause>,
        Error: From<SuppliedAdmissionError<Cause>>,
    >(
        &self,
        expected: &'a ValueType,
        value: &'a Value,
        path: PathToken,
        frontier: &mut Frontier<'_, 'a, impl ReservationPolicy>,
        on_boundary: &mut Callback<'_, Error>,
        numeric: &mut NumericChecks<'_, Error>,
    ) -> Result<(), Error> {
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
                    return Err(invalid(InvalidMembershipCause::ReferenceConformance, query).into());
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
                    return Err(invalid(InvalidMembershipCause::CompositeConformance, query).into());
                }
            }
            (ValueType::Enum(_), Value::Enum(_)) => {
                on_boundary(MembershipBoundary::Event(
                    MembershipEvent::DescriptorQuery,
                    query,
                ))?;
                let admitted = expected.admits(value);
                on_boundary(MembershipBoundary::AfterHelper(
                    MembershipEvent::DescriptorQuery,
                    query,
                ))?;
                if !admitted {
                    return Err(invalid(InvalidMembershipCause::Member, query).into());
                }
            }
            (ValueType::Composite(_), Value::Union(_)) => {
                on_boundary(MembershipBoundary::RequiresQualification(
                    UnsupportedHelperFamily::UnionPayloadDescriptor,
                    MembershipEvent::ValueVisit,
                    here,
                ))?;
                return Err(invalid(InvalidMembershipCause::UnsupportedUnion, here).into());
            }
            (ValueType::Option(payload), Value::Option(option)) => {
                same_type_walk::<Cause, Error>(
                    option.payload_type(),
                    payload,
                    path,
                    0,
                    on_boundary,
                    numeric.type_link,
                )?;
                if let Some(present) = option.payload() {
                    frontier.reserve::<Cause, Error>(
                        1,
                        path,
                        PathStep::OptionPayload,
                        on_boundary,
                    )?;
                    frontier.schedule::<Cause, Error>(
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
                    return Err(invalid(InvalidMembershipCause::Type, pair).into());
                }
                same_type_walk::<Cause, Error>(
                    actual.element(),
                    declared.element(),
                    path,
                    1,
                    on_boundary,
                    numeric.type_link,
                )?;
                let elements = collection.elements();
                if let Some(last) = elements.len().checked_sub(1) {
                    frontier.reserve::<Cause, Error>(
                        elements.len(),
                        path,
                        PathStep::CollectionElement(last),
                        on_boundary,
                    )?;
                }
                for (index, element) in elements.iter().enumerate().rev() {
                    frontier.schedule::<Cause, Error>(
                        path,
                        PathStep::CollectionElement(index),
                        (declared.element(), element),
                        on_boundary,
                    )?;
                }
            }
            _ => {
                let numeric_admitted = (numeric.value)(expected, value, here)?;
                if numeric_admitted.is_none()
                    && matches!(
                        (expected, value),
                        (ValueType::Decimal(_), Value::Decimal(_))
                    )
                {
                    on_boundary(MembershipBoundary::RequiresQualification(
                        UnsupportedHelperFamily::DecimalDomain,
                        MembershipEvent::ValueVisit,
                        here,
                    ))?;
                }
                if numeric_admitted.is_none()
                    && matches!(
                        (expected, value),
                        (ValueType::Int(_), Value::Integer(_))
                            | (ValueType::Rational(_), Value::Rational(_))
                    )
                {
                    on_boundary(MembershipBoundary::RequiresQualification(
                        UnsupportedHelperFamily::NumericDomain,
                        MembershipEvent::ValueVisit,
                        here,
                    ))?;
                }
                let admitted = numeric_admitted.unwrap_or_else(|| expected.admits(value));
                on_boundary(MembershipBoundary::AfterHelper(
                    MembershipEvent::ValueVisit,
                    here,
                ))?;
                if !admitted {
                    return Err(invalid(leaf_cause(expected, value), here).into());
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
        from_admitted, Cancel, CancelCause, CollectionKind, CollectionType, Decimal, DecimalType,
        EffectiveId, EnumMember, EnumShape, Identifier, Integer, IntegerInterval, NodeKey,
        ObjectId, ObjectReference, OptionValue, Rational, RationalDomain, RoundingMode,
        UnionMember, UnionValue, UniverseId, VariantId,
    };

    use super::*;
    use crate::declaration::{
        CompositeDeclaration, GlobalReservations, ObjectTypeDeclaration, StorageRequest,
        StorageUnit,
    };
    use crate::supplied_membership::{SuppliedMembershipBudget, UnsupportedHelperFamily};

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
    /// any child event.
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
    }

    /// The actual pending-worklist reserve rejects size overflow before a
    /// child can be scheduled or a reservation policy can run. A synthetic
    /// child count reaches this checked boundary without allocating it.
    /// Trace: FR-109-AC-5
    #[trace("TC-908", "FR-109-AC-5")]
    #[test]
    fn worklist_checked_size_overflow_is_typed_capacity() {
        let mut arena = MembershipPathArena::new().expect("arena");
        let root = arena.root(0);
        let expected = ValueType::Integer;
        let value = integer(1);
        let pending = vec![(&expected, &value, root)];
        let policy = DenyAll {
            seen: Cell::new(None),
        };
        let mut frontier = Frontier {
            arena: &mut arena,
            pending,
            reservation: &policy,
        };
        let mut after_reservation = false;
        let result = frontier.reserve::<InvalidMembershipCause, Stop>(
            usize::MAX,
            root,
            PathStep::CollectionElement(0),
            &mut |_| {
                after_reservation = true;
                Ok(())
            },
        );
        let request = StorageRequest::new(usize::MAX, StorageUnit::AdditionalElements);
        let locus = MembershipLocus {
            path: root,
            part: Child(CollectionElement(0)),
        };
        assert_eq!(result, Err(Stop::Capacity { request, locus }));
        assert!(!after_reservation);
        assert_eq!(policy.seen.get(), None);
        assert_eq!(frontier.pending.len(), 1);
        assert_eq!(
            arena.resolve(locus).expect("root remains valid").argument(),
            0
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

    /// The public supplied API preserves the owning ranked-member cause at
    /// the descriptor query for a member outside its admitted shape.
    /// Trace: FR-109-AC-4
    #[trace("TC-908", "FR-109-AC-4")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies checked enum identities without minting them in production"
    )]
    #[test]
    fn supplied_api_reports_enum_member_failure() {
        let admitted = VariantId::from_digest([1; 32]);
        let missing = VariantId::from_digest([2; 32]);
        let expected = ValueType::Enum(EnumShape::new(false, [admitted]));
        let value = Value::Enum(EnumMember::new(missing, 0));
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(0),
            part: Occurrence,
        };
        let mut budget = SuppliedMembershipBudget::new(2);
        let query = MembershipLocus {
            path: at.path,
            part: MembershipPart::Descriptor(0),
        };
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                at,
                &mut budget,
                &Cancel::new(),
            ),
            Err(Stop::Invalid {
                cause: InvalidMembershipCause::Member,
                locus: query,
            })
        );
        assert_eq!(budget.spent(), 2);
        assert_eq!(
            arena.resolve(query).expect("root remains valid").argument(),
            0
        );
    }

    /// The source-enumerated Enum path for a later ranked member is ValueVisit
    /// then one DescriptorQuery. Zero denies the visit, one denies the lookup,
    /// and two admit the member.
    /// Trace: FR-109-AC-1, FR-109-AC-4
    #[trace("TC-908", "FR-109-AC-1", "FR-109-AC-4")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies checked enum identities without minting them in production"
    )]
    #[test]
    fn supplied_enum_query_has_exact_two_event_boundary() {
        let first = VariantId::from_digest([1; 32]);
        let second = VariantId::from_digest([2; 32]);
        let expected = ValueType::Enum(EnumShape::new(false, [first, second]));
        let value = Value::Enum(EnumMember::new(second, 1));
        let cancel = Cancel::new();
        for ceiling in [0, 1, 2] {
            let mut arena = MembershipPathArena::new().expect("arena");
            let at = MembershipLocus {
                path: arena.root(0),
                part: Occurrence,
            };
            let query = MembershipLocus {
                path: at.path,
                part: MembershipPart::Descriptor(0),
            };
            let mut budget = SuppliedMembershipBudget::new(ceiling);
            let result = environment().admit_supplied::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                at,
                &mut budget,
                &cancel,
            );
            let expected_result = match ceiling {
                0 => Err(Stop::Limit {
                    ceiling: 0,
                    spent: 0,
                    requested: 1,
                    event: ValueVisit,
                    locus: at,
                }),
                1 => Err(Stop::Limit {
                    ceiling: 1,
                    spent: 1,
                    requested: 1,
                    event: MembershipEvent::DescriptorQuery,
                    locus: query,
                }),
                2 => Ok(()),
                _ => unreachable!("the fixture has only three ceilings"),
            };
            assert_eq!(result, expected_result);
            assert_eq!(budget.spent(), ceiling);
        }

        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(1),
            part: Occurrence,
        };
        let mut budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &expected,
                &Value::Boolean(true),
                &mut arena,
                at,
                &mut budget,
                &cancel,
            ),
            Err(Stop::Invalid {
                cause: InvalidMembershipCause::Kind,
                locus: at,
            })
        );
        assert_eq!(budget.spent(), 1);
    }

    /// Every numeric operand is checked against the explicit bit ceiling
    /// after its value-visit event has been charged.
    /// Trace: FR-109-AC-1, FR-109-AC-7
    #[trace("TC-908", "FR-109-AC-1", "FR-109-AC-7")]
    #[test]
    fn bounded_numeric_value_checks_all_three_integer_and_six_rational_operands() {
        use quire_exact::NumericMembershipOperand as Operand;

        let i = |value: i64| Integer::from(value);
        let interval = |lower, upper| IntegerInterval::spanning(i(lower), i(upper));
        let rational = |numerator, denominator| {
            Rational::new(i(numerator), i(denominator)).expect("positive denominator")
        };
        let domain = |nl, nu, dl, du| {
            RationalDomain::new(interval(nl, nu), interval(dl, du))
                .expect("positive denominator interval")
        };
        let cases = [
            (
                ValueType::Int(interval(0, 1)),
                integer(2),
                Operand::IntegerValue,
            ),
            (
                ValueType::Int(interval(2, 2)),
                integer(1),
                Operand::IntegerLower,
            ),
            (
                ValueType::Int(interval(0, 2)),
                integer(1),
                Operand::IntegerUpper,
            ),
            (
                ValueType::Rational(domain(0, 1, 1, 1)),
                Value::Rational(rational(2, 1)),
                Operand::RationalNumerator,
            ),
            (
                ValueType::Rational(domain(0, 1, 1, 2)),
                Value::Rational(rational(1, 2)),
                Operand::RationalDenominator,
            ),
            (
                ValueType::Rational(domain(2, 2, 1, 1)),
                Value::Rational(rational(1, 1)),
                Operand::RationalNumeratorLower,
            ),
            (
                ValueType::Rational(domain(0, 2, 1, 1)),
                Value::Rational(rational(1, 1)),
                Operand::RationalNumeratorUpper,
            ),
            (
                ValueType::Rational(domain(0, 1, 2, 2)),
                Value::Rational(rational(1, 1)),
                Operand::RationalDenominatorLower,
            ),
            (
                ValueType::Rational(domain(0, 1, 1, 2)),
                Value::Rational(rational(1, 1)),
                Operand::RationalDenominatorUpper,
            ),
        ];
        for (expected, value, operand) in cases {
            let mut arena = MembershipPathArena::new().expect("arena");
            let at = MembershipLocus {
                path: arena.root(4),
                part: MembershipPart::Value,
            };
            let mut budget = SuppliedMembershipBudget::new(1);
            assert_eq!(
                environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                    &expected,
                    &value,
                    &mut arena,
                    at,
                    &mut budget,
                    &Cancel::new(),
                    1,
                ),
                Err(BoundedSuppliedAdmissionError::NumericSize {
                    operand,
                    measured_bits: 2,
                    limit_bits: 1,
                    locus: at,
                })
            );
            assert_eq!(budget.spent(), 1);
            assert_eq!(arena.resolve(at).expect("rooted locus").argument(), 4);
        }
    }

    /// The value event precedes the numeric helper, and the same budget covers
    /// later arguments. Zero needs one bit, even when the integer is zero.
    /// Trace: FR-109-AC-1, FR-109-AC-2, FR-109-AC-3
    #[trace("TC-908", "FR-109-AC-1", "FR-109-AC-2", "FR-109-AC-3")]
    #[test]
    fn bounded_numeric_respects_event_order_zero_and_original_cancel() {
        let expected = ValueType::Int(IntegerInterval::spanning(Integer::zero(), Integer::one()));
        let value = integer(0);
        let mut arena = MembershipPathArena::new().expect("arena");
        let first = MembershipLocus {
            path: arena.root(0),
            part: MembershipPart::Value,
        };
        let second = MembershipLocus {
            path: arena.root(1),
            part: MembershipPart::Value,
        };
        let cancel = Cancel::new();
        let mut budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                first,
                &mut budget,
                &cancel,
                1,
            ),
            Ok(())
        );
        assert_eq!(budget.spent(), 1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                second,
                &mut budget,
                &cancel,
                0,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Limit {
                ceiling: 1,
                spent: 1,
                requested: 1,
                event: MembershipEvent::ValueVisit,
                locus: second,
            }))
        );
        assert_eq!(budget.spent(), 1);

        let mut zero_budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                second,
                &mut zero_budget,
                &cancel,
                0,
            ),
            Err(BoundedSuppliedAdmissionError::NumericSize {
                operand: quire_exact::NumericMembershipOperand::IntegerValue,
                measured_bits: 1,
                limit_bits: 0,
                locus: second,
            })
        );
        assert_eq!(zero_budget.spent(), 1);

        cancel.cancel(CancelCause::Requested);
        let mut cancelled_budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                second,
                &mut cancelled_budget,
                &cancel,
                0,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Cancelled {
                cause: CancelCause::Requested,
                spent: 0,
                event: MembershipEvent::ValueVisit,
                locus: second,
            }))
        );
        assert_eq!(cancelled_budget.spent(), 0);
        assert_eq!(cancel.tripped(), Some(CancelCause::Requested));
    }

    /// The seventh poll is the first borrowed-limb comparison after the
    /// value event, helper entry, three operand checks, and comparison entry.
    /// Trace: FR-109-AC-3
    #[trace("TC-908", "FR-109-AC-3")]
    #[test]
    fn bounded_numeric_maps_cancellation_during_kernel_comparison() {
        use core::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::{Arc, Mutex};

        let polls = Arc::new(AtomicUsize::new(0));
        let trigger = Arc::new(Mutex::new(None::<Cancel>));
        let observed_polls = Arc::clone(&polls);
        let observed_trigger = Arc::clone(&trigger);
        let cancel = Cancel::observing(move || {
            let ordinal = observed_polls.fetch_add(1, Ordering::SeqCst) + 1;
            if ordinal == 7 {
                observed_trigger
                    .lock()
                    .expect("test cancellation handle lock")
                    .take()
                    .expect("test cancellation handle")
                    .cancel(CancelCause::Deadline);
            }
        });
        *trigger.lock().expect("test cancellation handle lock") = Some(cancel.clone());

        let expected = ValueType::Int(IntegerInterval::spanning(
            Integer::from(10_i64),
            Integer::from(12_i64),
        ));
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(7),
            part: MembershipPart::Value,
        };
        let mut budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &integer(11),
                &mut arena,
                at,
                &mut budget,
                &cancel,
                4,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Cancelled {
                cause: CancelCause::Deadline,
                spent: 1,
                event: MembershipEvent::ValueVisit,
                locus: at,
            }))
        );
        assert_eq!(budget.spent(), 1);
        assert_eq!(polls.load(Ordering::SeqCst), 7);
        assert_eq!(cancel.tripped(), Some(CancelCause::Deadline));
        assert_eq!(arena.resolve(at).expect("rooted locus").argument(), 7);
    }

    /// The new caller path reports invalid membership and successful bounded
    /// numeric membership while retaining an independent root for each call.
    /// Trace: FR-109-AC-2, FR-109-AC-4
    #[trace("TC-908", "FR-109-AC-2", "FR-109-AC-4")]
    #[test]
    fn bounded_numeric_accepts_at_b_and_rejects_outside_domain() {
        let expected = ValueType::Int(IntegerInterval::spanning(
            Integer::zero(),
            Integer::from(2_i64),
        ));
        let mut arena = MembershipPathArena::new().expect("arena");
        let first = MembershipLocus {
            path: arena.root(0),
            part: MembershipPart::Value,
        };
        let second = MembershipLocus {
            path: arena.root(1),
            part: MembershipPart::Value,
        };
        let mut budget = SuppliedMembershipBudget::new(2);
        let cancel = Cancel::new();
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &integer(2),
                &mut arena,
                first,
                &mut budget,
                &cancel,
                2,
            ),
            Ok(())
        );
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &integer(3),
                &mut arena,
                second,
                &mut budget,
                &cancel,
                2,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Invalid {
                cause: InvalidMembershipCause::Payload,
                locus: second,
            }))
        );
        assert_eq!(budget.spent(), 2);

        let rational =
            Rational::new(Integer::one(), Integer::from(2_i64)).expect("positive denominator");
        let rational_type = ValueType::Rational(
            RationalDomain::new(
                IntegerInterval::spanning(Integer::zero(), Integer::one()),
                IntegerInterval::spanning(Integer::one(), Integer::from(2_i64)),
            )
            .expect("positive denominator interval"),
        );
        let third = MembershipLocus {
            path: arena.root(2),
            part: MembershipPart::Value,
        };
        let mut rational_budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &rational_type,
                &Value::Rational(rational),
                &mut arena,
                third,
                &mut rational_budget,
                &cancel,
                2,
            ),
            Ok(())
        );
        assert_eq!(rational_budget.spent(), 1);
    }

    /// Numeric links use one charged event and report a rooted typed endpoint
    /// stop before comparing any of the four or eight retained operands.
    /// Trace: FR-117-AC-1, FR-117-AC-2
    #[trace("TC-917", "FR-117-AC-1", "FR-117-AC-2")]
    #[test]
    fn bounded_numeric_type_links_check_expected_then_actual_endpoints() {
        use quire_exact::NumericTypeEndpoint as Endpoint;

        let i = |n: i64| Integer::from(n);
        let interval = |lower, upper| IntegerInterval::spanning(i(lower), i(upper));
        let domain = |nl, nu, dl, du| {
            RationalDomain::new(interval(nl, nu), interval(dl, du))
                .expect("positive denominator range")
        };
        let int_cases = [
            (
                interval(2, 2),
                interval(0, 1),
                Endpoint::IntegerExpectedLower,
            ),
            (
                interval(0, 2),
                interval(0, 1),
                Endpoint::IntegerExpectedUpper,
            ),
            (interval(0, 1), interval(2, 2), Endpoint::IntegerActualLower),
            (interval(0, 1), interval(0, 2), Endpoint::IntegerActualUpper),
        ];
        let rational_cases = [
            (
                domain(2, 2, 1, 1),
                domain(0, 1, 1, 1),
                Endpoint::RationalExpectedNumeratorLower,
            ),
            (
                domain(0, 2, 1, 1),
                domain(0, 1, 1, 1),
                Endpoint::RationalExpectedNumeratorUpper,
            ),
            (
                domain(0, 1, 2, 2),
                domain(0, 1, 1, 1),
                Endpoint::RationalExpectedDenominatorLower,
            ),
            (
                domain(0, 1, 1, 2),
                domain(0, 1, 1, 1),
                Endpoint::RationalExpectedDenominatorUpper,
            ),
            (
                domain(0, 1, 1, 1),
                domain(2, 2, 1, 1),
                Endpoint::RationalActualNumeratorLower,
            ),
            (
                domain(0, 1, 1, 1),
                domain(0, 2, 1, 1),
                Endpoint::RationalActualNumeratorUpper,
            ),
            (
                domain(0, 1, 1, 1),
                domain(0, 1, 2, 2),
                Endpoint::RationalActualDenominatorLower,
            ),
            (
                domain(0, 1, 1, 1),
                domain(0, 1, 1, 2),
                Endpoint::RationalActualDenominatorUpper,
            ),
        ];
        let cases = int_cases
            .into_iter()
            .map(|(expected, actual, position)| {
                (ValueType::Int(expected), ValueType::Int(actual), position)
            })
            .chain(
                rational_cases
                    .into_iter()
                    .map(|(expected, actual, position)| {
                        (
                            ValueType::Rational(expected),
                            ValueType::Rational(actual),
                            position,
                        )
                    }),
            );
        for (expected, actual, position) in cases {
            let mut arena = MembershipPathArena::new().expect("arena");
            let root = arena.root(9);
            let at = MembershipLocus {
                path: root,
                part: Occurrence,
            };
            let link = MembershipLocus {
                path: root,
                part: TypeLink(0),
            };
            let value = OptionValue::from_admitted(actual, None);
            let mut budget = SuppliedMembershipBudget::new(2);
            assert_eq!(
                environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                    &ValueType::option(expected.clone()),
                    &value,
                    &mut arena,
                    at,
                    &mut budget,
                    &Cancel::new(),
                    1,
                ),
                Err(BoundedSuppliedAdmissionError::NumericTypeSize {
                    position,
                    measured_bits: 2,
                    limit_bits: 1,
                    locus: link,
                })
            );
            assert_eq!(budget.spent(), 2);
            assert_eq!(arena.resolve(link).expect("link locus").argument(), 9);
            let mut at_bound_budget = SuppliedMembershipBudget::new(2);
            assert_eq!(
                environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                    &ValueType::option(expected),
                    &value,
                    &mut arena,
                    at,
                    &mut at_bound_budget,
                    &Cancel::new(),
                    2,
                ),
                Err(BoundedSuppliedAdmissionError::Admission(Stop::Invalid {
                    cause: InvalidMembershipCause::Type,
                    locus: link,
                }))
            );
            assert_eq!(at_bound_budget.spent(), 2);
        }

        for (numeric, first_position) in [
            (
                ValueType::Int(interval(2, 2)),
                Endpoint::IntegerExpectedLower,
            ),
            (
                ValueType::Rational(domain(2, 2, 1, 1)),
                Endpoint::RationalExpectedNumeratorLower,
            ),
        ] {
            let mut arena = MembershipPathArena::new().expect("arena");
            let root = arena.root(10);
            let at = MembershipLocus {
                path: root,
                part: Occurrence,
            };
            let link = MembershipLocus {
                path: root,
                part: TypeLink(0),
            };
            let value = OptionValue::from_admitted(numeric.clone(), None);
            let mut budget = SuppliedMembershipBudget::new(2);
            assert_eq!(
                environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                    &ValueType::option(numeric),
                    &value,
                    &mut arena,
                    at,
                    &mut budget,
                    &Cancel::new(),
                    1,
                ),
                Err(BoundedSuppliedAdmissionError::NumericTypeSize {
                    position: first_position,
                    measured_bits: 2,
                    limit_bits: 1,
                    locus: link,
                })
            );
            assert_eq!(budget.spent(), 2);
            assert_eq!(arena.resolve(link).expect("rooted link").argument(), 10);
        }
    }

    /// The nested collection header and numeric element link spend separately;
    /// a denied link never enters the exact helper.
    /// Trace: FR-117-AC-1, FR-117-AC-2, FR-117-AC-4
    #[trace("TC-917", "FR-117-AC-1", "FR-117-AC-2", "FR-117-AC-4")]
    #[test]
    fn bounded_numeric_type_link_budget_equality_and_zero_boundary() {
        let interval =
            |upper: i64| IntegerInterval::spanning(Integer::zero(), Integer::from(upper));
        let expected = ValueType::collection(sequence_of(ValueType::Int(interval(1))));
        for (actual_upper, ceiling, magnitude_bits, outcome, spent) in [
            (1, 2, 0, 0, 2), // denied link before helper
            (1, 3, 1, 1, 3), // equal at B, including zero lower endpoints
            (2, 3, 2, 2, 3), // unequal after a completed comparison
            (1, 3, 0, 3, 3), // zero needs one bit
        ] {
            let actual_type = sequence_of(ValueType::Int(interval(actual_upper)));
            let value = from_admitted(actual_type, vec![]);
            let mut arena = MembershipPathArena::new().expect("arena");
            let root = arena.root(3);
            let at = MembershipLocus {
                path: root,
                part: Occurrence,
            };
            let link = MembershipLocus {
                path: root,
                part: TypeLink(1),
            };
            let mut budget = SuppliedMembershipBudget::new(ceiling);
            let result = environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                at,
                &mut budget,
                &Cancel::new(),
                magnitude_bits,
            );
            match outcome {
                0 => assert_eq!(
                    result,
                    Err(BoundedSuppliedAdmissionError::Admission(Stop::Limit {
                        ceiling,
                        spent: 2,
                        requested: 1,
                        event: TypeLinkComparison,
                        locus: link,
                    }))
                ),
                1 => assert_eq!(result, Ok(())),
                2 => assert_eq!(
                    result,
                    Err(BoundedSuppliedAdmissionError::Admission(Stop::Invalid {
                        cause: InvalidMembershipCause::Type,
                        locus: link,
                    }))
                ),
                3 => assert_eq!(
                    result,
                    Err(BoundedSuppliedAdmissionError::NumericTypeSize {
                        position: quire_exact::NumericTypeEndpoint::IntegerExpectedLower,
                        measured_bits: 1,
                        limit_bits: 0,
                        locus: link,
                    })
                ),
                _ => unreachable!("fixed test case"),
            }
            assert_eq!(budget.spent(), spent);
            assert_eq!(arena.resolve(link).expect("link locus").argument(), 3);
        }
    }

    /// The tenth poll is the second borrowed limb step: two charged events,
    /// kernel entry and four endpoint checks precede the first two limbs.
    /// Trace: FR-117-AC-3
    #[trace("TC-917", "FR-117-AC-3")]
    #[test]
    fn bounded_numeric_type_link_preserves_late_kernel_cancellation() {
        use core::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::{Arc, Mutex};

        let polls = Arc::new(AtomicUsize::new(0));
        let trigger = Arc::new(Mutex::new(None::<Cancel>));
        let observed_polls = Arc::clone(&polls);
        let observed_trigger = Arc::clone(&trigger);
        let cancel = Cancel::observing(move || {
            let ordinal = observed_polls.fetch_add(1, Ordering::SeqCst) + 1;
            if ordinal == 10 {
                observed_trigger
                    .lock()
                    .expect("test cancellation handle lock")
                    .take()
                    .expect("test cancellation handle")
                    .cancel(CancelCause::Deadline);
            }
        });
        *trigger.lock().expect("test cancellation handle lock") = Some(cancel.clone());

        let endpoint = Integer::one().shifted_left(63);
        let numeric = ValueType::Int(IntegerInterval::spanning(endpoint.clone(), endpoint));
        let value = OptionValue::from_admitted(numeric.clone(), None);
        let mut arena = MembershipPathArena::new().expect("arena");
        let root = arena.root(7);
        let at = MembershipLocus {
            path: root,
            part: Occurrence,
        };
        let link = MembershipLocus {
            path: root,
            part: TypeLink(0),
        };
        let mut budget = SuppliedMembershipBudget::new(2);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &ValueType::option(numeric),
                &value,
                &mut arena,
                at,
                &mut budget,
                &cancel,
                64,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Cancelled {
                cause: CancelCause::Deadline,
                spent: 2,
                event: TypeLinkComparison,
                locus: link,
            }))
        );
        assert_eq!(budget.spent(), 2);
        assert_eq!(polls.load(Ordering::SeqCst), 10);
        assert_eq!(arena.resolve(link).expect("rooted link").argument(), 7);
    }

    /// A pre-cancelled bounded call stops at the original event even when its
    /// numeric type links would exceed B.
    /// Trace: FR-117-AC-3
    #[trace("TC-917", "FR-117-AC-3")]
    #[test]
    fn bounded_numeric_type_link_precancel_beats_oversized_endpoints() {
        let numeric = ValueType::Int(IntegerInterval::spanning(
            Integer::from(2_i64),
            Integer::from(2_i64),
        ));
        let value = OptionValue::from_admitted(numeric.clone(), None);
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(11),
            part: Occurrence,
        };
        let cancel = Cancel::new();
        cancel.cancel(CancelCause::Deadline);
        let mut budget = SuppliedMembershipBudget::new(2);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &ValueType::option(numeric),
                &value,
                &mut arena,
                at,
                &mut budget,
                &cancel,
                1,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Cancelled {
                cause: CancelCause::Deadline,
                spent: 0,
                event: ValueVisit,
                locus: at,
            }))
        );
        assert_eq!(budget.spent(), 0);
        assert_eq!(arena.resolve(at).expect("rooted occurrence").argument(), 11);
    }

    /// The direct API keeps the numeric helper unqualified. The caller's
    /// original cancellation and event denial retain their earlier priority.
    /// Trace: FR-117-AC-3, FR-117-AC-4
    #[trace("TC-917", "FR-117-AC-3", "FR-117-AC-4")]
    #[test]
    fn numeric_type_link_without_bound_preserves_event_precedence() {
        let numeric = ValueType::Int(IntegerInterval::spanning(Integer::zero(), Integer::one()));
        let value = OptionValue::from_admitted(numeric.clone(), None);
        let expected = ValueType::option(numeric);
        let mut arena = MembershipPathArena::new().expect("arena");
        let root = arena.root(12);
        let at = MembershipLocus {
            path: root,
            part: Occurrence,
        };
        let link = MembershipLocus {
            path: root,
            part: TypeLink(0),
        };

        let mut denied = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                at,
                &mut denied,
                &Cancel::new(),
            ),
            Err(Stop::Limit {
                ceiling: 1,
                spent: 1,
                requested: 1,
                event: TypeLinkComparison,
                locus: link,
            })
        );
        assert_eq!(denied.spent(), 1);

        let mut unqualified = SuppliedMembershipBudget::new(2);
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                at,
                &mut unqualified,
                &Cancel::new(),
            ),
            Err(Stop::UnsupportedHelper {
                family: UnsupportedHelperFamily::NumericDomain,
                locus: link,
            })
        );
        assert_eq!(unqualified.spent(), 2);

        let cancelled = Cancel::new();
        cancelled.cancel(CancelCause::Requested);
        let mut cancelled_budget = SuppliedMembershipBudget::new(2);
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                at,
                &mut cancelled_budget,
                &cancelled,
            ),
            Err(Stop::Cancelled {
                cause: CancelCause::Requested,
                spent: 0,
                event: ValueVisit,
                locus: at,
            })
        );
        assert_eq!(cancelled_budget.spent(), 0);
    }

    /// A single caller budget and B apply across two argument roots, including
    /// an equal rational type link on the second argument.
    /// Trace: FR-117-AC-1, FR-117-AC-2
    #[trace("TC-917", "FR-117-AC-1", "FR-117-AC-2")]
    #[test]
    fn bounded_numeric_type_links_share_budget_across_arguments() {
        let integer = ValueType::Int(IntegerInterval::spanning(Integer::zero(), Integer::one()));
        let rational = ValueType::Rational(
            RationalDomain::new(
                IntegerInterval::spanning(Integer::zero(), Integer::one()),
                IntegerInterval::spanning(Integer::one(), Integer::one()),
            )
            .expect("positive denominator range"),
        );
        let mut arena = MembershipPathArena::new().expect("arena");
        let first = MembershipLocus {
            path: arena.root(0),
            part: Occurrence,
        };
        let second = MembershipLocus {
            path: arena.root(1),
            part: Occurrence,
        };
        let mut budget = SuppliedMembershipBudget::new(4);
        let cancel = Cancel::new();
        for (at, numeric) in [(first, integer), (second, rational)] {
            let value = OptionValue::from_admitted(numeric.clone(), None);
            assert_eq!(
                environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                    &ValueType::option(numeric),
                    &value,
                    &mut arena,
                    at,
                    &mut budget,
                    &cancel,
                    1,
                ),
                Ok(())
            );
        }
        assert_eq!(budget.spent(), 4);
        assert_eq!(arena.resolve(second).expect("second root").argument(), 1);
    }

    /// Supplied numeric domains stop before unbounded BigInt comparisons,
    /// including those reached through a retained nested type. The plain
    /// Integer kind still has a fixed-time kind check.
    /// Trace: FR-109-AC-7
    #[trace("TC-908", "FR-109-AC-7")]
    #[test]
    fn supplied_api_reports_unqualified_numeric_domain_before_comparison() {
        let interval = IntegerInterval::spanning(Integer::zero(), Integer::from(10_i64));
        let rational = RationalDomain::new(
            interval.clone(),
            IntegerInterval::spanning(Integer::one(), Integer::from(2_i64)),
        )
        .expect("positive denominator range");
        for (expected, value) in [
            (ValueType::Int(interval.clone()), integer(1)),
            (
                ValueType::Rational(rational),
                Value::Rational(Rational::from_integer(Integer::one())),
            ),
        ] {
            let mut arena = MembershipPathArena::new().expect("arena");
            let at = MembershipLocus {
                path: arena.root(0),
                part: Occurrence,
            };
            let mut budget = SuppliedMembershipBudget::new(1);
            assert_eq!(
                environment().admit_supplied::<InvalidMembershipCause>(
                    &expected,
                    &value,
                    &mut arena,
                    at,
                    &mut budget,
                    &Cancel::new(),
                ),
                Err(Stop::UnsupportedHelper {
                    family: UnsupportedHelperFamily::NumericDomain,
                    locus: at,
                })
            );
            assert_eq!(budget.spent(), 1);
        }

        let numeric = ValueType::Int(interval);
        let nested = OptionValue::from_admitted(numeric.clone(), None);
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(1),
            part: Occurrence,
        };
        let mut budget = SuppliedMembershipBudget::new(2);
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &ValueType::option(numeric),
                &nested,
                &mut arena,
                at,
                &mut budget,
                &Cancel::new(),
            ),
            Err(Stop::UnsupportedHelper {
                family: UnsupportedHelperFamily::NumericDomain,
                locus: MembershipLocus {
                    path: at.path,
                    part: TypeLink(0),
                },
            })
        );
        assert_eq!(budget.spent(), 2);

        let mut kind_arena = MembershipPathArena::new().expect("arena");
        let kind_at = MembershipLocus {
            path: kind_arena.root(2),
            part: Occurrence,
        };
        let mut kind_budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &ValueType::Integer,
                &integer(1),
                &mut kind_arena,
                kind_at,
                &mut kind_budget,
                &Cancel::new(),
            ),
            Ok(())
        );
        assert_eq!(kind_budget.spent(), 1);
    }

    /// The supplied API stops before the bool-only Decimal domain helper,
    /// both at a direct value and at a retained nested type comparison.
    /// Trace: FR-109-AC-7
    #[trace("TC-908", "FR-109-AC-7")]
    #[test]
    fn supplied_api_reports_unqualified_decimal_before_the_helper() {
        let decimal_type = ValueType::Decimal(
            DecimalType::new(
                Integer::zero(),
                Integer::from(100_i64),
                2,
                2,
                RoundingMode::Exact,
            )
            .expect("bounded decimal fixture"),
        );
        let decimal = Value::Decimal(Decimal::new(Integer::one(), 0));
        let cancel = Cancel::new();
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(0),
            part: Occurrence,
        };
        let mut budget = SuppliedMembershipBudget::new(4);
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &decimal_type,
                &decimal,
                &mut arena,
                at,
                &mut budget,
                &cancel,
            ),
            Err(Stop::UnsupportedHelper {
                family: UnsupportedHelperFamily::DecimalDomain,
                locus: at,
            })
        );
        assert_eq!(budget.spent(), 1);

        let mut nested_arena = MembershipPathArena::new().expect("arena");
        let nested_at = MembershipLocus {
            path: nested_arena.root(1),
            part: Occurrence,
        };
        let mut nested_budget = SuppliedMembershipBudget::new(4);
        let nested_value = OptionValue::from_admitted(decimal_type.clone(), None);
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &ValueType::option(decimal_type),
                &nested_value,
                &mut nested_arena,
                nested_at,
                &mut nested_budget,
                &cancel,
            ),
            Err(Stop::UnsupportedHelper {
                family: UnsupportedHelperFamily::DecimalDomain,
                locus: MembershipLocus {
                    path: nested_at.path,
                    part: MembershipPart::TypeLink(0)
                },
            })
        );
        assert_eq!(nested_budget.spent(), 2);
    }

    /// Two Decimal arguments have exactly two owning value visits. The
    /// declared minimum scale bounds the normalization shift without a
    /// separate helper event or per-limb charge.
    /// Trace: FR-109-AC-1, FR-109-AC-2, FR-372-AC-5
    #[trace("TC-908", "FR-109-AC-1", "FR-109-AC-2", "FR-372-AC-5")]
    #[test]
    fn bounded_decimal_uses_cumulative_exact_event_boundary() {
        let expected = ValueType::Decimal(
            DecimalType::new(
                Integer::zero(),
                Integer::from(100_i64),
                2,
                2,
                RoundingMode::Exact,
            )
            .expect("bounded Decimal declaration"),
        );
        let value = Value::Decimal(Decimal::new(Integer::one(), 0));
        let mut arena = MembershipPathArena::new().expect("arena");
        let first = MembershipLocus {
            path: arena.root(0),
            part: MembershipPart::Value,
        };
        let second = MembershipLocus {
            path: arena.root(1),
            part: MembershipPart::Value,
        };
        let cancel = Cancel::new();

        let mut exact = SuppliedMembershipBudget::new(2);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected, &value, &mut arena, first, &mut exact, &cancel, 7,
            ),
            Ok(())
        );
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected, &value, &mut arena, second, &mut exact, &cancel, 7,
            ),
            Ok(())
        );
        assert_eq!(exact.spent(), 2);

        let mut one_less = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                first,
                &mut one_less,
                &cancel,
                7,
            ),
            Ok(())
        );
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                second,
                &mut one_less,
                &cancel,
                7,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Limit {
                ceiling: 1,
                spent: 1,
                requested: 1,
                event: MembershipEvent::ValueVisit,
                locus: second,
            }))
        );
        assert_eq!(one_less.spent(), 1);
        assert_eq!(arena.resolve(second).expect("second root").argument(), 1);

        let mut zero = SuppliedMembershipBudget::new(0);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected, &value, &mut arena, first, &mut zero, &cancel, 7,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Limit {
                ceiling: 0,
                spent: 0,
                requested: 1,
                event: MembershipEvent::ValueVisit,
                locus: first,
            }))
        );
        assert_eq!(zero.spent(), 0);
    }

    /// The magnitude premise is checked before invoking the helper; a
    /// completed nonmember is still a typed payload refusal at the same root.
    /// Trace: FR-109-AC-4, FR-372-AC-5
    #[trace("TC-908", "FR-109-AC-4", "FR-372-AC-5")]
    #[test]
    fn bounded_decimal_retains_size_and_nonmembership_locus() {
        let expected = ValueType::Decimal(
            DecimalType::new(
                Integer::zero(),
                Integer::from(100_i64),
                2,
                2,
                RoundingMode::Exact,
            )
            .expect("bounded Decimal declaration"),
        );
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(5),
            part: MembershipPart::Value,
        };
        let mut size_budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &Value::Decimal(Decimal::new(Integer::one(), 0)),
                &mut arena,
                at,
                &mut size_budget,
                &Cancel::new(),
                6,
            ),
            Err(BoundedSuppliedAdmissionError::DecimalSize {
                operand: DecimalMembershipOperand::Upper,
                measured_bits: 7,
                limit_bits: 6,
                locus: at,
            })
        );
        assert_eq!(size_budget.spent(), 1);

        let mut coefficient_budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &Value::Decimal(Decimal::new(Integer::from(128_i64), 0)),
                &mut arena,
                at,
                &mut coefficient_budget,
                &Cancel::new(),
                7,
            ),
            Err(BoundedSuppliedAdmissionError::DecimalSize {
                operand: DecimalMembershipOperand::Coefficient,
                measured_bits: 8,
                limit_bits: 7,
                locus: at,
            })
        );
        assert_eq!(coefficient_budget.spent(), 1);

        let lower_first = ValueType::Decimal(
            DecimalType::new(
                Integer::from(64_i64),
                Integer::from(65_i64),
                0,
                0,
                RoundingMode::Exact,
            )
            .expect("bounded Decimal declaration"),
        );
        let mut lower_budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &lower_first,
                &Value::Decimal(Decimal::new(Integer::one(), 0)),
                &mut arena,
                at,
                &mut lower_budget,
                &Cancel::new(),
                6,
            ),
            Err(BoundedSuppliedAdmissionError::DecimalSize {
                operand: DecimalMembershipOperand::Lower,
                measured_bits: 7,
                limit_bits: 6,
                locus: at,
            })
        );
        assert_eq!(lower_budget.spent(), 1);

        let mut invalid_budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &Value::Decimal(Decimal::new(Integer::from(101_i64), 0)),
                &mut arena,
                at,
                &mut invalid_budget,
                &Cancel::new(),
                7,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Invalid {
                cause: InvalidMembershipCause::Payload,
                locus: at,
            }))
        );
        assert_eq!(invalid_budget.spent(), 1);
    }

    /// The QSV projection preserves a constructed kernel resource failure's
    /// actual native request and rooted locus. Exact TC-920 tests the kernel's
    /// production allocation and checked-capacity mechanisms separately.
    /// Trace: FR-109-AC-8, FR-372-AC-5
    #[trace("TC-908", "FR-109-AC-8", "FR-372-AC-5")]
    #[test]
    fn decimal_kernel_resource_failures_project_without_partial_admission() {
        let arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(9),
            part: MembershipPart::Value,
        };
        let capacity: Result<Option<bool>, BoundedSuppliedAdmissionError<InvalidMembershipCause>> =
            decimal_value_result(
                Err(DecimalMembershipFailure::Capacity { magnitude_bits: 65 }),
                at,
                1,
            );
        assert_eq!(
            capacity,
            Err(BoundedSuppliedAdmissionError::DecimalCapacity {
                magnitude_bits: 65,
                locus: at,
            })
        );
        let allocation: Result<
            Option<bool>,
            BoundedSuppliedAdmissionError<InvalidMembershipCause>,
        > = decimal_value_result(
            Err(DecimalMembershipFailure::Allocation {
                additional_elements: 3,
                element_width: 4,
                layout_bytes: 12,
            }),
            at,
            1,
        );
        assert_eq!(
            allocation,
            Err(BoundedSuppliedAdmissionError::DecimalAllocation {
                additional_elements: 3,
                element_width: 4,
                layout_bytes: 12,
                locus: at,
            })
        );
        assert_eq!(arena.resolve(at).expect("resource locus").argument(), 9);
    }

    /// Cancellation observed by the kernel after the owning value event
    /// retains the original handle's cause and the already-spent budget.
    /// Trace: FR-109-AC-3, FR-109-AC-8, FR-372-AC-5
    #[trace("TC-908", "FR-109-AC-3", "FR-109-AC-8", "FR-372-AC-5")]
    #[test]
    fn bounded_decimal_maps_original_handle_cancellation_during_helper() {
        use core::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::{Arc, Mutex};

        let polls = Arc::new(AtomicUsize::new(0));
        let trigger = Arc::new(Mutex::new(None::<Cancel>));
        let observed_polls = Arc::clone(&polls);
        let observed_trigger = Arc::clone(&trigger);
        let cancel = Cancel::observing(move || {
            if observed_polls.fetch_add(1, Ordering::SeqCst) + 1 == 4 {
                observed_trigger
                    .lock()
                    .expect("test cancellation lock")
                    .take()
                    .expect("test cancellation handle")
                    .cancel(CancelCause::Deadline);
            }
        });
        *trigger.lock().expect("test cancellation lock") = Some(cancel.clone());

        let expected = ValueType::Decimal(
            DecimalType::new(
                Integer::zero(),
                Integer::from(100_i64),
                2,
                2,
                RoundingMode::Exact,
            )
            .expect("bounded Decimal declaration"),
        );
        let value = Value::Decimal(Decimal::new(Integer::one(), 0));
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(8),
            part: MembershipPart::Value,
        };
        let mut budget = SuppliedMembershipBudget::new(1);
        assert_eq!(
            environment().admit_supplied_bounded_numeric::<InvalidMembershipCause>(
                &expected,
                &value,
                &mut arena,
                at,
                &mut budget,
                &cancel,
                7,
            ),
            Err(BoundedSuppliedAdmissionError::Admission(Stop::Cancelled {
                cause: CancelCause::Deadline,
                spent: 1,
                event: MembershipEvent::ValueVisit,
                locus: at,
            }))
        );
        assert_eq!(budget.spent(), 1);
        assert_eq!(cancel.tripped(), Some(CancelCause::Deadline));
        assert!(polls.load(Ordering::SeqCst) >= 4);
    }

    /// Union descriptor/payload admission remains a typed dependency, not an
    /// invalid composite, until the owning public union contract is available.
    /// Trace: FR-109-AC-7
    #[trace("TC-908", "FR-109-AC-7")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies checked composite identities without minting them in production"
    )]
    #[test]
    fn supplied_api_reports_unqualified_union_before_payload_admission() {
        let declaration = NodeKey::from_digest([10; 32]);
        let member = NodeKey::from_digest([11; 32]);
        let value = UnionValue::from_admitted(
            UnionMember::from_admitted(
                declaration,
                member,
                Identifier::new("m").expect("identifier"),
            ),
            vec![integer(1)],
        );
        let mut arena = MembershipPathArena::new().expect("arena");
        let at = MembershipLocus {
            path: arena.root(0),
            part: Occurrence,
        };
        let mut budget = SuppliedMembershipBudget::new(2);
        assert_eq!(
            environment().admit_supplied::<InvalidMembershipCause>(
                &ValueType::Composite(declaration),
                &value,
                &mut arena,
                at,
                &mut budget,
                &Cancel::new(),
            ),
            Err(Stop::UnsupportedHelper {
                family: UnsupportedHelperFamily::UnionPayloadDescriptor,
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
            let path_requests = &requests[1..];
            let request_sample = &path_requests[..path_requests.len().min(16)];
            let transition_sample = &probe.transitions[..probe.transitions.len().min(16)];
            std::println!(
                "path_arena count={count} events={} nodes={nodes} path_reservations={} first_requests={request_sample:?} last_request={:?} first_transitions={transition_sample:?} last_transition={:?} copied_bound_bytes={copied_bound} elapsed={elapsed:?}",
                budget.spent(),
                path_requests.len(),
                path_requests.last(),
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
