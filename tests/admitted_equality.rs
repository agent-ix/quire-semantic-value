// SPDX-License-Identifier: AGPL-3.0-or-later
//! External reader/consumer fixtures; these supply already-admitted witnesses.

mod common;

use ix_trace_rs::trace;
use quire_exact::{
    EnumMember, EnumShape, Integer, LimitKind, Location, Meter, Origin, Outcome, Quantity,
    Rational, Role, ScalarLimits, TextPayload, TextProfile, TextType, UnitId, Value, ValueType,
    VariantId,
};
use quire_semantic_value::declaration::{
    CheckedEquality, CompositeDeclaration, CompositeShape, EqualityDecisionDescriptor,
    EqualityEnumDeclarationDescriptor, EqualityEnumMemberDescriptor, EqualityOperand,
    EqualityOperator, EqualitySchedule, EqualityUnitDescriptor, EqualityUnitEdgeDescriptor,
    EqualityUnitForm, TypeEnvironment,
};
use quire_semantic_value::enumeration::EnumMemberIndex;
use quire_semantic_value::reader_admission::{
    mint_admitted_equality_decision, AffineStep, EqualityMaterializationError,
    EqualityMaterializationLocus, ReaderSeal,
};

fn meter(work: u64) -> Meter {
    Meter::new(ScalarLimits {
        integer_bits: 1_000_000,
        decimal_digits: 1_000_000,
        scale_expansion: 1_000_000,
        text_input_bytes: 1_000_000,
        text_scalars: 1_000_000,
        normalized_scalars: 1_000_000,
        unit_edges: 1_000_000,
        value_occurrences: 1_000_000,
        work_units: work,
        result_units: 1_000_000,
    })
}
fn rational(n: i64) -> Rational {
    Rational::from_integer(Integer::from(n))
}
fn descriptor() -> EqualityDecisionDescriptor {
    EqualityDecisionDescriptor {
        location: Location::new(common::node(99), Origin::new(Role::new("claim"), 7)),
        operator: EqualityOperator::Equal,
        left: EqualityOperand::typed(ValueType::Boolean),
        right: EqualityOperand::typed(ValueType::Boolean),
        schedule: EqualitySchedule::Plan,
        units: vec![],
        enum_declarations: vec![],
        enum_members: vec![],
    }
}

#[trace("IR-1223")]
#[test]
fn plan_adoption_keeps_provenance_operator_and_source_order() {
    let mut original = descriptor();
    original.operator = EqualityOperator::NotEqual;
    let decision =
        mint_admitted_equality_decision(ReaderSeal, original.clone(), &mut meter(10_000)).unwrap();
    assert_eq!(decision.application(), original.location.node());
    assert_eq!(decision.occurrence().role().as_str(), "claim");
    assert_eq!(decision.occurrence().ordinal(), 7);
    assert_eq!(decision.location(), &original.location);
    assert_eq!(decision.operator(), EqualityOperator::NotEqual);
    assert_eq!(decision.left().source(), &ValueType::Boolean);
    assert_eq!(decision.left().target(), None);
    assert_eq!(decision.right(), &original.right);
    assert_eq!(decision.schedule(), EqualitySchedule::Plan);
    assert!(decision.units().is_empty());
    let equality = CheckedEquality::from_admitted_decision(&decision);
    assert_eq!(
        equality.evaluate(
            &TypeEnvironment::default(),
            &Value::Boolean(false),
            &Value::Boolean(true),
            &mut meter(10_000)
        ),
        Outcome::Completed(true)
    );
    original.location = Location::new(common::node(100), decision.occurrence().clone());
    let other = mint_admitted_equality_decision(ReaderSeal, original, &mut meter(10_000)).unwrap();
    assert_ne!(
        decision, other,
        "equal semantics at distinct occurrences retain distinct provenance"
    );
    assert_eq!(equality, CheckedEquality::from_admitted_decision(&other));
}

#[trace("IR-1223")]
#[test]
fn an_admitted_composite_needs_no_rechecking_environment() {
    let key = common::node(200);
    let environment = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key.clone(),
            "R",
            CompositeShape::Record(vec![]),
        )],
        [],
    )
    .unwrap();
    let operand = EqualityOperand::typed(ValueType::Composite(key.clone()));
    let oracle = environment
        .check_equality(
            EqualityOperator::Equal,
            operand.clone(),
            operand.clone(),
            &EnumMemberIndex::default(),
        )
        .unwrap();
    assert!(TypeEnvironment::default()
        .check_equality(
            EqualityOperator::Equal,
            operand.clone(),
            operand.clone(),
            &EnumMemberIndex::default()
        )
        .is_err());
    let mut original = descriptor();
    original.left = operand.clone();
    original.right = operand;
    let decision =
        mint_admitted_equality_decision(ReaderSeal, original, &mut meter(10_000)).unwrap();
    let adopted = CheckedEquality::from_admitted_decision(&decision);
    assert_eq!(
        adopted, oracle,
        "adoption succeeds without having the checking environment"
    );
    let value = environment.record(key, vec![]).unwrap();
    assert_eq!(
        adopted.evaluate(&environment, &value, &value, &mut meter(10_000)),
        Outcome::Completed(true)
    );
}

fn text(value: &str, ty: &TextType) -> Value {
    match quire_exact::admit_text(
        &TextPayload::from_utf8(value.as_bytes()).unwrap(),
        ty,
        &mut meter(10_000),
    ) {
        Outcome::Completed(value) => Value::Text(value),
        other => panic!("fixture text must admit: {other:?}"),
    }
}

#[trace("IR-1223")]
#[test]
fn text_schedule_retains_the_original_exact_type_and_normalization_profile() {
    let ty = TextType::new(0, 10, TextProfile::Nfc).unwrap();
    let mut original = descriptor();
    original.left = EqualityOperand::typed(ValueType::Text(ty.clone()));
    original.right = original.left.clone();
    original.schedule = EqualitySchedule::Text;
    let decision =
        mint_admitted_equality_decision(ReaderSeal, original, &mut meter(10_000)).unwrap();
    assert_eq!(decision.left().source(), &ValueType::Text(ty.clone()));
    assert_eq!(decision.schedule(), EqualitySchedule::Text);
    assert_eq!(
        CheckedEquality::from_admitted_decision(&decision).evaluate(
            &TypeEnvironment::default(),
            &text("é", &ty),
            &text("e\u{301}", &ty),
            &mut meter(10_000)
        ),
        Outcome::Completed(true)
    );
}

#[trace("IR-1223")]
#[test]
fn enum_materialization_preserves_utf8_cases_original_ids_positions_and_filter() {
    let declaration = common::node(300);
    let cases = ["e\u{301}", "not an identifier!", "é"];
    let variants: Vec<_> = cases
        .iter()
        .map(|case| VariantId::new(declaration.clone(), *case).unwrap())
        .collect();
    let mut original = descriptor();
    original.schedule = EqualitySchedule::Enum;
    original.left =
        EqualityOperand::typed(ValueType::Enum(EnumShape::new(false, variants.clone())));
    original.right = original.left.clone();
    original
        .enum_declarations
        .push(EqualityEnumDeclarationDescriptor {
            key: declaration.clone(),
            ordered: false,
            members: cases.iter().map(|case| (*case).to_owned()).collect(),
        });
    original.enum_members = variants
        .iter()
        .enumerate()
        .map(|(position, variant)| EqualityEnumMemberDescriptor {
            variant: variant.clone(),
            declaration: declaration.clone(),
            member: common::node(310 + u64::try_from(position).unwrap()),
            ordered: false,
            position,
            case: cases[position].into(),
        })
        .collect();
    let expected = original.enum_members.clone();
    let decision =
        mint_admitted_equality_decision(ReaderSeal, original, &mut meter(10_000)).unwrap();
    assert_eq!(decision.enum_members(), expected);
    assert_eq!(decision.enum_declarations()[0].members(), cases);
    for original in &expected {
        let retained = decision
            .resolved_enum_members()
            .resolve(original.variant.clone())
            .unwrap();
        assert_eq!(retained.declaration(), original.declaration);
        assert_eq!(retained.member(), original.member);
        assert_eq!(retained.position(), original.position);
        assert_eq!(retained.case(), original.case.as_ref());
        assert_eq!(retained.is_ordered(), original.ordered);
    }
    let foreign = VariantId::new(common::node(999), "foreign").unwrap();
    assert!(decision.resolved_enum_members().resolve(foreign).is_none());
    let equality = CheckedEquality::from_admitted_decision(&decision);
    let first = Value::Enum(EnumMember::new(variants[0].clone(), 0));
    let third = Value::Enum(EnumMember::new(variants[2].clone(), 2));
    assert_eq!(
        equality.evaluate(
            &TypeEnvironment::default(),
            &first,
            &first,
            &mut meter(10_000)
        ),
        Outcome::Completed(true)
    );
    assert_eq!(
        equality.evaluate(
            &TypeEnvironment::default(),
            &first,
            &third,
            &mut meter(10_000)
        ),
        Outcome::Completed(false)
    );
}

fn affine_descriptor() -> EqualityDecisionDescriptor {
    let dimension = common::node(400);
    let source = common::node(401);
    let intermediate = common::node(402);
    let root = common::node(403);
    let source_id = UnitId::declared(source.clone());
    let root_id = UnitId::declared(root.clone());
    let mut original = descriptor();
    original.schedule = EqualitySchedule::Quantity;
    original.left = EqualityOperand::converted(
        ValueType::Quantity(source_id.clone()),
        ValueType::Quantity(root_id.clone()),
    );
    original.right = EqualityOperand::typed(ValueType::Quantity(root_id.clone()));
    original.units = vec![
        EqualityUnitDescriptor {
            quantity_type: common::node(410),
            id: source_id,
            form: EqualityUnitForm::Declared {
                key: source,
                dimension_node: dimension.clone(),
                dimension_terms: vec![(dimension.clone(), Integer::one())],
                root: root.clone(),
                path: vec![
                    EqualityUnitEdgeDescriptor {
                        target: intermediate,
                        scale: rational(2),
                        offset: rational(3),
                    },
                    EqualityUnitEdgeDescriptor {
                        target: root.clone(),
                        scale: rational(5),
                        offset: rational(7),
                    },
                ],
            },
        },
        EqualityUnitDescriptor {
            quantity_type: common::node(411),
            id: root_id,
            form: EqualityUnitForm::Declared {
                key: root.clone(),
                dimension_node: dimension.clone(),
                dimension_terms: vec![(dimension, Integer::one())],
                root,
                path: vec![],
            },
        },
    ];
    original
}

#[trace("IR-1223")]
#[test]
fn selected_affine_path_materializes_canonical_without_reselecting_ids_or_conversion() {
    let original = affine_descriptor();
    let rows = original.units.clone();
    let decision =
        mint_admitted_equality_decision(ReaderSeal, original, &mut meter(100_000)).unwrap();
    assert_eq!(decision.units(), rows);
    let native = decision.resolved_units().get(rows[0].id.clone()).unwrap();
    let quire_semantic_value::quantity::QuantityUnit::Declared(unit) = native else {
        panic!("declared row");
    };
    assert_eq!(unit.canonical().scale(), &rational(10));
    assert_eq!(unit.canonical().offset(), &rational(22));
    assert_eq!(unit.root(), common::node(403));
    assert_eq!(unit.path().len(), 2);
    assert_eq!(
        decision.left().target(),
        Some(&ValueType::Quantity(rows[1].id.clone()))
    );
    let environment = TypeEnvironment::default().with_units(decision.resolved_units().clone());
    let left = Value::Quantity(Quantity::new(rational(1), rows[0].id.clone()));
    let right = Value::Quantity(Quantity::new(rational(32), rows[1].id.clone()));
    assert_eq!(
        CheckedEquality::from_admitted_decision(&decision).evaluate(
            &environment,
            &left,
            &right,
            &mut meter(100_000)
        ),
        Outcome::Completed(true)
    );
}

#[trace("IR-1223")]
#[test]
fn compound_descriptor_keeps_original_identity_dimensions_and_root_powers() {
    let root = common::node(500);
    let dimension = common::node(501);
    let roots = vec![(root.clone(), Integer::from(2))];
    let id = UnitId::compound(roots.clone());
    let mut original = descriptor();
    original.schedule = EqualitySchedule::Quantity;
    original.left = EqualityOperand::typed(ValueType::Quantity(id.clone()));
    original.right = original.left.clone();
    original.units.push(EqualityUnitDescriptor {
        quantity_type: common::node(502),
        id: id.clone(),
        form: EqualityUnitForm::Compound {
            dimension_terms: vec![(dimension.clone(), Integer::from(2))],
            terms: roots.clone(),
        },
    });
    let decision =
        mint_admitted_equality_decision(ReaderSeal, original, &mut meter(10_000)).unwrap();
    assert_eq!(decision.units()[0].id, id);
    let quire_semantic_value::quantity::QuantityUnit::Compound(native) =
        decision.resolved_units().get(id.clone()).unwrap()
    else {
        panic!("compound row");
    };
    assert_eq!(
        native
            .terms()
            .map(|(key, value)| (key, value.clone()))
            .collect::<Vec<_>>(),
        roots
    );
    assert_eq!(
        native
            .dimension()
            .exponents()
            .map(|(key, value)| (key, value.clone()))
            .collect::<Vec<_>>(),
        vec![(dimension, Integer::from(2))]
    );
    let value = Value::Quantity(Quantity::new(rational(3), id));
    let environment = TypeEnvironment::default().with_units(decision.resolved_units().clone());
    assert_eq!(
        CheckedEquality::from_admitted_decision(&decision).evaluate(
            &environment,
            &value,
            &value,
            &mut meter(10_000)
        ),
        Outcome::Completed(true)
    );
}

#[trace("IR-1223")]
#[test]
fn materialization_shares_remaining_work_and_returns_no_partial_value_at_n_minus_one() {
    let original = affine_descriptor();
    let mut complete = meter(100_000);
    complete
        .charge(quire_exact::Charge::new(
            quire_exact::ChargePoint::UnitIdentityRead,
        ))
        .unwrap();
    let prefix = complete.consumed(LimitKind::WorkUnits);
    mint_admitted_equality_decision(ReaderSeal, original.clone(), &mut complete).unwrap();
    let total = complete.consumed(LimitKind::WorkUnits);
    assert!(total > prefix);
    let mut exact = meter(total);
    exact
        .charge(quire_exact::Charge::new(
            quire_exact::ChargePoint::UnitIdentityRead,
        ))
        .unwrap();
    assert!(mint_admitted_equality_decision(ReaderSeal, original.clone(), &mut exact).is_ok());
    assert_eq!(exact.consumed(LimitKind::WorkUnits), total);
    let mut short = meter(total - 1);
    short
        .charge(quire_exact::Charge::new(
            quire_exact::ChargePoint::UnitIdentityRead,
        ))
        .unwrap();
    let error = mint_admitted_equality_decision(ReaderSeal, original, &mut short).unwrap_err();
    let EqualityMaterializationError::Resource { cause, locus } = error else {
        panic!("typed resource failure");
    };
    assert_eq!(cause.limit_kind, LimitKind::WorkUnits);
    assert_eq!(cause.limit, total - 1);
    assert!(matches!(
        locus,
        EqualityMaterializationLocus::Unit { .. } | EqualityMaterializationLocus::Edge { .. }
    ));
}

#[trace("IR-1223")]
#[test]
fn rational_growth_refuses_before_the_first_unpaid_fold_operation() {
    let mut limits = *meter(100_000).limits();
    limits.integer_bits = 2;
    let mut bounded = Meter::new(limits);
    let error =
        mint_admitted_equality_decision(ReaderSeal, affine_descriptor(), &mut bounded).unwrap_err();
    let EqualityMaterializationError::Resource { cause, locus } = error else {
        panic!("typed rational growth refusal");
    };
    assert_eq!(cause.limit_kind, LimitKind::IntegerBits);
    assert_eq!(
        cause.charge_point,
        quire_exact::ChargePoint::UnitRationalArithmetic
    );
    assert_eq!(
        locus,
        EqualityMaterializationLocus::Edge {
            row: 0,
            edge: 0,
            step: AffineStep::Scale
        }
    );
}
