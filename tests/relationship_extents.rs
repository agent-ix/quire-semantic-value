// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public admission, read, and resolution checks for FR-119.

use ix_trace_rs::trace;
use quire_exact::{
    EffectiveId, FieldValue, FloatType, IeeeValue, IeeeWidth, ObjectId, ObjectReference, Presence,
    UniverseId, Value, ValueType,
};
use quire_semantic_value::declaration::{FieldDeclaration, ObjectTypeDeclaration, TypeEnvironment};
use quire_semantic_value::object_closure::ObjectClosure;
use quire_semantic_value::relationship_extent::{
    ClosedPopulationManifest, EndSide, ExtentRefusal, ExtentState, ManifestFailure,
    ManifestOutcome, ManifestState, NavigationRelationship, NavigationRelationshipTable,
    PopulationEntry, PopulationEntryFailure, PopulationEnumerator, RelationshipEnd,
    RelationshipExtent, RelationshipExtentStore, Resolution, StopReason, TableRefusal,
};

#[allow(
    clippy::disallowed_methods,
    reason = "fixture supplies identities; QSV mints none"
)]
fn ty(byte: u8) -> EffectiveId {
    EffectiveId::from_digest([byte; 32])
}

const SYS: u8 = 1;
const PUMP: u8 = 2;
const TANK: u8 = 3;
const UNKNOWN: u8 = 4;

fn types() -> TypeEnvironment {
    TypeEnvironment::new(
        [],
        [
            ObjectTypeDeclaration::new(ty(SYS), "Sys", vec![]),
            ObjectTypeDeclaration::new(ty(PUMP), "Pump", vec![]).with_supertypes(vec![ty(SYS)]),
            ObjectTypeDeclaration::new(ty(TANK), "Tank", vec![]),
        ],
    )
    .expect("fixture model")
}

fn reference(universe: u8, object_type: u8, object: &str) -> ObjectReference {
    ObjectReference::new(
        UniverseId::from_digest([universe; 32]),
        ty(object_type),
        ObjectId::new(object).expect("fixture object id"),
    )
}

fn closure(types: &TypeEnvironment) -> ObjectClosure {
    ObjectClosure::new(
        types,
        [
            (reference(9, SYS, "s1"), vec![]),
            (reference(9, PUMP, "p1"), vec![]),
        ],
        &[],
    )
    .expect("fixture closure")
}

fn end(object_type: u8, unique: bool) -> RelationshipEnd {
    RelationshipEnd {
        object_type: ty(object_type),
        unique,
    }
}

fn declaration(
    key: &'static str,
    source: u8,
    target: u8,
    unique: bool,
) -> NavigationRelationship<&'static str> {
    NavigationRelationship {
        key,
        source: end(source, unique),
        target: end(target, unique),
    }
}

fn table(types: &TypeEnvironment, unique: bool) -> NavigationRelationshipTable<&'static str> {
    NavigationRelationshipTable::new(
        types,
        [
            declaration("composition", SYS, PUMP, unique),
            declaration("housing", PUMP, TANK, unique),
        ],
    )
    .expect("fixture table")
}

fn extent(
    relationship: &'static str,
    complete: bool,
    instances: Vec<(ObjectReference, ObjectReference)>,
) -> RelationshipExtent<&'static str> {
    RelationshipExtent {
        relationship,
        complete,
        instances,
    }
}

fn link() -> (ObjectReference, ObjectReference) {
    (reference(9, SYS, "s1"), reference(9, PUMP, "p1"))
}

#[derive(Clone)]
struct Rows<E> {
    rows: std::vec::IntoIter<Result<Option<PopulationEntry<&'static str>>, E>>,
}

impl<E> Rows<E> {
    fn new(rows: Vec<Result<Option<PopulationEntry<&'static str>>, E>>) -> Self {
        Self {
            rows: rows.into_iter(),
        }
    }
}

impl<E> PopulationEnumerator<&'static str> for Rows<E> {
    type Error = E;
    fn next_row(&mut self) -> Result<Option<PopulationEntry<&'static str>>, E> {
        self.rows
            .next()
            .expect("finite fixture enumerator has a terminal row")
    }
}

fn row(population: &'static str, covered: &[u8], complete: bool) -> PopulationEntry<&'static str> {
    PopulationEntry::try_new(population, covered.iter().map(|byte| ty(*byte)), complete)
        .expect("fixture population row")
}

fn manifest<'t>(
    types: &'t TypeEnvironment,
    rows: Vec<PopulationEntry<&'static str>>,
    limit: usize,
) -> ManifestOutcome<'t, &'static str, &'static str> {
    let mut source = Rows::new(
        rows.into_iter()
            .map(Some)
            .map(Ok)
            .chain([Ok(None)])
            .collect(),
    );
    ClosedPopulationManifest::from_enumerator(types, &mut source, limit).expect("fixture manifest")
}

#[trace("TC-919", "FR-119-AC-1")]
#[test]
fn table_refuses_duplicate_before_unknown_and_checks_both_end_memberships() {
    let types = types();
    let prior = table(&types, true);
    let before = prior.clone();
    assert_eq!(
        prior.ends(&"housing"),
        Some((end(PUMP, true), end(TANK, true)))
    );
    let duplicate = NavigationRelationshipTable::new(
        &types,
        [
            declaration("a", SYS, PUMP, true),
            declaration("a", UNKNOWN, PUMP, true),
        ],
    );
    assert_eq!(duplicate, Err(TableRefusal::DuplicateRelationshipKey("a")));
    for (source, target, side) in [
        (UNKNOWN, PUMP, EndSide::Source),
        (SYS, UNKNOWN, EndSide::Target),
        (UNKNOWN, UNKNOWN, EndSide::Source),
    ] {
        assert_eq!(
            NavigationRelationshipTable::new(&types, [declaration("a", source, target, true)]),
            Err(TableRefusal::UnknownEndType {
                relationship: "a",
                side,
                object_type: ty(UNKNOWN)
            })
        );
    }
    assert_eq!(prior, before);
}

#[trace("TC-919", "FR-119-AC-2")]
#[test]
fn unknown_relationship_refuses_whole_call_at_its_position() {
    let types = types();
    let table = table(&types, true);
    let prior =
        RelationshipExtentStore::new(&types, &table, [extent("composition", true, vec![link()])])
            .unwrap();
    let before = prior.clone();
    assert_eq!(
        RelationshipExtentStore::new(
            &types,
            &table,
            [
                extent("composition", true, vec![link()]),
                extent("pipe", true, vec![]),
            ]
        ),
        Err(ExtentRefusal::NotNavigationRelationship {
            relationship: "pipe",
            extent_index: 1
        })
    );
    assert_eq!(prior, before);
}

#[trace("TC-919", "FR-119-AC-3")]
#[test]
fn duplicate_extent_is_local_to_one_constructor_call() {
    let types = types();
    let table = table(&types, true);
    let first =
        RelationshipExtentStore::new(&types, &table, [extent("composition", true, vec![link()])])
            .unwrap();
    let before = first.clone();
    assert_eq!(
        RelationshipExtentStore::new(
            &types,
            &table,
            [
                extent("composition", true, vec![link()]),
                extent(
                    "composition",
                    true,
                    vec![(reference(9, TANK, "t1"), reference(9, TANK, "t1"))]
                ),
            ]
        ),
        Err(ExtentRefusal::DuplicateExtent {
            relationship: "composition",
            extent_index: 1
        })
    );
    let second =
        RelationshipExtentStore::new(&types, &table, [extent("composition", false, vec![])])
            .unwrap();
    assert_eq!(second.state(&"composition"), ExtentState::Incomplete);
    assert_eq!(first, before);
}

#[trace("TC-919", "FR-119-AC-4")]
#[test]
fn type_mismatch_checks_missing_and_unknown_ends_in_source_order() {
    let types = types();
    let table = table(&types, true);
    for (source_type, target_type, side, actual, declared) in [
        (TANK, PUMP, EndSide::Source, TANK, SYS),
        (SYS, TANK, EndSide::Target, TANK, PUMP),
        (UNKNOWN, PUMP, EndSide::Source, UNKNOWN, SYS),
        (TANK, TANK, EndSide::Source, TANK, SYS),
    ] {
        assert_eq!(
            RelationshipExtentStore::new(
                &types,
                &table,
                [extent(
                    "composition",
                    true,
                    vec![(
                        reference(9, source_type, "missing"),
                        reference(9, target_type, "missing")
                    ),]
                )]
            ),
            Err(ExtentRefusal::EndTypeMismatch {
                relationship: "composition",
                extent_index: 0,
                instance_index: 0,
                side,
                actual_type: ty(actual),
                declared_type: ty(declared),
            })
        );
    }
    let missing = reference(9, PUMP, "missing");
    let store = RelationshipExtentStore::new(
        &types,
        &table,
        [extent(
            "composition",
            true,
            vec![(reference(9, SYS, "s1"), missing.clone())],
        )],
    )
    .unwrap();
    assert_eq!(
        store
            .targets(&"composition", &reference(9, SYS, "s1"))
            .next(),
        Some(&missing)
    );
}

#[trace("TC-919", "FR-119-AC-5")]
#[test]
fn duplicate_triple_depends_on_end_uniqueness_and_precedes_later_faults() {
    let types = types();
    for unique in [true, false] {
        let table = NavigationRelationshipTable::new(
            &types,
            [NavigationRelationship {
                key: "composition",
                source: end(SYS, unique),
                target: end(PUMP, false),
            }],
        )
        .unwrap();
        let result = RelationshipExtentStore::new(
            &types,
            &table,
            [extent(
                "composition",
                true,
                vec![
                    link(),
                    link(),
                    (reference(9, TANK, "t"), reference(9, PUMP, "p")),
                ],
            )],
        );
        if unique {
            assert_eq!(
                result,
                Err(ExtentRefusal::DuplicateTriple {
                    relationship: "composition",
                    extent_index: 0,
                    instance_index: 1,
                })
            );
        } else {
            // The later type fault proves repeated triples were admitted first.
            assert_eq!(
                result,
                Err(ExtentRefusal::EndTypeMismatch {
                    relationship: "composition",
                    extent_index: 0,
                    instance_index: 2,
                    side: EndSide::Source,
                    actual_type: ty(TANK),
                    declared_type: ty(SYS),
                })
            );
            let store = RelationshipExtentStore::new(
                &types,
                &table,
                [extent("composition", true, vec![link(), link()])],
            )
            .unwrap();
            assert_eq!(store.targets(&"composition", &link().0).count(), 2);
            assert_eq!(store.sources(&"composition", &link().1).count(), 2);
        }
    }
    let table = table(&types, true);
    assert_eq!(
        RelationshipExtentStore::new(
            &types,
            &table,
            [
                extent(
                    "composition",
                    true,
                    vec![(reference(9, TANK, "t"), link().1)]
                ),
                extent("composition", true, vec![]),
            ]
        ),
        Err(ExtentRefusal::EndTypeMismatch {
            relationship: "composition",
            extent_index: 0,
            instance_index: 0,
            side: EndSide::Source,
            actual_type: ty(TANK),
            declared_type: ty(SYS),
        })
    );
}

#[trace("TC-919", "FR-119-AC-6")]
#[test]
fn extent_admission_has_no_count_bound() {
    let types = types();
    let table = table(&types, false);
    for count in [0, 1, 128] {
        let instances = (0..count)
            .map(|n| {
                (
                    reference(9, SYS, "s1"),
                    reference(9, PUMP, &format!("p{n}")),
                )
            })
            .collect();
        let store =
            RelationshipExtentStore::new(&types, &table, [extent("composition", true, instances)])
                .unwrap();
        assert_eq!(store.state(&"composition"), ExtentState::Complete);
        assert_eq!(
            store
                .targets(&"composition", &reference(9, SYS, "s1"))
                .count(),
            count
        );
    }
}

#[trace("TC-919", "FR-119-AC-7")]
#[test]
fn unlisted_incomplete_and_complete_empty_remain_distinct() {
    let types = types();
    let table = table(&types, true);
    let incomplete =
        RelationshipExtentStore::new(&types, &table, [extent("composition", false, vec![link()])])
            .unwrap();
    assert_eq!(incomplete.state(&"housing"), ExtentState::Unlisted);
    assert!(!incomplete.complete(&"housing"));
    assert!(incomplete.sources(&"housing", &link().1).next().is_none());
    assert_eq!(incomplete.state(&"composition"), ExtentState::Incomplete);
    assert!(!incomplete.complete(&"composition"));
    assert_eq!(incomplete.targets(&"composition", &link().0).count(), 1);
    let complete =
        RelationshipExtentStore::new(&types, &table, [extent("composition", true, vec![])])
            .unwrap();
    assert_eq!(complete.state(&"composition"), ExtentState::Complete);
    assert!(complete.complete(&"composition"));
    assert!(complete.targets(&"composition", &link().0).next().is_none());
    assert_eq!(complete.state(&"housing"), ExtentState::Unlisted);
}

#[trace("TC-919", "FR-119-AC-8")]
#[test]
fn reads_sort_full_reference_keys_and_retain_adjacent_repeats() {
    let types = types();
    let table = table(&types, false);
    let source = reference(9, SYS, "s1");
    let p1 = reference(9, PUMP, "p1");
    let p2 = reference(9, PUMP, "p2");
    let other_universe = reference(8, PUMP, "p1");
    let other_type = reference(9, SYS, "p1");
    let instances = vec![
        (source.clone(), p2.clone()),
        (source.clone(), p1.clone()),
        (source.clone(), p1.clone()),
        (source.clone(), other_universe.clone()),
        (other_type.clone(), p1.clone()),
    ];
    let t1 = reference(9, TANK, "t1");
    let housing = vec![(p2.clone(), t1.clone()), (p1.clone(), t1.clone())];
    let a = RelationshipExtentStore::new(
        &types,
        &table,
        [
            extent("composition", true, instances.clone()),
            extent("housing", true, housing.clone()),
        ],
    )
    .unwrap();
    let mut reversed = instances;
    reversed.reverse();
    let mut reversed_housing = housing;
    reversed_housing.reverse();
    let b = RelationshipExtentStore::new(
        &types,
        &table,
        [
            extent("housing", true, reversed_housing),
            extent("composition", true, reversed),
        ],
    )
    .unwrap();
    let expected = vec![other_universe, p1.clone(), p1.clone(), p2.clone()];
    assert_eq!(
        a.targets(&"composition", &source)
            .cloned()
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        b.targets(&"composition", &source)
            .cloned()
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        a.sources(&"composition", &p1).cloned().collect::<Vec<_>>(),
        vec![other_type, source.clone(), source.clone()]
    );
    assert_eq!(
        a.sources(&"housing", &t1).cloned().collect::<Vec<_>>(),
        vec![p1.clone(), p2.clone()]
    );
    assert_eq!(
        b.sources(&"housing", &t1).cloned().collect::<Vec<_>>(),
        vec![p1.clone(), p2]
    );
    assert_eq!(
        a.targets(&"composition", &reference(8, SYS, "s1")).count(),
        0
    );
    assert_eq!(
        a.targets(&"composition", &reference(9, PUMP, "s1")).count(),
        0
    );
}

#[trace("TC-919", "FR-119-AC-9")]
#[test]
fn held_object_is_present_even_when_manifest_is_partial_or_incomplete() {
    let types = types();
    let different = types.clone();
    let held = closure(&types);
    let x = reference(9, PUMP, "p1");
    let partial = manifest(&different, vec![row("Q", &[PUMP], true)], 0);
    assert_eq!(partial.state(), ManifestState::Partial);
    assert_eq!(held.resolve(&x, &types, &partial), Resolution::Present);
    let incomplete = manifest(&types, vec![row("Q", &[PUMP], false)], 1);
    assert_eq!(incomplete.state(), ManifestState::Closed);
    assert_eq!(held.resolve(&x, &types, &incomplete), Resolution::Present);
}

#[trace("TC-919", "FR-119-AC-10")]
#[test]
fn repeated_reads_are_pure_and_float_identity_uses_bits() {
    let types = types();
    let table = table(&types, true);
    let store =
        RelationshipExtentStore::new(&types, &table, [extent("composition", false, vec![link()])])
            .unwrap();
    let before = store.clone();
    let held = closure(&types);
    let held_before = held.clone();
    let partial = manifest(&types, vec![row("Q", &[SYS], false)], 1);
    let source = reference(9, SYS, "s1");
    let target = reference(9, PUMP, "p1");
    let spent = types.spent();
    for _ in 0..3 {
        assert!(!store.complete(&"composition"));
        assert_eq!(store.targets(&"composition", &source).count(), 1);
        assert_eq!(store.sources(&"composition", &target).count(), 1);
        assert!(store.targets(&"housing", &source).next().is_none());
        assert_eq!(held.resolve(&source, &types, &partial), Resolution::Present);
    }
    assert_eq!(types.spent(), spent);
    assert_eq!(store, before);
    assert_eq!(held, held_before);
    let float_types = TypeEnvironment::new(
        [],
        [ObjectTypeDeclaration::new(
            ty(SYS),
            "FloatOwner",
            vec![FieldDeclaration::new(
                "f",
                ValueType::Float(FloatType::exact(IeeeWidth::Binary64)),
                Presence::Required,
            )],
        )],
    )
    .unwrap();
    let with_bits = |bits| {
        ObjectClosure::new(
            &float_types,
            [(
                reference(9, SYS, "f1"),
                vec![(
                    "f",
                    FieldValue::Present(Value::Float(IeeeValue::binary64(bits))),
                )],
            )],
            &[],
        )
        .unwrap()
    };
    let nan = with_bits(0x7ff8_0000_0000_0001);
    assert_eq!(nan, nan);
    assert_eq!(nan, nan.clone());
    assert_ne!(with_bits(0), with_bits(0x8000_0000_0000_0000));
}

#[trace("TC-919", "FR-119-AC-12")]
#[test]
fn complete_covering_rows_prove_missing_exact_supertype_and_multi_type_references() {
    let types = types();
    let held = closure(&types);
    let missing = reference(9, PUMP, "missing");
    for covered in [vec![PUMP], vec![SYS], vec![TANK, SYS]] {
        let closed = manifest(&types, vec![row("Q", &covered, true)], 1);
        assert_eq!(
            held.resolve(&missing, &types, &closed),
            Resolution::Dangling
        );
    }
}

#[trace("TC-919", "FR-119-AC-13")]
#[test]
fn missing_object_is_undecidable_without_complete_covering_evidence() {
    let types = types();
    let held = closure(&types);
    let missing = reference(9, PUMP, "missing");
    let partial = manifest(&types, vec![row("Q", &[PUMP], true)], 0);
    let incomplete = manifest(&types, vec![row("Q", &[PUMP], false)], 1);
    let unrelated = manifest(&types, vec![row("Q", &[TANK], true)], 1);
    for outcome in [&partial, &incomplete, &unrelated] {
        assert_eq!(
            held.resolve(&missing, &types, outcome),
            Resolution::Undecidable
        );
    }
    let complete = manifest(&types, vec![row("Q", &[PUMP], true)], 1);
    assert_eq!(
        held.resolve(&missing, &types, &complete),
        Resolution::Dangling
    );
    assert_eq!(
        held.resolve(&reference(9, UNKNOWN, "missing"), &types, &complete),
        Resolution::Undecidable
    );
    let mut unknown = Rows::<&str>::new(vec![
        Ok(Some(row("Q", &[UNKNOWN], true))),
        Ok(None::<PopulationEntry<_>>),
    ]);
    assert_eq!(
        ClosedPopulationManifest::from_enumerator(&types, &mut unknown, 1),
        Err(ManifestFailure::UnknownCoveredType {
            population: "Q",
            covered_type: ty(UNKNOWN)
        })
    );
}

#[trace("TC-919", "FR-119-AC-14")]
#[test]
fn any_incomplete_covering_population_prevents_dangling() {
    let types = types();
    let held = closure(&types);
    let x = reference(9, PUMP, "missing");
    for broad in [vec![SYS], vec![TANK, SYS]] {
        let mixed = manifest(
            &types,
            vec![row("exact", &[PUMP], true), row("broad", &broad, false)],
            2,
        );
        assert_eq!(held.resolve(&x, &types, &mixed), Resolution::Undecidable);
        let complete = manifest(
            &types,
            vec![row("exact", &[PUMP], true), row("broad", &broad, true)],
            2,
        );
        assert_eq!(held.resolve(&x, &types, &complete), Resolution::Dangling);
    }
}

#[trace("TC-919", "FR-119-AC-15")]
#[test]
fn population_rows_and_manifest_refusals_preserve_order_and_typed_causes() {
    let types = types();
    assert_eq!(
        PopulationEntry::<&str>::try_new("Q", [], true),
        Err(PopulationEntryFailure::EmptyCoveredTypes)
    );
    assert_eq!(
        PopulationEntry::try_new("Q", [ty(SYS), ty(SYS)], true),
        Err(PopulationEntryFailure::DuplicateCoveredType(ty(SYS)))
    );
    let ordered = row("Q", &[TANK, SYS, PUMP], true);
    assert_eq!(ordered.covered_types(), &[ty(TANK), ty(SYS), ty(PUMP)]);
    let mut duplicate = Rows::<&str>::new(vec![
        Ok(Some(row("Q", &[SYS], true))),
        Ok(Some(row("Q", &[UNKNOWN], true))),
    ]);
    assert_eq!(
        ClosedPopulationManifest::from_enumerator(&types, &mut duplicate, 2),
        Err(ManifestFailure::DuplicatePopulation("Q"))
    );
    let mut unknown = Rows::<&str>::new(vec![Ok(Some(row("Q", &[UNKNOWN, 5], true)))]);
    assert_eq!(
        ClosedPopulationManifest::from_enumerator(&types, &mut unknown, 1),
        Err(ManifestFailure::UnknownCoveredType {
            population: "Q",
            covered_type: ty(UNKNOWN)
        })
    );
    let closed = manifest(
        &types,
        vec![row("Z", &[SYS], true), row("A", &[TANK], false)],
        2,
    );
    assert_eq!(
        closed
            .rows()
            .map(|entry| *entry.population())
            .collect::<Vec<_>>(),
        vec!["A", "Z"]
    );
    assert_eq!(closed.source_rows(), 2);
    assert_eq!(closed.stop(), None);
}

#[trace("TC-919", "FR-119-AC-16")]
#[test]
fn row_limit_probes_before_validating_duplicate_or_unknown_row() {
    let types = types();
    let held = closure(&types);
    for excess in [row("Q", &[SYS], true), row("X", &[UNKNOWN], true)] {
        let mut source =
            Rows::<&str>::new(vec![Ok(Some(row("Q", &[PUMP], true))), Ok(Some(excess))]);
        let partial = ClosedPopulationManifest::from_enumerator(&types, &mut source, 1).unwrap();
        assert_eq!(partial.state(), ManifestState::Partial);
        assert_eq!(partial.source_rows(), 2);
        assert_eq!(partial.partial().unwrap().source_rows(), 2);
        assert_eq!(partial.stop(), Some(&StopReason::RowLimit { limit: 1 }));
        assert_eq!(
            partial
                .rows()
                .map(|entry| *entry.population())
                .collect::<Vec<_>>(),
            vec!["Q"]
        );
        assert_eq!(
            held.resolve(&reference(9, PUMP, "missing"), &types, &partial),
            Resolution::Undecidable
        );
        assert_eq!(
            held.resolve(&reference(9, PUMP, "p1"), &types, &partial),
            Resolution::Present
        );
    }
}

#[trace("TC-919", "FR-119-AC-17")]
#[test]
fn exact_limit_and_zero_limit_check_exhaustion_before_stopping() {
    let types = types();
    let closed = manifest(&types, vec![row("Q", &[SYS], true)], 1);
    assert_eq!(closed.state(), ManifestState::Closed);
    assert_eq!(closed, closed.clone());
    assert_eq!(closed.closed().unwrap().source_rows(), 1);
    let empty = manifest(&types, vec![], 0);
    assert_eq!(empty.state(), ManifestState::Closed);
    assert_eq!(empty.source_rows(), 0);
    let partial = manifest(&types, vec![row("Q", &[SYS], true)], 0);
    assert_eq!(partial.state(), ManifestState::Partial);
    assert_eq!(partial.source_rows(), 1);
    assert_eq!(partial.rows().count(), 0);
    assert_eq!(partial.stop(), Some(&StopReason::RowLimit { limit: 0 }));
}

#[trace("TC-919", "FR-119-AC-18")]
#[test]
fn enumerator_error_keeps_sorted_rows_without_counting_failed_pull() {
    let types = types();
    let held = closure(&types);
    let mut source = Rows::new(vec![
        Ok(Some(row("Z", &[PUMP], true))),
        Ok(Some(row("A", &[TANK], true))),
        Err("source broke"),
    ]);
    let partial = ClosedPopulationManifest::from_enumerator(&types, &mut source, 3).unwrap();
    assert_eq!(partial.state(), ManifestState::Partial);
    assert_eq!(partial.source_rows(), 2);
    assert_eq!(partial.partial().unwrap().source_rows(), 2);
    assert_eq!(
        partial
            .rows()
            .map(|entry| *entry.population())
            .collect::<Vec<_>>(),
        vec!["A", "Z"]
    );
    assert_eq!(
        partial.stop(),
        Some(&StopReason::EnumeratorError("source broke"))
    );
    assert_eq!(
        held.resolve(&reference(9, PUMP, "missing"), &types, &partial),
        Resolution::Undecidable
    );
}

#[trace("TC-919", "FR-119-AC-19")]
#[test]
fn environment_identity_only_matters_for_missing_objects() {
    let types = types();
    let other = types.clone();
    let held = closure(&types);
    assert_eq!(types, other);
    let closed = manifest(&other, vec![row("Q", &[PUMP], true)], 1);
    assert_eq!(
        held.resolve(&reference(9, PUMP, "missing"), &types, &closed),
        Resolution::Undecidable
    );
    let partial = manifest(&other, vec![row("Q", &[PUMP], true)], 0);
    assert_eq!(
        held.resolve(&reference(9, PUMP, "p1"), &types, &partial),
        Resolution::Present
    );
    assert_ne!(closed, manifest(&types, vec![row("Q", &[PUMP], true)], 1));
}

#[trace("TC-919", "FR-119-AC-20")]
#[test]
fn child_only_population_does_not_cover_parent_reference() {
    let types = types();
    let held = closure(&types);
    let closed = manifest(&types, vec![row("Q", &[PUMP], true)], 1);
    assert_eq!(
        held.resolve(&reference(9, SYS, "missing"), &types, &closed),
        Resolution::Undecidable
    );
    assert_eq!(
        held.resolve(&reference(9, PUMP, "missing"), &types, &closed),
        Resolution::Dangling
    );
}
