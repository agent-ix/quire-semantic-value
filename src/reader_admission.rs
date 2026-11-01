// SPDX-License-Identifier: AGPL-3.0-or-later
//! Privileged IR-reader materialization. This module is intentionally hidden
//! from ordinary consumer documentation and is never re-exported by a facade.
//! Its public seal/mint remain callable by other Rust crates: the boundary is
//! an audited convention, not language-level cross-crate unforgeability.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Layout;
use quire_exact::{
    Charge, ChargePoint, Incomplete, Integer, LimitKind, Meter, NodeKey, Rational,
    RationalArithmetic,
};

use crate::declaration::{
    materialize_admitted, AdmittedEqualityDecision, EqualityDecisionDescriptor,
    EqualityUnitDescriptor, EqualityUnitForm,
};
use crate::enumeration::{EnumDeclaration, EnumMemberIndex, EnumValue};
use crate::quantity::{rational_event, QuantityUnit, UnitTable};
use crate::stop::Stop;
use crate::unit::{AffineArithmetic, CompoundUnit, Dimension, Unit, UnitEdge};

/// The IR reader's convention seal. This public item is not an unforgeable
/// Rust capability; structural API/dependency audits constrain supported use.
#[derive(Clone, Copy, Debug)]
pub struct ReaderSeal;

/// A mechanical affine fold event, never a semantic checking step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AffineStep {
    /// Visiting the original edge before using it.
    Edge,
    /// Multiplying the original scale by the accumulated scale.
    Scale,
    /// Multiplying the accumulated offset by the original scale.
    OffsetScale,
    /// Adding the original offset.
    OffsetAdd,
}

/// The original descriptor position where materialization stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EqualityMaterializationLocus {
    /// A resolved-unit row.
    Unit {
        /// Original row index.
        row: usize,
    },
    /// A normalized dimension term.
    DimensionTerm {
        /// Original unit row.
        row: usize,
        /// Original term index.
        term: usize,
    },
    /// A canonical compound root term.
    CompoundTerm {
        /// Original unit row.
        row: usize,
        /// Original term index.
        term: usize,
    },
    /// An original affine conversion edge and mechanical event.
    Edge {
        /// Original unit row.
        row: usize,
        /// Original edge index.
        edge: usize,
        /// First unpaid or failed event.
        step: AffineStep,
    },
    /// An original enum declaration or one of its cases.
    EnumDeclaration {
        /// Original declaration index.
        row: usize,
        /// Original case index, or the declaration itself.
        case: Option<usize>,
    },
    /// An original already-filtered enum member.
    EnumMember {
        /// Original member index.
        row: usize,
    },
}

/// Stable classification of a materialization failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EqualityMaterializationCode {
    /// Caller read-materialization allowance was exhausted.
    Resource,
    /// Exact arithmetic could not produce a value.
    Arithmetic,
    /// A real reservation failed.
    Allocation,
    /// A native allocation/count could not be represented.
    Capacity,
}

/// No opaque or partial decision is returned on these failures. The caller's
/// read meter retains its admitted prefix and original first denied charge.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum EqualityMaterializationError {
    /// Original typed meter denial, without a manufactured semantic refusal.
    #[error("equality materialization exhausted at {locus:?}: {cause:?}")]
    Resource {
        /// Original descriptor/event position.
        locus: EqualityMaterializationLocus,
        /// Original scalar resource record.
        cause: Incomplete,
    },
    /// Original exact arithmetic stop, without a fallback value.
    #[error("equality materialization arithmetic failed at {locus:?}: {cause:?}")]
    Arithmetic {
        /// Original descriptor/event position.
        locus: EqualityMaterializationLocus,
        /// Original computation stop.
        cause: Stop,
    },
    /// A fallible reservation's actual additional byte count.
    #[error("equality materialization cannot reserve {requested} bytes at {locus:?}")]
    Allocation {
        /// Original descriptor/event position.
        locus: EqualityMaterializationLocus,
        /// Actual requested bytes.
        requested: usize,
    },
    /// Count/layout arithmetic was impossible before reservation.
    #[error("equality materialization capacity overflow at {locus:?} for {count} entries")]
    Capacity {
        /// Original descriptor/event position.
        locus: EqualityMaterializationLocus,
        /// Unrepresentable entry count.
        count: usize,
    },
}

impl EqualityMaterializationError {
    /// Stable machine-readable classification.
    pub fn code(&self) -> EqualityMaterializationCode {
        match self {
            Self::Resource { .. } => EqualityMaterializationCode::Resource,
            Self::Arithmetic { .. } => EqualityMaterializationCode::Arithmetic,
            Self::Allocation { .. } => EqualityMaterializationCode::Allocation,
            Self::Capacity { .. } => EqualityMaterializationCode::Capacity,
        }
    }
}

fn charge(
    meter: &mut Meter,
    event: Charge,
    locus: EqualityMaterializationLocus,
) -> Result<(), EqualityMaterializationError> {
    meter
        .charge(event)
        .map_err(|cause| EqualityMaterializationError::Resource { locus, cause })
}

fn reserved<T>(
    count: usize,
    locus: EqualityMaterializationLocus,
) -> Result<Vec<T>, EqualityMaterializationError> {
    let layout = Layout::array::<T>(count)
        .map_err(|_| EqualityMaterializationError::Capacity { locus, count })?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| EqualityMaterializationError::Allocation {
            locus,
            requested: layout.size(),
        })?;
    Ok(values)
}

#[derive(Clone, Copy)]
enum TermKind {
    Dimension,
    Compound,
}

fn terms(
    source: &[(NodeKey, Integer)],
    row: usize,
    kind: TermKind,
    meter: &mut Meter,
) -> Result<Vec<(NodeKey, Integer)>, EqualityMaterializationError> {
    let mut values = reserved(source.len(), EqualityMaterializationLocus::Unit { row })?;
    for (term, (key, exponent)) in source.iter().enumerate() {
        let locus = match kind {
            TermKind::Compound => EqualityMaterializationLocus::CompoundTerm { row, term },
            TermKind::Dimension => EqualityMaterializationLocus::DimensionTerm { row, term },
        };
        charge(
            meter,
            Charge::new(ChargePoint::UnitIdentityRead)
                .size(LimitKind::IntegerBits, exponent.magnitude_bits()),
            locus,
        )?;
        let exponent = exponent.try_clone().map_err(|error| match error {
            quire_exact::IntegerCopyError::Allocation { requested } => {
                EqualityMaterializationError::Allocation { locus, requested }
            }
        })?;
        values.push((key.clone(), exponent));
    }
    Ok(values)
}

struct MeteredAffine<'a> {
    meter: &'a mut Meter,
    row: usize,
    edge: usize,
    multiplication: usize,
}

impl MeteredAffine<'_> {
    fn event(
        &mut self,
        operation: RationalArithmetic<'_>,
        step: AffineStep,
    ) -> Result<Rational, EqualityMaterializationError> {
        let locus = EqualityMaterializationLocus::Edge {
            row: self.row,
            edge: self.edge,
            step,
        };
        rational_event(operation, self.meter).map_err(|cause| match cause {
            Stop::Incomplete(cause) => EqualityMaterializationError::Resource { locus, cause },
            cause @ (Stop::Undefined(_) | Stop::Refused(_)) => {
                EqualityMaterializationError::Arithmetic { locus, cause }
            }
        })
    }
}

impl AffineArithmetic for MeteredAffine<'_> {
    type Error = EqualityMaterializationError;
    fn multiply(&mut self, left: &Rational, right: &Rational) -> Result<Rational, Self::Error> {
        let step = if self.multiplication == 0 {
            AffineStep::Scale
        } else {
            AffineStep::OffsetScale
        };
        self.multiplication += 1;
        self.event(RationalArithmetic::Multiply(left, right), step)
    }
    fn add(&mut self, left: &Rational, right: &Rational) -> Result<Rational, Self::Error> {
        self.event(RationalArithmetic::Add(left, right), AffineStep::OffsetAdd)
    }
}

fn unit(
    source: &EqualityUnitDescriptor,
    row: usize,
    edges: &mut u64,
    meter: &mut Meter,
) -> Result<QuantityUnit, EqualityMaterializationError> {
    let locus = EqualityMaterializationLocus::Unit { row };
    charge(meter, Charge::new(ChargePoint::UnitIdentityRead), locus)?;
    match &source.form {
        EqualityUnitForm::Declared {
            key,
            dimension_node,
            dimension_terms,
            root,
            path,
        } => {
            let dimension =
                Dimension::from_admitted(terms(dimension_terms, row, TermKind::Dimension, meter)?);
            let mut native_path = reserved(path.len(), locus)?;
            let mut canonical = UnitEdge::identity();
            for (edge, original) in path.iter().enumerate() {
                let locus = EqualityMaterializationLocus::Edge {
                    row,
                    edge,
                    step: AffineStep::Edge,
                };
                *edges = edges
                    .checked_add(1)
                    .ok_or(EqualityMaterializationError::Capacity {
                        locus,
                        count: usize::MAX,
                    })?;
                charge(
                    meter,
                    Charge::new(ChargePoint::UnitEdge)
                        .size(LimitKind::UnitEdges, *edges)
                        .size(
                            LimitKind::IntegerBits,
                            original
                                .scale
                                .max_part_bits()
                                .max(original.offset.max_part_bits()),
                        ),
                    locus,
                )?;
                let native =
                    UnitEdge::from_admitted(original.scale.clone(), original.offset.clone());
                canonical = canonical.followed_by(
                    &native,
                    &mut MeteredAffine {
                        meter,
                        row,
                        edge,
                        multiplication: 0,
                    },
                )?;
                native_path.push(native);
            }
            Ok(QuantityUnit::Declared(Box::new(Unit::from_admitted(
                key.clone(),
                dimension_node.clone(),
                dimension,
                root.clone(),
                native_path,
                canonical,
            ))))
        }
        EqualityUnitForm::Compound {
            dimension_terms,
            terms: roots,
        } => {
            let dimension =
                Dimension::from_admitted(terms(dimension_terms, row, TermKind::Dimension, meter)?);
            let roots = terms(roots, row, TermKind::Compound, meter)?;
            Ok(QuantityUnit::Compound(CompoundUnit::from_admitted(
                roots, dimension,
            )))
        }
    }
}

/// Materialize only facts already joined/admitted by the IR reader.
///
/// `meter` is the **same remaining read allowance** threaded by the caller,
/// never the runtime evaluation meter. Only the selected affine path is folded
/// to fill the native `Unit.canonical`; no identity, path, schedule, conversion
/// choice or membership is checked, selected, normalized or synthesized.
///
/// This is a public hidden convention surface, not a language-level capability.
/// Ordinary callers consume the reader's opaque value instead. Neither this
/// function nor its seal may be re-exported through IR/QSL consumer facades.
///
/// # Errors
/// Original typed resource/arithmetic failures retain their first unit/edge
/// position. Explicit vector/count reservation failures are typed separately.
/// No partial admitted decision is published.
pub fn mint_admitted_equality_decision(
    _seal: ReaderSeal,
    mut descriptor: EqualityDecisionDescriptor,
    meter: &mut Meter,
) -> Result<AdmittedEqualityDecision, EqualityMaterializationError> {
    let mut units = UnitTable::default();
    let mut edges = 0;
    for (row, original) in descriptor.units.iter().enumerate() {
        units.insert_held(original.id.clone(), unit(original, row, &mut edges, meter)?);
    }
    let mut declarations = reserved(
        descriptor.enum_declarations.len(),
        EqualityMaterializationLocus::EnumDeclaration { row: 0, case: None },
    )?;
    for (row, original) in core::mem::take(&mut descriptor.enum_declarations)
        .into_iter()
        .enumerate()
    {
        charge(
            meter,
            Charge::new(ChargePoint::EnumIdentityRead),
            EqualityMaterializationLocus::EnumDeclaration { row, case: None },
        )?;
        for (case, _) in original.members.iter().enumerate() {
            charge(
                meter,
                Charge::new(ChargePoint::EnumIdentityRead),
                EqualityMaterializationLocus::EnumDeclaration {
                    row,
                    case: Some(case),
                },
            )?;
        }
        declarations.push(EnumDeclaration::from_admitted(
            original.key,
            original.ordered,
            original.members,
        ));
    }
    let mut members = EnumMemberIndex::default();
    for (row, original) in descriptor.enum_members.iter().enumerate() {
        charge(
            meter,
            Charge::new(ChargePoint::EnumIdentityRead),
            EqualityMaterializationLocus::EnumMember { row },
        )?;
        let value = EnumValue::from_admitted(
            original.declaration.clone(),
            original.member.clone(),
            original.ordered,
            original.position,
            original.case.clone(),
        );
        members.record_admitted(original.variant.clone(), value);
    }
    Ok(materialize_admitted(
        descriptor,
        units,
        members,
        declarations,
    ))
}
