// SPDX-License-Identifier: AGPL-3.0-or-later
//! `Map` and `BoundedQuantity` in declaration admission, equality checking,
//! type resolution and supplied membership, through the public API (TC-927).
//! Fixture keys are producer-supplied identities, not a digest-minting oracle.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ix_trace_rs::trace;
use quire_exact::{
    from_admitted, BoundedQuantityType, Cancel, CancelCause, CardinalityBound, CollectionKind,
    CollectionType, EffectiveId, FieldValue, FloatType, Identifier, IeeeWidth, IllTypedCause,
    Integer, IntegerInterval, MapType, Meter, NodeKey, OptionValue, Outcome, Presence, Quantity,
    Rational, ScalarLimits, UnionMember, UnitId, Value, ValueType, VariantId,
};
use quire_semantic_value::declaration::{
    admits_equality_conversion, CompositeDeclaration, CompositeShape, DeclarationCause,
    EnvironmentFailure, EnvironmentLimitKind, EqualityOperand, EqualityOperator, EqualitySchedule,
    FieldDeclaration, RecursionEdges, TypeEnvironment, TypeEnvironmentLimits,
    UnionMemberDeclaration,
};
use quire_semantic_value::enumeration::EnumMemberIndex;
use quire_semantic_value::quantity::{UnitScope, UnitTable};
use quire_semantic_value::supplied_membership::{
    BoundedSuppliedAdmissionError, InvalidMembershipCause, MembershipLocus, MembershipPart,
    MembershipPathArena, QuantityBoundEndpoint, SuppliedAdmissionError, SuppliedMembershipBudget,
    UnsupportedHelperFamily,
};
use quire_semantic_value::unit::{DimensionNode, NominalDeclaration, UnitGraph, UnitNode};

#[expect(
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

/// Unit `U`, and `V`, a distinct unit of the same dimension, and `W`, a unit
/// of another dimension. `X` names no unit.
fn unit_u() -> UnitId {
    UnitId::declared(key(3))
}
fn unit_v() -> UnitId {
    UnitId::declared(key(4))
}
fn unit_w() -> UnitId {
    UnitId::declared(key(5))
}
fn unit_x() -> UnitId {
    UnitId::declared(key(6))
}

fn unit_table() -> UnitTable {
    let dimensions = BTreeMap::from([
        (
            key(1),
            DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
        ),
        (
            key(2),
            DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
        ),
    ]);
    let node = |dimension: u64, target: Option<u64>, scale: i64| {
        UnitNode::checked(
            *key(dimension).as_bytes(),
            target.map(|target| *key(target).as_bytes()),
            Rational::from_integer(Integer::from(scale)),
            Rational::from_integer(Integer::zero()),
            NominalDeclaration::default(),
        )
        .unwrap()
    };
    let units = BTreeMap::from([
        (key(3), node(1, None, 1)),
        (key(4), node(1, Some(3), 1000)),
        (key(5), node(2, None, 1)),
    ]);
    let graph = UnitGraph::from_checked_nodes(&dimensions, &units).unwrap();
    UnitTable::declared(&graph)
}

fn integer(n: i64) -> Integer {
    Integer::from(n)
}

fn quantity(n: i64, unit: UnitId) -> Quantity {
    Quantity::new(Rational::from_integer(integer(n)), unit)
}

/// `BoundedQuantity<unit>[min, max]`.
fn bounded(unit: UnitId, min: Option<i64>, max: Option<i64>) -> ValueType {
    ValueType::bounded_quantity(
        BoundedQuantityType::new(
            unit,
            min.map(|n| quantity(n, unit)),
            max.map(|n| quantity(n, unit)),
        )
        .unwrap(),
    )
}

fn bound(minimum: u64, maximum: u64) -> Option<CardinalityBound> {
    Some(CardinalityBound::new(minimum, maximum).unwrap())
}

fn map_of(values: ValueType, bound: Option<CardinalityBound>) -> ValueType {
    ValueType::map(MapType::new(values, bound))
}

fn sequence_of(element: ValueType) -> ValueType {
    ValueType::collection(CollectionType::new(CollectionKind::Sequence, element, None))
}

fn set_of(element: ValueType) -> ValueType {
    ValueType::collection(CollectionType::new(CollectionKind::Set, element, None))
}

fn float64() -> ValueType {
    ValueType::Float(FloatType::exact(IeeeWidth::Binary64))
}

fn float32() -> ValueType {
    ValueType::Float(FloatType::exact(IeeeWidth::Binary32))
}

fn field(name: &str, value_type: ValueType) -> FieldDeclaration {
    FieldDeclaration::new(name, value_type, Presence::Required)
}

fn record(n: u64, name: &str, fields: Vec<FieldDeclaration>) -> CompositeDeclaration {
    CompositeDeclaration::new(key(n), name, CompositeShape::Record(fields))
}

/// Key 1 is a record without a float; key 2 is a record holding one.
fn support() -> Vec<CompositeDeclaration> {
    vec![
        record(1, "Plain", vec![field("n", ValueType::Integer)]),
        record(2, "Floaty", vec![field("f", float64())]),
    ]
}

/// The environment declaring one record `Holder` with one field per type.
fn declare(types: &[ValueType]) -> Result<TypeEnvironment, DeclarationCause> {
    let fields = types
        .iter()
        .enumerate()
        .map(|(index, value_type)| field(&format!("f{index}"), value_type.clone()))
        .collect();
    let mut declarations = support();
    declarations.push(record(100, "Holder", fields));
    match TypeEnvironment::new(declarations, []) {
        Ok(environment) => Ok(environment),
        Err(EnvironmentFailure::Refused(refusal)) => Err(refusal.cause),
        Err(other) => panic!("unexpected failure {other:?}"),
    }
}

fn environment() -> TypeEnvironment {
    TypeEnvironment::new(support(), []).unwrap()
}

fn ill_typed(cause: IllTypedCause) -> DeclarationCause {
    DeclarationCause::Type(cause)
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

// ---------------------------------------------------------------- AC-1

/// `contains_ieee` reaches a float through `Map` links, and the metered walk
/// (a `Set` element's check) agrees and charges a `Map` link as a `Collection`
/// link.
#[trace("TC-927", "FR-127-AC-1")]
#[test]
fn ieee_containment_descends_map_links_in_both_walks() {
    let environment = environment();
    let with_float = [
        map_of(float64(), None),
        ValueType::option(map_of(float32(), None)),
        map_of(ValueType::option(sequence_of(float64())), bound(0, 3)),
        map_of(map_of(float64(), None), None),
        map_of(ValueType::Composite(key(2)), None),
    ];
    let without_float = [
        map_of(ValueType::Integer, None),
        map_of(bounded(unit_u(), Some(0), Some(10)), None),
        map_of(ValueType::Composite(key(1)), None),
        ValueType::option(map_of(ValueType::option(ValueType::Boolean), None)),
    ];
    for value_type in &with_float {
        assert!(environment.contains_ieee(value_type), "{value_type:?}");
        // A set element is refused exactly when it contains IEEE content.
        assert_eq!(
            declare(&[set_of(value_type.clone())]).err(),
            Some(ill_typed(IllTypedCause::OperatorIneligible)),
            "{value_type:?}"
        );
    }
    for value_type in &without_float {
        assert!(!environment.contains_ieee(value_type), "{value_type:?}");
        assert!(
            declare(&[set_of(value_type.clone())]).is_ok(),
            "{value_type:?}"
        );
    }
    // A record field of such a type is reached through its declaration.
    let holder = declare(&[map_of(float64(), None)]).unwrap();
    assert!(holder.contains_ieee(&ValueType::Composite(key(100))));
    let plain = declare(&[map_of(ValueType::Integer, None)]).unwrap();
    assert!(!plain.contains_ieee(&ValueType::Composite(key(100))));

    // The metered charge for a map link equals that for a collection link.
    let inner = ValueType::option(ValueType::option(ValueType::Integer));
    for wrap in [
        ValueType::option as fn(ValueType) -> ValueType,
        set_of,
        sequence_of,
    ] {
        let through_map = union_spent(wrap(map_of(inner.clone(), None)));
        let through_sequence = union_spent(wrap(sequence_of(inner.clone())));
        let leaf_only = union_spent(wrap(inner.clone()));
        assert_eq!(through_map, through_sequence);
        assert!(through_map > leaf_only);
    }
    // And the budget boundary falls on the same unit.
    let element = set_of(map_of(inner.clone(), None));
    let spent = union_spent(element.clone());
    let sequence_spent = union_spent(set_of(sequence_of(inner)));
    assert_eq!(spent, sequence_spent);
    let declarations = support()
        .into_iter()
        .chain([sealed_union_with_position(element)]);
    let Err(EnvironmentFailure::Limit(limit)) = TypeEnvironment::bounded(
        declarations,
        [],
        TypeEnvironmentLimits {
            work_units: spent - 1,
            ..TypeEnvironmentLimits::default()
        },
    ) else {
        panic!("one unit below the charge must stop");
    };
    assert_eq!(limit.kind(), EnvironmentLimitKind::WorkUnits);
}

// ---------------------------------------------------------------- AC-2

/// A map over floats is a valid declared type, as under a `Sequence`; a `Set`
/// whose element is such a map stays refused.
#[trace("TC-927", "FR-127-AC-2")]
#[test]
fn map_over_floats_is_admitted_and_set_of_such_a_map_is_refused() {
    let accepted = [
        map_of(float64(), None),
        ValueType::option(map_of(float32(), None)),
        sequence_of(map_of(float64(), None)),
        map_of(float64(), bound(1, 2)),
        map_of(map_of(float64(), None), None),
    ];
    let environment = environment();
    for value_type in &accepted {
        assert!(declare(core::slice::from_ref(value_type)).is_ok());
        assert_eq!(environment.check_type(value_type), Ok(()));
    }
    let refused = set_of(map_of(float64(), None));
    assert_eq!(
        declare(core::slice::from_ref(&refused)).err(),
        Some(ill_typed(IllTypedCause::OperatorIneligible))
    );
    assert_eq!(
        environment.check_type(&refused).unwrap_err().cause,
        IllTypedCause::OperatorIneligible
    );
}

// ---------------------------------------------------------------- AC-3

/// A map's value type receives every refusal an `Option` payload or a
/// collection element does, and the plain and metered walks agree.
#[trace("TC-927", "FR-127-AC-3")]
#[expect(
    clippy::disallowed_methods,
    reason = "fixture supplies a producer identity; production mints none"
)]
#[test]
fn map_value_type_is_refused_as_inside_option_and_collection() {
    let object_free = EffectiveId::from_digest([9; 32]);
    let cases = [
        (
            ValueType::Population(Some(2)),
            ill_typed(IllTypedCause::OperatorIneligible),
        ),
        (
            ValueType::option(ValueType::Population(None)),
            ill_typed(IllTypedCause::OperatorIneligible),
        ),
        (
            sequence_of(ValueType::Population(Some(1))),
            ill_typed(IllTypedCause::OperatorIneligible),
        ),
        (
            ValueType::Composite(key(77)),
            DeclarationCause::UnknownDeclaration(key(77)),
        ),
        (
            ValueType::Reference(object_free),
            ill_typed(IllTypedCause::TypeMismatch),
        ),
    ];
    let environment = environment();
    for (inner, cause) in cases {
        let through_map = map_of(inner.clone(), None);
        for wrapped in [
            ValueType::option(inner.clone()),
            sequence_of(inner.clone()),
            through_map.clone(),
            map_of(map_of(inner.clone(), None), bound(0, 2)),
            ValueType::option(map_of(inner.clone(), None)),
        ] {
            // Metered: environment admission.
            assert_eq!(
                declare(core::slice::from_ref(&wrapped)).err(),
                Some(cause.clone()),
                "{wrapped:?}"
            );
            // Plain: `check_type`, which reads an unknown declaration as a
            // type mismatch.
            let plain = match &cause {
                DeclarationCause::Type(cause) => *cause,
                _ => IllTypedCause::TypeMismatch,
            };
            assert_eq!(
                environment.check_type(&wrapped).unwrap_err().cause,
                plain,
                "{wrapped:?}"
            );
        }
    }
}

// ---------------------------------------------------------------- AC-4

fn sealed_union_with_position(position: ValueType) -> CompositeDeclaration {
    CompositeDeclaration::new(
        key(50),
        "U",
        CompositeShape::Union(vec![UnionMemberDeclaration::from_verified(
            UnionMember::from_admitted(key(50), key(51), Identifier::new("M").unwrap()),
            vec![position],
        )]),
    )
}

fn union_spent(position: ValueType) -> u64 {
    TypeEnvironment::new(
        support()
            .into_iter()
            .chain([sealed_union_with_position(position)]),
        [],
    )
    .unwrap()
    .spent()
}

/// A union member position through maps is walked link by link at the same
/// charge as the same shape through collections, and one unit fewer stops.
#[trace("TC-927", "FR-127-AC-4")]
#[test]
fn union_member_positions_are_walked_through_map_links() {
    let positions = [
        (
            map_of(ValueType::Integer, None),
            sequence_of(ValueType::Integer),
        ),
        (
            ValueType::option(map_of(ValueType::Composite(key(1)), None)),
            ValueType::option(sequence_of(ValueType::Composite(key(1)))),
        ),
        (
            map_of(map_of(ValueType::Boolean, None), bound(0, 5)),
            sequence_of(sequence_of(ValueType::Boolean)),
        ),
    ];
    for (through_map, through_sequence) in positions {
        let spent = union_spent(through_map.clone());
        assert_eq!(spent, union_spent(through_sequence));
        let leaf = union_spent(ValueType::Boolean);
        assert!(spent > leaf);
        let declarations = || {
            support()
                .into_iter()
                .chain([sealed_union_with_position(through_map.clone())])
        };
        assert!(TypeEnvironment::bounded(
            declarations(),
            [],
            TypeEnvironmentLimits {
                work_units: spent,
                ..TypeEnvironmentLimits::default()
            },
        )
        .is_ok());
        let Err(EnvironmentFailure::Limit(limit)) = TypeEnvironment::bounded(
            declarations(),
            [],
            TypeEnvironmentLimits {
                work_units: spent - 1,
                ..TypeEnvironmentLimits::default()
            },
        ) else {
            panic!("one unit fewer must stop with the work limit");
        };
        assert_eq!(limit.kind(), EnvironmentLimitKind::WorkUnits);
    }
}

// ---------------------------------------------------------------- AC-5

fn self_record(field_type: ValueType) -> Result<TypeEnvironment, DeclarationCause> {
    match TypeEnvironment::new([record(1, "Rec", vec![field("m", field_type)])], []) {
        Ok(environment) => Ok(environment),
        Err(EnvironmentFailure::Refused(refusal)) => Err(refusal.cause),
        Err(other) => panic!("unexpected failure {other:?}"),
    }
}

/// A cycle through a map escapes exactly when the map can be empty.
#[trace("TC-927", "FR-127-AC-5")]
#[test]
fn recursion_through_a_map_escapes_only_when_it_can_be_empty() {
    let this = || ValueType::Composite(key(1));
    assert!(self_record(map_of(this(), None)).is_ok());
    assert!(self_record(map_of(this(), bound(0, 4))).is_ok());
    assert!(self_record(ValueType::option(map_of(this(), bound(1, 4)))).is_ok());
    assert!(self_record(map_of(map_of(this(), bound(1, 4)), None)).is_ok());
    let non_escaping = DeclarationCause::Recursion {
        edges: RecursionEdges::NonEscaping,
        cycle: vec!["Rec".into(), "Rec".into()],
    };
    assert_eq!(
        self_record(map_of(this(), bound(1, 4))).err(),
        Some(non_escaping.clone())
    );
    assert_eq!(
        self_record(map_of(map_of(this(), None), bound(2, 4))).err(),
        None,
        "the inner unbounded map escapes"
    );
    assert_eq!(
        self_record(map_of(map_of(this(), bound(1, 4)), bound(1, 4))).err(),
        Some(non_escaping.clone())
    );
    // The same shape through a sequence behaves identically.
    assert_eq!(
        self_record(ValueType::collection(CollectionType::new(
            CollectionKind::Sequence,
            this(),
            bound(1, 4),
        )))
        .err(),
        Some(non_escaping)
    );
    // A tuple position is an unnamed edge, whatever the map's bound.
    let tuple = |position: ValueType| {
        TypeEnvironment::new(
            [CompositeDeclaration::new(
                key(1),
                "Rec",
                CompositeShape::Tuple(vec![position]),
            )],
            [],
        )
    };
    let Err(EnvironmentFailure::Refused(refusal)) = tuple(map_of(this(), None)) else {
        panic!("a tuple cycle through a map refuses");
    };
    assert_eq!(
        refusal.cause,
        DeclarationCause::Recursion {
            edges: RecursionEdges::Unnamed,
            cycle: vec!["Rec".into(), "Rec".into()],
        }
    );
    // A named union edge reached through a map keeps its recorded name.
    let union = |positions: Vec<ValueType>| {
        TypeEnvironment::new(
            [CompositeDeclaration::new(
                key(1),
                "A",
                CompositeShape::Union(vec![UnionMemberDeclaration::from_verified(
                    UnionMember::from_admitted(key(1), key(2), Identifier::new("Next").unwrap()),
                    positions,
                )]),
            )],
            [],
        )
    };
    assert!(union(vec![map_of(this(), None)]).is_ok());
    let Err(EnvironmentFailure::Refused(refusal)) = union(vec![map_of(this(), bound(1, 2))]) else {
        panic!("a named non-escaping cycle through a map refuses");
    };
    assert_eq!(
        refusal.cause,
        DeclarationCause::Recursion {
            edges: RecursionEdges::NonEscaping,
            cycle: vec!["A".into(), "A".into()],
        }
    );
}

// ---------------------------------------------------------------- AC-6

/// A bounded quantity is a leaf in every declaration walk.
#[trace("TC-927", "FR-127-AC-6")]
#[test]
fn bounded_quantity_is_a_leaf_everywhere_in_declaration() {
    let table = unit_table();
    let scope = UnitScope::new(&table);
    let shapes = [
        bounded(unit_u(), Some(0), Some(100)),
        bounded(unit_u(), None, None),
        bounded(unit_u(), Some(5), Some(3)),
        bounded(unit_x(), Some(0), None),
    ];
    for shape in &shapes {
        let types = [
            shape.clone(),
            ValueType::option(shape.clone()),
            sequence_of(shape.clone()),
            set_of(shape.clone()),
            map_of(shape.clone(), None),
        ];
        let environment = declare(&types).unwrap();
        for value_type in &types {
            assert!(!environment.contains_ieee(value_type));
            assert_eq!(environment.check_type(value_type), Ok(()));
        }
        assert!(!environment.contains_ieee(&ValueType::Composite(key(100))));
        assert!(admits_equality_conversion(shape, shape, &scope));
    }
    // It adds no cycle edge: the record is its own only declaration.
    assert!(self_record(bounded(unit_u(), Some(1), Some(2))).is_ok());
    // A bounded quantity at a union member position walks as a leaf.
    assert!(TypeEnvironment::new(
        support()
            .into_iter()
            .chain([sealed_union_with_position(bounded(unit_u(), None, None))]),
        [],
    )
    .is_ok());
}

// ---------------------------------------------------------------- AC-7

#[trace("TC-927", "FR-127-AC-7")]
#[test]
fn integer_sources_convert_to_neither_a_map_nor_a_bounded_quantity() {
    let table = unit_table();
    let scope = UnitScope::new(&table);
    let source = ValueType::Int(IntegerInterval::spanning(integer(0), integer(10)));
    for target in [
        map_of(ValueType::Integer, None),
        map_of(source.clone(), bound(0, 3)),
        bounded(unit_u(), Some(0), Some(10)),
        bounded(unit_u(), None, None),
    ] {
        assert!(!admits_equality_conversion(&source, &target, &scope));
    }
    // The existing rows are unchanged.
    assert!(admits_equality_conversion(
        &source,
        &ValueType::Integer,
        &scope
    ));
    assert!(admits_equality_conversion(
        &source,
        &ValueType::Int(IntegerInterval::spanning(integer(0), integer(20))),
        &scope
    ));
    assert!(!admits_equality_conversion(
        &source,
        &ValueType::Int(IntegerInterval::spanning(integer(0), integer(5))),
        &scope
    ));
    assert!(!admits_equality_conversion(
        &source,
        &ValueType::Quantity(unit_u()),
        &scope
    ));
}

// ---------------------------------------------------------------- AC-8

fn root_loci(
    arena: &mut MembershipPathArena,
    argument: usize,
) -> (MembershipLocus, MembershipLocus) {
    let root = arena.root(argument);
    (
        MembershipLocus {
            path: root,
            part: MembershipPart::Value,
        },
        MembershipLocus {
            path: root,
            part: MembershipPart::TypeLink(0),
        },
    )
}

type Admission = Result<(), SuppliedAdmissionError<InvalidMembershipCause>>;

fn admit(
    environment: &TypeEnvironment,
    expected: &ValueType,
    value: &Value,
) -> (Admission, u64, MembershipLocus) {
    let mut arena = MembershipPathArena::new().unwrap();
    let (at, _) = root_loci(&mut arena, 0);
    let mut budget = SuppliedMembershipBudget::new(u64::MAX);
    let result = environment.admit_supplied::<InvalidMembershipCause>(
        expected,
        value,
        &mut arena,
        at,
        &mut budget,
        &Cancel::new(),
    );
    (result, budget.spent(), at)
}

fn invalid(cause: InvalidMembershipCause, locus: MembershipLocus) -> Admission {
    Err(SuppliedAdmissionError::Invalid { cause, locus })
}

fn some_values(environment: &TypeEnvironment) -> Vec<Value> {
    vec![
        from_admitted(
            CollectionType::new(CollectionKind::Sequence, ValueType::Boolean, None),
            vec![Value::Boolean(true)],
        ),
        Value::String("text".into()),
        OptionValue::from_admitted(ValueType::Boolean, Some(Value::Boolean(false))),
        environment
            .record(
                key(1),
                vec![("n", FieldValue::Present(Value::Integer(integer(1))))],
            )
            .unwrap(),
        Value::Quantity(quantity(5, unit_u())),
        Value::Boolean(true),
    ]
}

/// No value of any variant is a member of a map type.
#[trace("TC-927", "FR-127-AC-8")]
#[test]
fn no_value_is_a_member_of_a_map_type() {
    let environment = environment();
    for expected in [
        map_of(ValueType::Boolean, None),
        map_of(ValueType::option(ValueType::Boolean), bound(1, 3)),
    ] {
        for value in some_values(&environment) {
            let (result, spent, at) = admit(&environment, &expected, &value);
            assert_eq!(
                result,
                invalid(InvalidMembershipCause::Kind, at),
                "{value:?}"
            );
            assert_eq!(spent, 1, "one ValueVisit event");
            // Structural membership agrees.
            assert_eq!(environment.admits(&expected, &value), Ok(false));
        }
    }
}

// ---------------------------------------------------------------- AC-9

fn quantity_value(numerator: i64, denominator: i64, unit: UnitId) -> Value {
    Value::Quantity(Quantity::new(
        Rational::new(integer(numerator), integer(denominator)).unwrap(),
        unit,
    ))
}

/// A bounded quantity admits exactly the in-range quantities of its unit, and
/// refuses with the cause its (type, value) pair names.
#[trace("TC-927", "FR-127-AC-9")]
#[test]
fn bounded_quantity_membership_admits_in_range_and_names_the_cause() {
    let environment = environment();
    let check = |expected: &ValueType, value: Value, cause: Option<InvalidMembershipCause>| {
        let (result, spent, at) = admit(&environment, expected, &value);
        match cause {
            None => assert_eq!(result, Ok(()), "{value:?} in {expected:?}"),
            Some(cause) => assert_eq!(result, invalid(cause, at), "{value:?} in {expected:?}"),
        }
        assert_eq!(spent, 1);
        assert_eq!(
            environment.admits(expected, &value),
            Ok(cause.is_none()),
            "{value:?}"
        );
    };
    let closed = bounded(unit_u(), Some(0), Some(100));
    for magnitude in [0, 50, 100] {
        check(&closed, quantity_value(magnitude, 1, unit_u()), None);
    }
    for magnitude in [-1, 101] {
        check(
            &closed,
            quantity_value(magnitude, 1, unit_u()),
            Some(InvalidMembershipCause::Payload),
        );
    }
    check(
        &closed,
        quantity_value(50, 1, unit_v()),
        Some(InvalidMembershipCause::Type),
    );
    for value in [
        Value::String("t".into()),
        Value::Boolean(true),
        OptionValue::from_admitted(ValueType::Boolean, None),
    ] {
        check(&closed, value, Some(InvalidMembershipCause::Kind));
    }
    // A fraction inside and outside a control range.
    check(
        &bounded(unit_u(), Some(1), Some(100)),
        quantity_value(1, 2, unit_u()),
        Some(InvalidMembershipCause::Payload),
    );
    check(&closed, quantity_value(1, 2, unit_u()), None);
    // An open side admits any magnitude on that side.
    let no_minimum = bounded(unit_u(), None, Some(100));
    check(&no_minimum, quantity_value(-1_000_000, 1, unit_u()), None);
    check(
        &no_minimum,
        quantity_value(101, 1, unit_u()),
        Some(InvalidMembershipCause::Payload),
    );
    let no_maximum = bounded(unit_u(), Some(0), None);
    check(&no_maximum, quantity_value(1_000_000, 1, unit_u()), None);
    check(
        &no_maximum,
        quantity_value(-1, 1, unit_u()),
        Some(InvalidMembershipCause::Payload),
    );
    // An inverted type refuses every value, by the same causes.
    let inverted = bounded(unit_u(), Some(5), Some(3));
    for magnitude in [3, 4, 5, 0, 10] {
        check(
            &inverted,
            quantity_value(magnitude, 1, unit_u()),
            Some(InvalidMembershipCause::Payload),
        );
    }
    check(
        &inverted,
        quantity_value(4, 1, unit_v()),
        Some(InvalidMembershipCause::Type),
    );
    check(
        &inverted,
        Value::Boolean(false),
        Some(InvalidMembershipCause::Kind),
    );
}

// --------------------------------------------------------------- AC-10

struct Walk {
    result: Admission,
    spent: u64,
    root: MembershipLocus,
}

fn link(root: &MembershipLocus, ordinal: u64) -> MembershipLocus {
    MembershipLocus {
        path: root.path,
        part: MembershipPart::TypeLink(ordinal),
    }
}

/// Admit `Option` whose retained payload type is `actual` against an expected
/// `Option<expected>`, with no numeric bound.
fn option_walk(environment: &TypeEnvironment, expected: &ValueType, actual: &ValueType) -> Walk {
    let value = OptionValue::from_admitted(actual.clone(), None);
    let (result, spent, root) = admit(environment, &ValueType::option(expected.clone()), &value);
    Walk {
        result,
        spent,
        root,
    }
}

/// Retained map types are compared by bound and then value type, each pair one
/// event, numeric links by the bounded operations.
#[trace("TC-927", "FR-127-AC-10")]
#[test]
fn retained_map_type_links_compare_bound_then_value_type() {
    let environment = environment();
    let map = |values: ValueType, b: Option<CardinalityBound>| map_of(values, b);
    let equal = option_walk(
        &environment,
        &map(ValueType::Boolean, bound(0, 3)),
        &map(ValueType::Boolean, bound(0, 3)),
    );
    assert_eq!(equal.result, Ok(()));
    // ValueVisit, the map pair, the value pair.
    assert_eq!(equal.spent, 3);

    let mismatches = [
        (
            map(ValueType::Boolean, bound(0, 3)),
            map(ValueType::Boolean, bound(0, 4)),
            0,
        ),
        (
            map(ValueType::Boolean, bound(0, 3)),
            map(ValueType::Boolean, bound(1, 3)),
            0,
        ),
        (
            map(ValueType::Boolean, bound(0, u64::MAX)),
            map(ValueType::Boolean, None),
            0,
        ),
        (
            map(ValueType::Boolean, None),
            map(ValueType::Boolean, bound(0, u64::MAX)),
            0,
        ),
        (map(ValueType::Boolean, None), map(ValueType::Uuid, None), 1),
        (
            map(ValueType::Boolean, None),
            sequence_of(ValueType::Boolean),
            0,
        ),
        (
            sequence_of(ValueType::Boolean),
            map(ValueType::Boolean, None),
            0,
        ),
        (
            map(ValueType::Boolean, None),
            ValueType::option(ValueType::Boolean),
            0,
        ),
    ];
    for (expected, actual, ordinal) in mismatches {
        let walk = option_walk(&environment, &expected, &actual);
        assert_eq!(
            walk.result,
            invalid(InvalidMembershipCause::Type, link(&walk.root, ordinal)),
            "{expected:?} against {actual:?}"
        );
    }

    // An `Int` inside the map is a numeric link of its own pair.
    let int = ValueType::Int(IntegerInterval::spanning(integer(0), integer(3)));
    let numeric = map(int.clone(), None);
    let unbounded = option_walk(&environment, &numeric, &numeric);
    assert_eq!(
        unbounded.result,
        Err(SuppliedAdmissionError::UnsupportedHelper {
            family: UnsupportedHelperFamily::NumericDomain,
            locus: link(&unbounded.root, 1),
        })
    );
    assert_eq!(unbounded.spent, 3);
    let bounded_walk = |expected: &ValueType, actual: &ValueType| {
        let mut arena = MembershipPathArena::new().unwrap();
        let (at, _) = root_loci(&mut arena, 0);
        let mut budget = SuppliedMembershipBudget::new(u64::MAX);
        let value = OptionValue::from_admitted(actual.clone(), None);
        let result = environment.admit_supplied_bounded_numeric::<InvalidMembershipCause>(
            &ValueType::option(expected.clone()),
            &value,
            &mut arena,
            at,
            &mut budget,
            &Cancel::new(),
            64,
        );
        (result, budget.spent(), at)
    };
    let (result, spent, _) = bounded_walk(&numeric, &numeric);
    assert_eq!(result, Ok(()));
    assert_eq!(spent, 3);
    let other = map(
        ValueType::Int(IntegerInterval::spanning(integer(0), integer(4))),
        None,
    );
    let (result, _, at) = bounded_walk(&numeric, &other);
    assert_eq!(
        result,
        Err(BoundedSuppliedAdmissionError::Admission(
            SuppliedAdmissionError::Invalid {
                cause: InvalidMembershipCause::Type,
                locus: link(&at, 1),
            }
        ))
    );
}

// --------------------------------------------------------------- AC-11

/// A retained bounded quantity matches only an equal bounded quantity.
#[trace("TC-927", "FR-127-AC-11")]
#[test]
fn retained_bounded_quantity_links_match_only_an_equal_type() {
    let environment = environment();
    let walk = |expected: &ValueType, actual: &ValueType| {
        let mut arena = MembershipPathArena::new().unwrap();
        let (at, _) = root_loci(&mut arena, 0);
        let mut budget = SuppliedMembershipBudget::new(u64::MAX);
        let value = OptionValue::from_admitted(actual.clone(), None);
        let result = environment.admit_supplied_bounded_numeric::<InvalidMembershipCause>(
            &ValueType::option(expected.clone()),
            &value,
            &mut arena,
            at,
            &mut budget,
            &Cancel::new(),
            64,
        );
        (result, at)
    };
    let base = bounded(unit_u(), Some(0), Some(100));
    assert_eq!(walk(&base, &base).0, Ok(()));
    assert_eq!(
        walk(&base, &bounded(unit_u(), Some(0), Some(100))).0,
        Ok(())
    );
    let different = [
        bounded(unit_v(), Some(0), Some(100)),
        bounded(unit_u(), Some(1), Some(100)),
        bounded(unit_u(), Some(0), Some(99)),
        bounded(unit_u(), None, Some(100)),
        bounded(unit_u(), Some(0), None),
        bounded(unit_u(), None, None),
        ValueType::Quantity(unit_u()),
    ];
    for other in &different {
        for (expected, actual) in [(&base, other), (other, &base)] {
            let (result, at) = walk(expected, actual);
            assert_eq!(
                result,
                Err(BoundedSuppliedAdmissionError::Admission(
                    SuppliedAdmissionError::Invalid {
                        cause: InvalidMembershipCause::Type,
                        locus: link(&at, 0),
                    }
                )),
                "{expected:?} against {actual:?}"
            );
        }
    }
    // Two unbounded bounded types of one unit are the same type; no bound
    // pair remains, so no magnitude ceiling is needed.
    let open = bounded(unit_u(), None, None);
    let walk = option_walk(&environment, &open, &open);
    assert_eq!(walk.result, Ok(()));
    assert_eq!(walk.spent, 2);
    // Without a ceiling a remaining bound pair is unsupported, a differing
    // unit or presence still a plain mismatch.
    let unbounded = option_walk(&environment, &base, &base);
    assert_eq!(
        unbounded.result,
        Err(SuppliedAdmissionError::UnsupportedHelper {
            family: UnsupportedHelperFamily::NumericDomain,
            locus: link(&unbounded.root, 0),
        })
    );
    assert_eq!(unbounded.spent, 2);
    for other in [bounded(unit_v(), Some(0), Some(100)), open.clone()] {
        let walk = option_walk(&environment, &base, &other);
        assert_eq!(
            walk.result,
            invalid(InvalidMembershipCause::Type, link(&walk.root, 0))
        );
    }
}

// --------------------------------------------------------------- AC-13

/// A bounded source is read as `Quantity<U>`; nothing converts into a bounded
/// type but itself.
#[trace("TC-927", "FR-127-AC-13")]
#[test]
fn bounded_quantity_conversion_follows_the_quantity_row() {
    let table = unit_table();
    let scope = UnitScope::new(&table);
    let to =
        |source: &ValueType, target: &ValueType| admits_equality_conversion(source, target, &scope);
    let sources = [
        bounded(unit_u(), Some(0), Some(10)),
        bounded(unit_u(), None, None),
        bounded(unit_u(), Some(5), Some(3)),
    ];
    for source in &sources {
        assert!(to(source, &ValueType::Quantity(unit_u())));
        for target in [unit_v(), unit_w(), unit_x()] {
            let expected = to(&ValueType::Quantity(unit_u()), &ValueType::Quantity(target));
            assert_eq!(to(source, &ValueType::Quantity(target)), expected);
        }
        assert!(to(source, source));
        // No other row reaches a bounded or map type.
        assert!(!to(&ValueType::Quantity(unit_u()), source));
        assert!(!to(source, &map_of(ValueType::Integer, None)));
        assert!(!to(&map_of(ValueType::Integer, None), source));
        assert!(!to(source, &ValueType::Integer));
    }
    assert!(to(
        &ValueType::Quantity(unit_u()),
        &ValueType::Quantity(unit_v())
    ));
    assert!(!to(
        &ValueType::Quantity(unit_u()),
        &ValueType::Quantity(unit_w())
    ));
    assert!(!to(&sources[0], &ValueType::Quantity(unit_w())));
    assert!(to(&sources[0], &ValueType::Quantity(unit_v())));
    // An unresolved unit admits no conversion.
    assert!(!to(
        &bounded(unit_x(), Some(0), Some(1)),
        &ValueType::Quantity(unit_u())
    ));
    assert!(!to(&sources[0], &ValueType::Quantity(unit_x())));
    // The same unit with different bounds is not the same type.
    assert!(!to(&sources[0], &sources[1]));
    assert!(!to(&sources[1], &sources[0]));
    // The bounds are never consulted: an empty type converts as any other.
    assert!(to(&sources[2], &ValueType::Quantity(unit_v())));
    // A map converts only to itself.
    let map = map_of(ValueType::Boolean, None);
    assert!(to(&map, &map));
    assert!(!to(&map, &map_of(ValueType::Boolean, bound(0, 1))));
}

// --------------------------------------------------------------- AC-14

fn check(
    environment: &TypeEnvironment,
    table: &UnitTable,
    left: EqualityOperand,
    right: EqualityOperand,
) -> Result<quire_semantic_value::declaration::CheckedEquality, IllTypedCause> {
    environment
        .check_equality_in(
            &UnitScope::new(table),
            EqualityOperator::Equal,
            left,
            right,
            &|_| EnumMemberIndex::default(),
        )
        .map_err(|error| error.cause)
}

fn typed(value_type: ValueType) -> EqualityOperand {
    EqualityOperand::typed(value_type)
}

/// A map is never comparable, an equal map pair included, and the refusal
/// precedes unit resolution and schedule selection.
#[trace("TC-927", "FR-127-AC-14")]
#[test]
fn equality_over_a_map_is_operator_ineligible() {
    let table = unit_table();
    let holder = declare(&[map_of(ValueType::Integer, None)]).unwrap();
    let map = map_of(ValueType::Integer, None);
    let boolean_map = map_of(ValueType::Boolean, None);
    let shapes = [
        map.clone(),
        ValueType::option(boolean_map.clone()),
        sequence_of(boolean_map.clone()),
        map_of(map.clone(), None),
        ValueType::Composite(key(100)),
        ValueType::option(ValueType::Composite(key(100))),
    ];
    for shape in &shapes {
        assert_eq!(
            check(&holder, &table, typed(shape.clone()), typed(shape.clone())).err(),
            Some(IllTypedCause::OperatorIneligible),
            "{shape:?}"
        );
        // Either side alone.
        assert_eq!(
            check(
                &holder,
                &table,
                typed(shape.clone()),
                typed(ValueType::Boolean)
            )
            .err(),
            Some(IllTypedCause::OperatorIneligible)
        );
        assert_eq!(
            check(
                &holder,
                &table,
                typed(ValueType::Boolean),
                typed(shape.clone())
            )
            .err(),
            Some(IllTypedCause::OperatorIneligible)
        );
    }
    // A conversion target that holds a map, before the conversion row is read.
    for target in [
        map.clone(),
        ValueType::option(map.clone()),
        ValueType::Composite(key(100)),
    ] {
        assert_eq!(
            check(
                &holder,
                &table,
                EqualityOperand::converted(
                    ValueType::Int(IntegerInterval::spanning(integer(0), integer(1))),
                    target,
                ),
                typed(ValueType::Integer),
            )
            .err(),
            Some(IllTypedCause::OperatorIneligible)
        );
    }
    // Before unit resolution: the other operand names an unresolved unit.
    assert_eq!(
        check(
            &holder,
            &table,
            typed(map.clone()),
            typed(ValueType::Quantity(unit_x()))
        )
        .err(),
        Some(IllTypedCause::OperatorIneligible)
    );
    assert_eq!(
        check(
            &holder,
            &table,
            typed(bounded(unit_x(), None, None)),
            typed(map)
        )
        .err(),
        Some(IllTypedCause::OperatorIneligible)
    );
    // A record without a map is still comparable.
    let plain = environment();
    assert_eq!(
        check(
            &plain,
            &table,
            typed(ValueType::Composite(key(1))),
            typed(ValueType::Composite(key(1)))
        )
        .unwrap()
        .schedule(),
        EqualitySchedule::Plan
    );
}

// --------------------------------------------------------------- AC-15

/// A top-level bounded operand is read as `Quantity<U>`; a nested one keeps
/// its own identity.
#[trace("TC-927", "FR-127-AC-15")]
#[test]
fn equality_reads_a_top_level_bounded_quantity_as_a_quantity() {
    let table = unit_table();
    let environment = environment();
    let b = |min, max| bounded(unit_u(), min, max);
    let schedule = |left: ValueType, right: ValueType| {
        check(&environment, &table, typed(left), typed(right)).map(|checked| checked.schedule())
    };
    let base = b(Some(0), Some(10));
    for other in [
        base.clone(),
        b(Some(5), Some(20)),
        b(Some(11), Some(5)),
        b(None, None),
        ValueType::Quantity(unit_u()),
    ] {
        assert_eq!(
            schedule(base.clone(), other.clone()),
            Ok(EqualitySchedule::Quantity)
        );
        assert_eq!(
            schedule(other, base.clone()),
            Ok(EqualitySchedule::Quantity)
        );
    }
    assert_eq!(
        schedule(base.clone(), ValueType::Quantity(unit_v())),
        Err(IllTypedCause::DistinctUnits)
    );
    assert_eq!(
        schedule(base.clone(), bounded(unit_v(), Some(0), Some(10))),
        Err(IllTypedCause::DistinctUnits)
    );
    assert_eq!(
        schedule(base.clone(), ValueType::Quantity(unit_w())),
        Err(IllTypedCause::IncompatibleDimensions)
    );
    assert_eq!(
        schedule(bounded(unit_w(), None, None), base.clone()),
        Err(IllTypedCause::IncompatibleDimensions)
    );
    assert_eq!(
        schedule(bounded(unit_x(), None, None), bounded(unit_x(), None, None)),
        Err(IllTypedCause::TypeMismatch)
    );
    assert_eq!(
        schedule(base.clone(), bounded(unit_x(), None, None)),
        Err(IllTypedCause::TypeMismatch)
    );
    // A conversion target reads the same way.
    assert_eq!(
        check(
            &environment,
            &table,
            EqualityOperand::converted(base.clone(), ValueType::Quantity(unit_v())),
            typed(ValueType::Quantity(unit_v())),
        )
        .map(|checked| checked.schedule()),
        Ok(EqualitySchedule::Quantity)
    );

    // The bounds play no part in the comparison.
    let checked = check(
        &environment,
        &table,
        typed(base.clone()),
        typed(base.clone()),
    )
    .unwrap();
    let compare = |left: i64, right: i64| {
        checked.evaluate(
            &environment,
            &Value::Quantity(quantity(left, unit_u())),
            &Value::Quantity(quantity(right, unit_u())),
            &mut meter(),
        )
    };
    assert!(matches!(compare(50, 3), Outcome::Completed(false)));
    assert!(matches!(compare(50, 50), Outcome::Completed(true)));
    assert!(matches!(compare(-7, -7), Outcome::Completed(true)));
    assert!(matches!(compare(5, 5), Outcome::Completed(true)));

    // A nested bounded quantity keeps its own identity.
    let nests: [fn(ValueType) -> ValueType; 3] = [ValueType::option, sequence_of, |element| {
        // A record field is reached through its declaration below.
        ValueType::option(sequence_of(element))
    }];
    for nest in nests {
        assert_eq!(
            schedule(nest(base.clone()), nest(base.clone())),
            Ok(EqualitySchedule::Plan)
        );
        for other in [
            ValueType::Quantity(unit_u()),
            b(Some(0), Some(20)),
            bounded(unit_v(), Some(0), Some(10)),
        ] {
            assert_eq!(
                schedule(nest(base.clone()), nest(other.clone())),
                Err(IllTypedCause::TypeMismatch),
                "{other:?}"
            );
        }
    }
    let record_with = |value_type: ValueType| {
        TypeEnvironment::new([record(1, "R", vec![field("q", value_type)])], []).unwrap()
    };
    let holder = record_with(base.clone());
    assert_eq!(
        check(
            &holder,
            &table,
            typed(ValueType::Composite(key(1))),
            typed(ValueType::Composite(key(1)))
        )
        .map(|checked| checked.schedule()),
        Ok(EqualitySchedule::Plan)
    );
}

// --------------------------------------------------------------- AC-16

fn sealed_environment(positions: Vec<ValueType>) -> TypeEnvironment {
    let mut environment = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "Shape",
            CompositeShape::Union(vec![UnionMemberDeclaration::resolved(
                Identifier::new("Only").unwrap(),
                positions,
            )]),
        )],
        [],
    )
    .unwrap();
    environment
        .seal_union_verified(
            key(1),
            key(40),
            vec![UnionMember::from_admitted(
                key(40),
                key(41),
                Identifier::new("Only").unwrap(),
            )],
        )
        .unwrap();
    environment
}

#[trace("TC-927", "FR-127-AC-16")]
#[test]
fn runtime_type_and_same_type_descend_map_links() {
    let sealed = sealed_environment(vec![]);
    let handle = ValueType::Composite(key(1));
    let final_key = ValueType::Composite(key(40));
    // A sealed handle resolves to its final key under a map, keeping the bound.
    for b in [None, bound(0, 4), bound(1, 3)] {
        assert_eq!(
            sealed.runtime_type(&map_of(handle.clone(), b)),
            Some(map_of(final_key.clone(), b))
        );
    }
    assert_eq!(
        sealed.runtime_type(&ValueType::option(map_of(handle.clone(), bound(1, 3)))),
        Some(ValueType::option(map_of(final_key.clone(), bound(1, 3))))
    );
    assert_eq!(
        sealed.runtime_type(&map_of(
            sequence_of(map_of(handle.clone(), bound(2, 3))),
            None
        )),
        Some(map_of(
            sequence_of(map_of(final_key.clone(), bound(2, 3))),
            None
        ))
    );
    // An unsealed union under a map has no runtime type.
    let unsealed = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key(1),
            "Shape",
            CompositeShape::Union(vec![UnionMemberDeclaration::resolved(
                Identifier::new("Only").unwrap(),
                vec![],
            )]),
        )],
        [],
    )
    .unwrap();
    assert_eq!(unsealed.runtime_type(&map_of(handle.clone(), None)), None);
    assert_eq!(
        unsealed.runtime_type(&ValueType::option(map_of(handle.clone(), bound(1, 3)))),
        None
    );
    // Leaves resolve to themselves.
    assert_eq!(
        sealed.runtime_type(&map_of(ValueType::Boolean, bound(0, 1))),
        Some(map_of(ValueType::Boolean, bound(0, 1)))
    );
    assert_eq!(
        sealed.runtime_type(&bounded(unit_u(), Some(0), Some(1))),
        Some(bounded(unit_u(), Some(0), Some(1)))
    );

    // `same_type` joins handles under a map and compares bound and value type.
    assert!(sealed.same_type(
        &map_of(handle.clone(), bound(0, 4)),
        &map_of(final_key.clone(), bound(0, 4))
    ));
    assert!(sealed.same_type(
        &map_of(handle.clone(), None),
        &map_of(final_key.clone(), None)
    ));
    assert!(!sealed.same_type(
        &map_of(handle.clone(), bound(0, 4)),
        &map_of(final_key.clone(), bound(0, 5))
    ));
    assert!(!sealed.same_type(
        &map_of(handle.clone(), bound(0, u64::MAX)),
        &map_of(final_key.clone(), None)
    ));
    assert!(!sealed.same_type(
        &map_of(handle.clone(), None),
        &map_of(ValueType::Boolean, None)
    ));
    assert!(!sealed.same_type(&map_of(handle.clone(), None), &sequence_of(handle.clone())));
    assert!(!sealed.same_type(
        &map_of(ValueType::Boolean, None),
        &ValueType::option(ValueType::Boolean)
    ));

    // A bounded quantity is a leaf compared by unit and both bounds.
    let option_bounded = |min, max| ValueType::option(bounded(unit_u(), min, max));
    let base = option_bounded(Some(0), Some(10));
    assert!(sealed.same_type(&base, &option_bounded(Some(0), Some(10))));
    assert!(!sealed.same_type(&base, &ValueType::option(ValueType::Quantity(unit_u()))));
    assert!(!sealed.same_type(&base, &option_bounded(Some(0), Some(20))));
    assert!(!sealed.same_type(&base, &option_bounded(Some(1), Some(10))));
    assert!(!sealed.same_type(&base, &option_bounded(None, Some(10))));
    assert!(!sealed.same_type(
        &base,
        &ValueType::option(bounded(unit_v(), Some(0), Some(10)))
    ));
    assert!(sealed.same_type(
        &bounded(unit_u(), None, None),
        &bounded(unit_u(), None, None)
    ));
    assert!(!sealed.same_type(
        &bounded(unit_u(), None, None),
        &ValueType::Quantity(unit_u())
    ));
}

// --------------------------------------------------------------- AC-17

fn rational_bound(numerator: i64, unit: UnitId) -> Quantity {
    quantity(numerator, unit)
}

fn quantity_type(min: Option<i64>, max: Option<i64>) -> ValueType {
    ValueType::bounded_quantity(
        BoundedQuantityType::new(
            unit_u(),
            min.map(|n| rational_bound(n, unit_u())),
            max.map(|n| rational_bound(n, unit_u())),
        )
        .unwrap(),
    )
}

struct Bounded {
    result: Result<(), BoundedSuppliedAdmissionError<InvalidMembershipCause>>,
    spent: u64,
    link: MembershipLocus,
    at: MembershipLocus,
}

/// Admit `Option` with retained payload `actual` against `Option<expected>`
/// under magnitude ceiling `limit`, an event ceiling of `events` and `cancel`.
fn bounded_walk(
    expected: &ValueType,
    actual: &ValueType,
    limit: u64,
    events: u64,
    cancel: &Cancel,
) -> Bounded {
    let environment = environment();
    let mut arena = MembershipPathArena::new().unwrap();
    let (at, link) = root_loci(&mut arena, 7);
    let mut budget = SuppliedMembershipBudget::new(events);
    let value = OptionValue::from_admitted(actual.clone(), None);
    let result = environment.admit_supplied_bounded_numeric::<InvalidMembershipCause>(
        &ValueType::option(expected.clone()),
        &value,
        &mut arena,
        at,
        &mut budget,
        cancel,
        limit,
    );
    Bounded {
        result,
        spent: budget.spent(),
        link,
        at,
    }
}

fn mismatch(walk: &Bounded) -> Result<(), BoundedSuppliedAdmissionError<InvalidMembershipCause>> {
    Err(BoundedSuppliedAdmissionError::Admission(
        SuppliedAdmissionError::Invalid {
            cause: InvalidMembershipCause::Type,
            locus: walk.link,
        },
    ))
}

fn oversize(
    endpoint: QuantityBoundEndpoint,
    walk: &Bounded,
) -> Result<(), BoundedSuppliedAdmissionError<InvalidMembershipCause>> {
    Err(BoundedSuppliedAdmissionError::QuantityBoundSize {
        endpoint,
        measured_bits: 9,
        limit_bits: 8,
        locus: walk.link,
    })
}

const FITS: i64 = 255;
const TOO_BIG: i64 = 256;

/// Equal bounds complete; a differing minimum, maximum, presence or unit is a
/// mismatch, and presence or unit measures nothing.
#[trace("TC-927", "FR-127-AC-17")]
#[test]
fn bounded_quantity_link_equal_and_differing_bounds() {
    let cancel = Cancel::new();
    let base = quantity_type(Some(0), Some(100));
    let equal = bounded_walk(&base, &quantity_type(Some(0), Some(100)), 64, 2, &cancel);
    assert_eq!(equal.result, Ok(()));
    assert_eq!(equal.spent, 2);
    let at_minimum = bounded_walk(&base, &quantity_type(Some(1), Some(100)), 64, 2, &cancel);
    assert_eq!(at_minimum.result, mismatch(&at_minimum));
    let at_maximum = bounded_walk(&base, &quantity_type(Some(0), Some(101)), 64, 2, &cancel);
    assert_eq!(at_maximum.result, mismatch(&at_maximum));
    // Fractions are compared exactly.
    let half = |n: i64, d: i64| {
        ValueType::bounded_quantity(
            BoundedQuantityType::new(
                unit_u(),
                Some(Quantity::new(
                    Rational::new(integer(n), integer(d)).unwrap(),
                    unit_u(),
                )),
                None,
            )
            .unwrap(),
        )
    };
    let reduced = bounded_walk(&half(1, 2), &half(2, 4), 64, 2, &cancel);
    assert_eq!(reduced.result, Ok(()));
    let unequal = bounded_walk(&half(1, 2), &half(1, 3), 64, 2, &cancel);
    assert_eq!(unequal.result, mismatch(&unequal));

    // Presence and unit differ: a mismatch with no magnitude measured, even
    // when a bound is above the ceiling.
    let huge = quantity_type(Some(TOO_BIG), Some(TOO_BIG));
    for (expected, actual) in [
        (
            quantity_type(Some(0), Some(1)),
            quantity_type(Some(0), None),
        ),
        (quantity_type(Some(0), None), quantity_type(None, Some(0))),
        (huge.clone(), quantity_type(Some(TOO_BIG), None)),
        (quantity_type(None, Some(TOO_BIG)), huge.clone()),
        (
            huge.clone(),
            ValueType::bounded_quantity(
                BoundedQuantityType::new(
                    unit_v(),
                    Some(quantity(TOO_BIG, unit_v())),
                    Some(quantity(TOO_BIG, unit_v())),
                )
                .unwrap(),
            ),
        ),
        (huge.clone(), ValueType::Quantity(unit_u())),
    ] {
        let walk = bounded_walk(&expected, &actual, 8, 2, &cancel);
        assert_eq!(walk.result, mismatch(&walk), "{expected:?} / {actual:?}");
        assert_eq!(walk.spent, 2);
    }
}

#[trace("TC-927", "FR-127-AC-17")]
#[test]
fn bounded_quantity_link_ceiling_is_exact_and_ordered() {
    let cancel = Cancel::new();
    // A bound of exactly the ceiling completes.
    let fits = quantity_type(Some(FITS), Some(FITS));
    let walk = bounded_walk(&fits, &fits, 8, 2, &cancel);
    assert_eq!(walk.result, Ok(()));
    let negative = quantity_type(Some(-FITS), Some(FITS));
    assert_eq!(
        bounded_walk(&negative, &negative, 8, 2, &cancel).result,
        Ok(())
    );
    // One bit over, on each endpoint alone.
    let cases = [
        (
            quantity_type(Some(TOO_BIG), Some(1)),
            quantity_type(Some(1), Some(1)),
            QuantityBoundEndpoint::ExpectedMinimum,
        ),
        (
            quantity_type(Some(1), Some(TOO_BIG)),
            quantity_type(Some(1), Some(1)),
            QuantityBoundEndpoint::ExpectedMaximum,
        ),
        (
            quantity_type(Some(1), Some(1)),
            quantity_type(Some(TOO_BIG), Some(1)),
            QuantityBoundEndpoint::ActualMinimum,
        ),
        (
            quantity_type(Some(1), Some(1)),
            quantity_type(Some(1), Some(TOO_BIG)),
            QuantityBoundEndpoint::ActualMaximum,
        ),
        // Several over: the stable order names the first.
        (
            quantity_type(Some(TOO_BIG), Some(TOO_BIG)),
            quantity_type(Some(TOO_BIG), Some(TOO_BIG)),
            QuantityBoundEndpoint::ExpectedMinimum,
        ),
        (
            quantity_type(Some(1), Some(TOO_BIG)),
            quantity_type(Some(TOO_BIG), Some(TOO_BIG)),
            QuantityBoundEndpoint::ExpectedMaximum,
        ),
        (
            quantity_type(Some(1), Some(1)),
            quantity_type(Some(TOO_BIG), Some(TOO_BIG)),
            QuantityBoundEndpoint::ActualMinimum,
        ),
        (
            quantity_type(Some(-TOO_BIG), None),
            quantity_type(Some(1), None),
            QuantityBoundEndpoint::ExpectedMinimum,
        ),
    ];
    for (expected, actual, endpoint) in cases {
        let walk = bounded_walk(&expected, &actual, 8, 2, &cancel);
        assert_eq!(
            walk.result,
            oversize(endpoint, &walk),
            "{expected:?} / {actual:?}"
        );
        assert_eq!(walk.spent, 2, "the event is charged once");
    }
    // The denominator counts: a part above the ceiling is oversized.
    let fraction = |denominator: i64| {
        ValueType::bounded_quantity(
            BoundedQuantityType::new(
                unit_u(),
                Some(Quantity::new(
                    Rational::new(integer(1), integer(denominator)).unwrap(),
                    unit_u(),
                )),
                None,
            )
            .unwrap(),
        )
    };
    let walk = bounded_walk(&fraction(TOO_BIG + 1), &fraction(3), 8, 2, &cancel);
    assert_eq!(
        walk.result,
        oversize(QuantityBoundEndpoint::ExpectedMinimum, &walk)
    );
    assert_eq!(
        bounded_walk(&fraction(FITS), &fraction(FITS), 8, 2, &cancel).result,
        Ok(())
    );
}

#[trace("TC-927", "FR-127-AC-17")]
#[test]
fn bounded_quantity_link_without_a_ceiling_needs_one_only_for_a_bound_pair() {
    let environment = environment();
    let walk =
        |expected: &ValueType, actual: &ValueType| option_walk(&environment, expected, actual);
    let present = quantity_type(Some(0), Some(100));
    let unsupported = walk(&present, &present);
    assert_eq!(
        unsupported.result,
        Err(SuppliedAdmissionError::UnsupportedHelper {
            family: UnsupportedHelperFamily::NumericDomain,
            locus: link(&unsupported.root, 0),
        })
    );
    // After the one TypeLinkComparison event, before any measurement.
    assert_eq!(unsupported.spent, 2);
    let one_side = quantity_type(Some(0), None);
    let unsupported = walk(&one_side, &one_side);
    assert!(matches!(
        unsupported.result,
        Err(SuppliedAdmissionError::UnsupportedHelper { .. })
    ));
    // A mismatch of presence or unit is decided without a ceiling.
    let differing = walk(&present, &one_side);
    assert_eq!(
        differing.result,
        invalid(InvalidMembershipCause::Type, link(&differing.root, 0))
    );
    // No bounds at all: equal units are the same type.
    let open = quantity_type(None, None);
    let same = walk(&open, &open);
    assert_eq!(same.result, Ok(()));
    assert_eq!(same.spent, 2);
}

#[trace("TC-927", "FR-127-AC-17")]
#[test]
fn bounded_quantity_link_event_denial_and_cancellation_keep_their_causes() {
    let huge = quantity_type(Some(TOO_BIG), Some(TOO_BIG));
    // A denied event at N - 1 measures nothing and the spend stays N - 1.
    let denied = bounded_walk(&huge, &huge, 8, 1, &Cancel::new());
    assert!(matches!(
        denied.result,
        Err(BoundedSuppliedAdmissionError::Admission(
            SuppliedAdmissionError::Limit {
                ceiling: 1,
                spent: 1,
                requested: 1,
                ..
            }
        ))
    ));
    assert_eq!(denied.spent, 1);
    let denied = bounded_walk(&huge, &huge, 8, 0, &Cancel::new());
    assert!(matches!(
        denied.result,
        Err(BoundedSuppliedAdmissionError::Admission(
            SuppliedAdmissionError::Limit { spent: 0, .. }
        ))
    ));
    // N events complete.
    let fits = quantity_type(Some(1), Some(2));
    let walk = bounded_walk(&fits, &fits, 8, 2, &Cancel::new());
    assert_eq!(walk.result, Ok(()));
    assert_eq!(walk.spent, 2);

    // Pre-cancellation beats an oversized bound and keeps the original cause.
    let cancel = Cancel::new();
    cancel.cancel(CancelCause::Deadline);
    let walk = bounded_walk(&huge, &huge, 8, 2, &cancel);
    assert_eq!(
        walk.result,
        Err(BoundedSuppliedAdmissionError::Admission(
            SuppliedAdmissionError::Cancelled {
                cause: CancelCause::Deadline,
                spent: 0,
                event: quire_semantic_value::supplied_membership::MembershipEvent::ValueVisit,
                locus: walk.at,
            }
        ))
    );
    assert_eq!(walk.spent, 0);
}

/// Count the polls of one run, then cancel at each ordinal in turn: every
/// stop keeps the original cause, and the run polls once before measuring,
/// once before each present bound pair and once before publishing.
#[trace("TC-927", "FR-127-AC-17")]
#[test]
fn bounded_quantity_link_polls_cancellation_before_each_step() {
    let run = |expected: &ValueType, actual: &ValueType, cancel_at: Option<usize>| {
        let polls = Arc::new(AtomicUsize::new(0));
        let trigger = Arc::new(Mutex::new(None::<Cancel>));
        let (observed_polls, observed_trigger) = (Arc::clone(&polls), Arc::clone(&trigger));
        let cancel = Cancel::observing(move || {
            let ordinal = observed_polls.fetch_add(1, Ordering::SeqCst) + 1;
            if Some(ordinal) == cancel_at {
                if let Some(handle) = observed_trigger.lock().unwrap().take() {
                    handle.cancel(CancelCause::Deadline);
                }
            }
        });
        *trigger.lock().unwrap() = Some(cancel.clone());
        let walk = bounded_walk(expected, actual, 64, 2, &cancel);
        (walk, polls.load(Ordering::SeqCst))
    };
    for (expected, actual, kernel_polls) in [
        // Poll, minimum pair, maximum pair, publish.
        (
            quantity_type(Some(0), Some(100)),
            quantity_type(Some(0), Some(100)),
            4,
        ),
        (
            quantity_type(Some(0), None),
            quantity_type(Some(0), None),
            3,
        ),
        (
            quantity_type(None, Some(9)),
            quantity_type(None, Some(9)),
            3,
        ),
    ] {
        let (walk, polls) = run(&expected, &actual, None);
        assert_eq!(walk.result, Ok(()));
        // The ValueVisit and TypeLinkComparison charges and the after-helper
        // boundary poll around the kernel steps.
        assert_eq!(polls, 3 + kernel_polls, "{expected:?}");
        for ordinal in 1..=polls {
            let (walk, _) = run(&expected, &actual, Some(ordinal));
            assert!(
                matches!(
                    walk.result,
                    Err(BoundedSuppliedAdmissionError::Admission(
                        SuppliedAdmissionError::Cancelled {
                            cause: CancelCause::Deadline,
                            ..
                        }
                    ))
                ),
                "cancel at poll {ordinal} of {polls}: {:?}",
                walk.result
            );
        }
    }
    // A mismatch at the minimum stops before the maximum pair is polled.
    let (walk, polls) = run(
        &quantity_type(Some(0), Some(100)),
        &quantity_type(Some(1), Some(100)),
        None,
    );
    assert_eq!(walk.result, mismatch(&walk));
    assert_eq!(polls, 3 + 3);
}

/// An oversized quantity bound names its side and bound on the wire.
#[trace("TC-927", "FR-127-AC-17")]
#[test]
fn quantity_bound_endpoints_have_distinct_wire_spellings() {
    use quire_semantic_value::supplied_stop::SuppliedNumericOperand;
    let spellings = [
        (
            QuantityBoundEndpoint::ExpectedMinimum,
            "type-link-expected-minimum",
        ),
        (
            QuantityBoundEndpoint::ExpectedMaximum,
            "type-link-expected-maximum",
        ),
        (
            QuantityBoundEndpoint::ActualMinimum,
            "type-link-actual-minimum",
        ),
        (
            QuantityBoundEndpoint::ActualMaximum,
            "type-link-actual-maximum",
        ),
    ];
    for (endpoint, spelling) in spellings {
        assert_eq!(
            SuppliedNumericOperand::QuantityBound(endpoint).as_str(),
            spelling
        );
    }
}

// ------------------------------------------------------- support checks

/// The fixture's expectations about the kernel the tests lean on.
#[test]
fn fixture_variants_are_distinct() {
    assert_ne!(variant(1), variant(2));
    assert_ne!(unit_u(), unit_v());
    assert_ne!(bounded(unit_u(), None, None), ValueType::Quantity(unit_u()));
    assert_eq!(
        map_of(ValueType::Boolean, None),
        map_of(ValueType::Boolean, None)
    );
}
