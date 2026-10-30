// SPDX-License-Identifier: AGPL-3.0-or-later
//! One structural identity fixture shared by unit and integration tests.
#![allow(
    dead_code,
    reason = "each integration test uses a subset of shared identity fixtures"
)]

use std::{format, string::String, vec, vec::Vec};

use quire_exact::{
    EffectiveId, NodeKey, OriginalDeclarationKey, PathSegment, UniverseId, VariantId,
};

#[allow(
    clippy::disallowed_methods,
    reason = "the single shared test fixture supplies structural identities; QSV production carries them"
)]
fn fixture_node(package: &str, path: Vec<PathSegment>, ordinal: u64) -> NodeKey {
    NodeKey::new(package, path, ordinal).expect("fixture node components are valid")
}

pub fn node(n: u64) -> NodeKey {
    fixture_node("qsv-test", vec![PathSegment::Text(format!("{n:020}"))], 0)
}

pub fn node_parts(package: &str, path: &str, ordinal: u64) -> NodeKey {
    fixture_node(package, vec![PathSegment::Text(path.into())], ordinal)
}

#[allow(
    clippy::disallowed_methods,
    reason = "test fixtures supply structural identities; QSV production never constructs them"
)]
fn fixture_effective(owner: Option<EffectiveId>, package: &str, original: String) -> EffectiveId {
    EffectiveId::new(owner, OriginalDeclarationKey::new(package, original))
}

pub fn effective(n: u64) -> EffectiveId {
    fixture_effective(None, "qsv-test", format!("Type{n:03}"))
}

pub fn effective_parts(package: &str, original: &str, owner: Option<EffectiveId>) -> EffectiveId {
    fixture_effective(owner, package, original.into())
}

pub fn universe(n: u64) -> UniverseId {
    UniverseId::new(node(n))
}

pub fn variant(n: u64) -> VariantId {
    VariantId::new(node(n), "Case").expect("fixture case is nonempty")
}

pub fn nested_node() -> NodeKey {
    let inner = fixture_node("P", vec![PathSegment::Text("inner".into())], 1);
    fixture_node(
        "P",
        vec![PathSegment::Node(inner), PathSegment::Text("outer".into())],
        2,
    )
}

pub fn effective_child(owner: EffectiveId, n: u64) -> EffectiveId {
    fixture_effective(Some(owner), "qsv-test", format!("Member{n:03}"))
}
