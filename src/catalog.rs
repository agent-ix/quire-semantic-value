// SPDX-License-Identifier: AGPL-3.0-or-later
//! The native-diagnostics catalog vocabulary an evaluation result carries
//! (ADR-013 O-16, O-17): the outcome [`Category`], the catalog code and cause
//! of a refusal ([`CatalogCode`]), the internal fault ([`InternalFault`]) and
//! the two traits through which a family holds its own evaluation-time cause
//! ([`CatalogCoded`], [`UndefinedCoded`]).
//!
//! These types live in this leaf so the evaluation interface can name them
//! without depending on a compiler crate. Building a refusal *record* at a
//! source locus stays with the crate that owns source loci.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write as _;
use core::hash::Hash;

/// The catalog code prefix of the `StateModel` family (ADR-012 §5.1 S1): the
/// string `FamilyKind::StateModel.catalog_code_prefix()` returns, and the
/// prefix every `StateModel` evaluation-time cause carries.
pub const STATE_MODEL_CATALOG_PREFIX: &str = "state-model";

/// The payload rendering of a member identity. A population member's
/// identity is a JSON string, so a well-formed identity is valid UTF-8 and
/// renders as itself. Bytes that are not UTF-8 render as
/// `identity bytes 0x<lowercase hex> (not UTF-8)`: two distinct malformed
/// identities never render alike, and a malformed identity never renders as
/// a lossy decoding that could equal a real member's.
pub fn render_identity(identity: &[u8]) -> String {
    match core::str::from_utf8(identity) {
        Ok(identity) => identity.to_string(),
        Err(_) => {
            let mut rendered = String::from("identity bytes 0x");
            for byte in identity {
                // Writing to a `String` cannot fail.
                let _ = write!(rendered, "{byte:02x}");
            }
            rendered.push_str(" (not UTF-8)");
            rendered
        }
    }
}

/// ADR-013 O-16: the outcome category every evaluation, negotiation and proof
/// result maps into. Exactly the eight values ADR-013 §3 O-16's category
/// table names in its first column; not a kernel type (`quire-exact` carries
/// no category), and this crate is its only owner. Category-mapping
/// functions from each family's own outcome (C-08, C-09, C-23) match this
/// enum exhaustively with no `_` arm, so a ninth category fails every one of
/// them to compile rather than silently falling into an existing row.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Category {
    /// A completed value, or a claim's `Completed(false)` (a violation is
    /// its own, separate category below).
    Success,
    /// A claim's `Completed(false)`: a false predicate, not a refusal.
    Violation,
    /// The operation has no mathematical value (kernel `Undefined`).
    Undefined,
    /// The operation is defined but its result, or the request itself, is
    /// not admitted.
    Refusal,
    /// A well-formed request naming a real, catalogued capability this
    /// build, backend or solver does not supply.
    Unsupported,
    /// A named charge was unavailable and no partial value exists: timeout,
    /// cancellation or bound exhaustion.
    Incomplete,
    /// Neither proved nor refuted -- including a vacuous proof and a replay
    /// parity disagreement.
    Inconclusive,
    /// A runtime, evaluator or mapping invariant broke; never a `Refusal`.
    InternalFailure,
}

impl Category {
    /// Every value, in the ADR-013 O-16 category table's row order.
    pub const ALL: [Self; 8] = [
        Self::Success,
        Self::Violation,
        Self::Undefined,
        Self::Refusal,
        Self::Unsupported,
        Self::Incomplete,
        Self::Inconclusive,
        Self::InternalFailure,
    ];

    /// A stable display spelling. ADR-013 O-16 fixes the eight values and
    /// states that category values "compare lexically on their wire
    /// strings", but names no QSpec wire contract for this QSL-internal
    /// type, so this spelling is QSL's own and carries no QSpec authority.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Violation => "violation",
            Self::Undefined => "undefined",
            Self::Refusal => "refusal",
            Self::Unsupported => "unsupported",
            Self::Incomplete => "incomplete",
            Self::Inconclusive => "inconclusive",
            Self::InternalFailure => "internal-failure",
        }
    }

    /// FR-285 (ADR-029 CB-4): the one exit function, QSpec FR-301's code
    /// for an item of this category. Undefined folds to 10 with violation;
    /// inconclusive is a proof or `analyze` item's, which did not settle
    /// its claim and so is incomplete (22). A supplied-trace clause still
    /// pending at the trace's end takes [`Self::trace_exit_code`] instead.
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Violation | Self::Undefined => 10,
            Self::Refusal => 20,
            Self::Unsupported => 21,
            Self::Incomplete | Self::Inconclusive => 22,
            Self::InternalFailure => 30,
        }
    }

    /// FR-285's row for a supplied-trace clause (FR-283): one still pending
    /// when the trace ends is inconclusive but observed no violation, so it
    /// exits 0. Every other category exits as [`Self::exit_code`] states.
    pub const fn trace_exit_code(self) -> u8 {
        match self {
            Self::Inconclusive => 0,
            other => other.exit_code(),
        }
    }

    /// FR-285's severity fold: the most severe of `codes` under QSpec
    /// FR-301's order 30, 20, 21, 22, 10, 0, or `None` when `codes` is
    /// empty. A code outside that set ranks most severe, so it is never
    /// hidden behind a known one.
    pub fn most_severe(codes: impl IntoIterator<Item = u8>) -> Option<u8> {
        const ORDER: [u8; 6] = [30, 20, 21, 22, 10, 0];
        codes.into_iter().min_by_key(|code| {
            ORDER
                .iter()
                .position(|known| known == code)
                .map_or(0, |rank| rank + 1)
        })
    }
}

impl core::fmt::Display for Category {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// ADR-013 O-17: one `quire.native.diagnostics/v1` code and cause, exactly as
/// the `quire.native.diagnostics/v1` catalog spells them. Every
/// `catalog_code()` across every stage returns this same
/// type (C-15); there is no conversion back to a typed cause, and no
/// consumer reads the catalog's message -- only the code, the cause and the
/// structured fields the catalog defines for that pair. Equality is lexical
/// on both fields, matching ADR-013 O-17's stated equality kind.
#[derive(Clone, Copy, Debug)]
pub struct CatalogCode {
    code: &'static str,
    cause: &'static str,
    prefix: Option<&'static str>,
}

// O-17 equality remains lexical on the published pair; ownership is
// rendering metadata and is inspected separately through prefix().
impl PartialEq for CatalogCode {
    fn eq(&self, other: &Self) -> bool {
        (self.code, self.cause) == (other.code, other.cause)
    }
}

impl Eq for CatalogCode {}

impl Hash for CatalogCode {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&(self.code, self.cause), state);
    }
}

impl CatalogCode {
    /// Construct a code/cause pair. Callers name codes and causes exactly as
    /// the catalog spells them (ADR-013 R-07: a typed-cause-to-code
    /// conversion never invents a code the catalog does not define).
    pub const fn new(code: &'static str, cause: &'static str) -> Self {
        Self {
            code,
            cause,
            prefix: None,
        }
    }

    /// Attribute the rendered pair without changing its published raw fields.
    pub const fn with_prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = Some(prefix);
        self
    }

    /// The producing family's rendering prefix, when attributed.
    pub const fn prefix(&self) -> Option<&'static str> {
        self.prefix
    }

    /// The catalog's top-level code, e.g. `"runtime_invariant"`.
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// The catalog's cause under that code, e.g.
    /// `"established-invariant-broken"`.
    pub const fn cause(&self) -> &'static str {
        self.cause
    }
}

impl core::fmt::Display for CatalogCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(prefix) = self.prefix {
            write!(f, "{prefix}/")?;
        }
        write!(f, "{}/{}", self.code, self.cause)
    }
}

/// ADR-013 T-4: a broken runtime, evaluator or mapping invariant, raised
/// from a compiler stage, the I2 reader, `replay` or `route` (and, for
/// evaluation, `CheckedPackage::call`). Names the stage and the violated
/// invariant, each by a stable identifier -- not a display string (ADR-013
/// R-05) and not itself a closed enum: T-4 gives `LimitKind` alone the
/// "closed enum" qualifier, and the set of stages and invariants that can
/// raise a fault is expected to grow as later slices add stages, without
/// widening this type. `InternalFault` maps to exactly one catalog code
/// (`runtime_invariant`/`established-invariant-broken`, confirmed by the
/// catalog's revision `1-draft.6` note) and exactly one category
/// (`Category::InternalFailure`), and is never a `Refusal` (ADR-013 T-4).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InternalFault {
    stage: &'static str,
    invariant: &'static str,
    kernel_cause: Option<quire_exact::CheckedInvariantCause>,
    named: Vec<(&'static str, String)>,
}

impl InternalFault {
    /// `stage` and `invariant` are stable identifiers the raising site
    /// names, never derived from a display string or message (ADR-013 R-05).
    pub const fn new(stage: &'static str, invariant: &'static str) -> Self {
        Self {
            stage,
            invariant,
            kernel_cause: None,
            named: Vec::new(),
        }
    }

    /// This fault with the values it concerns, each under a stable field
    /// name (for example the node and declarations of a rebound
    /// correspondence entry).
    #[must_use]
    pub fn with_named(mut self, named: Vec<(&'static str, String)>) -> Self {
        self.named = named;
        self
    }

    /// The values this fault concerns, by stable field name; empty when the
    /// invariant names none.
    pub fn named(&self) -> &[(&'static str, String)] {
        &self.named
    }

    /// A received kernel invariant, retaining its original typed provenance.
    pub const fn from_kernel(
        stage: &'static str,
        invariant: &'static str,
        cause: quire_exact::CheckedInvariantCause,
    ) -> Self {
        Self {
            stage,
            invariant,
            kernel_cause: Some(cause),
            named: Vec::new(),
        }
    }

    /// The received kernel cause; QSL-owned invariant faults have no kernel cause.
    pub const fn kernel_cause(&self) -> Option<quire_exact::CheckedInvariantCause> {
        self.kernel_cause
    }

    /// The stage, reader or module that raised this fault.
    pub const fn stage(&self) -> &'static str {
        self.stage
    }

    /// The stable identifier of the violated invariant.
    pub const fn invariant(&self) -> &'static str {
        self.invariant
    }

    /// Always `runtime_invariant`/`established-invariant-broken` (ADR-013
    /// T-4): an internal fault is never a `Refusal` and never any other
    /// catalog code.
    pub fn catalog_code(&self) -> CatalogCode {
        CatalogCode::new("runtime_invariant", "established-invariant-broken")
    }

    /// Always `Category::InternalFailure` (ADR-013 O-16).
    pub fn category(&self) -> Category {
        Category::InternalFailure
    }
}

/// ADR-013 O-16/O-17: a family-owned evaluation-time refusal
/// cause, held only through this trait so the layer-3 `check` core and this
/// crate itself never name the concrete cause type -- "a cause belongs to
/// the family whose construct produces it" (ADR-013 O-16). The supertraits
/// make a `FamilyResult`/`FamilyOutcome` built from a `Box<dyn CatalogCoded>`
/// `Debug`, able to cross a thread, and free of any borrow (ADR-013 O-16
/// "Representation").
pub trait CatalogCoded: core::fmt::Debug + Send + Sync + 'static {
    /// This cause's catalog code (O-17's method); its O-16 category is
    /// always `Category::Refusal`.
    fn catalog_code(&self) -> CatalogCode;

    /// The structured payload the catalog row requires for the cause
    /// [`Self::catalog_code`] names (ADR-013 O-17, FR-096), with the same
    /// map shape as [`UndefinedRecord::fields`]: one entry for each required
    /// payload item other than a location, valued by the item's rendering.
    /// A location item is the refusal record's locus, not a field. Read from the
    /// cause's own variant, never from a message.
    ///
    /// `None` for a cause FR-096's key table has no row for: it has no
    /// fields to give, and an empty map would pass for a cause whose row
    /// requires none.
    fn catalog_fields(&self) -> Option<BTreeMap<&'static str, String>>;
}

/// ADR-013 O-16: a family-owned evaluation-time undefined cause,
/// held only through this trait -- the undefined-category counterpart of
/// [`CatalogCoded`].
pub trait UndefinedCoded: core::fmt::Debug + Send + Sync + 'static {
    /// This cause's [`UndefinedRecord`]; its O-16 category is always
    /// `Category::Undefined`.
    fn undefined_record(&self) -> UndefinedRecord;

    /// The producing family's prefix, separate from the published reason.
    fn catalog_prefix(&self) -> Option<&'static str> {
        None
    }

    /// Render ownership and the unchanged catalog reason together.
    fn catalog_code(&self) -> String {
        let record = self.undefined_record();
        match self.catalog_prefix() {
            Some(prefix) => format!("{prefix}/{}", record.reason),
            None => record.reason.to_string(),
        }
    }
}

/// ADR-013 O-16: the closed reason set of the
/// `quire.native.diagnostics/v1` "Undefined reasons" table -- the catalog
/// states this is not a refusal code or cause. Grows by one variant each
/// time a family adds a new evaluation-time undefined result, the same way
/// a family-dispatch cause set grows.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum UndefinedReason {
    /// FR-151 dispatch: an FR-151 dispatched call's selected method's
    /// effective precondition evaluated to `false`.
    PreconditionFalse,
    /// FR-153: a `lookup<T>(p, r) absent undefined` query's reference `r`
    /// names no member of the population bound to `p`.
    AbsentKey,
    /// FR-090-AC-21: an `allInstances<T>` query cannot establish subtype
    /// closure for the admitted population declaration.
    IncompleteClosure,
    /// FR-139: an `only(x in P where c: e)` form found no object of the
    /// population satisfying `c`, or several.
    OnlyNotOne,
}

impl UndefinedReason {
    /// The catalog's own reason spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreconditionFalse => "precondition-false",
            Self::AbsentKey => "absent-key",
            Self::IncompleteClosure => "incomplete-closure",
            Self::OnlyNotOne => "only-not-one",
        }
    }
}

impl core::fmt::Display for UndefinedReason {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// ADR-013 O-16: the undefined-side counterpart of O-17's refusal record:
/// `reason` plus the catalog's
/// own structured payload for that reason. [`UndefinedCoded::undefined_record`]
/// returns this record, not the bare reason, because the catalog requires
/// the payload and a consumer outside the producing family holds only the
/// trait object -- without the record it could reach the payload only by
/// downcasting to the family's own cause type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndefinedRecord {
    /// The catalog's closed undefined reason.
    pub reason: UndefinedReason,
    /// The catalog's structured payload for `reason`, keyed by the
    /// catalog's own field names.
    pub fields: BTreeMap<&'static str, String>,
}

#[cfg(test)]
mod tests {
    use super::{
        render_identity, UndefinedCoded, UndefinedReason, UndefinedRecord,
        STATE_MODEL_CATALOG_PREFIX,
    };
    use alloc::collections::BTreeMap;
    use ix_trace_rs::trace;

    /// FR-126-AC-6: a malformed identity renders as its hex bytes, so two
    /// distinct malformed identities never render alike, and a well-formed
    /// one renders as itself.
    #[trace("TC-925", "FR-126-AC-6")]
    #[test]
    fn render_identity_renders_non_utf8_bytes_as_hex() {
        assert_eq!(render_identity(b"c9"), "c9");
        assert_eq!(
            render_identity(&[0xFF, 0xFE]),
            "identity bytes 0xfffe (not UTF-8)"
        );
        assert_ne!(render_identity(&[0xFF]), render_identity(&[0xFE]));
    }

    #[derive(Debug)]
    struct Prefixed;

    impl UndefinedCoded for Prefixed {
        fn undefined_record(&self) -> UndefinedRecord {
            UndefinedRecord {
                reason: UndefinedReason::AbsentKey,
                fields: BTreeMap::new(),
            }
        }

        fn catalog_prefix(&self) -> Option<&'static str> {
            Some(STATE_MODEL_CATALOG_PREFIX)
        }
    }

    /// FR-126-AC-7: a cause prefixed with the `StateModel` constant renders
    /// under `state-model/`.
    #[trace("TC-925", "FR-126-AC-7")]
    #[test]
    fn state_model_prefix_leads_the_rendered_catalog_code() {
        assert_eq!(Prefixed.catalog_code(), "state-model/absent-key");
    }
}
