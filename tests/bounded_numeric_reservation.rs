// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public bounded supplied admission through the production reservation path.

use std::cell::Cell;

use ix_trace_rs::trace;
use quire_exact::{
    Cancel, Decimal, DecimalType, Integer, OptionValue, RoundingMode, Value, ValueType,
};
use quire_semantic_value::declaration::{StorageRequest, StorageUnit, TypeEnvironment};
use quire_semantic_value::supplied_membership::{
    BoundedSuppliedAdmissionError, InvalidMembershipCause, MembershipLocus, MembershipPart,
    MembershipPathArena, PathStep, SuppliedAdmissionError, SuppliedMembershipBudget,
};

/// Trace: FR-109-AC-5
#[trace("TC-908", "FR-109-AC-5")]
#[test]
fn bounded_decimal_worklist_denial_retains_request_and_retry_succeeds() {
    let environment = TypeEnvironment::new([], []).expect("empty environment admits");
    let domain = DecimalType::new(
        Integer::from(0_i64),
        Integer::from(9_i64),
        0,
        0,
        RoundingMode::Exact,
    )
    .expect("bounded Decimal declaration");
    let expected = ValueType::option(ValueType::Decimal(domain.clone()));
    let value = OptionValue::present(
        ValueType::Decimal(domain),
        Value::Decimal(Decimal::new(Integer::from(3_i64), 0)),
    )
    .expect("matching Option payload");
    let cancel = Cancel::new();
    let mut arena = MembershipPathArena::new().expect("arena");
    let at = MembershipLocus {
        path: arena.root(4),
        part: MembershipPart::Value,
    };
    let mut budget = SuppliedMembershipBudget::new(4);
    let observed = Cell::new(None);
    let denied = environment
        .admit_supplied_bounded_numeric_with_reservation::<InvalidMembershipCause>(
            &expected,
            &value,
            &mut arena,
            at,
            &mut budget,
            &cancel,
            4,
            &|request| {
                observed.set(Some(request));
                false
            },
        );
    let Err(BoundedSuppliedAdmissionError::Admission(SuppliedAdmissionError::Allocation {
        request,
        locus,
    })) = denied
    else {
        panic!("expected measured bounded-worklist denial: {denied:?}");
    };
    assert_eq!(observed.get(), Some(request));
    assert_eq!(
        request,
        StorageRequest::new(1, StorageUnit::AdditionalElements)
    );
    let resolved = arena.resolve(locus).expect("rooted denial locus");
    assert_eq!(resolved.argument(), 4);
    assert_eq!(
        resolved.part(),
        MembershipPart::Child(PathStep::OptionPayload)
    );
    assert_eq!(resolved.steps_from_leaf().count(), 0);
    assert_eq!(budget.spent(), 2);

    let mut retry_arena = MembershipPathArena::new().expect("retry arena");
    let retry_at = MembershipLocus {
        path: retry_arena.root(4),
        part: MembershipPart::Value,
    };
    let mut retry_budget = SuppliedMembershipBudget::new(4);
    let allowed_requests = Cell::new(0);
    assert_eq!(
        environment.admit_supplied_bounded_numeric_with_reservation::<InvalidMembershipCause>(
            &expected,
            &value,
            &mut retry_arena,
            retry_at,
            &mut retry_budget,
            &cancel,
            4,
            &|request| {
                assert_eq!(
                    request,
                    StorageRequest::new(1, StorageUnit::AdditionalElements)
                );
                allowed_requests.set(allowed_requests.get() + 1);
                true
            },
        ),
        Ok(())
    );
    assert_eq!(allowed_requests.get(), 2);
    assert_eq!(retry_budget.spent(), 4);
}
