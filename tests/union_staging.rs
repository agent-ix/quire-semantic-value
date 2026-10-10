// SPDX-License-Identifier: AGPL-3.0-or-later
//! Resolved topology and verified sealing through the real shared registry.
//! Fixture keys are producer-supplied identities, not a digest-minting oracle.

use ix_trace_rs::trace;
use quire_exact::{
    Cancel, CollectionKind, CollectionType, EffectiveId, FieldValue, FloatType, Identifier,
    IeeeWidth, IllTypedCause, Integer, Meter, NodeKey, ObjectId, ObjectReference, OptionValue,
    Outcome, Presence, ScalarLimits, UnionMember, UnionValue, UniverseId, Value, ValueType,
    VariantId,
};
use quire_semantic_value::declaration::{
    CompositeDeclaration, CompositeShape, DeclarationCause, EnvironmentFailure,
    EnvironmentLimitKind, EqualityOperand, EqualityOperator, FieldDeclaration,
    ObjectTypeDeclaration, RecursionEdges, TypeEnvironment, UnionMemberDeclaration, WorkBudget,
};
use quire_semantic_value::enumeration::EnumMemberIndex;
use quire_semantic_value::object_closure::{ObjectClosure, ObjectClosureCause};

#[allow(
    clippy::disallowed_methods,
    reason = "fixture supplies producer identities; production mints none"
)]
fn key(n: u64) -> NodeKey {
    let mut bytes = [0; 32];
    bytes[..8].copy_from_slice(&n.to_be_bytes());
    NodeKey::from_digest(bytes)
}

fn variant(n: u64) -> VariantId {
    VariantId::from_digest(*key(n).as_bytes())
}

fn resolved(name: &str, positions: Vec<ValueType>) -> UnionMemberDeclaration {
    UnionMemberDeclaration::resolved(Identifier::new(name).unwrap(), positions)
}

fn binding(final_key: NodeKey, n: u64, name: &str) -> UnionMember {
    UnionMember::from_admitted(final_key, key(n), Identifier::new(name).unwrap())
}

fn shape() -> CompositeDeclaration {
    CompositeDeclaration::new(
        key(1),
        "Shape",
        CompositeShape::Union(vec![
            resolved("Empty", vec![]),
            resolved("Rect", vec![ValueType::Integer, ValueType::Integer]),
        ]),
    )
}

fn bindings(final_key: NodeKey) -> Vec<UnionMember> {
    vec![
        binding(final_key, 10, "Empty"),
        binding(final_key, 11, "Rect"),
    ]
}

fn integer(n: i64) -> Value {
    Value::Integer(Integer::from(n))
}

/// Trace: FR-321-AC-4
#[trace("FR-321-AC-4")]
#[test]
fn topology_sealing_and_member_search_share_a_cumulative_budget() {
    let declaration = || {
        CompositeDeclaration::new(
            key(1),
            "Tree",
            CompositeShape::Union(vec![
                resolved("End", vec![]),
                resolved(
                    "Next",
                    vec![ValueType::option(ValueType::Composite(key(1)))],
                ),
            ]),
        )
    };
    // Registration 4; member/type checking 6; two recursion walks 8 each.
    let mut short = WorkBudget::new(25, Cancel::new());
    let Err(EnvironmentFailure::Limit(limit)) =
        TypeEnvironment::bounded_with_budget([declaration()], [], 0, &mut short)
    else {
        panic!("topology N-1 must stop");
    };
    assert_eq!(limit.configured_bound(), 25);
    assert_eq!(limit.actual(), 26);
    let mut work = WorkBudget::new(30, Cancel::new());
    let mut env = TypeEnvironment::bounded_with_budget([declaration()], [], 0, &mut work).unwrap();
    assert_eq!(work.spent(), 26);
    env.seal_union_verified_with_budget(
        key(1),
        key(40),
        vec![binding(key(40), 10, "End"), binding(key(40), 11, "Next")],
        &mut work,
    )
    .unwrap();
    assert_eq!(work.spent(), 28);
    assert_eq!(
        env.union_member_named_with_budget(key(40), "Next", &mut work)
            .unwrap()
            .unwrap()
            .0,
        1
    );
    assert_eq!(work.spent(), 30);
    let limit = env
        .union_member_named_with_budget(key(1), "End", &mut work)
        .unwrap_err();
    assert_eq!(limit.configured_bound(), 30);
    assert_eq!(limit.actual(), 31);
    assert_eq!(work.spent(), 30);
}

/// Trace: FR-323-AC-3, FR-321-AC-4
#[trace("FR-323-AC-3", "FR-321-AC-4")]
#[test]
fn transitive_ieee_and_nested_type_walks_preserve_named_stops() {
    let mut env = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "Tree",
            CompositeShape::Union(vec![
                resolved("End", vec![]),
                resolved(
                    "Next",
                    vec![ValueType::option(ValueType::Composite(key(1)))],
                ),
            ]),
        )],
        [],
    )
    .unwrap();
    let mut short = WorkBudget::new(5, Cancel::new());
    let limit = env
        .contains_ieee_with_budget(&ValueType::Composite(key(1)), &mut short)
        .unwrap_err();
    assert_eq!(limit.configured_bound(), 5);
    assert_eq!(limit.actual(), 6);
    let mut exact = WorkBudget::new(6, Cancel::new());
    assert_eq!(
        env.contains_ieee_with_budget(&ValueType::Composite(key(1)), &mut exact),
        Ok(false)
    );
    assert_eq!(exact.spent(), 6);
    let set = ValueType::collection(CollectionType::new(
        CollectionKind::Set,
        ValueType::Composite(key(1)),
        None,
    ));
    assert!(env
        .check_type_with_budget(&set, &mut WorkBudget::new(7, Cancel::new()))
        .is_err());
    assert_eq!(
        env.check_type_with_budget(&set, &mut WorkBudget::new(8, Cancel::new())),
        Ok(Ok(()))
    );
    let internal = ValueType::option(ValueType::option(ValueType::Composite(key(1))));
    assert_eq!(
        env.runtime_type_with_budget(&internal, &mut WorkBudget::new(3, Cancel::new())),
        Ok(None)
    );
    env.seal_union_verified(
        key(1),
        key(40),
        vec![binding(key(40), 10, "End"), binding(key(40), 11, "Next")],
    )
    .unwrap();
    let final_type = ValueType::option(ValueType::option(ValueType::Composite(key(40))));
    let limit = env
        .runtime_type_with_budget(&internal, &mut WorkBudget::new(4, Cancel::new()))
        .unwrap_err();
    assert_eq!(limit.actual(), 5);
    assert_eq!(
        env.runtime_type_with_budget(&internal, &mut WorkBudget::new(5, Cancel::new())),
        Ok(Some(final_type.clone()))
    );
    assert_eq!(
        env.same_type_with_budget(
            &internal,
            &final_type,
            &mut WorkBudget::new(3, Cancel::new())
        ),
        Ok(true)
    );
    assert_eq!(
        env.same_type_with_budget(
            &internal,
            &final_type,
            &mut WorkBudget::new(2, Cancel::new())
        )
        .unwrap_err()
        .actual(),
        3
    );
    let cancelled = Cancel::new();
    cancelled.cancel(quire_exact::CancelCause::Requested);
    assert!(env
        .contains_ieee_with_budget(
            &ValueType::Composite(key(1)),
            &mut WorkBudget::new(100, cancelled.clone())
        )
        .is_err());
    assert!(env
        .runtime_type_with_budget(&internal, &mut WorkBudget::new(100, cancelled.clone()))
        .is_err());
    assert!(env
        .same_type_with_budget(
            &internal,
            &final_type,
            &mut WorkBudget::new(100, cancelled.clone())
        )
        .is_err());
    assert!(env
        .union_member_named_with_budget(key(1), "End", &mut WorkBudget::new(100, cancelled.clone()))
        .is_err());
    assert_eq!(cancelled.cause(), Some(quire_exact::CancelCause::Requested));
}

/// Trace: FR-321-AC-2, FR-321-AC-4
#[trace("FR-321-AC-2", "FR-321-AC-4")]
#[allow(
    clippy::disallowed_methods,
    reason = "fixture supplies object identity; production mints none"
)]
#[test]
fn closure_name_matching_admission_and_reference_walk_use_one_budget() {
    let object_type = EffectiveId::from_digest([1; 32]);
    let mut env = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "Holder",
            CompositeShape::Union(vec![resolved(
                "Some",
                vec![ValueType::Reference(object_type)],
            )]),
        )],
        [ObjectTypeDeclaration::new(
            object_type,
            "Object",
            vec![FieldDeclaration::new(
                "holder",
                ValueType::Composite(key(1)),
                Presence::Required,
            )],
        )],
    )
    .unwrap();
    env.seal_union_verified(key(1), key(40), vec![binding(key(40), 10, "Some")])
        .unwrap();
    let reference = |name| {
        ObjectReference::new(
            UniverseId::from_digest([9; 32]),
            object_type,
            ObjectId::new(name).unwrap(),
        )
    };
    let owner = reference("owner");
    let objects = |target: ObjectReference| {
        vec![(
            owner.clone(),
            vec![(
                "holder",
                FieldValue::Present(
                    env.union(key(1), variant(10), vec![Value::Reference(target)])
                        .unwrap(),
                ),
            )],
        )]
    };
    // Object 1; field matching 2; slot 1; admission 5; closure 5 =14.
    let mut short = WorkBudget::new(13, Cancel::new());
    let limit =
        ObjectClosure::new_with_budget(&env, objects(owner.clone()), &[], &mut short).unwrap_err();
    assert_eq!(limit.configured_bound(), 13);
    assert_eq!(limit.actual(), 14);
    let mut exact = WorkBudget::new(14, Cancel::new());
    let closure = ObjectClosure::new_with_budget(&env, objects(owner.clone()), &[], &mut exact)
        .unwrap()
        .unwrap();
    assert!(closure.contains(&owner));
    assert_eq!(exact.spent(), 14);
    let missing = reference("missing");
    let refusal = ObjectClosure::new_with_budget(
        &env,
        objects(missing.clone()),
        &[],
        &mut WorkBudget::new(14, Cancel::new()),
    )
    .unwrap()
    .unwrap_err();
    assert_eq!(*refusal.object, owner);
    assert_eq!(
        refusal.cause,
        ObjectClosureCause::DanglingReference(Box::new(missing))
    );
    let cancel = Cancel::new();
    cancel.cancel(quire_exact::CancelCause::Requested);
    assert!(ObjectClosure::new_with_budget(
        &env,
        objects(owner.clone()),
        &[],
        &mut WorkBudget::new(100, cancel)
    )
    .is_err());
}

fn meter() -> Meter {
    Meter::new(ScalarLimits {
        integer_bits: u64::MAX,
        decimal_digits: u64::MAX,
        scale_expansion: u64::MAX,
        text_input_bytes: u64::MAX,
        text_scalars: u64::MAX,
        normalized_scalars: u64::MAX,
        unit_edges: u64::MAX,
        value_occurrences: u64::MAX,
        work_units: u64::MAX,
        result_units: u64::MAX,
    })
}

/// Trace: FR-321-AC-1, FR-321-AC-4
#[trace("FR-321-AC-1", "FR-321-AC-4")]
#[test]
fn bounded_construction_keeps_work_stops_separate_from_prior_runtime_stop() {
    let mut env = TypeEnvironment::new([shape()], []).unwrap();
    env.seal_union_verified(key(1), key(40), bindings(key(40))).unwrap();
    // Member lookup1 + retained positions2 + two type/value admissions2 each.
    let limit = env.union_with_budget(key(1), variant(11), vec![integer(2), integer(3)], &mut WorkBudget::new(6, Cancel::new())).unwrap_err();
    assert_eq!(limit.configured_bound(), 6);
    assert_eq!(limit.actual(), 7);
    let mut work = WorkBudget::new(7, Cancel::new());
    let value = env.union_with_budget(key(40), variant(11), vec![integer(2), integer(3)], &mut work).unwrap().unwrap();
    assert_eq!(work.spent(), 7);
    assert!(env.admits(&ValueType::Composite(key(1)), &value));
    let wrong = env.union_with_budget(key(1), variant(11), vec![integer(2)], &mut WorkBudget::new(0, Cancel::new())).unwrap().unwrap_err();
    assert_eq!(wrong.cause, quire_semantic_value::declaration::ConstructionCause::WrongArity { declared: 2, supplied: 1 });
    let calls = std::cell::Cell::new(0);
    let cancel = Cancel::new();
    let mut work = WorkBudget::new(3, cancel.clone());
    let result = env.evaluate_union_with_budget(key(1), variant(11), vec![
        Box::new(|_| { calls.set(calls.get() + 1); cancel.cancel(quire_exact::CancelCause::Requested); Outcome::Undefined(quire_exact::Undefined::DivisionByZero) }),
        Box::new(|_| { calls.set(calls.get() + 1); Outcome::Completed(integer(3)) }),
    ], &mut meter(), &mut work).unwrap().unwrap();
    assert!(matches!(result, Outcome::Undefined(quire_exact::Undefined::DivisionByZero)));
    assert_eq!(calls.get(), 1);
    assert_eq!(work.spent(), 3);
    assert_eq!(cancel.cause(), Some(quire_exact::CancelCause::Requested));
}

/// Trace: FR-321-AC-1, FR-321-AC-4
#[trace("FR-321-AC-1", "FR-321-AC-4")]
#[test]
fn bounded_equality_uses_shared_type_ieee_and_final_join_walks() {
    let mut env = TypeEnvironment::new([shape()], []).unwrap();
    env.seal_union_verified(key(1), key(40), bindings(key(40))).unwrap();
    let operands = || (EqualityOperand::typed(ValueType::Composite(key(1))), EqualityOperand::typed(ValueType::Composite(key(40))));
    // Two type checks1 each, two runtime joins1 each, two IEEE walks7 each,
    // and one same-type joined leaf =19, independent of runtime value events.
    let (left, right) = operands();
    let limit = env.check_equality_with_budget(EqualityOperator::Equal, left, right, &EnumMemberIndex::default(), &mut WorkBudget::new(18, Cancel::new())).unwrap_err();
    assert_eq!(limit.configured_bound(), 18);
    assert_eq!(limit.actual(), 19);
    let (left, right) = operands();
    let mut work = WorkBudget::new(19, Cancel::new());
    let equality = env.check_equality_with_budget(EqualityOperator::Equal, left, right, &EnumMemberIndex::default(), &mut work).unwrap().unwrap();
    assert_eq!(work.spent(), 19);
    let a = env.union(key(1), variant(10), vec![]).unwrap();
    let b = env.union(key(40), variant(10), vec![]).unwrap();
    assert!(matches!(equality.evaluate(&a, &b, &mut meter()), Outcome::Completed(true)));
    let refusal = env.check_equality_with_budget(EqualityOperator::Equal,
        EqualityOperand::typed(ValueType::Composite(key(99))), EqualityOperand::typed(ValueType::Composite(key(1))),
        &EnumMemberIndex::default(), &mut WorkBudget::new(19, Cancel::new())).unwrap().unwrap_err();
    assert_eq!(refusal.cause, IllTypedCause::TypeMismatch);
    let cancel = Cancel::new();
    cancel.cancel(quire_exact::CancelCause::Requested);
    let (left, right) = operands();
    assert!(env.check_equality_with_budget(EqualityOperator::Equal, left, right, &EnumMemberIndex::default(), &mut WorkBudget::new(19, cancel)).is_err());
}

/// Cancellation robustness of the public descriptor API; this empty fixture
/// does not claim source grammar admits an empty authored union declaration.
#[test]
fn zero_member_seal_polls_cancellation_without_attaching_a_final_key() {
    let mut env = TypeEnvironment::new([CompositeDeclaration::new(key(1), "EmptyFixture", CompositeShape::Union(vec![]))], []).unwrap();
    let original = env.clone();
    let cancel = Cancel::new();
    cancel.cancel(quire_exact::CancelCause::Requested);
    let Err(EnvironmentFailure::Limit(limit)) = env.seal_union_verified_with_budget(key(1), key(40), vec![], &mut WorkBudget::new(0, cancel)) else { panic!("zero-member seal must poll cancellation"); };
    assert_eq!(limit.configured_bound(), 0);
    assert_eq!(limit.actual(), 0);
    assert_eq!(env, original);
    assert_eq!(env.union_key(key(1)), None);
}

/// Trace: FR-321-AC-1
#[trace("FR-321-AC-1")]
#[test]
fn resolved_topology_is_visible_and_runtime_union_is_denied_until_sealed() {
    let mut env = TypeEnvironment::new([shape()], []).unwrap();
    let (position, rect) = env.union_member_named(key(1), "Rect").unwrap();
    assert_eq!(position, 1);
    assert_eq!(rect.identifier().as_str(), "Rect");
    assert_eq!(rect.positions(), &[ValueType::Integer, ValueType::Integer]);
    assert!(rect.member().is_none());
    assert_eq!(env.union_key(key(1)), None);
    assert_eq!(env.runtime_type(&ValueType::Composite(key(1))), None);
    assert!(env.union_member(key(1), variant(10)).is_none());
    assert_eq!(
        env.check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(ValueType::Composite(key(1))),
            EqualityOperand::typed(ValueType::Composite(key(1))),
            &EnumMemberIndex::default()
        )
        .unwrap_err()
        .cause,
        IllTypedCause::TypeMismatch
    );
    assert!(env.union(key(1), variant(10), vec![]).is_err());
    let supplied = UnionValue::from_admitted(binding(key(40), 10, "Empty"), vec![]);
    assert!(!env.admits(&ValueType::Composite(key(1)), &supplied));
    let calls = std::cell::Cell::new(0);
    assert!(env
        .evaluate_union(
            key(1),
            variant(11),
            vec![
                Box::new(|_| {
                    calls.set(calls.get() + 1);
                    Outcome::Completed(integer(2))
                }),
                Box::new(|_| {
                    calls.set(calls.get() + 1);
                    Outcome::Completed(integer(3))
                }),
            ],
            &mut meter()
        )
        .is_err());
    assert_eq!(calls.get(), 0);
    env.seal_union_verified(key(1), key(40), bindings(key(40)))
        .unwrap();
    assert_eq!(env.union_key(key(1)), Some(key(40)));
    assert_eq!(env.union_handle(key(40)), Some(key(1)));
    assert_eq!(env.composite(key(40)).unwrap().key(), key(1));
    let (position, rect) = env.union_member(key(40), variant(11)).unwrap();
    assert_eq!(position, 1);
    assert_eq!(rect.member().unwrap().declaration(), key(40));
    assert_eq!(rect.member().unwrap().variant(), variant(11));
    assert!(env.admits(&ValueType::Composite(key(1)), &supplied));
    assert!(env.admits(&ValueType::Composite(key(40)), &supplied));
    let value = env
        .union(key(1), variant(11), vec![integer(2), integer(3)])
        .unwrap();
    let Value::Union(value) = value else {
        panic!("sealed union carrier required");
    };
    assert_eq!(value.declaration(), key(40));
    assert_eq!(value.payload().len(), 2);
    let wrong_handle_value = UnionValue::from_admitted(binding(key(1), 10, "Empty"), vec![]);
    assert!(!env.admits(&ValueType::Composite(key(1)), &wrong_handle_value));
}

/// Trace: FR-321-AC-1
#[trace("FR-321-AC-1")]
#[test]
fn invalid_seals_leave_every_descriptor_and_join_unchanged() {
    let original = TypeEnvironment::new([shape()], []).unwrap();
    for (members, cause) in [
        (
            vec![binding(key(40), 10, "Empty")],
            DeclarationCause::Type(IllTypedCause::TypeMismatch),
        ),
        (
            vec![binding(key(40), 10, "Rect"), binding(key(40), 11, "Empty")],
            DeclarationCause::Type(IllTypedCause::TypeMismatch),
        ),
        (
            vec![binding(key(40), 10, "Empty"), binding(key(41), 11, "Rect")],
            DeclarationCause::Type(IllTypedCause::TypeMismatch),
        ),
        (
            vec![binding(key(40), 10, "Empty"), binding(key(40), 10, "Rect")],
            DeclarationCause::DuplicateKey,
        ),
    ] {
        let mut env = original.clone();
        let Err(EnvironmentFailure::Refused(actual)) =
            env.seal_union_verified(key(1), key(40), members)
        else {
            panic!("invalid seal must refuse");
        };
        assert_eq!(actual.declaration, "Shape");
        assert_eq!(actual.cause, cause);
        assert_eq!(env, original);
    }
    let mut env = original;
    env.seal_union_verified(key(1), key(40), bindings(key(40)))
        .unwrap();
    let sealed = env.clone();
    let Err(EnvironmentFailure::Refused(actual)) =
        env.seal_union_verified(key(1), key(41), bindings(key(41)))
    else {
        panic!("resealing must refuse");
    };
    assert_eq!(actual.cause, DeclarationCause::DuplicateKey);
    assert_eq!(env, sealed);
}

/// Trace: FR-321-AC-1
#[trace("FR-321-AC-1")]
#[test]
fn final_key_collisions_cannot_alias_records_or_another_union() {
    let mut env = TypeEnvironment::new(
        [
            shape(),
            CompositeDeclaration::new(
                key(2),
                "Other",
                CompositeShape::Union(vec![resolved("Empty", vec![])]),
            ),
            CompositeDeclaration::new(key(40), "Record", CompositeShape::Record(vec![])),
        ],
        [],
    )
    .unwrap();
    let original = env.clone();
    let Err(EnvironmentFailure::Refused(actual)) =
        env.seal_union_verified(key(1), key(40), bindings(key(40)))
    else {
        panic!("record key collision must refuse");
    };
    assert_eq!(actual.cause, DeclarationCause::DuplicateKey);
    assert_eq!(env, original);
    env.seal_union_verified(key(1), key(41), bindings(key(41)))
        .unwrap();
    let sealed = env.clone();
    let Err(EnvironmentFailure::Refused(actual)) =
        env.seal_union_verified(key(2), key(41), vec![binding(key(41), 20, "Empty")])
    else {
        panic!("another union cannot acquire the same final key");
    };
    assert_eq!(actual.cause, DeclarationCause::DuplicateKey);
    assert_eq!(env, sealed);
    let record = env.record(key(40), vec![]).unwrap();
    assert!(env.admits(&ValueType::Composite(key(40)), &record));
    assert_eq!(
        env.runtime_type(&ValueType::Composite(key(40))),
        Some(ValueType::Composite(key(40)))
    );
}

/// Trace: FR-323-AC-3, FR-321-AC-4
#[trace("FR-323-AC-3", "FR-321-AC-4")]
#[test]
fn ieee_and_named_escape_checks_run_before_any_binding_exists() {
    let env = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "Tree",
            CompositeShape::Union(vec![
                resolved(
                    "Leaf",
                    vec![ValueType::Float(FloatType::exact(IeeeWidth::Binary64))],
                ),
                resolved(
                    "Node",
                    vec![ValueType::option(ValueType::Composite(key(1)))],
                ),
            ]),
        )],
        [],
    )
    .unwrap();
    assert_eq!(env.union_key(key(1)), None);
    assert!(env.contains_ieee(&ValueType::Composite(key(1))));
    let set = ValueType::collection(CollectionType::new(
        CollectionKind::Set,
        ValueType::Composite(key(1)),
        None,
    ));
    assert_eq!(
        env.check_type(&set).unwrap_err().cause,
        IllTypedCause::OperatorIneligible
    );
    let sequence = ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Composite(key(1)),
        None,
    ));
    assert_eq!(env.check_type(&sequence), Ok(()));
    let Err(EnvironmentFailure::Refused(actual)) = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "Loop",
            CompositeShape::Union(vec![resolved("Next", vec![ValueType::Composite(key(1))])]),
        )],
        [],
    ) else {
        panic!("non-escaping resolved topology must refuse");
    };
    assert_eq!(actual.declaration, "Loop");
    assert_eq!(
        actual.cause,
        DeclarationCause::Recursion {
            edges: RecursionEdges::NonEscaping,
            cycle: vec!["Loop".into(), "Loop".into()],
        }
    );
}

/// Trace: FR-321-AC-4
#[trace("FR-321-AC-4")]
#[test]
fn seal_work_limits_are_exact_and_do_not_attach_a_prefix() {
    let original = TypeEnvironment::new([shape()], []).unwrap();
    let mut env = original.clone();
    let Err(EnvironmentFailure::Limit(limit)) =
        env.seal_union_verified_with_cancel(key(1), key(40), bindings(key(40)), 1, &Cancel::new())
    else {
        panic!("N minus one must deny sealing");
    };
    assert_eq!(limit.kind(), EnvironmentLimitKind::WorkUnits);
    assert_eq!(limit.configured_bound(), 1);
    assert_eq!(limit.actual(), 2);
    assert_eq!(env, original);
    let cancelled = Cancel::new();
    cancelled.cancel(quire_exact::CancelCause::Requested);
    let Err(EnvironmentFailure::Limit(_)) =
        env.seal_union_verified_with_cancel(key(1), key(40), bindings(key(40)), 2, &cancelled)
    else {
        panic!("cancellation must leave the registry unresolved");
    };
    assert_eq!(env, original);
    assert_eq!(cancelled.cause(), Some(quire_exact::CancelCause::Requested));
    env.seal_union_verified_with_cancel(key(1), key(40), bindings(key(40)), 2, &Cancel::new())
        .unwrap();
    assert_eq!(env.union_key(key(1)), Some(key(40)));
    assert_eq!(env.union_member(key(1), variant(11)).unwrap().0, 1);
}

/// Trace: FR-321-AC-4
#[trace("FR-321-AC-4")]
#[test]
fn recursive_handle_final_key_joins_admit_payloads_and_checked_equality() {
    let declaration = |handle, name, other| {
        CompositeDeclaration::new(
            key(handle),
            name,
            CompositeShape::Union(vec![
                resolved("End", vec![]),
                resolved(
                    "Next",
                    vec![ValueType::option(ValueType::Composite(key(other)))],
                ),
            ]),
        )
    };
    let mut env =
        TypeEnvironment::new([declaration(1, "A", 2), declaration(2, "B", 1)], []).unwrap();
    env.seal_union_verified(
        key(1),
        key(40),
        vec![binding(key(40), 10, "End"), binding(key(40), 11, "Next")],
    )
    .unwrap();
    assert_eq!(
        env.runtime_type(&ValueType::option(ValueType::Composite(key(2)))),
        None
    );
    env.seal_union_verified(
        key(2),
        key(41),
        vec![binding(key(41), 20, "End"), binding(key(41), 21, "Next")],
    )
    .unwrap();
    let end = env.union(key(2), variant(20), vec![]).unwrap();
    let child = OptionValue::present(ValueType::Composite(key(41)), end).unwrap();
    let value = env.union(key(1), variant(11), vec![child]).unwrap();
    assert!(env.admits(&ValueType::Composite(key(1)), &value));
    assert!(env.admits(&ValueType::Composite(key(40)), &value));
    assert_eq!(
        env.runtime_type(&ValueType::option(ValueType::Composite(key(2)))),
        Some(ValueType::option(ValueType::Composite(key(41))))
    );
    let equality = env
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(ValueType::Composite(key(1))),
            EqualityOperand::typed(ValueType::Composite(key(40))),
            &EnumMemberIndex::default(),
        )
        .unwrap();
    let independent_end = env.union(key(41), variant(20), vec![]).unwrap();
    let independent = env
        .union(
            key(40),
            variant(11),
            vec![OptionValue::present(ValueType::Composite(key(41)), independent_end).unwrap()],
        )
        .unwrap();
    assert!(matches!(
        equality.evaluate(&value, &independent, &mut meter()),
        Outcome::Completed(true)
    ));
    let expected_collection = ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::option(ValueType::Composite(key(1))),
        None,
    ));
    let actual_collection = CollectionType::new(
        CollectionKind::Sequence,
        ValueType::option(ValueType::Composite(key(40))),
        None,
    );
    assert_eq!(
        env.runtime_type(&expected_collection),
        Some(ValueType::collection(actual_collection.clone()))
    );
    let contained = OptionValue::present(ValueType::Composite(key(40)), value.clone()).unwrap();
    let collection = quire_exact::from_admitted(actual_collection, vec![contained]);
    assert!(env.admits(&expected_collection, &collection));
    let wrong = UnionValue::from_admitted(
        binding(key(1), 11, "Next"),
        vec![OptionValue::none(ValueType::Composite(key(2)))],
    );
    assert!(!env.admits(&ValueType::Composite(key(1)), &wrong));
}

/// Trace: FR-321-AC-4
#[trace("FR-321-AC-4")]
#[test]
fn ten_thousand_supplied_links_follow_sealed_keys_with_internal_position_types() {
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let handle_type = ValueType::Composite(key(1));
            let final_type = ValueType::Composite(key(40));
            let mut env = TypeEnvironment::new(
                [CompositeDeclaration::new(
                    key(1),
                    "Tree",
                    CompositeShape::Union(vec![
                        resolved("End", vec![]),
                        resolved("Next", vec![ValueType::option(handle_type.clone())]),
                    ]),
                )],
                [],
            )
            .unwrap();
            env.seal_union_verified(
                key(1),
                key(40),
                vec![binding(key(40), 10, "End"), binding(key(40), 11, "Next")],
            )
            .unwrap();
            let end = env
                .union_member(key(1), variant(10))
                .unwrap()
                .1
                .member()
                .unwrap()
                .clone();
            let next = env
                .union_member(key(40), variant(11))
                .unwrap()
                .1
                .member()
                .unwrap()
                .clone();
            let supplied = || {
                let mut value = UnionValue::from_admitted(end.clone(), vec![]);
                for _ in 0..10_000 {
                    value = UnionValue::from_admitted(
                        next.clone(),
                        vec![OptionValue::from_admitted(final_type.clone(), Some(value))],
                    );
                }
                value
            };
            let left = supplied();
            let right = supplied();
            assert_eq!(left.occ(), Integer::from(20_001_u64));
            // Work includes type validation, payload scheduling and option
            // type comparison, not just kernel occurrence count.
            assert_eq!(env.admits_bounded(&handle_type, &left, 50_002), Ok(true));
            assert_eq!(env.admits_bounded(&final_type, &left, 50_002), Ok(true));
            let limit = env.admits_bounded(&handle_type, &left, 50_001).unwrap_err();
            assert_eq!(limit.kind(), EnvironmentLimitKind::WorkUnits);
            assert_eq!(limit.configured_bound(), 50_001);
            assert_eq!(limit.actual(), 50_002);
            let equality = env
                .check_equality(
                    EqualityOperator::Equal,
                    EqualityOperand::typed(handle_type),
                    EqualityOperand::typed(final_type),
                    &EnumMemberIndex::default(),
                )
                .unwrap();
            assert!(matches!(
                equality.evaluate(&left, &right, &mut meter()),
                Outcome::Completed(true)
            ));
            assert_eq!(
                quire_exact::compare_keys(&left, &right),
                Some(std::cmp::Ordering::Equal)
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Trace: FR-321-AC-2
#[trace("FR-321-AC-2")]
#[allow(
    clippy::disallowed_methods,
    reason = "fixture supplies object identity; production mints none"
)]
#[test]
fn sealed_reference_positions_validate_conformance_and_closure() {
    let object_type = EffectiveId::from_digest([1; 32]);
    let mut env = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "Holder",
            CompositeShape::Union(vec![resolved(
                "Some",
                vec![ValueType::Reference(object_type)],
            )]),
        )],
        [ObjectTypeDeclaration::new(
            object_type,
            "Object",
            vec![FieldDeclaration::new(
                "holder",
                ValueType::Composite(key(1)),
                Presence::Required,
            )],
        )],
    )
    .unwrap();
    env.seal_union_verified(key(1), key(40), vec![binding(key(40), 10, "Some")])
        .unwrap();
    let reference = |name| {
        ObjectReference::new(
            UniverseId::from_digest([9; 32]),
            object_type,
            ObjectId::new(name).unwrap(),
        )
    };
    let owner = reference("owner");
    let target = reference("target");
    let owner_value = env
        .union(key(1), variant(10), vec![Value::Reference(target.clone())])
        .unwrap();
    let target_value = env
        .union(key(40), variant(10), vec![Value::Reference(owner.clone())])
        .unwrap();
    let closure = ObjectClosure::new(
        &env,
        [
            (
                owner.clone(),
                vec![("holder", FieldValue::Present(owner_value.clone()))],
            ),
            (
                target.clone(),
                vec![("holder", FieldValue::Present(target_value))],
            ),
        ],
        &[],
    )
    .unwrap();
    assert!(closure.contains(&owner));
    assert!(closure.contains(&target));
    let bad = ObjectClosure::new(
        &env,
        [(
            owner.clone(),
            vec![("holder", FieldValue::Present(owner_value))],
        )],
        &[],
    )
    .unwrap_err();
    assert_eq!(*bad.object, owner);
    assert_eq!(
        bad.cause,
        ObjectClosureCause::DanglingReference(Box::new(target))
    );
    assert!(env
        .union(key(1), variant(10), vec![Value::Boolean(true)])
        .is_err());
}

/// Trace: FR-321-AC-4
#[trace("FR-321-AC-4")]
#[test]
fn full_size_resolved_registry_can_be_sealed_without_replacing_topology() {
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let mut deep = ValueType::Composite(key(1));
            for _ in 0..1_000 {
                deep = ValueType::option(deep);
            }
            let mut members: Vec<_> = (0..4_999)
                .map(|i| resolved(&format!("M{i}"), vec![]))
                .collect();
            members.push(resolved("Deep", vec![deep.clone()]));
            let mut env = TypeEnvironment::new(
                [CompositeDeclaration::new(
                    key(1),
                    "Wide",
                    CompositeShape::Union(members),
                )],
                [],
            )
            .unwrap();
            let CompositeShape::Union(members) = env.composite(key(1)).unwrap().shape() else {
                panic!("resolved union required");
            };
            assert_eq!(members.len(), 5_000);
            assert!(members.iter().all(|member| member.member().is_none()));
            assert_eq!(
                env.union_member_named(key(1), "Deep")
                    .unwrap()
                    .1
                    .positions(),
                &[deep.clone()]
            );
            let before = env.clone();
            let bindings = || {
                (0..5_000)
                    .map(|i| {
                        binding(
                            key(40),
                            i + 100,
                            if i == 4_999 {
                                "Deep".to_owned()
                            } else {
                                format!("M{i}")
                            }
                            .as_str(),
                        )
                    })
                    .collect()
            };
            let Err(EnvironmentFailure::Limit(limit)) = env.seal_union_verified_with_cancel(
                key(1),
                key(40),
                bindings(),
                4_999,
                &Cancel::new(),
            ) else {
                panic!("full seal must deny at N minus one");
            };
            assert_eq!(limit.actual(), 5_000);
            assert_eq!(env, before);
            env.seal_union_verified_with_cancel(key(1), key(40), bindings(), 5_000, &Cancel::new())
                .unwrap();
            assert_eq!(env.union_member(key(40), variant(5_099)).unwrap().0, 4_999);
            assert_eq!(
                env.union_member_named(key(1), "Deep")
                    .unwrap()
                    .1
                    .positions(),
                &[deep.clone()]
            );
            let mut expected = ValueType::Composite(key(40));
            for _ in 0..1_000 {
                expected = ValueType::option(expected);
            }
            assert_eq!(env.runtime_type(&deep), Some(expected));
            assert_eq!(env.composite(key(40)).unwrap().key(), key(1));
        })
        .unwrap()
        .join()
        .unwrap();
}
