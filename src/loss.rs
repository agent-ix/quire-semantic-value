// SPDX-License-Identifier: AGPL-3.0-or-later
//! Evaluation loss records: the information one
//! completed operation discarded ([`ValueLoss`]) and the expression that
//! produced it ([`LocatedLoss`]). A completed evaluation reports them in
//! evaluation order.

use quire_exact::{DecimalLoss, IeeeExactLoss, IeeeFlags};

use crate::location::Location;

/// Information one completed operation discarded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueLoss {
    /// A rounded decimal operation or conversion (QSpec FR-140).
    Decimal(DecimalLoss),
    /// An IEEE-to-exact conversion (QSpec FR-148).
    IeeeExact(IeeeExactLoss),
    /// The non-empty flag set an IEEE operation raised (QSpec FR-148).
    IeeeFlags(IeeeFlags),
}

/// A loss record and the expression that produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocatedLoss {
    /// The producing expression.
    pub location: Location,
    /// What was discarded.
    pub loss: ValueLoss,
}
