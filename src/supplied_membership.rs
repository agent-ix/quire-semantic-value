// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-owned accounting and typed stops for supplied-value membership.
//!
//! The traversal owns event ordering and storage reservations. This module
//! records only events it explicitly requests; it does not inspect values or
//! debit the evaluator's meter.

use quire_exact::{Cancel, CancelCause};

use crate::declaration::StorageRequest;

/// The phase that owns every result from supplied-value membership.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppliedAdmissionPhase {
    /// Checking a supplied runtime value against its expected type.
    Membership,
}

/// One logical event requested by the supplied-membership traversal.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MembershipEvent {
    /// Visit one actual value occurrence before inspecting its runtime kind.
    ValueVisit,
    /// Compare one expected and retained type-chain link pair.
    TypeLinkComparison,
    /// Schedule one actual child occurrence for a later membership visit.
    ChildScheduling,
    /// Query one admitted descriptor or ranked member shape.
    DescriptorQuery,
}

/// The part of an occurrence at which an event was requested.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MembershipPart {
    /// The occurrence itself.
    Value,
    /// A zero-based type-chain link pair.
    TypeLink(u64),
    /// A zero-based child position of the current occurrence.
    Child(usize),
    /// A descriptor lookup attached to the current occurrence.
    Descriptor,
}

/// The supplied argument and actual occurrence at an event boundary.
///
/// `occurrence` is the traversal's ordinal for an actual visit, including a
/// repeated reference to a shared value. `part` identifies work within that
/// occurrence without allocating a path solely for an error.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MembershipLocus {
    /// Zero-based supplied argument position in the invocation.
    pub argument: usize,
    /// Zero-based actual occurrence visit in that argument.
    pub occurrence: u64,
    /// The local part of that occurrence.
    pub part: MembershipPart,
}

/// Why a supplied value failed membership after permitted work.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InvalidMembershipCause {
    /// The value's runtime kind differs from the expected kind.
    Kind,
    /// The expected and retained types differ.
    Type,
    /// A ranked or declared member does not belong to its descriptor.
    Member,
    /// A required payload differs from its admitted shape or type.
    Payload,
}

/// A typed stop from the supplied-admission phase.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SuppliedAdmissionError {
    /// The input failed a membership rule at the given locus.
    Invalid {
        /// The failed membership rule.
        cause: InvalidMembershipCause,
        /// Where the rule failed.
        locus: MembershipLocus,
    },
    /// The next one-unit event would exceed the caller's ceiling.
    Limit {
        /// The configured supplied-membership ceiling.
        ceiling: u64,
        /// Successful cumulative work, excluding the denied event.
        spent: u64,
        /// The denied request, always one logical event unit.
        requested: u64,
        /// The event that was not performed.
        event: MembershipEvent,
        /// Where the event would have occurred.
        locus: MembershipLocus,
    },
    /// The caller's original cancellation handle stopped this boundary.
    Cancelled {
        /// The cause recorded on the original handle.
        cause: CancelCause,
        /// Successful cumulative work before cancellation.
        spent: u64,
        /// The event or helper boundary at which cancellation was observed.
        event: MembershipEvent,
        /// Where cancellation was observed.
        locus: MembershipLocus,
    },
    /// A real supplied-worklist reservation was denied.
    Allocation {
        /// The measured request at the reservation boundary.
        request: StorageRequest,
        /// Where the reservation was needed.
        locus: MembershipLocus,
    },
    /// Checked worklist size or representation arithmetic overflowed.
    Capacity {
        /// The measured request whose size could not be represented.
        request: StorageRequest,
        /// Where the reservation was needed.
        locus: MembershipLocus,
    },
}

impl SuppliedAdmissionError {
    /// The owning phase, independent of the failure classification.
    pub const fn phase(&self) -> SuppliedAdmissionPhase {
        SuppliedAdmissionPhase::Membership
    }
}

/// Cumulative logical work owned by one supplied-admission invocation.
///
/// Pass the same mutable budget through each argument and nested helper. A
/// denied event never changes `spent`; zero is a zero-work ceiling.
#[derive(Debug, Eq, PartialEq)]
pub struct SuppliedMembershipBudget {
    ceiling: u64,
    spent: u64,
}

impl SuppliedMembershipBudget {
    /// Create a budget with no successful events yet.
    pub const fn new(ceiling: u64) -> Self {
        Self { ceiling, spent: 0 }
    }

    /// The configured logical-work ceiling.
    pub const fn ceiling(&self) -> u64 {
        self.ceiling
    }

    /// The successful cumulative logical work.
    pub const fn spent(&self) -> u64 {
        self.spent
    }

    /// Request one event before performing it, polling the original handle.
    pub fn charge(
        &mut self,
        event: MembershipEvent,
        locus: MembershipLocus,
        cancel: &Cancel,
    ) -> Result<(), SuppliedAdmissionError> {
        self.check_cancel(event, locus, cancel)?;
        match self.spent.checked_add(1) {
            Some(next) if next <= self.ceiling => {
                self.spent = next;
                Ok(())
            }
            _ => Err(SuppliedAdmissionError::Limit {
                ceiling: self.ceiling,
                spent: self.spent,
                requested: 1,
                event,
                locus,
            }),
        }
    }

    /// Poll after an indivisible helper, without charging another event.
    pub fn check_cancel(
        &self,
        event: MembershipEvent,
        locus: MembershipLocus,
        cancel: &Cancel,
    ) -> Result<(), SuppliedAdmissionError> {
        let _ = cancel.poll();
        if let Some(cause) = cancel.cause() {
            return Err(SuppliedAdmissionError::Cancelled {
                cause,
                spent: self.spent,
                event,
                locus,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use quire_exact::{Cancel, CancelCause};

    use super::{
        MembershipEvent, MembershipLocus, MembershipPart, SuppliedAdmissionError,
        SuppliedAdmissionPhase, SuppliedMembershipBudget,
    };

    fn locus(argument: usize, occurrence: u64, part: MembershipPart) -> MembershipLocus {
        MembershipLocus {
            argument,
            occurrence,
            part,
        }
    }

    /// Trace: FR-109-AC-1, FR-109-AC-2
    #[ix_trace_rs::trace("TC-908", "FR-109-AC-1", "FR-109-AC-2")]
    #[test]
    fn exact_one_less_and_zero_share_cumulative_event_boundaries() {
        // The budget receives an independently enumerated event order across
        // two arguments, including a nested type and one scheduled child.
        let events = [
            (
                MembershipEvent::ValueVisit,
                locus(0, 0, MembershipPart::Value),
            ),
            (
                MembershipEvent::ValueVisit,
                locus(1, 0, MembershipPart::Value),
            ),
            (
                MembershipEvent::TypeLinkComparison,
                locus(1, 0, MembershipPart::TypeLink(0)),
            ),
            (
                MembershipEvent::TypeLinkComparison,
                locus(1, 0, MembershipPart::TypeLink(1)),
            ),
            (
                MembershipEvent::ChildScheduling,
                locus(1, 0, MembershipPart::Child(0)),
            ),
            (
                MembershipEvent::DescriptorQuery,
                locus(1, 0, MembershipPart::Descriptor),
            ),
        ];
        let cancel = Cancel::new();
        for ceiling in [0, 5, 6] {
            let mut budget = SuppliedMembershipBudget::new(ceiling);
            let result = events
                .iter()
                .copied()
                .try_for_each(|(event, at)| budget.charge(event, at, &cancel));
            if ceiling == 6 {
                assert_eq!(result, Ok(()));
                assert_eq!(budget.spent(), 6);
            } else {
                let (event, at) = events[ceiling as usize];
                assert_eq!(
                    result,
                    Err(SuppliedAdmissionError::Limit {
                        ceiling,
                        spent: ceiling,
                        requested: 1,
                        event,
                        locus: at,
                    })
                );
                assert_eq!(budget.spent(), ceiling);
                assert_eq!(
                    result.unwrap_err().phase(),
                    SuppliedAdmissionPhase::Membership
                );
            }
        }
    }

    /// Trace: FR-109-AC-3
    #[ix_trace_rs::trace("TC-908", "FR-109-AC-3")]
    #[test]
    fn original_handle_cancellation_stops_before_next_event() {
        let cancel = Cancel::new();
        let mut budget = SuppliedMembershipBudget::new(3);
        let first = locus(0, 0, MembershipPart::Value);
        let next = locus(0, 0, MembershipPart::TypeLink(0));
        budget
            .charge(MembershipEvent::ValueVisit, first, &cancel)
            .unwrap();
        cancel.cancel(CancelCause::Requested);
        assert_eq!(
            budget.charge(MembershipEvent::TypeLinkComparison, next, &cancel),
            Err(SuppliedAdmissionError::Cancelled {
                cause: CancelCause::Requested,
                spent: 1,
                event: MembershipEvent::TypeLinkComparison,
                locus: next,
            })
        );
        assert_eq!(budget.spent(), 1);
        assert_eq!(cancel.tripped(), Some(CancelCause::Requested));
    }

    /// Trace: FR-109-AC-3
    #[ix_trace_rs::trace("TC-908", "FR-109-AC-3")]
    #[test]
    fn cancellation_after_bounded_helper_keeps_successful_event_spend() {
        let cancel = Cancel::new();
        let mut budget = SuppliedMembershipBudget::new(1);
        let at = locus(0, 0, MembershipPart::Descriptor);
        budget
            .charge(MembershipEvent::DescriptorQuery, at, &cancel)
            .unwrap();
        cancel.cancel(CancelCause::Deadline);
        assert_eq!(
            budget.check_cancel(MembershipEvent::DescriptorQuery, at, &cancel),
            Err(SuppliedAdmissionError::Cancelled {
                cause: CancelCause::Deadline,
                spent: 1,
                event: MembershipEvent::DescriptorQuery,
                locus: at,
            })
        );
        assert_eq!(budget.spent(), 1);
    }
}
