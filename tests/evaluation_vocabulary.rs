// SPDX-License-Identifier: AGPL-3.0-or-later
//! The catalog vocabulary and supplied-value stop records through the public API (FR-126).

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use ix_trace_rs::trace;
use quire_semantic_value::catalog::{
    CatalogCode, Category, InternalFault, UndefinedCoded, UndefinedReason, UndefinedRecord,
};
use quire_semantic_value::supplied_stop::{
    AttemptedCount, SuppliedCause, SuppliedContext, SuppliedFailure, SuppliedLimit,
    SuppliedLimitKind, SuppliedPhase, SuppliedStop,
};

fn hash_of(code: &CatalogCode) -> u64 {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    hasher.finish()
}

#[test]
#[trace("FR-126-AC-1")]
fn category_exit_codes_follow_the_declared_order() {
    let exits: Vec<u8> = Category::ALL.iter().map(|c| c.exit_code()).collect();
    assert_eq!(exits, [0, 10, 10, 20, 21, 22, 22, 30]);
    assert_eq!(Category::Inconclusive.trace_exit_code(), 0);
    assert_eq!(Category::Incomplete.trace_exit_code(), 22);
    assert_eq!(Category::most_severe([]), None);
    assert_eq!(Category::most_severe([0, 22, 30, 10]), Some(30));
}

#[test]
#[trace("FR-126-AC-2")]
fn catalog_code_identity_ignores_the_rendering_prefix() {
    let plain = CatalogCode::new("ill_typed", "x");
    let prefixed = plain.with_prefix("value");
    assert_eq!(plain, prefixed);
    assert_eq!(hash_of(&plain), hash_of(&prefixed));
    assert_ne!(plain, CatalogCode::new("ill_typed", "y"));
    assert_eq!(plain.to_string(), "ill_typed/x");
    assert_eq!(prefixed.to_string(), "value/ill_typed/x");
    assert_eq!(prefixed.prefix(), Some("value"));
}

#[test]
#[trace("FR-126-AC-3")]
fn internal_fault_is_one_code_and_one_category() {
    let fault = InternalFault::new("S6a", "broken").with_named(vec![("node", "n1".to_string())]);
    assert_eq!(
        fault.catalog_code(),
        CatalogCode::new("runtime_invariant", "established-invariant-broken")
    );
    assert_eq!(fault.category(), Category::InternalFailure);
    assert_eq!((fault.stage(), fault.invariant()), ("S6a", "broken"));
    assert_eq!(fault.named(), [("node", "n1".to_string())]);
}

#[derive(Debug)]
struct Absent(Option<&'static str>);

impl UndefinedCoded for Absent {
    fn undefined_record(&self) -> UndefinedRecord {
        UndefinedRecord {
            reason: UndefinedReason::AbsentKey,
            fields: BTreeMap::new(),
        }
    }

    fn catalog_prefix(&self) -> Option<&'static str> {
        self.0
    }
}

#[test]
#[trace("FR-126-AC-4")]
fn undefined_cause_renders_its_prefix_then_the_reason() {
    assert_eq!(Absent(None).catalog_code(), "absent-key");
    assert_eq!(Absent(Some("value")).catalog_code(), "value/absent-key");
}

#[test]
#[trace("FR-126-AC-5")]
fn supplied_stop_names_its_field_and_wraps_as_a_stop() {
    assert_eq!(SuppliedLimitKind::WorkUnits.field(), "work_units");
    assert_eq!(
        SuppliedLimitKind::ValueOccurrences.field(),
        "value_occurrences"
    );
    assert_eq!(SuppliedLimitKind::ScaleExpansion.field(), "scale_expansion");
    for (kind, phase, field) in [
        (
            SuppliedLimitKind::AdmissionWorkUnits,
            SuppliedPhase::Admission,
            "supplied.admission_work_units",
        ),
        (
            SuppliedLimitKind::ConversionWorkUnits,
            SuppliedPhase::Conversion,
            "supplied.conversion_work_units",
        ),
    ] {
        assert_eq!(kind.field(), field);
        assert_eq!(phase.field(), field);
    }
    assert_eq!(
        SuppliedLimitKind::NumericMagnitudeBits.field(),
        "supplied.numeric_magnitude_bits"
    );
    let context = SuppliedContext {
        phase: SuppliedPhase::Admission,
        parameter: Some(0),
        event: None,
        locus: None,
        consumed: 3,
    };
    let limited = SuppliedStop {
        context: context.clone(),
        cause: SuppliedCause::Limit(SuppliedLimit {
            limit_kind: SuppliedLimitKind::AdmissionWorkUnits,
            limit: 3,
            next_charge: 1,
            counter: AttemptedCount::Count(4),
            operand: None,
        }),
    };
    assert_eq!(
        limited.limit().map(SuppliedLimit::field),
        Some("supplied.admission_work_units")
    );
    let cancelled = SuppliedStop {
        context,
        cause: SuppliedCause::Cancelled(quire_exact::CancelCause::Requested),
    };
    assert!(cancelled.limit().is_none());
    assert!(matches!(
        SuppliedFailure::from(cancelled),
        SuppliedFailure::Stop(_)
    ));
}
