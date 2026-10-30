// SPDX-License-Identifier: AGPL-3.0-or-later
//! QSpec FR-142 dimensions, declared units, their admitted unit graph and the
//! evaluator-owned compound-unit value, at runtime.
//!
//! Base dimensions and units are nominal nodes. A dimension is a sorted
//! map from base-dimension key to a nonzero mathematical exponent. The units
//! of one dimension node form an acyclic graph with exactly one targetless
//! canonical root; each edge maps `target_value = scale × source_value +
//! offset` exactly.
//!
//! This module owns the graph's topology and conversion logic: given nodes
//! that already passed their per-node checks, [`UnitGraph::from_checked_nodes`]
//! resolves their references, refuses a malformed graph and composes each
//! unit's exact path to its root. The compile side admits structural keys and
//! checks their owner before building the graph.
//!
//! It never constructs a `NodeKey`: only the checking stage mints one. The
//! admitted structural key resolves by lookup among the graph's keys.
//!
//! A declared unit's kernel [`UnitId`] carries its admitted node key. A
//! compound unit's id carries its ordered root terms and exponents.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};

use quire_canonical::{Encode, Error, Limits, Sink, Writer};

use quire_exact::{
    Integer, InvalidNodeKey, NodeKey, NodeKeyView, PathSegmentView, Rational, UnitId,
};

use crate::semantic_node::{
    check_terms, IdentityRefusal, InvalidSemanticGraph, SemanticGraphCause, IDENTITY_LIMITS,
};

/// The graph refusal of `cause`.
fn refuse(cause: SemanticGraphCause) -> InvalidSemanticGraph {
    InvalidSemanticGraph { cause }
}

// ---- dimensions --------------------------------------------------------------

/// A normalized dimension: base-dimension node keys to nonzero exponents.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Dimension(BTreeMap<NodeKey, Integer>);

impl Dimension {
    /// The empty (dimensionless) map.
    pub fn dimensionless() -> Self {
        Self::default()
    }

    /// Whether every exponent is absent.
    pub fn is_dimensionless(&self) -> bool {
        self.0.is_empty()
    }

    /// Ascending `(base dimension, exponent)` terms; no exponent is zero.
    pub fn exponents(&self) -> impl Iterator<Item = (NodeKey, &Integer)> {
        self.0.iter().map(|(key, exponent)| (key.clone(), exponent))
    }

    /// Add exponents.
    pub fn multiply(&self, other: &Self) -> Self {
        Self(combine(&self.0, &other.0, Integer::add))
    }

    /// Subtract exponents.
    pub fn divide(&self, other: &Self) -> Self {
        Self(combine(&self.0, &other.0, Integer::sub))
    }

    /// Multiply every exponent by `exponent`.
    pub fn power(&self, exponent: &Integer) -> Self {
        Self(scale_exponents(&self.0, exponent))
    }
}

/// Combine two exponent maps key by key and drop zero exponents.
fn combine(
    left: &BTreeMap<NodeKey, Integer>,
    right: &BTreeMap<NodeKey, Integer>,
    operation: fn(&Integer, &Integer) -> Integer,
) -> BTreeMap<NodeKey, Integer> {
    let keys: BTreeSet<_> = left.keys().chain(right.keys()).cloned().collect();
    let zero = Integer::zero();
    keys.into_iter()
        .map(|key| {
            let exponent = operation(
                left.get(&key).unwrap_or(&zero),
                right.get(&key).unwrap_or(&zero),
            );
            (key, exponent)
        })
        .filter(|(_, exponent)| !exponent.is_zero())
        .collect()
}

fn scale_exponents(
    terms: &BTreeMap<NodeKey, Integer>,
    exponent: &Integer,
) -> BTreeMap<NodeKey, Integer> {
    terms
        .iter()
        .map(|(key, value)| (key.clone(), value.mul(exponent)))
        .filter(|(_, value)| !value.is_zero())
        .collect()
}

// ---- units -------------------------------------------------------------------

/// One exact affine edge `target = scale × source + offset`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UnitEdge {
    scale: Rational,
    offset: Rational,
}

impl UnitEdge {
    /// A unit node's edge after its semantic checks, in order: a zero scale
    /// refuses, then a canonical root (`is_root`) whose edge is not the
    /// identity (scale one, offset zero).
    fn checked(
        scale: Rational,
        offset: Rational,
        is_root: bool,
    ) -> Result<Self, SemanticGraphCause> {
        if scale.is_zero() {
            return Err(SemanticGraphCause::ZeroScale);
        }
        let identity = scale == Rational::from_integer(Integer::one()) && offset.is_zero();
        if is_root && !identity {
            return Err(SemanticGraphCause::NonIdentityRoot);
        }
        Ok(Self { scale, offset })
    }

    /// Nonzero exact scale.
    pub fn scale(&self) -> &Rational {
        &self.scale
    }

    /// Exact offset.
    pub fn offset(&self) -> &Rational {
        &self.offset
    }
}

/// An admitted declared unit and its exact path to the canonical root.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Unit {
    key: NodeKey,
    dimension_node: NodeKey,
    dimension: Dimension,
    root: NodeKey,
    path: Vec<UnitEdge>,
    canonical: UnitEdge,
}

impl Unit {
    /// The unit node key.
    pub fn key(&self) -> NodeKey {
        self.key.clone()
    }

    /// The declared-arm [`UnitId`]: this admitted unit's node key under the
    /// `quire.checked-semantic-node/v1` label.
    pub fn id(&self) -> UnitId {
        UnitId::declared(self.key.clone())
    }

    /// The unit's dimension node key.
    pub fn dimension_node(&self) -> NodeKey {
        self.dimension_node.clone()
    }

    /// The normalized dimension map.
    pub fn dimension(&self) -> &Dimension {
        &self.dimension
    }

    /// The canonical root unit of the unit's dimension node.
    pub fn root(&self) -> NodeKey {
        self.root.clone()
    }

    /// Edges from this unit to the root, in source-to-root order.
    pub fn path(&self) -> &[UnitEdge] {
        &self.path
    }

    /// The composed exact mapping to the canonical root.
    pub fn canonical(&self) -> &UnitEdge {
        &self.canonical
    }

    /// Whether this unit is affine: a nonzero-offset point unit that admits
    /// only conversion and comparison. The composed canonical offset
    /// decides, so a zero-offset unit that targets an affine unit is affine.
    pub fn is_affine(&self) -> bool {
        !self.canonical.offset.is_zero()
    }
}

/// What lowering builds a dimension's or unit's nominal node from (QSL FR-094),
/// beyond the graph's topology: the node's qualified declaration name and
/// the RFC 8785 bytes of its `quire.dimension-node/v1` or
/// `quire.unit-node/v1` body. The compile side reads and checks both; this
/// layer only carries them. Its structural key is admitted separately.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NominalDeclaration {
    /// The preimage's qualified declaration segments.
    pub qualified_declaration: Vec<String>,
    /// The preimage's RFC 8785 bytes.
    pub preimage: Vec<u8>,
}

/// An admitted dimension or unit node as lowering builds it (QSL FR-094): its
/// key, its [`NominalDeclaration`], and the admitted nodes its preimage
/// names, each resolved to its key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NominalUnitNode {
    /// The node key.
    pub key: NodeKey,
    /// The node's name and preimage bytes.
    pub declaration: NominalDeclaration,
    /// Whether it is a dimension or a unit, and the nodes it names.
    pub form: NominalUnitForm,
}

/// The kind of a [`NominalUnitNode`], with its preimage's terms resolved
/// to admitted keys.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "exact unit edge scales and offsets stay inline; boxing them adds heap allocations and changes the public form"
)]
pub enum NominalUnitForm {
    /// A dimension node: its `(base dimension, exponent)` terms, ascending
    /// by key, empty for a base dimension.
    Dimension {
        /// The derived dimension's terms.
        terms: Vec<(NodeKey, Integer)>,
    },
    /// A unit node: its dimension, its target unit (`None` for a root) and
    /// its edge `target = scale × unit + offset` (the identity for a root).
    Unit {
        /// The unit's dimension node.
        dimension: NodeKey,
        /// The unit's target unit node.
        target: Option<NodeKey>,
        /// The edge's exact scale.
        scale: Rational,
        /// The edge's exact offset.
        offset: Rational,
    },
}

/// A dimension node after its per-node checks, as read: its
/// `(base dimension id, exponent)` terms, empty for a base dimension, and
/// its [`NominalDeclaration`]. Each id is an admitted structural node key.
/// Built only by
/// [`DimensionNode::checked`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DimensionNode {
    terms: Vec<(NodeKey, Integer)>,
    declaration: NominalDeclaration,
}

impl DimensionNode {
    /// A dimension node's terms after `check_terms`, in order: a zero
    /// exponent refuses, then a repeated base dimension, then terms not
    /// strictly ascending by id.
    pub fn checked(
        terms: Vec<(NodeKey, Integer)>,
        declaration: NominalDeclaration,
    ) -> Result<Self, SemanticGraphCause> {
        check_terms(&terms)?;
        Ok(Self { terms, declaration })
    }
}

/// A unit node after its per-node checks, as read, with its
/// [`NominalDeclaration`]. Its dimension and target ids are admitted
/// structural node keys, resolved by lookup among the admitted keys. Built
/// only by [`UnitNode::checked`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitNode {
    dimension: NodeKey,
    target: Option<NodeKey>,
    edge: UnitEdge,
    declaration: NominalDeclaration,
}

impl UnitNode {
    /// A unit node whose edge `target = scale × source + offset` passed its
    /// semantic checks, in order: a zero scale refuses, then a canonical
    /// root (`target` is `None`) whose edge is not the identity (scale one,
    /// offset zero).
    pub fn checked(
        dimension: NodeKey,
        target: Option<NodeKey>,
        scale: Rational,
        offset: Rational,
        declaration: NominalDeclaration,
    ) -> Result<Self, SemanticGraphCause> {
        let edge = UnitEdge::checked(scale, offset, target.is_none())?;
        Ok(Self {
            dimension,
            target,
            edge,
            declaration,
        })
    }
}

/// An admitted closed set of dimension and unit nodes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnitGraph {
    dimensions: BTreeMap<NodeKey, Dimension>,
    units: BTreeMap<NodeKey, Unit>,
    /// Each admitted dimension and unit node as lowering builds it, by key.
    nominal: BTreeMap<NodeKey, NominalUnitNode>,
}

/// A unit node whose dimension and target resolved to admitted keys.
struct ResolvedUnit {
    dimension: NodeKey,
    target: Option<NodeKey>,
    edge: UnitEdge,
}

impl UnitGraph {
    /// Admit the topology of dimension and unit nodes, keyed by their
    /// distinct retained keys. Each node's own checks are its constructor's
    /// ([`DimensionNode::checked`], [`UnitNode::checked`]).
    ///
    /// This checks topology only. The caller admits each structural key and
    /// its owner. A graph built from unchecked keys carries no provenance
    /// guarantee.
    ///
    /// The graph, referenced by node ids, is checked for unknown or derived
    /// dimension terms, unknown dimensions, unknown and cross-dimension
    /// targets, per-dimension root count and target cycles, in that order.
    /// Every refusal is `invalid_semantic_graph`; the typed cause names the
    /// first failed check.
    pub fn from_checked_nodes(
        dimensions: &BTreeMap<NodeKey, DimensionNode>,
        units: &BTreeMap<NodeKey, UnitNode>,
    ) -> Result<Self, InvalidSemanticGraph> {
        let dimension_maps = dimension_maps(dimensions)?;
        let resolved = resolve_units(units, dimensions)?;
        let mut nominal = BTreeMap::new();
        for (key, node) in dimensions {
            let terms = match dimension_maps.get(key) {
                Some(map) if !node.terms.is_empty() => map
                    .0
                    .iter()
                    .map(|(base, exponent)| (base.clone(), exponent.clone()))
                    .collect(),
                _ => Vec::new(),
            };
            nominal.insert(
                key.clone(),
                NominalUnitNode {
                    key: key.clone(),
                    declaration: node.declaration.clone(),
                    form: NominalUnitForm::Dimension { terms },
                },
            );
        }
        for (key, node) in units {
            if let Some(unit) = resolved.get(key) {
                nominal.insert(
                    key.clone(),
                    NominalUnitNode {
                        key: key.clone(),
                        declaration: node.declaration.clone(),
                        form: NominalUnitForm::Unit {
                            dimension: unit.dimension.clone(),
                            target: unit.target.clone(),
                            scale: unit.edge.scale.clone(),
                            offset: unit.edge.offset.clone(),
                        },
                    },
                );
            }
        }
        let units = unit_paths(&resolved, &dimension_maps)?;
        Ok(Self {
            dimensions: dimension_maps,
            units,
            nominal,
        })
    }

    /// Every admitted dimension and unit node as lowering builds it, by
    /// key.
    pub(crate) fn nominal_nodes(&self) -> impl Iterator<Item = &NominalUnitNode> {
        self.nominal.values()
    }

    /// The normalized map of an admitted dimension node.
    pub fn dimension(&self, key: NodeKey) -> Option<&Dimension> {
        self.dimensions.get(&key)
    }

    /// An admitted unit.
    pub fn unit(&self, key: NodeKey) -> Option<&Unit> {
        self.units.get(&key)
    }

    /// Every admitted unit, ascending by node key.
    pub fn units(&self) -> impl Iterator<Item = &Unit> {
        self.units.values()
    }

    /// A unit node key as a declared-arm [`UnitId`]. Only the
    /// key of a unit in this graph converts; any other node key, such as a
    /// dimension's, is refused. The caller must supply a checked structural
    /// key, as the compile side does before [`Self::from_checked_nodes`].
    pub fn declared_unit_id(&self, key: NodeKey) -> Result<UnitId, NotAUnitKey> {
        self.unit(key.clone())
            .map(Unit::id)
            .ok_or(NotAUnitKey { key })
    }

    /// Construct a compound unit from schema-valid `(unit node id,
    /// exponent)` terms: every term must name an admitted canonical root
    /// unit with a nonzero exponent, strictly ascending by id.
    pub fn compound_unit(
        &self,
        terms: &[(NodeKey, Integer)],
    ) -> Result<CompoundUnit, InvalidCompoundUnit> {
        check_terms(terms).map_err(|cause| InvalidCompoundUnit {
            cause: match cause {
                SemanticGraphCause::ZeroExponent => CompoundUnitCause::ZeroExponent,
                SemanticGraphCause::DuplicateTerm => CompoundUnitCause::DuplicateTerm,
                // `check_terms` raises only the three term causes.
                _ => CompoundUnitCause::UnsortedTerms,
            },
        })?;
        let mut dimension = Dimension::dimensionless();
        let mut compound_terms = BTreeMap::new();
        for (id, exponent) in terms {
            let (key, root) = self
                .units
                .get_key_value(id)
                .filter(|(key, unit)| unit.root == **key)
                .ok_or(InvalidCompoundUnit {
                    cause: CompoundUnitCause::NotRootUnit,
                })?;
            dimension = dimension.multiply(&root.dimension.power(exponent));
            compound_terms.insert(key.clone(), exponent.clone());
        }
        Ok(CompoundUnit::new(compound_terms, dimension))
    }
}

/// The refusal: the node key names no admitted unit.
#[derive(Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[error("node key {key} names no admitted unit")]
pub struct NotAUnitKey {
    /// The refused node key.
    pub key: NodeKey,
}

/// A base dimension maps to itself; a derived dimension to its base terms,
/// each resolved by lookup among the admitted dimension keys.
fn dimension_maps(
    dimensions: &BTreeMap<NodeKey, DimensionNode>,
) -> Result<BTreeMap<NodeKey, Dimension>, InvalidSemanticGraph> {
    dimensions
        .iter()
        .map(|(key, node)| {
            if node.terms.is_empty() {
                return Ok((
                    key.clone(),
                    Dimension([(key.clone(), Integer::one())].into()),
                ));
            }
            let mut terms = BTreeMap::new();
            for (term, exponent) in &node.terms {
                let base = dimensions
                    .get_key_value(term)
                    .map(|(key, _)| key.clone())
                    .ok_or(refuse(SemanticGraphCause::UnknownDimension))?;
                if dimensions
                    .get(&base)
                    .is_some_and(|base| !base.terms.is_empty())
                {
                    return Err(refuse(SemanticGraphCause::NonBaseDimensionTerm));
                }
                terms.insert(base, exponent.clone());
            }
            Ok((key.clone(), Dimension(terms)))
        })
        .collect()
}

/// Resolve every unit's dimension, then every unit's target, by lookup among
/// the admitted keys.
fn resolve_units(
    units: &BTreeMap<NodeKey, UnitNode>,
    dimensions: &BTreeMap<NodeKey, DimensionNode>,
) -> Result<BTreeMap<NodeKey, ResolvedUnit>, InvalidSemanticGraph> {
    let resolved_dimensions = units
        .values()
        .map(|unit| {
            dimensions
                .get_key_value(&unit.dimension)
                .map(|(key, _)| key.clone())
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(refuse(SemanticGraphCause::UnknownDimension))?;
    units
        .iter()
        .zip(resolved_dimensions)
        .map(|((key, unit), dimension)| {
            let target = unit
                .target
                .as_ref()
                .map(|id| {
                    units
                        .get_key_value(id)
                        .map(|(key, _)| key.clone())
                        .ok_or(refuse(SemanticGraphCause::UnknownTarget))
                })
                .transpose()?;
            Ok((
                key.clone(),
                ResolvedUnit {
                    dimension,
                    target,
                    edge: unit.edge.clone(),
                },
            ))
        })
        .collect()
}

/// Check unit graph topology and compose each unit's exact path to its root.
fn unit_paths(
    units: &BTreeMap<NodeKey, ResolvedUnit>,
    dimensions: &BTreeMap<NodeKey, Dimension>,
) -> Result<BTreeMap<NodeKey, Unit>, InvalidSemanticGraph> {
    let targets = || {
        units
            .values()
            .filter_map(|unit| Some((unit, unit.target.as_ref()?)))
    };
    if targets().any(|(unit, target)| {
        units
            .get(target)
            .is_some_and(|target| target.dimension != unit.dimension)
    }) {
        return Err(refuse(SemanticGraphCause::CrossDimensionTarget));
    }
    let mut roots: BTreeMap<NodeKey, Vec<NodeKey>> = BTreeMap::new();
    for (key, unit) in units {
        let slot = roots.entry(unit.dimension.clone()).or_default();
        if unit.target.is_none() {
            slot.push(key.clone());
        }
    }
    if roots.values().any(Vec::is_empty) {
        return Err(refuse(SemanticGraphCause::MissingRoot));
    }
    if roots.values().any(|found| found.len() > 1) {
        return Err(refuse(SemanticGraphCause::DuplicateRoot));
    }
    units
        .iter()
        .map(|(key, unit)| {
            let mut path = Vec::new();
            let mut canonical = UnitEdge {
                scale: Rational::from_integer(Integer::one()),
                offset: Rational::from_integer(Integer::zero()),
            };
            let mut current = (key, unit);
            while let Some(target) = current.1.target.as_ref() {
                if path.len() >= units.len() {
                    return Err(refuse(SemanticGraphCause::TargetCycle));
                }
                let edge = current.1.edge.clone();
                canonical = UnitEdge {
                    scale: edge.scale.mul(&canonical.scale),
                    offset: edge.scale.mul(&canonical.offset).add(&edge.offset),
                };
                path.push(edge);
                let next = units
                    .get(target)
                    .ok_or(refuse(SemanticGraphCause::UnknownTarget))?;
                current = (target, next);
            }
            let dimension = dimensions
                .get(&unit.dimension)
                .cloned()
                .ok_or(refuse(SemanticGraphCause::UnknownDimension))?;
            Ok((
                key.clone(),
                Unit {
                    key: key.clone(),
                    dimension_node: unit.dimension.clone(),
                    dimension,
                    root: current.0.clone(),
                    path,
                    canonical,
                },
            ))
        })
        .collect()
}

// ---- compound units ----------------------------------------------------------

/// Why a compound-unit preimage was refused at construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[error("invalid compound unit: {cause:?}")]
pub struct InvalidCompoundUnit {
    /// The typed reason.
    pub cause: CompoundUnitCause,
}

/// The check that refused a compound-unit preimage.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CompoundUnitCause {
    /// The preimage does not satisfy `value-compound-unit.schema.json`.
    NonCanonicalPreimage,
    /// A term exponent is zero.
    ZeroExponent,
    /// A root unit appears twice.
    DuplicateTerm,
    /// Terms are not strictly ascending by node key.
    UnsortedTerms,
    /// A term does not name an admitted canonical root unit.
    NotRootUnit,
}

// RFC 8785 JCS: `quire-canonical` orders members itself, so field
// declaration order carries no meaning.
/// The compound-arm [`UnitId`] of exactly these structural root terms.
/// Their FR-201 `terms` arm is encoded under `limits` solely to bound the
/// work of carrying the identity; neither the bytes nor a digest are retained.
///
/// # Errors
///
/// [`IdentityRefusal::InputBytes`] when the preimage's canonical bytes would
/// exceed `limits` (QSL FR-259 Behavior 4): nothing bounds the number of
/// terms or an exponent's size, so a large enough unit reaches any byte
/// limit. [`IdentityRefusal::Allocation`] reports failed term-vector, encoder
/// stack, and structural-node-copy reservations with the requested bytes.
/// The existing infallible `Integer` clone and decimal formatting remain the
/// separate IR-1149 allocation residual.
pub fn compound_unit_id<'a>(
    terms: impl IntoIterator<Item = (&'a NodeKey, &'a Integer)>,
    limits: Limits,
) -> Result<UnitId, IdentityRefusal> {
    let mut retained = Vec::new();
    for term in terms {
        retained
            .try_reserve(1)
            .map_err(|_| IdentityRefusal::Allocation {
                requested: retained
                    .len()
                    .saturating_add(1)
                    .saturating_mul(core::mem::size_of::<(&NodeKey, &Integer)>()),
            })?;
        retained.push(term);
    }
    quire_canonical::encode(&mut CountingSink, &StructuralTerms(&retained), limits)?;
    let mut owned = Vec::new();
    owned
        .try_reserve(retained.len())
        .map_err(|_| IdentityRefusal::Allocation {
            requested: retained
                .len()
                .saturating_mul(core::mem::size_of::<(NodeKey, Integer)>()),
        })?;
    for (unit, exponent) in retained {
        let unit = unit.try_clone().map_err(|error| match error {
            InvalidNodeKey::Allocation { requested } => IdentityRefusal::Allocation { requested },
            _ => IdentityRefusal::NonCanonical,
        })?;
        owned.push((unit, exponent.clone()));
    }
    Ok(UnitId::compound(owned))
}

struct CountingSink;

impl Sink for CountingSink {
    fn write_bytes(&mut self, _bytes: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

struct StructuralTerms<'a>(&'a [(&'a NodeKey, &'a Integer)]);

impl Encode for StructuralTerms<'_> {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("terms")?;
        writer.begin_array()?;
        for (unit, exponent) in self.0 {
            writer.begin_object()?;
            writer.name("exponent")?;
            writer.string(&exponent.to_string())?;
            writer.name("unit")?;
            encode_node_id(writer, unit)?;
            writer.end_object()?;
        }
        writer.end_array()?;
        writer.end_object()
    }
}

/// Emit a nested structural node id through an explicit stack. A path may
/// contain node ids, so recursive serialization would use input-sized stack.
fn encode_node_id<S: Sink + ?Sized>(
    writer: &mut Writer<'_, S>,
    node: &NodeKey,
) -> Result<(), Error> {
    enum Event<'a> {
        Node(NodeKeyView<'a>),
        Text(&'a str),
        EndArray,
        EndObject,
    }
    let mut events = Vec::new();
    events.try_reserve(1).map_err(|_| Error::Allocation {
        requested: core::mem::size_of::<Event>(),
    })?;
    events.push(Event::Node(node.view()));
    while let Some(event) = events.pop() {
        match event {
            Event::Node(node) => {
                writer.begin_object()?;
                writer.name("ordinal")?;
                writer.integer(i128::from(node.ordinal()))?;
                writer.name("package")?;
                writer.string(node.package())?;
                writer.name("path")?;
                writer.begin_array()?;
                let additional = node.path().count().saturating_add(2);
                events
                    .try_reserve(additional)
                    .map_err(|_| Error::Allocation {
                        requested: events
                            .len()
                            .saturating_add(additional)
                            .saturating_mul(core::mem::size_of::<Event>()),
                    })?;
                let path_start = events.len().saturating_add(2);
                events.push(Event::EndObject);
                events.push(Event::EndArray);
                for segment in node.path() {
                    events.push(match segment {
                        PathSegmentView::Text(text) => Event::Text(text),
                        PathSegmentView::Node(node) => Event::Node(node),
                    });
                }
                events[path_start..].reverse();
            }
            Event::Text(text) => writer.string(text)?,
            Event::EndArray => writer.end_array()?,
            Event::EndObject => writer.end_object()?,
        }
    }
    Ok(())
}

/// A normalized compound unit: canonical root-unit keys to nonzero exponents.
/// The empty map is the sole dimensionless unit. Two compound units are equal
/// exactly when their terms are, which is also when their ids are.
#[derive(Clone, Debug)]
pub struct CompoundUnit {
    terms: BTreeMap<NodeKey, Integer>,
    dimension: Dimension,
}

impl PartialEq for CompoundUnit {
    fn eq(&self, other: &Self) -> bool {
        self.terms == other.terms
    }
}

impl Eq for CompoundUnit {}

impl Hash for CompoundUnit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.terms.hash(state);
    }
}

impl CompoundUnit {
    fn new(terms: BTreeMap<NodeKey, Integer>, dimension: Dimension) -> Self {
        Self { terms, dimension }
    }

    /// The single-term compound unit `root^1` of a declared unit's root.
    pub(crate) fn of_root(unit: &Unit) -> Self {
        Self::new(
            [(unit.root.clone(), Integer::one())].into(),
            unit.dimension.clone(),
        )
    }

    /// Ascending `(root unit, exponent)` terms.
    pub fn terms(&self) -> impl Iterator<Item = (NodeKey, &Integer)> {
        self.terms
            .iter()
            .map(|(key, exponent)| (key.clone(), exponent))
    }

    /// The normalized dimension map.
    pub fn dimension(&self) -> &Dimension {
        &self.dimension
    }

    /// The structural compound-arm [`UnitId`] after its ordered terms pass
    /// the [`IDENTITY_LIMITS`] encoding bound.
    ///
    /// # Errors
    ///
    /// As [`compound_unit_id`].
    pub fn id(&self) -> Result<UnitId, IdentityRefusal> {
        compound_unit_id(self.terms.iter(), IDENTITY_LIMITS)
    }

    pub(crate) fn multiply(&self, other: &Self) -> Self {
        Self::new(
            combine(&self.terms, &other.terms, Integer::add),
            self.dimension.multiply(&other.dimension),
        )
    }

    pub(crate) fn divide(&self, other: &Self) -> Self {
        Self::new(
            combine(&self.terms, &other.terms, Integer::sub),
            self.dimension.divide(&other.dimension),
        )
    }

    pub(crate) fn power(&self, exponent: &Integer) -> Self {
        Self::new(
            scale_exponents(&self.terms, exponent),
            self.dimension.power(exponent),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use ix_trace_rs::trace;

    fn id(byte: u8) -> NodeKey {
        crate::test_identity::node(u64::from(byte))
    }

    /// QSL FR-259 Behavior 4: a compound-unit preimage over the byte limit
    /// it is encoded under refuses with that limit and the bytes it needed,
    /// never a panic and never a malformed value. Under the default limit
    /// the same terms have an id.
    #[trace("TC-728", "FR-259-AC-2")]
    #[test]
    fn a_compound_unit_over_the_identity_byte_limit_refuses_with_the_limit() {
        let (one, two) = (Integer::from(1_i64), Integer::from(-2_i64));
        let first = id(1);
        let second = id(2);
        let terms = || [(&first, &one), (&second, &two)];
        assert!(compound_unit_id(terms(), IDENTITY_LIMITS).is_ok());
        let Err(IdentityRefusal::InputBytes { bound, required }) =
            compound_unit_id(terms(), Limits::new(64))
        else {
            panic!("a two-term preimage is longer than 64 bytes");
        };
        assert_eq!(bound, 64);
        assert!(required > 64, "the refusal names the bytes it needed");
    }

    /// A nested structural path is measured as the complete FR-201 node form.
    #[trace("TC-728", "FR-259-AC-2")]
    #[test]
    fn a_nested_node_path_is_measured_before_carrying_the_compound_id() {
        let outer = crate::test_identity::nested_node();
        let exponent = Integer::one();
        let canonical = br#"{"terms":[{"exponent":"1","unit":{"ordinal":2,"package":"P","path":[{"ordinal":1,"package":"P","path":["inner"]},"outer"]}}]}"#;
        let required = u64::try_from(canonical.len()).unwrap();
        assert_eq!(
            compound_unit_id([(&outer, &exponent)], Limits::new(required)),
            Ok(UnitId::compound(vec![(outer.clone(), exponent.clone())]))
        );
        assert_eq!(
            compound_unit_id([(&outer, &exponent)], Limits::new(required - 1)),
            Err(IdentityRefusal::InputBytes {
                bound: required - 1,
                required,
            })
        );
    }

    fn rational(numerator: i64) -> Rational {
        Rational::from_integer(Integer::from(numerator))
    }

    /// A dimension node cannot hold terms `check_terms` refuses, so
    /// `UnitGraph::from_checked_nodes` never sees a zero exponent, a repeat
    /// or unsorted terms; the refusals come in `check_terms`' order.
    #[test]
    fn a_dimension_node_refuses_zero_then_repeated_then_unsorted_terms() {
        let term = |key, exponent: i64| (id(key), Integer::from(exponent));
        assert!(DimensionNode::checked(
            vec![term(1, 2), term(2, -1)],
            NominalDeclaration::default()
        )
        .is_ok());
        assert_eq!(
            DimensionNode::checked(
                vec![term(2, 1), term(1, 0), term(1, 1)],
                NominalDeclaration::default()
            ),
            Err(SemanticGraphCause::ZeroExponent)
        );
        assert_eq!(
            DimensionNode::checked(vec![term(1, 1), term(1, 1)], NominalDeclaration::default()),
            Err(SemanticGraphCause::DuplicateTerm)
        );
        assert_eq!(
            DimensionNode::checked(vec![term(2, 1), term(1, 1)], NominalDeclaration::default()),
            Err(SemanticGraphCause::UnsortedTerms)
        );
    }

    /// A unit node refuses a zero scale before a non-identity root, and its
    /// root-ness is its own `target`, so the edge check cannot disagree with
    /// the node it is stored in.
    #[test]
    fn a_unit_node_refuses_a_zero_scale_then_a_non_identity_root() {
        assert_eq!(
            UnitNode::checked(
                id(1),
                None,
                rational(0),
                rational(5),
                NominalDeclaration::default()
            ),
            Err(SemanticGraphCause::ZeroScale)
        );
        assert_eq!(
            UnitNode::checked(
                id(1),
                None,
                rational(2),
                rational(0),
                NominalDeclaration::default()
            ),
            Err(SemanticGraphCause::NonIdentityRoot)
        );
        assert!(UnitNode::checked(
            id(1),
            None,
            rational(1),
            rational(0),
            NominalDeclaration::default()
        )
        .is_ok());
        assert!(UnitNode::checked(
            id(1),
            Some(id(2)),
            rational(2),
            rational(3),
            NominalDeclaration::default()
        )
        .is_ok());
    }
}
