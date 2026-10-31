// SPDX-License-Identifier: AGPL-3.0-or-later
//! The authored `absent` keyword of a `lookup` operation (QSL FR-400).
//!
//! This value is shared by parsed forms and checked terms. It records the
//! authored keyword independently of the lookup's result.

/// The authored absence mode of a `lookup` operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AbsenceMode {
    /// `absent undefined`.
    Undefined,
    /// `absent empty`.
    Empty,
    /// `absent refused`.
    Refused,
}

impl AbsenceMode {
    /// The mode's keyword, also used as the checked V2 operation's `absence` spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Empty => "empty",
            Self::Refused => "refused",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AbsenceMode;

    #[test]
    fn each_absence_mode_spells_its_authored_keyword() {
        assert_eq!(AbsenceMode::Undefined.as_str(), "undefined");
        assert_eq!(AbsenceMode::Empty.as_str(), "empty");
        assert_eq!(AbsenceMode::Refused.as_str(), "refused");
    }
}
