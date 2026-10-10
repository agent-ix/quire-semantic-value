// SPDX-License-Identifier: AGPL-3.0-or-later
//! QSpec FR-143 finite-value construction from a containment graph.
//!
//! A [`ValueGraph`] names constructor nodes whose slots may contain other
//! nodes. Building a root admits nodes bottom-up: a node reached twice becomes
//! one shared immutable value (a DAG), and a containment back-edge refuses at
//! the node that closes the cycle. Sharing never creates object identity.

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::declaration::{Component, ConstructionCause, ConstructionRefusal, EnvironmentLimit, TypeEnvironment, WorkBudget};
use quire_exact::{FieldValue, NodeKey, Value, VariantId};

/// A graph-local node name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GraphNodeId(pub u64);

/// One slot of a graph node.
#[derive(Clone, Debug)]
pub enum GraphSlot {
    /// An already completed value.
    Value(Value),
    /// The value built from another graph node.
    Node(GraphNodeId),
    /// Explicit absence.
    Absent,
    /// Explicit `null`.
    Null,
}

/// A constructor node.
#[derive(Clone, Debug)]
pub enum GraphNode {
    /// A record of `declaration`.
    Record {
        /// The record declaration.
        declaration: NodeKey,
        /// Supplied fields by name.
        fields: Vec<(String, GraphSlot)>,
    },
    /// One union member of `declaration`, resolved by its supplied member key.
    Union {
        /// The union declaration.
        declaration: NodeKey,
        /// The active member's verified key bytes.
        variant: VariantId,
        /// Supplied payload positions.
        positions: Vec<GraphSlot>,
    },
    /// A tuple of `declaration`.
    Tuple {
        /// The tuple declaration.
        declaration: NodeKey,
        /// Supplied positions.
        positions: Vec<GraphSlot>,
    },
}

impl GraphNode {
    fn children(&self) -> impl Iterator<Item = GraphNodeId> + '_ {
        let slots: Box<dyn Iterator<Item = &GraphSlot>> = match self {
            Self::Record { fields, .. } => Box::new(fields.iter().map(|(_, slot)| slot)),
            Self::Tuple { positions, .. } | Self::Union { positions, .. } => {
                Box::new(positions.iter())
            }
        };
        slots.filter_map(|slot| match slot {
            GraphSlot::Node(id) => Some(*id),
            GraphSlot::Value(_) | GraphSlot::Absent | GraphSlot::Null => None,
        })
    }
}

/// A named containment graph.
#[derive(Clone, Debug, Default)]
pub struct ValueGraph {
    nodes: BTreeMap<GraphNodeId, GraphNode>,
}

/// A graph construction refusal at its originating node.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("value graph refused at node {node:?}: {cause:?}")]
pub struct GraphRefusal {
    /// The originating node.
    pub node: GraphNodeId,
    /// The typed cause.
    pub cause: GraphCause,
}

/// The typed cause of a [`GraphRefusal`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphCause {
    /// Two nodes share one name.
    DuplicateNode,
    /// A slot or the root names no node.
    UnknownNode,
    /// This node contains itself through value containment.
    ContainmentCycle,
    /// The node's constructor refuses.
    Construction(ConstructionRefusal),
}

impl ValueGraph {
    /// A graph from named nodes.
    pub fn new(
        nodes: impl IntoIterator<Item = (GraphNodeId, GraphNode)>,
    ) -> Result<Self, GraphRefusal> {
        let mut admitted = BTreeMap::new();
        for (id, node) in nodes {
            if admitted.insert(id, node).is_some() {
                return Err(GraphRefusal {
                    node: id,
                    cause: GraphCause::DuplicateNode,
                });
            }
        }
        Ok(Self { nodes: admitted })
    }
}

enum Visit {
    Enter(GraphNodeId),
    Exit(GraphNodeId),
}

impl TypeEnvironment {
    /// Build the finite value with one caller-owned cumulative budget.
    /// Work stops are separate from the original graph refusal and node locus.
    pub fn build(
        &self,
        graph: &ValueGraph,
        root: GraphNodeId,
        budget: &mut WorkBudget,
    ) -> Result<Result<Value, GraphRefusal>, EnvironmentLimit> {
        budget.charge(1)?;
        let mut built: BTreeMap<GraphNodeId, Value> = BTreeMap::new();
        let mut on_path = BTreeSet::new();
        let mut visits = vec![Visit::Enter(root)];
        while let Some(visit) = visits.pop() {
            budget.charge(1)?;
            match visit {
                Visit::Enter(id) => {
                    if built.contains_key(&id) { continue; }
                    if on_path.contains(&id) {
                        return Ok(Err(GraphRefusal { node: id, cause: GraphCause::ContainmentCycle }));
                    }
                    let Some(node) = graph.nodes.get(&id) else {
                        return Ok(Err(GraphRefusal { node: id, cause: GraphCause::UnknownNode }));
                    };
                    on_path.insert(id);
                    visits.push(Visit::Exit(id));
                    for child in node.children() {
                        budget.charge(1)?;
                        visits.push(Visit::Enter(child));
                    }
                }
                Visit::Exit(id) => {
                    on_path.remove(&id);
                    let Some(node) = graph.nodes.get(&id) else {
                        return Ok(Err(GraphRefusal { node: id, cause: GraphCause::UnknownNode }));
                    };
                    match self.construct(node, &built, budget)? {
                        Ok(value) => { built.insert(id, value); }
                        Err(cause) => return Ok(Err(GraphRefusal { node: id, cause })),
                    }
                }
            }
        }
        Ok(built.remove(&root).ok_or(GraphRefusal { node: root, cause: GraphCause::UnknownNode }))
    }

    fn construct<'g>(
        &self,
        node: &'g GraphNode,
        built: &BTreeMap<GraphNodeId, Value>,
        budget: &mut WorkBudget,
    ) -> Result<Result<Value, GraphCause>, EnvironmentLimit> {
        let resolve = |slot: &GraphSlot| match slot {
            GraphSlot::Value(value) => Ok(FieldValue::Present(value.clone())),
            GraphSlot::Node(id) => built
                .get(id)
                .map(|value| FieldValue::Present(value.clone()))
                .ok_or(GraphCause::UnknownNode),
            GraphSlot::Absent => Ok(FieldValue::Absent),
            GraphSlot::Null => Ok(FieldValue::Null),
        };
        let fields = |fields: &'g [(String, GraphSlot)]| {
            fields
                .iter()
                .map(|(name, slot)| resolve(slot).map(|value| (name.as_str(), value)))
                .collect::<Result<Vec<_>, _>>()
        };
        let union_variant = match node {
            GraphNode::Union { variant, .. } => Some(*variant),
            GraphNode::Record { .. } | GraphNode::Tuple { .. } => None,
        };
        let constructed = match node {
            GraphNode::Record {
                declaration,
                fields: supplied,
            } => {
                budget.charge(supplied.len())?;
                let supplied = match fields(supplied) {
                    Ok(fields) => fields,
                    Err(cause) => return Ok(Err(cause)),
                };
                self.record(*declaration, supplied, budget)?
            },
            GraphNode::Tuple {
                declaration,
                positions,
            }
            | GraphNode::Union {
                declaration,
                positions,
                ..
            } => {
                budget.charge(positions.len())?;
                let mut values = Vec::with_capacity(positions.len());
                for (index, slot) in positions.iter().enumerate() {
                    let resolved = match resolve(slot) {
                        Ok(slot) => slot,
                        Err(cause) => return Ok(Err(cause)),
                    };
                    let cause = match resolved {
                        FieldValue::Present(value) => {
                            values.push(value);
                            continue;
                        }
                        FieldValue::Absent | FieldValue::Null => ConstructionCause::TypeMismatch,
                    };
                    return Ok(Err(GraphCause::Construction(ConstructionRefusal {
                        component: Component::Position(index),
                        cause,
                    })));
                }
                match union_variant {
                    Some(variant) => self.union(*declaration, variant, values, budget)?,
                    None => self.tuple(*declaration, values, budget)?,
                }
            }
        };
        Ok(constructed.map_err(GraphCause::Construction))
    }
}
