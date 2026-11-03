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
    original.left = EqualityOperand::typed(ValueType::Text(ty));
    original.right = original.left.clone();
    original.schedule = EqualitySchedule::Text;
    let decision =
        mint_admitted_equality_decision(ReaderSeal, original, &mut meter(10_000)).unwrap();
    assert_eq!(decision.left().source(), &ValueType::Text(ty));
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
        assert_eq!(retained.declaration_members(), cases);
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

#[trace("FR-131-AC-1")]
#[test]
fn enum_materialization_refuses_ambiguous_same_key_declarations() {
    let key = common::node(800);
    let variant = VariantId::new(key.clone(), "A").unwrap();
    let mut original = descriptor();
    original.enum_declarations = vec![
        EqualityEnumDeclarationDescriptor {
            key: key.clone(),
            ordered: true,
            members: vec!["A".into(), "B".into()],
        },
        EqualityEnumDeclarationDescriptor {
            key: key.clone(),
            ordered: true,
            members: vec!["A".into(), "C".into()],
        },
    ];
    original.enum_members.push(EqualityEnumMemberDescriptor {
        variant: variant.clone(),
        declaration: key,
        member: common::node(801),
        ordered: true,
        position: 0,
        case: "A".into(),
    });
    assert_eq!(
        mint_admitted_equality_decision(ReaderSeal, original.clone(), &mut meter(10_000)),
        Err(EqualityMaterializationError::IdentityForm {
            locus: EqualityMaterializationLocus::EnumMember { row: 0 }
        })
    );

    original.enum_declarations[1].members = vec!["A".into(), "B".into()];
    let decision = mint_admitted_equality_decision(ReaderSeal, original, &mut meter(10_000))
        .expect("identical declarations are unambiguous");
    assert_eq!(
        decision
            .resolved_enum_members()
            .resolve(variant)
            .unwrap()
            .declaration_members(),
        ["A", "B"]
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
            id: source_id,
            form: EqualityUnitForm::Declared {
                dimension_node: dimension.clone(),
                dimension_terms: vec![(dimension.clone(), Integer::one())],
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
            id: root_id,
            form: EqualityUnitForm::Declared {
                dimension_node: dimension.clone(),
                dimension_terms: vec![(dimension, Integer::one())],
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
    assert_eq!(unit.key(), rows[0].id.declared_key().unwrap().clone());
    let quire_semantic_value::quantity::QuantityUnit::Declared(root_unit) =
        decision.resolved_units().get(rows[1].id.clone()).unwrap()
    else {
        panic!("root declared row");
    };
    assert_eq!(root_unit.key(), rows[1].id.declared_key().unwrap().clone());
    assert_eq!(root_unit.root(), root_unit.key());
    assert!(root_unit.path().is_empty());
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
    let roots = vec![(root.clone(), Integer::from(2_i64))];
    let id = UnitId::compound(roots.clone());
    let mut original = descriptor();
    original.schedule = EqualitySchedule::Quantity;
    original.left = EqualityOperand::typed(ValueType::Quantity(id.clone()));
    original.right = original.left.clone();
    original.units.push(EqualityUnitDescriptor {
        id: id.clone(),
        form: EqualityUnitForm::Compound {
            dimension_terms: vec![(dimension.clone(), Integer::from(2_i64))],
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
        vec![(dimension, Integer::from(2_i64))]
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
    assert_eq!(
        locus,
        EqualityMaterializationLocus::DimensionTerm { row: 1, term: 0 }
    );
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

#[trace("IR-1223")]
#[test]
fn each_unpaid_affine_event_keeps_its_original_typed_locus() {
    for (occurrence, edge, step) in [
        (1, 0, AffineStep::Scale),
        (2, 0, AffineStep::OffsetScale),
        (3, 0, AffineStep::OffsetAdd),
        (4, 1, AffineStep::Scale),
        (5, 1, AffineStep::OffsetScale),
        (6, 1, AffineStep::OffsetAdd),
    ] {
        let mut denied = meter(100_000).with_injected_denial(quire_exact::InjectedDenial {
            point: quire_exact::ChargePoint::UnitRationalArithmetic,
            occurrence: std::num::NonZeroU64::new(occurrence).unwrap(),
        });
        let error = mint_admitted_equality_decision(ReaderSeal, affine_descriptor(), &mut denied)
            .unwrap_err();
        let EqualityMaterializationError::Resource { cause, locus } = error else {
            panic!("typed original denial");
        };
        assert_eq!(
            cause.charge_point,
            quire_exact::ChargePoint::UnitRationalArithmetic
        );
        assert_eq!(
            locus,
            EqualityMaterializationLocus::Edge { row: 0, edge, step }
        );
    }
}

#[trace("IR-1223")]
#[test]
fn a_long_retained_path_materializes_on_a_small_stack_without_a_depth_cap() {
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(|| {
            let count = 2_048_u64;
            let root = common::node(10_000 + count);
            let source = common::node(10_000);
            let id = UnitId::declared(source.clone());
            let mut original = descriptor();
            original.schedule = EqualitySchedule::Quantity;
            original.left = EqualityOperand::typed(ValueType::Quantity(id.clone()));
            original.right = original.left.clone();
            let path: Vec<_> = (1..=count)
                .map(|n| EqualityUnitEdgeDescriptor {
                    target: common::node(10_000 + n),
                    scale: rational(1),
                    offset: rational(0),
                })
                .collect();
            original.units.push(EqualityUnitDescriptor {
                id: id.clone(),
                form: EqualityUnitForm::Declared {
                    dimension_node: common::node(9_998),
                    dimension_terms: vec![(common::node(9_998), Integer::one())],
                    path: path.clone(),
                },
            });
            let decision =
                mint_admitted_equality_decision(ReaderSeal, original, &mut meter(1_000_000))
                    .unwrap();
            let EqualityUnitForm::Declared { path: retained, .. } = &decision.units()[0].form
            else {
                panic!("declared");
            };
            assert_eq!(retained, &path);
            let quire_semantic_value::quantity::QuantityUnit::Declared(native) =
                decision.resolved_units().get(id).unwrap()
            else {
                panic!("declared");
            };
            assert_eq!(native.root(), root);
            assert_eq!(native.canonical().scale(), &rational(1));
            assert_eq!(native.canonical().offset(), &rational(0));
            assert_eq!(native.path().len(), usize::try_from(count).unwrap());
        })
        .unwrap()
        .join()
        .unwrap();
}

#[trace("IR-1223")]
#[test]
fn enum_case_copy_respects_the_callers_byte_provision_without_normalizing() {
    let mut limits = *meter(10_000).limits();
    limits.text_input_bytes = 2;
    let declaration = common::node(700);
    let case = "e\u{301}";
    let variant = VariantId::new(declaration.clone(), case).unwrap();
    let mut original = descriptor();
    original.schedule = EqualitySchedule::Enum;
    original.left =
        EqualityOperand::typed(ValueType::Enum(EnumShape::new(false, [variant.clone()])));
    original.right = original.left.clone();
    original
        .enum_declarations
        .push(EqualityEnumDeclarationDescriptor {
            key: declaration.clone(),
            ordered: false,
            members: vec![case.into()],
        });
    original.enum_members.push(EqualityEnumMemberDescriptor {
        variant,
        declaration,
        member: common::node(701),
        ordered: false,
        position: 0,
        case: case.into(),
    });
    let error =
        mint_admitted_equality_decision(ReaderSeal, original.clone(), &mut Meter::new(limits))
            .unwrap_err();
    let EqualityMaterializationError::Resource { cause, locus } = error else {
        panic!("typed original case-copy refusal");
    };
    assert_eq!(cause.limit_kind, LimitKind::TextInputBytes);
    assert_eq!(cause.limit, 2);
    assert_eq!(cause.next_charge, Integer::from(3_usize));
    assert_eq!(locus, EqualityMaterializationLocus::EnumMember { row: 0 });
    limits.text_input_bytes = 3;
    let decision =
        mint_admitted_equality_decision(ReaderSeal, original, &mut Meter::new(limits)).unwrap();
    assert_eq!(decision.enum_members()[0].case.as_ref(), case);
    assert_eq!(
        decision
            .resolved_enum_members()
            .resolve(decision.enum_members()[0].variant.clone())
            .unwrap()
            .case(),
        case
    );
}

#[trace("IR-1223")]
#[test]
fn materialization_codes_have_a_closed_round_trip_catalog() {
    use quire_semantic_value::reader_admission::EqualityMaterializationCode as Code;
    assert_eq!(
        Code::ALL.map(Code::as_str),
        [
            "resource",
            "arithmetic",
            "allocation",
            "capacity",
            "identity_form"
        ]
    );
    for code in Code::ALL {
        assert_eq!(Code::from_code(code.as_str()), Some(code));
    }
    for unknown in ["", "RESOURCE", "unknown", "capacity "] {
        assert_eq!(Code::from_code(unknown), None);
    }
}

#[trace("IR-1223")]
#[test]
fn a_mismatched_unit_identity_arm_is_a_typed_materialization_failure() {
    use quire_semantic_value::reader_admission::EqualityMaterializationCode;
    for compound_id in [false, true] {
        let mut original = descriptor();
        original.units.push(EqualityUnitDescriptor {
            id: if compound_id {
                UnitId::compound(vec![])
            } else {
                UnitId::declared(common::node(700))
            },
            form: if compound_id {
                EqualityUnitForm::Declared {
                    dimension_node: common::node(701),
                    dimension_terms: vec![],
                    path: vec![],
                }
            } else {
                EqualityUnitForm::Compound {
                    dimension_terms: vec![],
                }
            },
        });
        let error =
            mint_admitted_equality_decision(ReaderSeal, original, &mut meter(100)).unwrap_err();
        assert_eq!(error.code(), EqualityMaterializationCode::IdentityForm);
        assert_eq!(
            error,
            EqualityMaterializationError::IdentityForm {
                locus: EqualityMaterializationLocus::Unit { row: 0 },
            }
        );
    }
}
