// SPDX-License-Identifier: AGPL-3.0-or-later
//! The applied model operation a transition system steps by: [`ModelTransition`].
//!
//! A transition carries the declaration identity text of one operation, its
//! optional receiver and the ordered arguments it was applied to, each already
//! one canonical RFC 8785 document in QSL's typed canonical form. The crate
//! interprets none of them (FR-060): it checks that each text is canonical,
//! yields the transition identity of QSpec FR-181 as RFC 8785 bytes, encodes
//! itself through [`quire_canonical::Encode`] and orders by those bytes.
//!
//! Nothing here recurses once per nesting level of an operation, receiver or
//! argument: texts are read and embedded by `quire-canonical`'s arena reader
//! and its explicit-stack document encoder, and comparison walks the identity
//! bytes without building them.

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

use quire_canonical::{read, to_vec, Encode, Error, Limits, ReadError, Sink, Writer};

use crate::semantic_node::IdentityRefusal;

/// The `type` member value of every transition identity.
const TRANSITION_TYPE: &str = "transition";

/// One applied model operation: the operation's declaration identity text, its
/// optional receiver and the ordered arguments it was applied to.
///
/// Every text is one canonical RFC 8785 document. Equality is field-wise and
/// agrees with equality of [`ModelTransition::canonical_bytes`]; the order is
/// the unsigned-lexicographic order of those bytes, not a field-wise order.
/// The type holds exactly an operation, a receiver and arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelTransition {
    operation: String,
    receiver: Option<String>,
    arguments: Vec<String>,
}

/// Which text of a [`ModelTransition`] a refusal names.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TransitionSite {
    /// The operation's declaration identity text.
    Operation,
    /// The receiver text.
    Receiver,
    /// The argument at this zero-based index.
    Argument(usize),
}

/// Why a text of a [`ModelTransition`] was refused.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TransitionCause {
    /// The text is not one JSON value equal to its own RFC 8785 re-encoding.
    NotCanonicalJson,
    /// The receiver document is `null`; an absent receiver is the only `null`
    /// receiver in the identity.
    NullReceiver,
    /// The text, or its re-encoding, exceeds the byte limit.
    InputBytes {
        /// The byte limit the text was read and re-encoded under.
        bound: u64,
        /// The bytes needed when it was refused; always more than `bound`.
        required: u64,
    },
    /// A heap reservation for the text's read tree or re-encoding failed.
    Allocation {
        /// The size in bytes of the reservation that failed.
        requested: usize,
    },
}

/// A [`ModelTransition`] was refused: which text, and why.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[error("invalid model transition at {site:?}: {cause:?}")]
pub struct InvalidModelTransition {
    site: TransitionSite,
    cause: TransitionCause,
}

impl InvalidModelTransition {
    /// The text that was refused.
    #[must_use]
    pub const fn site(&self) -> TransitionSite {
        self.site
    }

    /// Why it was refused.
    #[must_use]
    pub const fn cause(&self) -> &TransitionCause {
        &self.cause
    }
}

impl ModelTransition {
    /// Retain `operation`, `receiver` and `arguments` verbatim after checking
    /// that each is one canonical RFC 8785 document.
    ///
    /// The operation is checked first, then the receiver when present, then
    /// each argument in order; the first failure is returned. `limits` bounds
    /// each text on its own, as the input bound of its read and the byte bound
    /// of its re-encoding. Arity is not checked.
    ///
    /// # Errors
    ///
    /// [`InvalidModelTransition`] naming the first refused text and its cause.
    pub fn new(
        operation: String,
        receiver: Option<String>,
        arguments: Vec<String>,
        limits: Limits,
    ) -> Result<Self, InvalidModelTransition> {
        check(&operation, limits).map_err(|cause| refusal(TransitionSite::Operation, cause))?;
        if let Some(text) = &receiver {
            check(text, limits).map_err(|cause| refusal(TransitionSite::Receiver, cause))?;
            if text == NULL_DOCUMENT {
                return Err(refusal(
                    TransitionSite::Receiver,
                    TransitionCause::NullReceiver,
                ));
            }
        }
        for (index, text) in arguments.iter().enumerate() {
            check(text, limits).map_err(|cause| refusal(TransitionSite::Argument(index), cause))?;
        }
        Ok(Self {
            operation,
            receiver,
            arguments,
        })
    }

    /// The operation's declaration identity text.
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }

    /// The receiver's canonical text, or `None` for an operation with no
    /// receiver.
    #[must_use]
    pub fn receiver(&self) -> Option<&str> {
        self.receiver.as_deref()
    }

    /// The arguments' canonical texts in parameter declaration order.
    #[must_use]
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    /// The RFC 8785 bytes of the transition identity: the object with members
    /// `arguments`, `operation`, `receiver` (`null` when absent) and `type`
    /// (`"transition"`).
    ///
    /// # Errors
    ///
    /// [`IdentityRefusal::InputBytes`] when the identity exceeds
    /// `limits.max_bytes()`, [`IdentityRefusal::Allocation`] when a heap
    /// reservation fails.
    pub fn canonical_bytes(&self, limits: Limits) -> Result<Vec<u8>, IdentityRefusal> {
        Ok(to_vec(self, limits)?)
    }

    /// The identity bytes, produced one at a time without allocating. Each
    /// stored text is canonical, so it is embedded as written; only the
    /// operation is escaped, as the JSON string it is.
    fn identity_bytes(&self) -> impl Iterator<Item = u8> + '_ {
        let arguments = self.arguments.iter().enumerate().flat_map(|(index, text)| {
            let separator = (index > 0).then_some(b',');
            separator.into_iter().chain(text.bytes())
        });
        let receiver = self.receiver.as_deref().unwrap_or(NULL_DOCUMENT);
        b"{\"arguments\":["
            .iter()
            .copied()
            .chain(arguments)
            .chain(b"],\"operation\":\"".iter().copied())
            .chain(self.operation.bytes().flat_map(Escaped::new))
            .chain(b"\",\"receiver\":".iter().copied())
            .chain(receiver.bytes())
            .chain(b",\"type\":\"".iter().copied())
            .chain(TRANSITION_TYPE.bytes())
            .chain(b"\"}".iter().copied())
    }
}

/// The only document spelled `null`; also the receiver member of an absent
/// receiver.
const NULL_DOCUMENT: &str = "null";

fn refusal(site: TransitionSite, cause: TransitionCause) -> InvalidModelTransition {
    InvalidModelTransition { site, cause }
}

/// Check that `text` is one JSON value equal to its own RFC 8785 re-encoding.
fn check(text: &str, limits: Limits) -> Result<(), TransitionCause> {
    let document = read(text.as_bytes(), limits.max_bytes()).map_err(|error| match error {
        ReadError::Limit(limit) => TransitionCause::InputBytes {
            bound: limit.bound,
            required: limit.required,
        },
        ReadError::Allocation { requested } => TransitionCause::Allocation { requested },
        ReadError::Malformed { .. } | ReadError::NumberOutOfRange { .. } => {
            TransitionCause::NotCanonicalJson
        }
        // `ReadError` is non-exhaustive: `_` is for refusals that do not exist
        // yet, so a new one is reviewed here rather than assumed to be bad text.
        _ => TransitionCause::NotCanonicalJson,
    })?;
    let canonical = to_vec(&document, limits).map_err(|error| match error {
        Error::Limit(limit) => TransitionCause::InputBytes {
            bound: limit.bound,
            required: limit.required,
        },
        Error::Allocation { requested } => TransitionCause::Allocation { requested },
        Error::NonFiniteNumber(_)
        | Error::IntegerMagnitudeAboveMaximum(_)
        | Error::UnsignedIntegerMagnitudeAboveMaximum(_)
        | Error::WideIntegerMagnitudeAboveMaximum(_)
        | Error::NonStringMemberName { .. }
        | Error::DuplicateMemberName { .. }
        | Error::SerdeJsonPrivateToken(_)
        | Error::Serialize(_) => TransitionCause::NotCanonicalJson,
        // A fault of the encoder, not of the text: FR-120 has no cause for it
        // (decision pending on PR #41), so it is reported as the closest
        // listed refusal and named here so it is not absorbed by `_`.
        Error::Protocol(_) | Error::Internal { .. } => TransitionCause::NotCanonicalJson,
        // `Error` is non-exhaustive: `_` is for refusals that do not exist yet.
        _ => TransitionCause::NotCanonicalJson,
    })?;
    if canonical == text.as_bytes() {
        Ok(())
    } else {
        Err(TransitionCause::NotCanonicalJson)
    }
}

/// Read a stored text again and embed its document as one value.
fn embed<S: Sink + ?Sized>(writer: &mut Writer<'_, S>, text: &str) -> Result<(), Error> {
    let document = read(text.as_bytes(), u64::MAX).map_err(|error| match error {
        ReadError::Limit(limit) => Error::Limit(limit),
        ReadError::Allocation { requested } => Error::Allocation { requested },
        _ => Error::Internal {
            invariant: "a stored transition text was admitted as canonical JSON",
        },
    })?;
    document.encode_into(writer)
}

impl Encode for ModelTransition {
    /// Push the identity as events. The shape above the stored texts is fixed
    /// (object, array, texts); each text is embedded by `quire-canonical`'s
    /// own explicit-stack encoding, so no step follows a text's nesting.
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("arguments")?;
        writer.begin_array()?;
        for text in &self.arguments {
            embed(writer, text)?;
        }
        writer.end_array()?;
        writer.name("operation")?;
        writer.string(&self.operation)?;
        writer.name("receiver")?;
        match &self.receiver {
            Some(text) => embed(writer, text)?,
            None => writer.null()?,
        }
        writer.name("type")?;
        writer.string(TRANSITION_TYPE)?;
        writer.end_object()
    }
}

impl Ord for ModelTransition {
    /// Ascending unsigned-lexicographic order of [`ModelTransition::canonical_bytes`],
    /// computed without building the bytes: it takes no limits, cannot fail
    /// and allocates nothing.
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity_bytes().cmp(other.identity_bytes())
    }
}

impl PartialOrd for ModelTransition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// One byte of a JSON string body, as RFC 8785 escapes it: up to six bytes.
struct Escaped {
    bytes: [u8; 6],
    next: usize,
    length: usize,
}

impl Escaped {
    fn new(byte: u8) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let (bytes, length) = match byte {
            b'"' => ([b'\\', b'"', 0, 0, 0, 0], 2),
            b'\\' => ([b'\\', b'\\', 0, 0, 0, 0], 2),
            0x08 => ([b'\\', b'b', 0, 0, 0, 0], 2),
            0x09 => ([b'\\', b't', 0, 0, 0, 0], 2),
            0x0a => ([b'\\', b'n', 0, 0, 0, 0], 2),
            0x0c => ([b'\\', b'f', 0, 0, 0, 0], 2),
            0x0d => ([b'\\', b'r', 0, 0, 0, 0], 2),
            0x00..=0x1f => (
                [
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    HEX[usize::from(byte >> 4)],
                    HEX[usize::from(byte & 0x0f)],
                ],
                6,
            ),
            0x20..=0x21 | 0x23..=0x5b | 0x5d..=0xff => ([byte, 0, 0, 0, 0, 0], 1),
        };
        Self {
            bytes,
            next: 0,
            length,
        }
    }
}

impl Iterator for Escaped {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.next >= self.length {
            return None;
        }
        let byte = self.bytes.get(self.next).copied();
        self.next += 1;
        byte
    }
}
