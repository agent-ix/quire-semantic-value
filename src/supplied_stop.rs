// SPDX-License-Identifier: AGPL-3.0-or-later
//! The supplied-value stop records (FR-277, QSpec FR-323): the phase, event,
//! locus, limit, storage and cause a pre-call stop carries, and the failure
//! that wraps them.
//!
//! The budgets and the run that produce these records stay with the crate
//! that walks supplied values; the records live here so the evaluation
//! interface can carry them in a call failure without depending on it.

use alloc::boxed::Box;
use alloc::vec::Vec;

use quire_exact::{CancelCause, NumericMembershipOperand, NumericTypeEndpoint};

use crate::call::InputRefusal;
use crate::catalog::InternalFault;
use crate::supplied_membership::{
    DecimalMembershipOperand, InvalidMembershipCause, MembershipPart, PathStep,
    UnsupportedHelperFamily,
};

/// The setting name of A, FR-277's caller-owned admission ceiling.
pub const ADMISSION_WORK_UNITS: &str = "supplied.admission_work_units";

/// The setting name of H, FR-277's caller-owned conversion ceiling.
pub const CONVERSION_WORK_UNITS: &str = "supplied.conversion_work_units";

/// The setting name of B, FR-277's caller-owned numeric magnitude bound.
pub const NUMERIC_MAGNITUDE_BITS: &str = "supplied.numeric_magnitude_bits";

/// The phase that initiated a supplied-value event and owns its budget.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppliedPhase {
    /// Direct or final checked-call membership, charged to A.
    Admission,
    /// Replay witness conversion, charged to H.
    Conversion,
}

impl SuppliedPhase {
    /// The wire spelling of the phase.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admission => "admission",
            Self::Conversion => "conversion",
        }
    }

    /// The caller field that bounds this phase's helper work.
    #[must_use]
    pub const fn field(self) -> &'static str {
        match self {
            Self::Admission => ADMISSION_WORK_UNITS,
            Self::Conversion => CONVERSION_WORK_UNITS,
        }
    }
}

/// The actual logical event a supplied-phase stop happened at (QSpec FR-323
/// "Pre-call result context").
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppliedEvent {
    /// QSV: one runtime occurrence visit.
    ValueVisit,
    /// QSV: one compared pair of type-chain links.
    TypeComparison,
    /// QSV: one child scheduling operation.
    ChildScheduling,
    /// QSV or conversion: one descriptor query.
    DescriptorQuery,
    /// Conversion: reading and locally validating one witness node.
    WitnessPreflight,
    /// Conversion: constructing one converted kernel node.
    WitnessMaterialization,
    /// FR-098's per-argument converted-node counter C.
    ConvertedNode,
    /// The per-argument semantic `value_occurrences` counter.
    ValueOccurrence,
}

impl SuppliedEvent {
    /// The wire spelling of the event.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ValueVisit => "value-visit",
            Self::TypeComparison => "type-comparison",
            Self::ChildScheduling => "child-scheduling",
            Self::DescriptorQuery => "descriptor-query",
            Self::WitnessPreflight => "witness-preflight",
            Self::WitnessMaterialization => "witness-materialization",
            Self::ConvertedNode => "converted-node",
            Self::ValueOccurrence => "value-occurrence",
        }
    }
}

/// Where in a supplied argument a stop happened, in its producer's own
/// typed terms. No fabricated empty path is ever carried: an unavailable
/// locus is `None` on the context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuppliedLocus {
    /// A membership event: the zero-based supplied argument, the retained
    /// child edges from the current occurrence back to that argument's root
    /// (leaf first), and the local part of the occurrence.
    Membership {
        /// The zero-based supplied argument at the path root.
        argument: usize,
        /// Child edges from the current occurrence to the root.
        steps_from_leaf: Vec<PathStep>,
        /// The final event part.
        part: MembershipPart,
    },
    /// A conversion event: the zero-based argument and the zero-based
    /// preorder position of the witness node within it.
    Witness {
        /// The zero-based supplied argument.
        argument: usize,
        /// The node's preorder position in that argument's witness.
        position: u64,
    },
}

/// Which counter a limit stop denied.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppliedLimitKind {
    /// A: cumulative admission helper events.
    AdmissionWorkUnits,
    /// H: cumulative conversion helper events.
    ConversionWorkUnits,
    /// FR-098's converted nodes C, bounded by `work_units`.
    WorkUnits,
    /// The semantic occurrence count, bounded by `value_occurrences`.
    ValueOccurrences,
    /// A decimal quantity magnitude's power-of-ten expansion, bounded by
    /// `scale_expansion`.
    ScaleExpansion,
    /// B: one numeric operand's bit length, bounded per operand with no
    /// cumulative counter.
    NumericMagnitudeBits,
}

impl SuppliedLimitKind {
    /// The caller field that raises this bound.
    #[must_use]
    pub const fn field(self) -> &'static str {
        match self {
            Self::AdmissionWorkUnits => ADMISSION_WORK_UNITS,
            Self::ConversionWorkUnits => CONVERSION_WORK_UNITS,
            Self::WorkUnits => "work_units",
            Self::ValueOccurrences => "value_occurrences",
            Self::ScaleExpansion => "scale_expansion",
            Self::NumericMagnitudeBits => NUMERIC_MAGNITUDE_BITS,
        }
    }
}

/// The checked count a denied event would have reached.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AttemptedCount {
    /// The exact attempted count.
    Count(u64),
    /// The counter's representation overflowed; nothing wrapped.
    Overflow,
}

/// The pre-call limit record (FR-277 "Projection into existing QSL
/// carriers"): the numerical meanings of `Incomplete` without a charge point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SuppliedLimit {
    /// The denied counter.
    pub limit_kind: SuppliedLimitKind,
    /// The configured ceiling.
    pub limit: u64,
    /// The exact denied amount (one for A and H).
    pub next_charge: u64,
    /// The checked attempted count.
    pub counter: AttemptedCount,
    /// The numeric operand B denied; `None` for every other counter.
    pub operand: Option<SuppliedNumericOperand>,
}

/// The numeric operand a B stop names (QSpec FR-323 "Numeric magnitude
/// bound"): a value-in-domain operand, or an endpoint of a compared pair of
/// type-chain links.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuppliedNumericOperand {
    /// An operand of one value-in-domain comparison.
    Value(NumericMembershipOperand),
    /// An endpoint of one type-link comparison.
    TypeLink(NumericTypeEndpoint),
    /// An operand of one Decimal value-in-domain comparison.
    Decimal(DecimalMembershipOperand),
}

impl SuppliedNumericOperand {
    /// The wire spelling of the operand.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Value(operand) => match operand {
                NumericMembershipOperand::IntegerValue => "value",
                NumericMembershipOperand::IntegerLower => "lower",
                NumericMembershipOperand::IntegerUpper => "upper",
                NumericMembershipOperand::RationalNumerator => "value-numerator",
                NumericMembershipOperand::RationalDenominator => "value-denominator",
                NumericMembershipOperand::RationalNumeratorLower => "numerator-lower",
                NumericMembershipOperand::RationalNumeratorUpper => "numerator-upper",
                NumericMembershipOperand::RationalDenominatorLower => "denominator-lower",
                NumericMembershipOperand::RationalDenominatorUpper => "denominator-upper",
            },
            Self::Decimal(operand) => match operand {
                DecimalMembershipOperand::Coefficient => "coefficient",
                DecimalMembershipOperand::Lower => "lower",
                DecimalMembershipOperand::Upper => "upper",
            },
            Self::TypeLink(endpoint) => match endpoint {
                NumericTypeEndpoint::IntegerExpectedLower => "type-link-expected-lower",
                NumericTypeEndpoint::IntegerExpectedUpper => "type-link-expected-upper",
                NumericTypeEndpoint::IntegerActualLower => "type-link-actual-lower",
                NumericTypeEndpoint::IntegerActualUpper => "type-link-actual-upper",
                NumericTypeEndpoint::RationalExpectedNumeratorLower => {
                    "type-link-expected-lower-numerator"
                }
                NumericTypeEndpoint::RationalExpectedNumeratorUpper => {
                    "type-link-expected-upper-numerator"
                }
                NumericTypeEndpoint::RationalExpectedDenominatorLower => {
                    "type-link-expected-lower-denominator"
                }
                NumericTypeEndpoint::RationalExpectedDenominatorUpper => {
                    "type-link-expected-upper-denominator"
                }
                NumericTypeEndpoint::RationalActualNumeratorLower => {
                    "type-link-actual-lower-numerator"
                }
                NumericTypeEndpoint::RationalActualNumeratorUpper => {
                    "type-link-actual-upper-numerator"
                }
                NumericTypeEndpoint::RationalActualDenominatorLower => {
                    "type-link-actual-lower-denominator"
                }
                NumericTypeEndpoint::RationalActualDenominatorUpper => {
                    "type-link-actual-upper-denominator"
                }
                NumericTypeEndpoint::DecimalExpectedLower => "type-link-expected-lower-coefficient",
                NumericTypeEndpoint::DecimalExpectedUpper => "type-link-expected-upper-coefficient",
                NumericTypeEndpoint::DecimalActualLower => "type-link-actual-lower-coefficient",
                NumericTypeEndpoint::DecimalActualUpper => "type-link-actual-upper-coefficient",
            },
        }
    }
}

impl SuppliedLimit {
    /// The caller field that raises this bound.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.limit_kind.field()
    }
}

/// The measured unit of a failed reservation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppliedStorageUnit {
    /// Native bytes.
    Bytes,
    /// Additional collection elements.
    AdditionalElements,
    /// Distinct path-arena identifiers.
    ArenaIdentifiers,
    /// Magnitude bits of the largest Decimal operand whose scratch layout
    /// could not be represented.
    MagnitudeBits,
}

/// The measured request of a storage or capacity stop, in its native unit.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SuppliedStorage {
    /// The exact amount submitted to the reservation.
    pub requested: u64,
    /// Its native unit.
    pub unit: SuppliedStorageUnit,
}

impl SuppliedStorage {}

/// The five closed causes of a supplied-phase stop (QSpec FR-323).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuppliedCause {
    /// A helper or counter ceiling denied the next event.
    Limit(SuppliedLimit),
    /// The input failed membership; the original refusal and, where QSV
    /// reported it, the membership cause.
    Input {
        /// The original input refusal.
        refusal: InputRefusal,
        /// QSV's membership cause, when membership found the fault.
        membership: Option<InvalidMembershipCause>,
    },
    /// The original caller [`quire_exact::Cancel`] stopped the phase.
    Cancelled(CancelCause),
    /// A fallible reservation was denied by the allocator.
    Storage(SuppliedStorage),
    /// A reservation size could not be represented.
    Capacity(SuppliedStorage),
}

/// The context every supplied-phase stop carries (QSpec FR-323):
/// `{ phase, parameter, event, locus, consumed, cause }` with the cause in
/// [`SuppliedStop::cause`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuppliedContext {
    /// The actual initiating owner.
    pub phase: SuppliedPhase,
    /// The zero-based declared parameter position, when known.
    pub parameter: Option<usize>,
    /// The actual owning event; `None` for a cancellation outside a
    /// scheduled event.
    pub event: Option<SuppliedEvent>,
    /// The producer's typed locus, when available.
    pub locus: Option<SuppliedLocus>,
    /// The successful prefix of the owning counter: cumulative for A and H,
    /// per argument for C and `value_occurrences`. A denied step is never in
    /// it.
    pub consumed: u64,
}

/// A phase-marked pre-call stop: no evaluation ran, nothing was charged to
/// the evaluator's meter and no partial value exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuppliedStop {
    /// The phase context.
    pub context: SuppliedContext,
    /// Why the phase stopped.
    pub cause: SuppliedCause,
}

impl SuppliedStop {
    /// The limit record, when a ceiling denied the event.
    #[must_use]
    pub const fn limit(&self) -> Option<&SuppliedLimit> {
        match &self.cause {
            SuppliedCause::Limit(limit) => Some(limit),
            SuppliedCause::Input { .. }
            | SuppliedCause::Cancelled(_)
            | SuppliedCause::Storage(_)
            | SuppliedCause::Capacity(_) => None,
        }
    }
}

/// A supplied-admission walk reached a helper family whose supplied-value
/// contract QSV has not qualified (IR-714): the walk stopped before that
/// helper ran. It is distinct from invalid input, a limit, a cancellation,
/// storage and capacity, and never a [`SuppliedCause`] arm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedHelperStop {
    /// The phase context at the stop.
    pub context: SuppliedContext,
    /// The helper family that remains an implementation dependency.
    pub family: UnsupportedHelperFamily,
}

/// Why a supplied phase stopped before any call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuppliedFailure {
    /// A limit, input, cancellation, storage or capacity stop.
    Stop(Box<SuppliedStop>),
    /// An unqualified helper family.
    Unsupported(Box<UnsupportedHelperStop>),
    /// A broken kernel contract: an environment ceiling or declaration
    /// refusal answering a membership query.
    Fault(InternalFault),
}

impl From<SuppliedStop> for SuppliedFailure {
    fn from(stop: SuppliedStop) -> Self {
        Self::Stop(Box::new(stop))
    }
}
