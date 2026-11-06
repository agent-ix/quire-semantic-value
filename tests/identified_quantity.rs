// SPDX-License-Identifier: AGPL-3.0-or-later
//! `Conversion::identified_quantity` through the public `quantity` API.
#![allow(
    clippy::disallowed_methods,
    reason = "the fixtures supply checked unit identities without minting them in production"
)]

mod common;

use std::collections::BTreeMap;
use std::num::NonZeroU64;

use ix_trace_rs::trace;
use quire_exact::{
    ChargePoint, DecimalType, InexactTarget, InjectedDenial, Integer, IntegerInterval, LimitKind,
    Meter, NodeKey, Outcome, Quantity, QuantityCarrier, Rational, Refusal, RoundingMode,
    ScalarLimits, UnitDomain, UnitId,
};
use quire_semantic_value::quantity::{
    convert_quantity, Conversion, ConvertedValue, HandoffRefusal, QuantityTarget, QuantityUnit,
    UnitScope, UnitTable,
};
use quire_semantic_value::semantic_node::IdentityRefusal;
use quire_semantic_value::unit::{
    CompoundUnit, DimensionNode, NominalDeclaration, UnitGraph, UnitNode,
};

fn meter() -> Meter {
    Meter::new(ScalarLimits {
        integer_bits: 1024,
        decimal_digits: 1024,
        scale_expansion: 1024,
        text_input_bytes: 1024,
        text_scalars: 1024,
        normalized_scalars: 1024,
        unit_edges: 1024,
        value_occurrences: 1024,
        work_units: 1024,
        result_units: 1024,
    })
}

fn tight_meter(
    work_units: u64,
    result_units: u64,
    value_occurrences: u64,
    decimal_digits: u64,
) -> Meter {
    Meter::new(ScalarLimits {
        integer_bits: 2,
        decimal_digits,
        scale_expansion: 0,
        text_input_bytes: 0,
        text_scalars: 0,
        normalized_scalars: 0,
        unit_edges: 0,
        value_occurrences,
        work_units,
        result_units,
    })
}

fn int(n: i64) -> Integer {
    Integer::from(n)
}

fn rational(n: i64) -> Rational {
    Rational::from_integer(int(n))
}

fn key(n: u8) -> NodeKey {
    common::node(u64::from(n))
}

fn dimension(terms: &[(u8, i64)]) -> DimensionNode {
    let terms = terms
        .iter()
        .map(|(base, e)| (key(*base), int(*e)))
        .collect();
    DimensionNode::checked(terms, NominalDeclaration::default()).unwrap()
}

fn unit(dimension: u8, target: Option<u8>, scale: Rational) -> UnitNode {
    UnitNode::checked(
        key(dimension),
        target.map(key),
        scale,
        rational(0),
        NominalDeclaration::default(),
    )
    .unwrap()
}

const MASS: u8 = 1;
const LENGTH: u8 = 2;
const TIME: u8 = 3;
const ENERGY: u8 = 4;
const KILOGRAM: u8 = 11;
const METRE: u8 = 12;
const SECOND: u8 = 13;
const JOULE: u8 = 14;
const MILLIJOULE: u8 = 15;

/// The compound unit kg m^2 s^-2 as `graph` builds it.
fn si_energy(graph: &UnitGraph) -> CompoundUnit {
    graph
        .compound_unit(&[
            (key(KILOGRAM), int(1)),
            (key(METRE), int(2)),
            (key(SECOND), int(-2)),
        ])
        .unwrap()
}

/// QSpec TC-187 U25's units: joule (root of energy = M L^2 T^-2), the
/// millijoule, and, when `twin` is given, a declared unit of energy under the
/// joule whose key bytes are `twin`.
fn energy_graph() -> UnitGraph {
    let dimensions = BTreeMap::from([
        (key(MASS), dimension(&[])),
        (key(LENGTH), dimension(&[])),
        (key(TIME), dimension(&[])),
        (
            key(ENERGY),
            dimension(&[(MASS, 1), (LENGTH, 2), (TIME, -2)]),
        ),
    ]);
    let units = BTreeMap::from([
        (key(KILOGRAM), unit(MASS, None, rational(1))),
        (key(METRE), unit(LENGTH, None, rational(1))),
        (key(SECOND), unit(TIME, None, rational(1))),
        (key(JOULE), unit(ENERGY, None, rational(1))),
        (
            key(MILLIJOULE),
            unit(
                ENERGY,
                Some(JOULE),
                Rational::new(int(1), int(1000)).unwrap(),
            ),
        ),
    ]);
    UnitGraph::from_checked_nodes(&dimensions, &units).unwrap()
}

fn declared(table: &UnitTable, node: u8) -> QuantityUnit {
    table.get(UnitId::declared(key(node))).unwrap().clone()
}

fn exact(
    table: &UnitTable,
    quantity: &Quantity,
    unit: &QuantityUnit,
    meter: &mut Meter,
) -> Conversion {
    let Outcome::Completed(conversion) = convert_quantity(
        table.resolve(quantity).unwrap(),
        unit,
        &QuantityTarget::Exact,
        meter,
    )
    .unwrap() else {
        panic!("the conversion completes");
    };
    conversion
}

/// Trace: FR-124-AC-1
#[trace("TC-923", "FR-124-AC-1")]
#[test]
fn an_exact_conversion_hands_over_its_quantity_and_identified_unit() {
    let graph = energy_graph();
    let table = UnitTable::declared(&graph);
    let joule = Quantity::new(rational(1), UnitId::declared(key(JOULE)));
    let compound = QuantityUnit::Compound(si_energy(&graph));
    let milli = declared(&table, MILLIJOULE);
    for (target, expected) in [(compound, 1), (milli, 1000)] {
        let conversion = exact(&table, &joule, &target, &mut meter());
        let (quantity, identified) = conversion.identified_quantity().unwrap();
        assert_eq!(quantity.magnitude(), &rational(expected));
        assert_eq!(quantity.unit(), &identified.id());
        assert_eq!(identified.id(), target.id().unwrap());
        assert_eq!(identified.unit(), conversion.unit());
    }
}

/// Trace: FR-124-AC-2
#[trace("TC-923", "FR-124-AC-2", "FR-128-AC-3")]
#[test]
fn the_handed_over_quantity_resolves_and_converts_on_to_joules() {
    let graph = energy_graph();
    let table = UnitTable::declared(&graph);
    let joule_id = UnitId::declared(key(JOULE));
    let start = Quantity::new(rational(1), joule_id.clone());
    let compound = QuantityUnit::Compound(si_energy(&graph));

    let mut first = meter();
    let conversion = exact(&table, &start, &compound, &mut first);
    let (quantity, identified) = conversion.identified_quantity().unwrap();

    let mut scope = UnitScope::new(&table);
    assert_eq!(scope.form(identified), quantity.unit().clone());
    let resolved = scope.resolve(&quantity).unwrap();
    assert_eq!(resolved.unit(), conversion.unit());

    let mut second = meter();
    let Outcome::Completed(back) = convert_quantity(
        resolved,
        &declared(&table, JOULE),
        &QuantityTarget::Exact,
        &mut second,
    )
    .unwrap() else {
        panic!("the conversion to joules completes");
    };
    let (joules, _) = back.identified_quantity().unwrap();
    assert_eq!(joules.magnitude(), &rational(1));
    assert_eq!(joules.unit(), &joule_id);

    // One identity read, one target domain and one result retain per
    // conversion, and no edge: the source and target are roots.
    for meter in [&first, &second] {
        assert_eq!(meter.admission_count(), 3);
        assert_eq!(meter.consumed(quire_exact::LimitKind::UnitEdges), 0);
    }

    // The same, per charge point, through the injected-denial seam: denying
    // occurrence 1 stops the conversion at that point, and denying
    // occurrence 2 (or any edge) never fires because each is charged once.
    let hop_back = declared(&table, JOULE);
    let hops = [
        (&start, &compound, &table),
        (
            &quantity,
            &hop_back,
            &table_with(&table, &identified_unit(&conversion)),
        ),
    ];
    for (source, target, table) in hops {
        for point in [
            ChargePoint::UnitIdentityRead,
            ChargePoint::UnitTargetDomain,
            ChargePoint::UnitResultRetain,
        ] {
            let denied = convert_denying(table, source, target, point, 1);
            assert!(
                matches!(&denied, Outcome::Incomplete(record) if record.charge_point == point),
                "occurrence 1 of {point:?} fires"
            );
            assert!(
                matches!(
                    convert_denying(table, source, target, point, 2),
                    Outcome::Completed(_)
                ),
                "occurrence 2 of {point:?} never fires"
            );
        }
        assert!(matches!(
            convert_denying(table, source, target, ChargePoint::UnitEdge, 1),
            Outcome::Completed(_)
        ));
    }
}

fn identified_unit(conversion: &Conversion) -> quire_semantic_value::quantity::IdentifiedUnit {
    conversion.identified_quantity().unwrap().1
}

/// A copy of `table` that also holds the identified unit.
fn table_with(
    table: &UnitTable,
    unit: &quire_semantic_value::quantity::IdentifiedUnit,
) -> UnitTable {
    let mut scope = UnitScope::new(table);
    scope.form(unit.clone());
    let mut copy = UnitTable::default();
    copy.append(table.clone());
    copy.append(scope.into_formed());
    copy
}

fn convert_denying(
    table: &UnitTable,
    source: &Quantity,
    target: &QuantityUnit,
    point: ChargePoint,
    occurrence: u64,
) -> Outcome<Conversion> {
    let mut denying = meter().with_injected_denial(InjectedDenial {
        point,
        occurrence: NonZeroU64::new(occurrence).unwrap(),
    });
    convert_quantity(
        table.resolve(source).unwrap(),
        target,
        &QuantityTarget::Exact,
        &mut denying,
    )
    .unwrap()
}

/// Trace: FR-124-AC-3
#[trace("TC-923", "FR-124-AC-3")]
#[test]
fn a_decimal_or_integer_conversion_is_not_a_quantity() {
    let graph = energy_graph();
    let table = UnitTable::declared(&graph);
    let joule = Quantity::new(rational(1), UnitId::declared(key(JOULE)));
    let target = declared(&table, MILLIJOULE);
    let source = table.resolve(&joule).unwrap();
    let decimal =
        quire_exact::DecimalType::new(int(0), int(10_000), 0, 0, RoundingMode::Exact).unwrap();
    for representation in [
        QuantityTarget::Decimal(decimal),
        QuantityTarget::Integer {
            domain: IntegerInterval::spanning(int(0), int(10_000)),
            rounding: RoundingMode::Exact,
        },
    ] {
        let Outcome::Completed(conversion) =
            convert_quantity(source, &target, &representation, &mut meter()).unwrap()
        else {
            panic!("the conversion completes");
        };
        assert_eq!(
            conversion.identified_quantity(),
            Err(HandoffRefusal::NotExact)
        );
    }
}

/// A carried target has a distinct exact-placement refusal from an ordinary
/// decimal or integer conversion, and non-exact modes retain loss records.
#[trace("TC-1287", "FR-497-AC-4", "FR-497-AC-5", "FR-497-AC-6")]
#[test]
fn carried_quantity_targets_map_exact_refusals_and_preserve_rounding() {
    let graph = energy_graph();
    let table = UnitTable::declared(&graph);
    let source_quantity = Quantity::new(rational(1), UnitId::declared(key(MILLIJOULE)));
    let source = table.resolve(&source_quantity).unwrap();
    let target_unit = declared(&table, JOULE);

    let exact_decimal = DecimalType::new(int(1), int(2), 2, 2, RoundingMode::Exact).unwrap();
    let exact_integer = IntegerInterval::spanning(int(1), int(2));
    let carried_targets = [
        (
            QuantityCarrier::Decimal(exact_decimal.clone()),
            InexactTarget::Decimal(Box::new(exact_decimal)),
        ),
        (
            QuantityCarrier::Integer {
                domain: exact_integer.clone(),
                rounding: RoundingMode::Exact,
            },
            InexactTarget::Integer(Box::new(exact_integer)),
        ),
    ];

    for (carrier, expected) in carried_targets {
        let Outcome::Refused(refusal) = convert_quantity(
            source,
            &target_unit,
            &QuantityTarget::Carried(carrier),
            &mut meter(),
        )
        .unwrap() else {
            panic!("exact carried placement refuses");
        };
        assert_eq!(refusal, Refusal::InexactConversion { target: expected });
        assert_eq!(refusal.code(), Some("inexact_conversion"));
        assert_eq!(refusal.cause(), Some("inexact-conversion"));
    }

    let rounded_decimal =
        DecimalType::new(int(-1), int(1), 2, 2, RoundingMode::NearestEven).unwrap();
    let Outcome::Completed(conversion) = convert_quantity(
        source,
        &target_unit,
        &QuantityTarget::Carried(QuantityCarrier::Decimal(rounded_decimal)),
        &mut meter(),
    )
    .unwrap() else {
        panic!("nearest-even carried Decimal placement completes");
    };
    let ConvertedValue::Decimal(result) = conversion.value() else {
        panic!("carried Decimal placement retains a Decimal result");
    };
    assert!(result.loss().is_some());
    assert_eq!(result.value().normalized().coefficient(), &Integer::zero());

    let rounded_integer = IntegerInterval::spanning(int(-1), int(1));
    let Outcome::Completed(conversion) = convert_quantity(
        source,
        &target_unit,
        &QuantityTarget::Carried(QuantityCarrier::Integer {
            domain: rounded_integer,
            rounding: RoundingMode::NearestEven,
        }),
        &mut meter(),
    )
    .unwrap() else {
        panic!("nearest-even carried Int placement completes");
    };
    let ConvertedValue::Integer { value, loss } = conversion.value() else {
        panic!("carried Int placement retains an Integer result");
    };
    assert!(loss.is_some());
    assert_eq!(value.value(), &Integer::zero());
}

/// QSpec CQ-E13 and TC-187 U29: an exact integer carrier keeps its retained
/// value at the published two-work boundary.
#[trace("FR-124-AC-7", "FR-142-AC-28")]
#[test]
fn exact_integer_conversion_retains_at_two_work_units() {
    let graph = energy_graph();
    let table = UnitTable::declared(&graph);
    let source = Quantity::new(rational(2), UnitId::declared(key(METRE)));
    let target_unit = declared(&table, METRE);
    let target = QuantityTarget::Carried(QuantityCarrier::Integer {
        domain: IntegerInterval::spanning(int(-2), int(2)),
        rounding: RoundingMode::Exact,
    });
    let mut meter = tight_meter(2, 1, 1, 0);

    let Outcome::Completed(conversion) = convert_quantity(
        table.resolve(&source).unwrap(),
        &target_unit,
        &target,
        &mut meter,
    )
    .unwrap() else {
        panic!("CQ-E13 completes at its published limits");
    };
    let ConvertedValue::Integer { value, loss } = conversion.value() else {
        panic!("CQ-E13 retains an integer");
    };
    assert_eq!(value.value(), &int(2));
    assert_eq!(loss, &None);
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 2);
    assert_eq!(meter.consumed(LimitKind::ResultUnits), 1);
    assert_eq!(meter.consumed(LimitKind::ValueOccurrences), 1);
    assert_eq!(meter.admission_count(), 3);
}

/// The integer result charge remains atomic and named, and target-domain work
/// denial precedes retention at the one-work boundary.
#[trace("FR-124-AC-7", "FR-142-AC-28")]
#[test]
fn integer_conversion_retention_boundaries_are_atomic() {
    let graph = energy_graph();
    let table = UnitTable::declared(&graph);
    let source = Quantity::new(rational(2), UnitId::declared(key(METRE)));
    let target_unit = declared(&table, METRE);
    let target = QuantityTarget::Carried(QuantityCarrier::Integer {
        domain: IntegerInterval::spanning(int(-2), int(2)),
        rounding: RoundingMode::Exact,
    });

    for (mut meter, kind, point, consumed, work, occurrences) in [
        (
            tight_meter(1, 1, 1, 0),
            LimitKind::WorkUnits,
            ChargePoint::UnitTargetDomain,
            1,
            1,
            1,
        ),
        (
            tight_meter(2, 0, 1, 0),
            LimitKind::ResultUnits,
            ChargePoint::UnitResultRetain,
            0,
            2,
            1,
        ),
        (
            tight_meter(2, 1, 0, 0),
            LimitKind::ValueOccurrences,
            ChargePoint::UnitIdentityRead,
            0,
            0,
            0,
        ),
    ] {
        let Outcome::Incomplete(incomplete) = convert_quantity(
            table.resolve(&source).unwrap(),
            &target_unit,
            &target,
            &mut meter,
        )
        .unwrap() else {
            panic!("the selected boundary denies conversion");
        };
        assert_eq!(incomplete.limit_kind, kind);
        assert_eq!(incomplete.charge_point, point);
        assert_eq!(incomplete.consumed, consumed);
        assert_eq!(incomplete.next_charge, int(1));
        assert_eq!(meter.consumed(LimitKind::WorkUnits), work);
        assert_eq!(meter.consumed(LimitKind::ResultUnits), 0);
        assert_eq!(meter.consumed(LimitKind::ValueOccurrences), occurrences);
        assert_eq!(meter.admission_count(), work);
    }

    let mut injected = tight_meter(2, 1, 1, 0).with_injected_denial(InjectedDenial {
        point: ChargePoint::UnitResultRetain,
        occurrence: NonZeroU64::new(1).unwrap(),
    });
    let Outcome::Incomplete(incomplete) = convert_quantity(
        table.resolve(&source).unwrap(),
        &target_unit,
        &target,
        &mut injected,
    )
    .unwrap() else {
        panic!("the named integer retention charge is injectable");
    };
    assert_eq!(incomplete.limit_kind, LimitKind::WorkUnits);
    assert_eq!(incomplete.charge_point, ChargePoint::UnitResultRetain);
    assert_eq!(incomplete.consumed, 2);
    assert_eq!(incomplete.next_charge, int(0));
    assert_eq!(injected.consumed(LimitKind::ResultUnits), 0);
    assert_eq!(injected.admission_count(), 2);
}

/// Exact rational and decimal conversion retain their existing one-work
/// charge; the integer exception does not change their values.
#[trace("FR-124-AC-7")]
#[test]
fn noninteger_conversion_retention_keeps_one_work_unit() {
    let graph = energy_graph();
    let table = UnitTable::declared(&graph);
    let source = Quantity::new(rational(2), UnitId::declared(key(METRE)));
    let target_unit = declared(&table, METRE);
    let decimal = DecimalType::new(int(-2), int(2), 0, 0, RoundingMode::Exact).unwrap();

    for target in [QuantityTarget::Exact, QuantityTarget::Decimal(decimal)] {
        let decimal_digits = if matches!(&target, QuantityTarget::Decimal(_)) {
            1
        } else {
            0
        };
        let mut tight = tight_meter(2, 1, 1, decimal_digits);
        let Outcome::Incomplete(incomplete) = convert_quantity(
            table.resolve(&source).unwrap(),
            &target_unit,
            &target,
            &mut tight,
        )
        .unwrap() else {
            panic!("noninteger retention needs a third work unit");
        };
        assert_eq!(incomplete.limit_kind, LimitKind::WorkUnits);
        assert_eq!(incomplete.charge_point, ChargePoint::UnitResultRetain);
        assert_eq!(incomplete.consumed, 2);
        assert_eq!(incomplete.next_charge, int(1));
        assert_eq!(tight.consumed(LimitKind::ResultUnits), 0);

        let mut admitted = tight_meter(3, 1, 1, decimal_digits);
        let Outcome::Completed(conversion) = convert_quantity(
            table.resolve(&source).unwrap(),
            &target_unit,
            &target,
            &mut admitted,
        )
        .unwrap() else {
            panic!("noninteger conversion completes with three work units");
        };
        match conversion.value() {
            ConvertedValue::Exact(value) => assert_eq!(value, &rational(2)),
            ConvertedValue::Decimal(value) => {
                assert_eq!(value.value().normalized().coefficient(), &int(2));
                assert_eq!(value.loss(), None);
            }
            ConvertedValue::Integer { .. } => panic!("the target was not integer"),
        }
        assert_eq!(admitted.consumed(LimitKind::WorkUnits), 3);
        assert_eq!(admitted.consumed(LimitKind::ResultUnits), 1);
    }
}

/// Ordinary conversion targets keep their old refusal identity, while carried
/// targets preserve exact-before-membership and rounded-membership ordering.
#[trace("TC-1287", "FR-497-AC-4", "FR-497-AC-5")]
#[test]
fn carried_refusals_precede_membership_without_changing_legacy_targets() {
    let graph = energy_graph();
    let table = UnitTable::declared(&graph);
    let source_quantity = Quantity::new(rational(1), UnitId::declared(key(MILLIJOULE)));
    let source = table.resolve(&source_quantity).unwrap();
    let target_unit = declared(&table, JOULE);

    let legacy_decimal = DecimalType::new(int(1), int(2), 2, 2, RoundingMode::Exact).unwrap();
    let legacy_integer = IntegerInterval::spanning(int(1), int(2));
    for (target, expected) in [
        (
            QuantityTarget::Decimal(legacy_decimal.clone()),
            InexactTarget::Decimal(Box::new(legacy_decimal)),
        ),
        (
            QuantityTarget::Integer {
                domain: legacy_integer.clone(),
                rounding: RoundingMode::Exact,
            },
            InexactTarget::Integer(Box::new(legacy_integer)),
        ),
    ] {
        let Outcome::Refused(refusal) =
            convert_quantity(source, &target_unit, &target, &mut meter()).unwrap()
        else {
            panic!("legacy exact placement refuses");
        };
        assert_eq!(refusal, Refusal::InexactDecimal { target: expected });
        assert_eq!(refusal.code(), Some("inexact_decimal"));
    }

    let exact_outside = DecimalType::new(int(1), int(2), 2, 2, RoundingMode::Exact).unwrap();
    assert!(matches!(
        convert_quantity(
            source,
            &target_unit,
            &QuantityTarget::Carried(QuantityCarrier::Decimal(exact_outside)),
            &mut meter(),
        )
        .unwrap(),
        Outcome::Refused(Refusal::InexactConversion { .. })
    ));

    let rounded_outside =
        DecimalType::new(int(1), int(2), 2, 2, RoundingMode::NearestEven).unwrap();
    assert!(matches!(
        convert_quantity(
            source,
            &target_unit,
            &QuantityTarget::Carried(QuantityCarrier::Decimal(rounded_outside)),
            &mut meter(),
        )
        .unwrap(),
        Outcome::Refused(Refusal::DecimalOutOfDomain { .. })
    ));

    let rounded_integer_outside = IntegerInterval::spanning(int(1), int(2));
    assert!(matches!(
        convert_quantity(
            source,
            &target_unit,
            &QuantityTarget::Carried(QuantityCarrier::Integer {
                domain: rounded_integer_outside,
                rounding: RoundingMode::NearestEven,
            }),
            &mut meter(),
        )
        .unwrap(),
        Outcome::Refused(Refusal::IntegerOutOfDomain { .. })
    ));
}

/// A source unit with no terms, and a target of `roots` dimension-cancelling
/// roots: the first half with exponent +1, the second with -1, over distinct dimension
/// nodes that share one base map.
fn cancelling(roots: u32) -> (UnitGraph, CompoundUnit) {
    let base = common::node(3_000_000);
    let id = |tag: u8, n: u32| common::node(u64::from(tag) * 1_000_000 + u64::from(n));
    let mut dimensions = BTreeMap::from([(
        base.clone(),
        DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
    )]);
    let mut units = BTreeMap::new();
    let mut terms = Vec::new();
    for n in 0..roots {
        dimensions.insert(
            id(1, n),
            DimensionNode::checked(vec![(base.clone(), int(1))], NominalDeclaration::default())
                .unwrap(),
        );
        units.insert(
            id(2, n),
            UnitNode::checked(
                id(1, n),
                None,
                rational(1),
                rational(0),
                NominalDeclaration::default(),
            )
            .unwrap(),
        );
        terms.push((id(2, n), int(if n < roots / 2 { 1 } else { -1 })));
    }
    let graph = UnitGraph::from_checked_nodes(&dimensions, &units).unwrap();
    let target = graph.compound_unit(&terms).unwrap();
    (graph, target)
}

/// Trace: FR-124-AC-4
#[trace("TC-923", "FR-124-AC-4", "FR-128-AC-3")]
#[test]
fn an_unidentifiable_target_is_refused_with_the_identity_refusal() {
    let (graph, target) = cancelling(200_000);
    let target = QuantityUnit::Compound(target);
    let source_unit = QuantityUnit::Compound(graph.compound_unit(&[]).unwrap());
    assert!(target.has_dimension_of(&source_unit));

    let mut table = UnitTable::declared(&graph);
    let source_id = table.insert(source_unit.clone()).unwrap();
    let scope = UnitScope::new(&table);
    let source = Quantity::new(rational(1), source_id);
    let conversion = exact(&table, &source, &target, &mut meter());

    let Err(refusal @ IdentityRefusal::InputBytes { bound, required }) = target.id() else {
        panic!("the structural term encoding exceeds its input bound");
    };
    assert_eq!(bound, 16_777_216);
    assert!(required > bound);
    assert_eq!(target.id(), Err(refusal));
    assert_eq!(
        conversion.identified_quantity(),
        Err(HandoffRefusal::Identity(refusal))
    );
    // `identified_quantity` takes `&self` and no scope, so it cannot form a
    // unit; the scope still resolves only what the caller inserted.
    assert_eq!(scope.resolve(&source).unwrap().unit(), &source_unit);
}

/// Trace: FR-124-AC-5
#[trace("TC-923", "FR-124-AC-5", "FR-128-AC-3")]
#[test]
fn a_declared_root_and_a_single_root_compound_keep_distinct_ids() {
    let graph = energy_graph();
    let table = UnitTable::declared(&graph);
    let joule = Quantity::new(rational(1), UnitId::declared(key(JOULE)));
    let compound = QuantityUnit::Compound(graph.compound_unit(&[(key(JOULE), int(1))]).unwrap());
    let twin = declared(&table, JOULE);

    let (declared_quantity, declared_unit) = exact(&table, &joule, &twin, &mut meter())
        .identified_quantity()
        .unwrap();
    let (compound_quantity, compound_unit) = exact(&table, &joule, &compound, &mut meter())
        .identified_quantity()
        .unwrap();
    assert_eq!(declared_unit.id().declared_key(), Some(&key(JOULE)));
    assert_eq!(
        compound_unit.id().compound_terms(),
        Some(&[(key(JOULE), int(1))][..])
    );
    assert_eq!(declared_unit.id().domain(), UnitDomain::Declared);
    assert_eq!(compound_unit.id().domain(), UnitDomain::Compound);
    assert_ne!(declared_unit.id(), compound_unit.id());

    let mut scope = UnitScope::new(&table);
    scope.form(compound_unit);
    assert_eq!(scope.resolve(&compound_quantity).unwrap().unit(), &compound);
    assert_ne!(scope.resolve(&compound_quantity).unwrap().unit(), &twin);
    assert_eq!(declared_quantity.unit(), &declared_unit.id());
}

/// Trace: FR-124-AC-3
#[trace("TC-923", "FR-124-AC-3")]
#[test]
fn a_non_exact_conversion_into_an_unidentifiable_target_is_not_exact() {
    let (graph, target) = cancelling(200_000);
    let target = QuantityUnit::Compound(target);
    let source_unit = QuantityUnit::Compound(graph.compound_unit(&[]).unwrap());
    let mut table = UnitTable::declared(&graph);
    let source_id = table.insert(source_unit).unwrap();
    let source = Quantity::new(rational(1), source_id);
    let decimal =
        quire_exact::DecimalType::new(int(0), int(10), 0, 0, RoundingMode::Exact).unwrap();
    let Outcome::Completed(conversion) = convert_quantity(
        table.resolve(&source).unwrap(),
        &target,
        &QuantityTarget::Decimal(decimal),
        &mut meter(),
    )
    .unwrap() else {
        panic!("the conversion completes");
    };
    assert!(target.id().is_err());
    assert_eq!(
        conversion.identified_quantity(),
        Err(HandoffRefusal::NotExact)
    );
}
