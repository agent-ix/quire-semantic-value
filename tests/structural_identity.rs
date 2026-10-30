// SPDX-License-Identifier: AGPL-3.0-or-later
//! Discriminating structural identity controls for FR-128.

mod common;

use std::collections::BTreeMap;

use ix_trace_rs::trace;
use quire_exact::{IllTypedCause, Integer, Rational, ValueType, VariantId};
use quire_semantic_value::declaration::{
    CompositeDeclaration, CompositeShape, DeclarationCause, EnvironmentFailure, FieldDeclaration,
    ObjectTypeDeclaration, TypeEnvironment,
};
use quire_semantic_value::enumeration::{EnumDeclaration, EnumMemberIndex};
use quire_semantic_value::semantic_node::{InvalidSemanticGraph, SemanticGraphCause};
use quire_semantic_value::unit::{DimensionNode, NominalDeclaration, UnitGraph, UnitNode};

fn root(dimension: quire_exact::NodeKey) -> UnitNode {
    UnitNode::checked(
        dimension,
        None,
        Rational::from_integer(Integer::one()),
        Rational::from_integer(Integer::zero()),
        NominalDeclaration::default(),
    )
    .expect("root fixture is canonical")
}

/// Package, path and ordinal each participate in all structural node lookups.
#[trace("FR-128-AC-1")]
#[test]
fn node_components_separate_dimension_unit_composite_and_enum_lookups() {
    let admitted = common::node_parts("P", "same", 1);
    let unit = common::node_parts("P", "unit", 1);
    let dimensions = BTreeMap::from([(
        admitted.clone(),
        DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
    )]);
    let units = BTreeMap::from([(unit.clone(), root(admitted.clone()))]);
    let graph = UnitGraph::from_checked_nodes(&dimensions, &units).unwrap();
    let types = TypeEnvironment::new(
        [CompositeDeclaration::new(
            admitted.clone(),
            "Record",
            CompositeShape::Record(vec![]),
        )],
        [],
    )
    .unwrap();
    let enum_type = EnumDeclaration::new(admitted.clone(), false, vec!["Case".into()]).unwrap();
    let mut enum_index = EnumMemberIndex::default();
    enum_index.record(enum_type.member("Case", unit.clone()).unwrap());

    assert!(graph.dimension(admitted.clone()).is_some());
    assert!(graph.unit(unit.clone()).is_some());
    assert!(types.composite(admitted.clone()).is_some());
    assert!(enum_index
        .resolve(VariantId::new(admitted.clone(), "Case").unwrap())
        .is_some());

    for foreign in [
        common::node_parts("Other", "same", 1),
        common::node_parts("P", "other", 1),
        common::node_parts("P", "same", 2),
    ] {
        assert!(graph.dimension(foreign.clone()).is_none());
        assert!(types.composite(foreign.clone()).is_none());
        assert!(enum_index
            .resolve(VariantId::new(foreign.clone(), "Case").unwrap())
            .is_none());
        assert_eq!(
            graph.declared_unit_id(foreign.clone()).unwrap_err().key,
            foreign
        );
    }
    for foreign in [
        common::node_parts("Other", "unit", 1),
        common::node_parts("P", "other", 1),
        common::node_parts("P", "unit", 2),
    ] {
        assert!(graph.unit(foreign).is_none());
    }
}

/// Root ownership, owner chain and original declaration are separate keys.
#[trace("FR-128-AC-2")]
#[test]
fn effective_components_separate_object_and_reference_checks() {
    let admitted = common::effective_parts("P", "Type", None);
    let foreign = [
        common::effective_parts("Other", "Type", None),
        common::effective_parts("P", "Other", None),
        common::effective_parts(
            "P",
            "Type",
            Some(common::effective_parts("P", "Owner", None)),
        ),
    ];
    let types = TypeEnvironment::new(
        [],
        [ObjectTypeDeclaration::new(admitted.clone(), "Type", vec![])],
    )
    .unwrap();
    assert!(types.object_type(admitted.clone()).is_some());
    for key in foreign {
        assert!(types.object_type(key.clone()).is_none());
        assert!(!types.conforms(key.clone(), admitted.clone()));
        assert!(!types.conforms(admitted.clone(), key.clone()));
        let refusal = TypeEnvironment::new(
            [CompositeDeclaration::new(
                common::node_parts("P", "record", 1),
                "Record",
                CompositeShape::Record(vec![FieldDeclaration::new(
                    "ref",
                    ValueType::Reference(key.clone()),
                    quire_exact::Presence::Required,
                )]),
            )],
            [ObjectTypeDeclaration::new(admitted.clone(), "Type", vec![])],
        );
        assert!(
            matches!(
                refusal,
                Err(EnvironmentFailure::Refused(ref invalid))
                    if invalid.cause == DeclarationCause::Type(IllTypedCause::TypeMismatch)
            ),
            "{refusal:?}"
        );
    }
}

/// A near-matching key cannot replace the first absent dimension or target.
#[trace("FR-128-AC-5")]
#[test]
fn missing_references_refuse_before_partial_identity_selection() {
    let dimension = common::node_parts("P", "dimension", 1);
    let root_key = common::node_parts("P", "root", 1);
    let missing_dimension = common::node_parts("Other", "dimension", 1);
    let missing_target = common::node_parts("P", "root", 2);
    let dimensions = BTreeMap::from([(
        dimension.clone(),
        DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
    )]);
    let missing_dimension_unit = BTreeMap::from([(
        root_key.clone(),
        UnitNode::checked(
            missing_dimension.clone(),
            Some(missing_target.clone()),
            Rational::from_integer(Integer::one()),
            Rational::from_integer(Integer::zero()),
            NominalDeclaration::default(),
        )
        .unwrap(),
    )]);
    // Both references are absent. The dimension check must still refuse first.
    // UnitGraph takes no Meter and makes no charge on either validation path.
    assert_eq!(
        UnitGraph::from_checked_nodes(&dimensions, &missing_dimension_unit),
        Err(InvalidSemanticGraph {
            cause: SemanticGraphCause::UnknownDimension,
        })
    );

    let present_root = root(dimension.clone());
    let pointed = UnitNode::checked(
        dimension,
        Some(missing_target),
        Rational::from_integer(Integer::one()),
        Rational::from_integer(Integer::zero()),
        NominalDeclaration::default(),
    )
    .unwrap();
    let units = BTreeMap::from([
        (root_key, present_root),
        (common::node_parts("P", "child", 1), pointed),
    ]);
    assert_eq!(
        UnitGraph::from_checked_nodes(&dimensions, &units),
        Err(InvalidSemanticGraph {
            cause: SemanticGraphCause::UnknownTarget,
        })
    );
}
