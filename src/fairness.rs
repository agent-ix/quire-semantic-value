// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared checked property-form and fairness values.

#[cfg(test)]
mod tests {
    use std::collections::{hash_map::DefaultHasher, HashSet};
    use std::hash::{Hash, Hasher};
    use std::string::String;
    use std::vec;

    use ix_trace_rs::trace;
    use quire_canonical::{to_vec, Error, Limits};
    use quire_exact::EffectiveId;

    use super::{
        FairnessConstraint, FairnessGranularity, FairnessKind, FairnessOperation, PropertyForm,
    };

    #[allow(
        clippy::disallowed_methods,
        reason = "a test fixture supplies EffectiveId; production code mints none"
    )]
    fn id(byte: u8) -> EffectiveId {
        EffectiveId::from_digest([byte; 32])
    }

    fn limits(bytes: u64) -> Limits {
        Limits::new(bytes)
    }

    fn constraint(
        kind: FairnessKind,
        id_byte: u8,
        name: &str,
        granularity: FairnessGranularity,
    ) -> FairnessConstraint {
        FairnessConstraint::new(
            kind,
            FairnessOperation::new(id(id_byte), String::from(name)),
            granularity,
        )
    }

    #[trace("TC-921", "FR-121-AC-1")]
    #[test]
    fn public_values_round_trip_every_form_kind_and_granularity() {
        let forms = [
            PropertyForm::ReachableInvariant,
            PropertyForm::BoundedMltl,
            PropertyForm::Safety,
            PropertyForm::Liveness,
        ];
        assert_eq!(forms.iter().copied().collect::<HashSet<_>>().len(), 4);
        let kinds = [FairnessKind::Weak, FairnessKind::Strong];
        let granularities = [FairnessGranularity::Whole, FairnessGranularity::Each];
        for kind in kinds {
            for granularity in granularities {
                let value = constraint(kind, 7, "tick", granularity);
                assert_eq!(value.kind(), kind);
                assert_eq!(value.granularity(), granularity);
                assert_eq!(value.operation().declaring_type(), id(7));
                assert_eq!(value.operation().name(), "tick");
            }
        }
    }

    #[trace("TC-921", "FR-121-AC-2")]
    #[test]
    fn identity_equality_and_hash_distinguish_each_fairness_field() {
        let base = constraint(FairnessKind::Weak, 0, "tick", FairnessGranularity::Whole);
        let equal = constraint(FairnessKind::Weak, 0, "tick", FairnessGranularity::Whole);
        let changed_kind = constraint(FairnessKind::Strong, 0, "tick", FairnessGranularity::Whole);
        let changed_type = constraint(FairnessKind::Weak, 1, "tick", FairnessGranularity::Whole);
        let changed_name = constraint(FairnessKind::Weak, 0, "tock", FairnessGranularity::Whole);
        let changed_granularity =
            constraint(FairnessKind::Weak, 0, "tick", FairnessGranularity::Each);
        assert_eq!(base, equal);
        let hash = |value: &FairnessConstraint| {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(hash(&base), hash(&equal));
        assert_ne!(base.operation(), changed_type.operation());
        assert_ne!(base.operation(), changed_name.operation());
        let mut set = HashSet::new();
        for value in [
            base.clone(),
            equal,
            changed_kind,
            changed_type,
            changed_name,
            changed_granularity,
        ] {
            set.insert(value);
        }
        assert_eq!(set.len(), 5);
        assert!(set.contains(&base));
    }

    #[trace("TC-921", "FR-121-AC-3")]
    #[test]
    fn total_order_follows_variants_then_declaring_type_and_name() {
        let mut forms = vec![
            PropertyForm::Liveness,
            PropertyForm::Safety,
            PropertyForm::BoundedMltl,
            PropertyForm::ReachableInvariant,
        ];
        forms.sort();
        assert_eq!(
            forms,
            [
                PropertyForm::ReachableInvariant,
                PropertyForm::BoundedMltl,
                PropertyForm::Safety,
                PropertyForm::Liveness
            ]
        );
        assert!(FairnessKind::Weak < FairnessKind::Strong);
        assert!(FairnessGranularity::Whole < FairnessGranularity::Each);
        let expected = [
            constraint(FairnessKind::Weak, 0, "alpha", FairnessGranularity::Whole),
            constraint(FairnessKind::Weak, 0, "tick", FairnessGranularity::Whole),
            constraint(FairnessKind::Weak, 1, "tick", FairnessGranularity::Whole),
            constraint(FairnessKind::Weak, 1, "tick", FairnessGranularity::Each),
            constraint(FairnessKind::Strong, 0, "tick", FairnessGranularity::Whole),
        ];
        let mut reverse = expected.to_vec();
        reverse.reverse();
        reverse.sort();
        assert_eq!(reverse, expected);
        assert!(expected[1].operation() < expected[2].operation());
        for left in &expected {
            for right in &expected {
                assert_eq!(left.cmp(right) == std::cmp::Ordering::Equal, left == right);
                assert_eq!(left.partial_cmp(right), Some(left.cmp(right)));
            }
        }
    }

    #[trace("TC-921", "FR-121-AC-4")]
    #[test]
    fn canonical_constraint_bytes_reflect_every_field_and_escape_names() {
        let base = constraint(FairnessKind::Weak, 0, "tick", FairnessGranularity::Whole);
        let expected = b"{\"granularity\":\"whole\",\"kind\":\"weak\",\"operation\":{\"declaring_type\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"name\":\"tick\"}}";
        let encoded = to_vec(&base, limits(4096)).expect("canonical constraint");
        assert_eq!(encoded, expected);
        for changed in [
            constraint(FairnessKind::Strong, 0, "tick", FairnessGranularity::Whole),
            constraint(FairnessKind::Weak, 1, "tick", FairnessGranularity::Whole),
            constraint(FairnessKind::Weak, 0, "tock", FairnessGranularity::Whole),
            constraint(FairnessKind::Weak, 0, "tick", FairnessGranularity::Each),
        ] {
            assert_ne!(
                to_vec(&changed, limits(4096)).expect("changed encoding"),
                encoded
            );
        }
        let quoted = constraint(FairnessKind::Weak, 0, "t\"ick", FairnessGranularity::Whole);
        let quoted_bytes = to_vec(&quoted, limits(4096)).expect("quoted name");
        let expected_quoted = b"{\"granularity\":\"whole\",\"kind\":\"weak\",\"operation\":{\"declaring_type\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"name\":\"t\\\"ick\"}}";
        assert_eq!(quoted_bytes, expected_quoted);
    }

    #[trace("TC-921", "FR-121-AC-5")]
    #[test]
    fn canonical_forms_operation_and_limit_have_exact_results() {
        for (form, expected) in [
            (
                PropertyForm::ReachableInvariant,
                b"\"reachable-invariant\"".as_slice(),
            ),
            (PropertyForm::BoundedMltl, b"\"bounded-mltl\"".as_slice()),
            (PropertyForm::Safety, b"\"safety\"".as_slice()),
            (PropertyForm::Liveness, b"\"liveness\"".as_slice()),
        ] {
            assert_eq!(to_vec(&form, limits(4096)).expect("form"), expected);
        }
        let value = constraint(FairnessKind::Weak, 0, "tick", FairnessGranularity::Whole);
        let operation = value.operation();
        let expected = b"{\"declaring_type\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"name\":\"tick\"}";
        assert_eq!(
            to_vec(operation, limits(4096)).expect("operation"),
            expected
        );
        let complete = to_vec(&value, limits(4096)).expect("constraint");
        let length = u64::try_from(complete.len()).expect("test output length");
        assert_eq!(
            to_vec(&value, limits(length)).expect("exact limit"),
            complete
        );
        assert!(matches!(
            to_vec(&value, limits(length - 1)),
            Err(Error::Limit(_))
        ));
    }
}
