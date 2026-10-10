// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared registry/admission tests, distinct from trusted kernel construction.
//! Structural fixture keys are already-verified inputs, not FR-441 minting evidence.

use ix_trace_rs::trace;
use quire_exact::{
    CollectionKind, CollectionType, EffectiveId, EnumMember, FieldValue, FloatType, Identifier,
    IeeeWidth, IllTypedCause, Integer, NodeKey, ObjectId, ObjectReference, OptionValue, Presence,
    UnionMember, UnionValue, UniverseId, Value, ValueType, VariantId,
};
use quire_semantic_value::containment::{
    GraphCause, GraphNode, GraphNodeId, GraphSlot, ValueGraph,
};
use quire_semantic_value::declaration::{
    Component, CompositeDeclaration, CompositeShape, ConstructionCause, DeclarationCause,
    EnvironmentFailure, EnvironmentLimitKind, FieldDeclaration, ObjectTypeDeclaration,
    RecursionEdges, TypeEnvironment, TypeEnvironmentLimits, UnionMemberDeclaration,
};
use quire_semantic_value::object_closure::{ObjectClosure, ObjectClosureCause};

#[allow(clippy::disallowed_methods, reason = "fixture supplies producer identities; production mints none")]
fn key(n: u64) -> NodeKey {
    let mut bytes = [0; 32];
    bytes[..8].copy_from_slice(&n.to_be_bytes());
    NodeKey::from_digest(bytes)
}

fn member(
    declaration: NodeKey,
    n: u64,
    name: &str,
    positions: Vec<ValueType>,
) -> UnionMemberDeclaration {
    UnionMemberDeclaration::from_verified(
        UnionMember::from_admitted(declaration, key(n), Identifier::new(name).unwrap()),
        positions,
    )
}

fn shape() -> CompositeDeclaration {
    CompositeDeclaration::new(
        key(1),
        "Shape",
        CompositeShape::Union(vec![
            member(
                key(1),
                12,
                "Rect",
                vec![ValueType::Integer, ValueType::Integer],
            ),
            member(key(1), 10, "Empty", vec![]),
            member(key(1), 11, "Circle", vec![ValueType::Integer]),
        ]),
    )
}

fn integer(n: i64) -> Value {
    Value::Integer(Integer::from(n))
}
fn variant(n: u64) -> VariantId {
    VariantId::from_digest(*key(n).as_bytes())
}

/// Trace: FR-321-AC-1
#[trace("FR-321-AC-1")]
#[test]
fn ordered_resolution_and_real_construction_validate_positions() {
    let env = TypeEnvironment::new([shape()], []).unwrap();
    let (position, rect) = env.union_member_named(key(1), "Rect").unwrap();
    assert_eq!(position, 0);
    assert_eq!(rect.positions(), &[ValueType::Integer, ValueType::Integer]);
    assert_eq!(rect.member().unwrap().variant(), variant(12));
    assert_eq!(env.union_member(key(1), variant(10)).unwrap().0, 1);
    assert!(env.union_member_named(key(1), "rect").is_none());
    let empty = env.union(key(1), variant(10), vec![]).unwrap();
    assert_eq!(empty.occ(), Integer::one());
    let value = env
        .union(key(1), variant(12), vec![integer(2), integer(3)])
        .unwrap();
    assert!(env.admits(&ValueType::Composite(key(1)), &value));
    assert_eq!(value.occ(), Integer::from(3_u64));
    let Value::Union(value) = value else {
        panic!("union carrier required");
    };
    assert_eq!(value.member().identifier().as_str(), "Rect");
    assert_eq!(value.payload().len(), 2);
    assert!(matches!(&value.payload()[0], Value::Integer(n) if n == &Integer::from(2_u64)));
    assert!(matches!(&value.payload()[1], Value::Integer(n) if n == &Integer::from(3_u64)));
    let wrong = env
        .union(key(1), variant(12), vec![integer(2)])
        .unwrap_err();
    assert_eq!(wrong.component, Component::Value);
    assert_eq!(
        wrong.cause,
        ConstructionCause::WrongArity {
            declared: 2,
            supplied: 1
        }
    );
    let wrong = env
        .union(key(1), variant(12), vec![integer(2), Value::Boolean(true)])
        .unwrap_err();
    assert_eq!(wrong.component, Component::Position(1));
    assert_eq!(wrong.cause, ConstructionCause::TypeMismatch);
}

/// Trace: FR-321-AC-1
#[trace("FR-321-AC-1")]
#[test]
fn supplied_trusted_carriers_still_require_registry_admission() {
    let env = TypeEnvironment::new([shape()], []).unwrap();
    let ty = ValueType::Composite(key(1));
    let supplied = |decl, id, name, payload| {
        UnionValue::from_admitted(
            member(decl, id, name, vec![]).member().unwrap().clone(),
            payload,
        )
    };
    for bad in [
        supplied(key(2), 12, "Rect", vec![integer(2), integer(3)]),
        supplied(key(1), 99, "Rect", vec![integer(2), integer(3)]),
        supplied(key(1), 12, "Spoof", vec![integer(2), integer(3)]),
        supplied(key(1), 12, "Rect", vec![integer(2)]),
        supplied(key(1), 11, "Circle", vec![Value::Boolean(true)]),
        Value::Enum(EnumMember::new(variant(12), 0)),
        quire_exact::from_admitted_slots(key(1), vec![].into_boxed_slice()),
    ] {
        assert!(!env.admits(&ty, &bad));
    }
    assert!(env.union(key(1), variant(99), vec![]).is_err());
    assert!(env.union(key(2), variant(12), vec![]).is_err());
    assert!(env.admits(
        &ty,
        &supplied(key(1), 12, "Rect", vec![integer(2), integer(3)])
    ));
}

/// Trace: FR-321-AC-1
#[trace("FR-321-AC-1")]
#[test]
fn duplicate_names_keys_foreign_bindings_and_unknown_types_refuse() {
    let declare = |members| CompositeDeclaration::new(key(1), "U", CompositeShape::Union(members));
    for (members, expected) in [
        (
            vec![
                member(key(1), 10, "A", vec![]),
                member(key(1), 11, "A", vec![]),
            ],
            DeclarationCause::DuplicateMember("A".into()),
        ),
        (
            vec![
                member(key(1), 10, "A", vec![]),
                member(key(1), 10, "B", vec![]),
            ],
            DeclarationCause::DuplicateKey,
        ),
        (
            vec![member(key(2), 10, "A", vec![])],
            DeclarationCause::Type(IllTypedCause::TypeMismatch),
        ),
        (
            vec![member(key(1), 10, "A", vec![ValueType::Composite(key(99))])],
            DeclarationCause::UnknownDeclaration(key(99)),
        ),
    ] {
        let Err(EnvironmentFailure::Refused(actual)) = TypeEnvironment::new([declare(members)], [])
        else {
            panic!("invalid registry must refuse");
        };
        assert_eq!(actual.declaration, "U");
        assert_eq!(actual.cause, expected);
    }
}

/// Trace: FR-323-AC-3
#[trace("FR-323-AC-3")]
#[test]
fn unused_ieee_member_denies_all_keyed_kinds_but_sequence_admits() {
    let env = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "F",
            CompositeShape::Union(vec![
                member(key(1), 10, "Empty", vec![]),
                member(
                    key(1),
                    11,
                    "Measured",
                    vec![ValueType::Float(FloatType::exact(IeeeWidth::Binary64))],
                ),
            ]),
        )],
        [],
    )
    .unwrap();
    assert!(env.contains_ieee(&ValueType::Composite(key(1))));
    let empty = env.union(key(1), variant(10), vec![]).unwrap();
    assert!(env.admits(&ValueType::Composite(key(1)), &empty));
    for kind in [
        CollectionKind::Set,
        CollectionKind::Bag,
        CollectionKind::OrderedSet,
    ] {
        assert_eq!(
            env.check_type(&ValueType::collection(CollectionType::new(
                kind,
                ValueType::Composite(key(1)),
                None,
            )))
            .unwrap_err()
            .cause,
            IllTypedCause::OperatorIneligible
        );
    }
    assert_eq!(
        env.check_type(&ValueType::collection(CollectionType::new(
            CollectionKind::Sequence,
            ValueType::Composite(key(1)),
            None,
        ))),
        Ok(())
    );
}

/// Trace: FR-321-AC-4
#[trace("FR-321-AC-4")]
#[test]
fn named_union_edges_require_escape_and_report_exact_cycle() {
    let declare = |key_id, name, next, escape| {
        CompositeDeclaration::new(
            key(key_id),
            name,
            CompositeShape::Union(vec![member(
                key(key_id),
                key_id + 10,
                "Next",
                vec![if escape {
                    ValueType::option(ValueType::Composite(key(next)))
                } else {
                    ValueType::Composite(key(next))
                }],
            )]),
        )
    };
    let Err(EnvironmentFailure::Refused(actual)) =
        TypeEnvironment::new([declare(1, "A", 2, false), declare(2, "B", 1, false)], [])
    else {
        panic!("non-escaping named cycle must refuse");
    };
    assert_eq!(actual.declaration, "A");
    assert_eq!(
        actual.cause,
        DeclarationCause::Recursion {
            edges: RecursionEdges::NonEscaping,
            cycle: vec!["A".into(), "B".into(), "A".into()],
        }
    );
    let env =
        TypeEnvironment::new([declare(1, "A", 2, false), declare(2, "B", 1, true)], []).unwrap();
    assert_eq!(env.composites().count(), 2);
    let Err(EnvironmentFailure::Refused(actual)) = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "Tuple",
            CompositeShape::Tuple(vec![ValueType::option(ValueType::Composite(key(1)))]),
        )],
        [],
    ) else {
        panic!("unnamed recursion still refuses");
    };
    assert_eq!(
        actual.cause,
        DeclarationCause::Recursion {
            edges: RecursionEdges::Unnamed,
            cycle: vec!["Tuple".into(), "Tuple".into()],
        }
    );
}

/// Trace: FR-321-AC-4
#[trace("FR-321-AC-4")]
#[test]
fn five_thousand_members_and_thousand_options_use_named_work_limit() {
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let declarations = || {
                let mut ty = ValueType::Integer;
                for _ in 0..1_000 {
                    ty = ValueType::option(ty);
                }
                let mut members: Vec<_> = (0..4_999)
                    .map(|i| member(key(1), i + 10, &format!("M{i}"), vec![]))
                    .collect();
                members.push(member(key(1), 5_009, "Deep", vec![ty]));
                [CompositeDeclaration::new(
                    key(1),
                    "Wide",
                    CompositeShape::Union(members),
                )]
            };
            let env = TypeEnvironment::bounded(
                declarations(),
                [],
                TypeEnvironmentLimits {
                    work_units: 6_001,
                    ..TypeEnvironmentLimits::default()
                },
            )
            .unwrap();
            let CompositeShape::Union(members) = env.composite(key(1)).unwrap().shape() else {
                panic!("union registry required");
            };
            assert_eq!(members.len(), 5_000);
            assert_eq!(env.union_member_named(key(1), "Deep").unwrap().0, 4_999);
            let Err(EnvironmentFailure::Limit(limit)) = TypeEnvironment::bounded(
                declarations(),
                [],
                TypeEnvironmentLimits {
                    work_units: 6_000,
                    ..TypeEnvironmentLimits::default()
                },
            ) else {
                panic!("N minus one must deny");
            };
            assert_eq!(limit.kind(), EnvironmentLimitKind::WorkUnits);
            assert_eq!(limit.configured_bound(), 6_000);
            assert_eq!(limit.actual(), 6_001);
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Trace: FR-321-AC-4
#[trace("FR-321-AC-4")]
#[test]
fn ten_thousand_supplied_union_links_are_admitted_on_small_stack() {
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let ty = ValueType::Composite(key(1));
            let env = TypeEnvironment::new(
                [CompositeDeclaration::new(
                    key(1),
                    "Tree",
                    CompositeShape::Union(vec![
                        member(key(1), 10, "Leaf", vec![]),
                        member(key(1), 11, "Node", vec![ValueType::option(ty.clone())]),
                    ]),
                )],
                [],
            )
            .unwrap();
            let leaf = env
                .union_member_named(key(1), "Leaf")
                .unwrap()
                .1
                .member()
                .unwrap()
                .clone();
            let node = env
                .union_member_named(key(1), "Node")
                .unwrap()
                .1
                .member()
                .unwrap()
                .clone();
            // Deliberately bypass checked construction to exercise supplied-value validation.
            let mut value = UnionValue::from_admitted(leaf, vec![]);
            for _ in 0..10_000 {
                value = UnionValue::from_admitted(
                    node.clone(),
                    vec![OptionValue::from_admitted(ty.clone(), Some(value))],
                );
            }
            let mut independent = env.union(key(1), variant(10), vec![]).unwrap();
            for _ in 0..10_000 {
                independent = UnionValue::from_admitted(
                    node.clone(),
                    vec![OptionValue::from_admitted(ty.clone(), Some(independent))],
                );
            }
            assert_eq!(
                quire_exact::plan_equality(&value, &independent)
                    .unwrap()
                    .pair_events(),
                &Integer::from(20_001_u64)
            );
            assert_eq!(
                quire_exact::compare_keys(&value, &independent),
                Some(std::cmp::Ordering::Equal)
            );
            assert_eq!(value.occ(), Integer::from(20_001_u64));
            assert!(env.admits(&ty, &value));
            assert_eq!(env.admits_bounded(&ty, &value, 20_001), Ok(true));
            let limit = env.admits_bounded(&ty, &value, 20_000).unwrap_err();
            assert_eq!(limit.kind(), EnvironmentLimitKind::WorkUnits);
            assert_eq!(limit.configured_bound(), 20_000);
            assert_eq!(limit.actual(), 20_001);
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Trace: FR-321-AC-2
#[trace("FR-321-AC-2")]
#[allow(
    clippy::disallowed_methods,
    reason = "fixture supplies object identities; production mints none"
)]
#[test]
fn union_references_close_and_dangling_payloads_name_the_target() {
    let object_type = EffectiveId::from_digest([1; 32]);
    let reference = |name| {
        ObjectReference::new(
            UniverseId::from_digest([9; 32]),
            object_type,
            ObjectId::new(name).unwrap(),
        )
    };
    let owner = reference("owner");
    let target = reference("target");
    let env = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "Holder",
            CompositeShape::Union(vec![
                member(key(1), 10, "Some", vec![ValueType::Reference(object_type)]),
                member(key(1), 11, "Nothing", vec![]),
            ]),
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
    let value = env
        .union(key(1), variant(10), vec![Value::Reference(target.clone())])
        .unwrap();
    let nothing = env.union(key(1), variant(11), vec![]).unwrap();
    let closure = ObjectClosure::new(
        &env,
        [
            (
                owner.clone(),
                vec![("holder", FieldValue::Present(value.clone()))],
            ),
            (
                target.clone(),
                vec![("holder", FieldValue::Present(nothing))],
            ),
        ],
        &[],
    )
    .unwrap();
    assert!(closure.contains(&owner));
    assert!(closure.contains(&target));
    let bad = ObjectClosure::new(
        &env,
        [(owner.clone(), vec![("holder", FieldValue::Present(value))])],
        &[],
    )
    .unwrap_err();
    assert_eq!(*bad.object, owner);
    assert_eq!(
        bad.cause,
        ObjectClosureCause::DanglingReference(Box::new(target))
    );
}

/// Trace: FR-321-AC-1
#[trace("FR-321-AC-1")]
#[test]
fn union_graph_construction_preserves_cycle_locus_and_positional_payloads() {
    let env = TypeEnvironment::new([shape()], []).unwrap();
    let graph = ValueGraph::new([(
        GraphNodeId(1),
        GraphNode::Union {
            declaration: key(1),
            variant: variant(12),
            positions: vec![GraphSlot::Value(integer(2)), GraphSlot::Value(integer(3))],
        },
    )])
    .unwrap();
    let value = env.build(&graph, GraphNodeId(1)).unwrap();
    assert!(env.admits(&ValueType::Composite(key(1)), &value));
    let cycle = ValueGraph::new([(
        GraphNodeId(7),
        GraphNode::Union {
            declaration: key(1),
            variant: variant(12),
            positions: vec![
                GraphSlot::Node(GraphNodeId(7)),
                GraphSlot::Value(integer(3)),
            ],
        },
    )])
    .unwrap();
    let refusal = env.build(&cycle, GraphNodeId(7)).unwrap_err();
    assert_eq!(refusal.node, GraphNodeId(7));
    assert_eq!(refusal.cause, GraphCause::ContainmentCycle);
}

/// Trace: FR-323-AC-1
#[trace("FR-323-AC-1")]
#[test]
fn registry_binding_drives_identifier_key_order_and_checked_equality() {
    use quire_exact::{compare_keys, Meter, Outcome, ScalarLimits};
    use quire_semantic_value::declaration::{EqualityOperand, EqualityOperator};
    use quire_semantic_value::enumeration::EnumMemberIndex;
    let env = TypeEnvironment::new([shape()], []).unwrap();
    let rect = env
        .union(key(1), variant(12), vec![integer(2), integer(3)])
        .unwrap();
    let same = env
        .union(key(1), variant(12), vec![integer(2), integer(3)])
        .unwrap();
    let circle = env.union(key(1), variant(11), vec![integer(5)]).unwrap();
    let empty = env.union(key(1), variant(10), vec![]).unwrap();
    // Declaration position is Rect, Empty, Circle; digest order is Empty, Circle, Rect.
    // Independently specified ASCII key order is Circle, Empty, Rect.
    assert_eq!(
        compare_keys(&circle, &empty),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(compare_keys(&empty, &rect), Some(std::cmp::Ordering::Less));
    let equality = env
        .check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(ValueType::Composite(key(1))),
            EqualityOperand::typed(ValueType::Composite(key(1))),
            &EnumMemberIndex::default(),
        )
        .unwrap();
    let mut meter = Meter::new(ScalarLimits {
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
    });
    assert!(matches!(
        equality.evaluate(&rect, &same, &mut meter),
        Outcome::Completed(true)
    ));
    assert!(matches!(
        equality.evaluate(&rect, &circle, &mut meter),
        Outcome::Completed(false)
    ));
    let changed = env
        .union(key(1), variant(12), vec![integer(3), integer(2)])
        .unwrap();
    assert!(matches!(
        equality.evaluate(&rect, &changed, &mut meter),
        Outcome::Completed(false)
    ));
    assert_eq!(
        env.check_equality(
            EqualityOperator::Equal,
            EqualityOperand::typed(ValueType::Composite(key(1))),
            EqualityOperand::typed(ValueType::Composite(key(2))),
            &EnumMemberIndex::default()
        )
        .unwrap_err()
        .cause,
        IllTypedCause::TypeMismatch
    );
}

/// Trace: FR-321-AC-1
#[trace("FR-321-AC-1")]
#[test]
fn nested_union_admission_traverses_record_tuple_option_and_collection() {
    let env = TypeEnvironment::new(
        [
            shape(),
            CompositeDeclaration::new(
                key(2),
                "Record",
                CompositeShape::Record(vec![FieldDeclaration::new(
                    "shape",
                    ValueType::Composite(key(1)),
                    Presence::Required,
                )]),
            ),
            CompositeDeclaration::new(
                key(3),
                "Tuple",
                CompositeShape::Tuple(vec![ValueType::Composite(key(2))]),
            ),
        ],
        [],
    )
    .unwrap();
    let wrap = |shape| {
        let record = quire_exact::from_admitted_slots(
            key(2),
            vec![FieldValue::Present(shape)].into_boxed_slice(),
        );
        let tuple = quire_exact::from_admitted_slots(
            key(3),
            vec![FieldValue::Present(record)].into_boxed_slice(),
        );
        let ty = ValueType::Composite(key(3));
        let collection = quire_exact::from_admitted(
            CollectionType::new(CollectionKind::Sequence, ty.clone(), None),
            vec![tuple],
        );
        OptionValue::from_admitted(
            ValueType::collection(CollectionType::new(CollectionKind::Sequence, ty, None)),
            Some(collection),
        )
    };
    let ty = ValueType::option(ValueType::collection(CollectionType::new(
        CollectionKind::Sequence,
        ValueType::Composite(key(3)),
        None,
    )));
    let good = wrap(env.union(key(1), variant(10), vec![]).unwrap());
    assert!(env.admits(&ty, &good));
    let bad = wrap(UnionValue::from_admitted(
        env.union_member_named(key(1), "Circle")
            .unwrap()
            .1
            .member()
            .unwrap()
            .clone(),
        vec![Value::Boolean(true)],
    ));
    assert!(!env.admits(&ty, &bad));
}

/// Trace: FR-321-AC-1
#[trace("FR-321-AC-1")]
#[test]
fn registry_evaluation_orders_payloads_and_propagates_first_stop() {
    use quire_exact::{
        CheckedInvariantCause, Deferred, Meter, Outcome, Refusal, ScalarLimits, Undefined,
    };
    use std::cell::RefCell;
    let env = TypeEnvironment::new([shape()], []).unwrap();
    let meter = || {
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
    };
    let visits = RefCell::new(Vec::new());
    let first = |_: &mut Meter| {
        visits.borrow_mut().push(0);
        Outcome::Completed(integer(2))
    };
    let second = |_: &mut Meter| {
        visits.borrow_mut().push(1);
        Outcome::Completed(integer(3))
    };
    let payload: Vec<Deferred<'_>> = vec![Box::new(first), Box::new(second)];
    let result = env
        .evaluate_union(key(1), variant(12), payload, &mut meter())
        .unwrap();
    let Outcome::Completed(value) = result else {
        panic!("completed registry construction required");
    };
    assert_eq!(*visits.borrow(), vec![0, 1]);
    assert_eq!(value.occ(), Integer::from(3_u64));
    visits.borrow_mut().clear();
    let stop = |_: &mut Meter| {
        visits.borrow_mut().push(0);
        Outcome::Undefined(Undefined::DivisionByZero)
    };
    let second = |_: &mut Meter| {
        visits.borrow_mut().push(1);
        Outcome::Completed(integer(3))
    };
    assert!(matches!(
        env.evaluate_union(
            key(1),
            variant(12),
            vec![Box::new(stop), Box::new(second)],
            &mut meter()
        )
        .unwrap(),
        Outcome::Undefined(Undefined::DivisionByZero)
    ));
    assert_eq!(*visits.borrow(), vec![0]);
    let wrong = |_: &mut Meter| Outcome::Completed(Value::Boolean(true));
    assert!(matches!(
        env.evaluate_union(key(1), variant(11), vec![Box::new(wrong)], &mut meter())
            .unwrap(),
        Outcome::Refused(Refusal::CheckedInvariant {
            cause: CheckedInvariantCause::DeferredResultNotAdmitted
        })
    ));
    assert!(
        matches!(env.evaluate_union(key(1), variant(10), vec![], &mut meter()).unwrap(),
        Outcome::Completed(value) if value.occ() == Integer::one())
    );
}
