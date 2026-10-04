// SPDX-License-Identifier: AGPL-3.0-or-later
//! The early-exit carrier for this crate's computations, always converted into
//! a [`quire_exact::Outcome`] by [`outcome_from_stop`], and back by
//! [`outcome_into_stop`]. It is not a kernel type and not a spec-owned type,
//! which is why it holds no catalog code of its own and is not part of the
//! kernel's outcome or refusal families. It is shared by this crate's
//! computations (`quantity`, `declaration`, `enumeration`) and by the evaluators
//! above it, which all convert through
//! [`outcome_from_stop`]/[`outcome_into_stop`], which is why an `?`-friendly
//! shape earns its own module rather than inline handling at each call site.
//!
//! `quire_exact::Outcome<T>` is foreign to this crate, so
//! `outcome_from_stop`/`outcome_into_stop` cannot be an inherent `impl` on it
//! (E0116, the orphan rule). They are plain functions instead: a
//! single-implementation trait would add a name to learn with no seam or
//! polymorphism to justify it.
//!
//! No `Halt` variant carries a fault into a `Stop`, and no `Stop`-returning
//! helper can pass one to [`outcome_from_stop`]: an evaluator carries its own
//! crate-private `Halt` for an invariant break, and no `From<Halt> for Stop`
//! conversion exists here, so a fault stays unrepresentable in `Stop` by
//! construction, not by convention.

use quire_exact::{Incomplete, Outcome, Refusal, Undefined};

/// Why a `value` computation stopped without a value: the kernel's own
/// [`Undefined`]/[`Refusal`] reasons, or a meter [`Incomplete`] charge
/// denial. Holds kernel causes only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stop {
    /// The kernel reports the computation undefined.
    Undefined(Undefined),
    /// The kernel refuses the computation.
    Refused(Refusal),
    /// A meter charge was denied.
    Incomplete(Incomplete),
}

impl From<Incomplete> for Stop {
    fn from(record: Incomplete) -> Self {
        Self::Incomplete(record)
    }
}

/// Converts a `value` computation's `Result<T, Stop>` early exit into the
/// kernel [`Outcome<T>`] it represents, variant by variant.
pub fn outcome_from_stop<T>(result: Result<T, Stop>) -> Outcome<T> {
    match result {
        Ok(value) => Outcome::Completed(value),
        Err(Stop::Undefined(reason)) => Outcome::Undefined(reason),
        Err(Stop::Refused(reason)) => Outcome::Refused(reason),
        Err(Stop::Incomplete(record)) => Outcome::Incomplete(record),
    }
}

/// The inverse of [`outcome_from_stop`], for further `?` propagation.
pub fn outcome_into_stop<T>(outcome: Outcome<T>) -> Result<T, Stop> {
    match outcome {
        Outcome::Completed(value) => Ok(value),
        Outcome::Undefined(reason) => Err(Stop::Undefined(reason)),
        Outcome::Refused(reason) => Err(Stop::Refused(reason)),
        Outcome::Incomplete(record) => Err(Stop::Incomplete(record)),
    }
}
