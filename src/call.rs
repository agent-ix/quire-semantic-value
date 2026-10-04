// SPDX-License-Identifier: AGPL-3.0-or-later
//! The call-admission refusal: a runtime input a call or evaluation refuses
//! before any charge. An evaluator's admission code produces it; a backend
//! admitting arguments to checked code refuses the same way.
//!
//! Each refusal names its stable catalog code as a string
//! ([`InputRefusal::code`]) and its closed cause tag ([`InputRefusal::cause`]).
//! This crate depends on `quire-exact` only, so it cannot name a diagnostic
//! `Code` enum; a consumer maps each refusal to its own enum, and pins that
//! enum's spelling to this string for every variant.

use alloc::string::String;

/// A runtime input a call or evaluation refuses before any charge.
///
/// quire:canonical
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum InputRefusal {
    /// No function of this name: `missing_declaration` / `missing-name`.
    #[error("no function named {0}")]
    UnknownFunction(String),
    /// The argument count differs from the parameter count:
    /// `invalid_runtime_input` / `wrong-value-kind`.
    #[error("{supplied} arguments for {declared} parameters")]
    Arity {
        /// Declared parameters.
        declared: usize,
        /// Supplied arguments.
        supplied: usize,
    },
    /// An argument is not a value of its parameter type:
    /// `invalid_runtime_input` / `wrong-value-kind`.
    #[error("argument {parameter} is not a value of its declared type")]
    WrongValueKind {
        /// The parameter index.
        parameter: usize,
    },
    /// An argument holds a reference with no object in the complete
    /// population: `dangling_reference` / `absent-target-in-complete-population`.
    #[error("argument {parameter} holds a reference with no target object")]
    DanglingReference {
        /// The parameter index.
        parameter: usize,
    },
    /// QSL FR-107: `evaluate_clause`'s name resolves to no state clause of this
    /// package: `missing_declaration` / `missing-name`.
    #[error("no state clause named {0}")]
    UnknownClause(String),
    /// QSL FR-107: `evaluate_clause`'s observations were admitted for a
    /// different clause than the one named, or (QSL FR-115) `evaluate_frame`'s
    /// invocation for a different frame: `invalid_runtime_input` /
    /// `wrong-role-mapping`.
    #[error("the observations were admitted for another clause or frame")]
    ObservationsMismatch,
}

impl InputRefusal {
    /// The stable catalog code: `missing_declaration`,
    /// `invalid_runtime_input` or `dangling_reference`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownFunction(_) | Self::UnknownClause(_) => "missing_declaration",
            Self::Arity { .. } | Self::WrongValueKind { .. } | Self::ObservationsMismatch => {
                "invalid_runtime_input"
            }
            Self::DanglingReference { .. } => "dangling_reference",
        }
    }

    /// The closed cause tag.
    pub fn cause(&self) -> &'static str {
        match self {
            Self::UnknownFunction(_) | Self::UnknownClause(_) => "missing-name",
            Self::Arity { .. } | Self::WrongValueKind { .. } => "wrong-value-kind",
            Self::DanglingReference { .. } => "absent-target-in-complete-population",
            Self::ObservationsMismatch => "wrong-role-mapping",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each refusal names its catalog code and its closed cause tag.
    #[test]
    fn each_refusal_names_its_code_and_cause() {
        let cases = [
            (
                InputRefusal::UnknownFunction(String::from("f")),
                "missing_declaration",
                "missing-name",
            ),
            (
                InputRefusal::UnknownClause(String::from("c")),
                "missing_declaration",
                "missing-name",
            ),
            (
                InputRefusal::Arity {
                    declared: 1,
                    supplied: 2,
                },
                "invalid_runtime_input",
                "wrong-value-kind",
            ),
            (
                InputRefusal::WrongValueKind { parameter: 0 },
                "invalid_runtime_input",
                "wrong-value-kind",
            ),
            (
                InputRefusal::DanglingReference { parameter: 0 },
                "dangling_reference",
                "absent-target-in-complete-population",
            ),
            (
                InputRefusal::ObservationsMismatch,
                "invalid_runtime_input",
                "wrong-role-mapping",
            ),
        ];
        for (refusal, code, cause) in cases {
            assert_eq!(refusal.code(), code, "{refusal:?}");
            assert_eq!(refusal.cause(), cause, "{refusal:?}");
        }
    }
}
