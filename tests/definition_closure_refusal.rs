// SPDX-License-Identifier: AGPL-3.0-or-later
//! The definition-closure refusal vocabulary through the public `definition` API (TC-924).

use ix_trace_rs::trace;
use quire_semantic_value::definition::{
    PackageCause, PackageRefusal, PackageRefusalCode, SelectionRefusalCode,
};

const CAUSES: [PackageCause; 3] = [
    PackageCause::MissingMember,
    PackageCause::ConflictingDefinition,
    PackageCause::IncompatibleDefinition,
];

/// An exhaustive match with no wildcard arm: this compiles only while the enum is closed and
/// not `#[non_exhaustive]`, which this integration-test crate (not the defining one) enforces.
fn cause_index(cause: PackageCause) -> usize {
    match cause {
        PackageCause::MissingMember => 0,
        PackageCause::ConflictingDefinition => 1,
        PackageCause::IncompatibleDefinition => 2,
    }
}

/// Exhaustive over the single diagnostic code, again with no wildcard arm.
fn code_spelling(code: PackageRefusalCode) -> &'static str {
    match code {
        PackageRefusalCode::InvalidPackage => "invalid_package",
    }
}

#[test]
#[trace("TC-924", "FR-125-AC-1")]
fn spellings_match_fr_272_invalid_package_row() {
    assert_eq!(PackageCause::MissingMember.as_str(), "missing-member");
    assert_eq!(
        PackageCause::ConflictingDefinition.as_str(),
        "conflicting-definition"
    );
    assert_eq!(
        PackageCause::IncompatibleDefinition.as_str(),
        "incompatible-definition"
    );
    assert_eq!(
        PackageRefusalCode::InvalidPackage.as_str(),
        "invalid_package"
    );
}

#[test]
#[trace("TC-924", "FR-125-AC-2")]
fn invalid_package_pairs_each_cause_with_the_single_code() {
    for cause in CAUSES {
        let refusal = PackageRefusal::invalid_package(cause);
        assert_eq!(refusal.code, PackageRefusalCode::InvalidPackage);
        assert_eq!(refusal.cause, cause);
        assert_eq!(code_spelling(refusal.code), refusal.code.as_str());
    }
}

#[test]
#[trace("TC-924", "FR-125-AC-3")]
fn package_cause_is_closed_and_has_no_digest_mismatch_spelling() {
    let indices: Vec<usize> = CAUSES.into_iter().map(cause_index).collect();
    assert_eq!(indices, [0, 1, 2]);
    for cause in CAUSES {
        assert!(!matches!(
            cause.as_str(),
            "byte-digest-mismatch" | "digest-domain-mismatch" | "content-mismatch"
        ));
    }
}

#[test]
#[trace("TC-924", "FR-125-AC-4")]
fn selection_codes_are_in_check_order_and_round_trip() {
    let expected = [
        "selection_unknown_trigger",
        "selection_duplicate_trigger",
        "selection_unknown_role",
        "selection_duplicate_role",
        "selection_required_missing",
        "selection_alternative_conflict",
        "selection_trigger_unsatisfied",
        "selection_untriggered_profile",
    ];
    let spellings: Vec<&str> = SelectionRefusalCode::ALL
        .into_iter()
        .map(SelectionRefusalCode::as_str)
        .collect();
    assert_eq!(spellings, expected);
    for code in SelectionRefusalCode::ALL {
        assert_eq!(SelectionRefusalCode::from_code(code.as_str()), Some(code));
    }
    for other in [
        "invalid_package",
        "",
        "SELECTION_UNKNOWN_TRIGGER",
        "selection_unknown_trigger ",
    ] {
        assert_eq!(SelectionRefusalCode::from_code(other), None);
    }
}
