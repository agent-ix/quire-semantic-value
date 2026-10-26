// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public reservation control for real type-environment admission boundaries.

use std::cell::{Cell, RefCell};

use ix_trace_rs::trace;
use quire_exact::{Integer, NodeKey, OptionValue, Value, ValueType};
use quire_semantic_value::declaration::{
    CompositeDeclaration, CompositeShape, EnvironmentFailure, FieldDeclaration,
    ReservationDecision, StorageRequest, StorageUnit, TypeEnvironment, TypeEnvironmentLimits,
};

#[allow(
    clippy::disallowed_methods,
    reason = "the fixture supplies a checked declaration key without minting one in production"
)]
fn key() -> NodeKey {
    NodeKey::from_digest([17; 32])
}

fn declaration() -> CompositeDeclaration {
    CompositeDeclaration::new(
        key(),
        "Pair",
        CompositeShape::Record(vec![FieldDeclaration::new(
            "left",
            ValueType::Integer,
            quire_exact::Presence::Required,
        )]),
    )
}

/// The public callback receives the actual registry request and the chosen
/// classification is returned as the ordinary type-environment error.
#[trace("FR-108-AC-1", "QSL-1150")]
#[test]
fn constructor_reservation_controls_return_allocation_and_capacity() {
    for (decision, is_capacity) in [
        (ReservationDecision::AllocationDenied, false),
        (ReservationDecision::CapacityExceeded, true),
    ] {
        let seen = Cell::new(None);
        let result = TypeEnvironment::new_with_reservation([declaration()], [], &|request| {
            seen.set(Some(request));
            decision
        });
        let request = StorageRequest::new(1, StorageUnit::AdditionalElements);
        assert_eq!(seen.get(), Some(request));
        assert_eq!(
            result,
            Err(if is_capacity {
                EnvironmentFailure::Capacity(request)
            } else {
                EnvironmentFailure::Allocation(request)
            })
        );
    }

    assert_eq!(
        TypeEnvironment::new([declaration()], [])
            .expect("ordinary admission still reaches the allocator")
            .composites()
            .count(),
        1
    );
}

/// Membership injection reaches the live worklist reservation while checking
/// a present option; it does not substitute a prebuilt error or value.
#[trace("FR-109-AC-5", "QSL-1150")]
#[test]
fn membership_reservation_controls_return_allocation_and_capacity() {
    let environment = TypeEnvironment::new([], []).expect("empty environment admits");
    let expected = ValueType::option(ValueType::Integer);
    let value = OptionValue::present(ValueType::Integer, Value::Integer(Integer::from(3_i64)))
        .expect("the fixture option is well-typed");

    for (decision, is_capacity) in [
        (ReservationDecision::AllocationDenied, false),
        (ReservationDecision::CapacityExceeded, true),
    ] {
        let seen = Cell::new(None);
        let result = environment.admits_with_reservation(&expected, &value, &|request| {
            seen.set(Some(request));
            decision
        });
        let request = StorageRequest::new(1, StorageUnit::AdditionalElements);
        assert_eq!(seen.get(), Some(request));
        assert_eq!(
            result,
            Err(if is_capacity {
                EnvironmentFailure::Capacity(request)
            } else {
                EnvironmentFailure::Allocation(request)
            })
        );
    }

    assert!(environment
        .admits(&expected, &value)
        .expect("ordinary membership remains available"));
}

/// The bounded constructor exposes the same real reservation policy.
#[trace("QSL-1150")]
#[test]
fn bounded_constructor_uses_the_public_reservation_control() {
    let seen = Cell::new(None);
    let result = TypeEnvironment::bounded_with_reservation(
        [declaration()],
        [],
        TypeEnvironmentLimits::default(),
        &|request| {
            seen.set(Some(request));
            ReservationDecision::Permit
        },
    )
    .expect("permit reaches the allocator");
    assert_eq!(
        seen.get(),
        Some(StorageRequest::new(1, StorageUnit::AdditionalElements))
    );
    assert_eq!(result.composites().count(), 1);
}

/// The path-vector reservation preserves a capacity classification after the
/// membership walk has permitted its separate worklist reservation.
#[trace("QSL-1150")]
#[test]
fn membership_preserves_capacity_at_a_retained_path_reservation() {
    let environment = TypeEnvironment::new([], []).expect("empty environment admits");
    let expected = ValueType::option(ValueType::Integer);
    let value = OptionValue::present(ValueType::Integer, Value::Integer(Integer::from(3_i64)))
        .expect("the fixture option is well-typed");
    let seen = RefCell::new(Vec::new());
    let result = environment.admits_with_reservation(&expected, &value, &|request| {
        let mut seen = seen.borrow_mut();
        seen.push(request);
        if seen.len() == 1 {
            ReservationDecision::Permit
        } else {
            ReservationDecision::CapacityExceeded
        }
    });
    let request = StorageRequest::new(1, StorageUnit::AdditionalElements);
    assert_eq!(seen.borrow().as_slice(), [request, request]);
    assert_eq!(result, Err(EnvironmentFailure::Capacity(request)));
}
