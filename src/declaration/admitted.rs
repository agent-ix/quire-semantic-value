// SPDX-License-Identifier: AGPL-3.0-or-later
//! Retained producer decisions used without repeating semantic checking.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use quire_exact::{Integer, Location, NodeKey, Origin, Rational, UnitId, ValueType, VariantId};

use super::{CheckedEquality, EqualityOperand, EqualityOperator, EqualitySchedule};
use crate::enumeration::{EnumDeclaration, EnumMemberIndex};
use crate::quantity::UnitTable;

/// A selected affine conversion edge, in original source-to-root order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EqualityUnitEdgeDescriptor {
    /// Original target unit identity.
    pub target: NodeKey,
    /// Exact multiplicative conversion factor.
    pub scale: Rational,
    /// Exact additive conversion offset.
    pub offset: Rational,
}

/// Producer-retained facts of a resolved unit; these descriptors grant no admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EqualityUnitForm {
    /// A declared unit with its already selected conversion path.
    Declared {
        /// Original unit node identity.
        key: NodeKey,
        /// Original dimension declaration.
        dimension_node: NodeKey,
        /// Original normalized base-dimension powers.
        dimension_terms: Vec<(NodeKey, Integer)>,
        /// Original canonical root identity.
        root: NodeKey,
        /// Original conversion edges, including their target identities.
        path: Vec<EqualityUnitEdgeDescriptor>,
    },
    /// A compound unit with its already normalized dimensions and root powers.
    Compound {
        /// Original normalized base-dimension powers.
        dimension_terms: Vec<(NodeKey, Integer)>,
        /// Original ordered canonical root powers.
        terms: Vec<(NodeKey, Integer)>,
    },
}

/// One original resolved-unit row used by the equality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EqualityUnitDescriptor {
    /// Original quantity type node.
    pub quantity_type: NodeKey,
    /// Original producer-issued identity; materialization never recomputes it.
    pub id: UnitId,
    /// Already-admitted declared or compound facts.
    pub form: EqualityUnitForm,
}

/// An already-admitted declaration, preserving even non-identifier UTF-8 cases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EqualityEnumDeclarationDescriptor {
    /// Original declaration identity.
    pub key: NodeKey,
    /// Producer-selected ordering semantics.
    pub ordered: bool,
    /// Original member names in their admitted order.
    pub members: Vec<String>,
}

/// One original member of the compared enum's already-filtered index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EqualityEnumMemberDescriptor {
    /// Original producer-issued variant identity.
    pub variant: VariantId,
    /// Original declaration identity.
    pub declaration: NodeKey,
    /// Original member identity.
    pub member: NodeKey,
    /// Producer-selected ordering semantics.
    pub ordered: bool,
    /// Original position, joined to the native witness without a lossy cast.
    pub position: usize,
    /// Original case bytes, never normalized or revalidated here.
    pub case: Box<str>,
}

/// Already-admitted native facts supplied to the reader's privileged mint.
///
/// This is data, not an admission capability. Only the audited reader boundary
/// turns it into an [`AdmittedEqualityDecision`]. Operand types must be the
/// original joined Exact witnesses, including Text, Enum, Quantity and Plan
/// forms; a partial type projection is not a substitute for a missing witness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EqualityDecisionDescriptor {
    /// Original equality application and exact occurrence.
    pub location: Location,
    /// Producer-selected operator.
    pub operator: EqualityOperator,
    /// Original left source type and optional conversion target.
    pub left: EqualityOperand,
    /// Original right source type and optional conversion target.
    pub right: EqualityOperand,
    /// Producer-selected comparison procedure.
    pub schedule: EqualitySchedule,
    /// Full selected unit facts in source order.
    pub units: Vec<EqualityUnitDescriptor>,
    /// Full compared declarations in source order.
    pub enum_declarations: Vec<EqualityEnumDeclarationDescriptor>,
    /// Already-filtered member facts in source order.
    pub enum_members: Vec<EqualityEnumMemberDescriptor>,
}

/// A reader-retained equality decision with private construction fields.
///
/// The supported consumer path reads this value from the admitted IR reader
/// and passes it to [`CheckedEquality::from_admitted_decision`]. The separate
/// hidden public mint is an audited API convention: another Rust crate can
/// call it with a seal. Field privacy does **not** establish language-level
/// cross-crate unforgeability. IR and QSL own structural API/dependency audits.
///
/// An external caller cannot construct the retained value from raw fields:
/// ```compile_fail,E0451
/// use quire_semantic_value::declaration::{AdmittedEqualityDecision, CheckedEquality};
/// fn forge(checked: CheckedEquality) -> AdmittedEqualityDecision {
///     AdmittedEqualityDecision {
///         checked,
///         location: unimplemented!(),
///         units: unimplemented!(),
///         enum_declarations: unimplemented!(),
///         enum_members: unimplemented!(),
///     }
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedEqualityDecision {
    checked: CheckedEquality,
    location: Location,
    units: Vec<EqualityUnitDescriptor>,
    enum_declarations: Vec<EnumDeclaration>,
    enum_members: Vec<EqualityEnumMemberDescriptor>,
}

impl AdmittedEqualityDecision {
    /// Original equality application identity.
    pub fn application(&self) -> &NodeKey {
        self.location.node()
    }
    /// Original occurrence role and ordinal.
    pub fn occurrence(&self) -> &Origin {
        self.location.occurrence()
    }
    /// Original complete lower-layer location.
    pub fn location(&self) -> &Location {
        &self.location
    }
    /// Producer-selected operator.
    pub fn operator(&self) -> EqualityOperator {
        self.checked.operator
    }
    /// Original left operand and conversion choice.
    pub fn left(&self) -> &EqualityOperand {
        &self.checked.left
    }
    /// Original right operand and conversion choice.
    pub fn right(&self) -> &EqualityOperand {
        &self.checked.right
    }
    /// Producer-selected schedule.
    pub fn schedule(&self) -> EqualitySchedule {
        self.checked.schedule
    }
    /// Full original unit rows, identities, dimensions and conversion paths.
    pub fn units(&self) -> &[EqualityUnitDescriptor] {
        &self.units
    }
    /// Original admitted declarations, with verbatim case bytes and ordering.
    pub fn enum_declarations(&self) -> &[EnumDeclaration] {
        &self.enum_declarations
    }
    /// Full original filtered member rows.
    pub fn enum_members(&self) -> &[EqualityEnumMemberDescriptor] {
        &self.enum_members
    }
}

impl EqualityOperand {
    /// Original source type, before any selected conversion.
    pub fn source(&self) -> &ValueType {
        &self.source
    }
}

impl CheckedEquality {
    /// Adopt the completed retained value without checking, folding, selecting
    /// a schedule, filtering members or consulting a type environment.
    pub fn from_admitted_decision(decision: &AdmittedEqualityDecision) -> Self {
        decision.checked.clone()
    }
}

pub(crate) fn materialize_admitted(
    descriptor: EqualityDecisionDescriptor,
    units: UnitTable,
    members: EnumMemberIndex,
    declarations: Vec<EnumDeclaration>,
) -> AdmittedEqualityDecision {
    AdmittedEqualityDecision {
        checked: CheckedEquality {
            operator: descriptor.operator,
            left: descriptor.left,
            right: descriptor.right,
            schedule: descriptor.schedule,
            units,
            enum_members: members,
        },
        location: descriptor.location,
        units: descriptor.units,
        enum_declarations: declarations,
        enum_members: descriptor.enum_members,
    }
}
