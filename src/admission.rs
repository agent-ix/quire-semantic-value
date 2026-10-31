// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared input-path admission diagnostics (FR-130).
//!
//! QSL and call evaluators populate this record where they classify an input
//! defect. Admission decisions and observation check order belong to callers.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::catalog::CatalogCode;
use crate::catalog::InternalFault;
use quire_exact::ConstructionCause;

/// A step from a checked value to a nested value, ordered outermost first.
/// An empty sequence names the checked value itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathStep {
    /// Enter a named field, including names that look like numbers.
    Field(String),
    /// Enter the value held by an option.
    OptionValue,
    /// Enter a positional element.
    Index(usize),
}

/// The original catalog and input-path evidence for an admission defect.
///
/// Callers supply the catalog pair and named fields at the site that classifies
/// the defect. `path` is ordered outermost first; `kernel` retains a typed
/// construction cause when one produced the defect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionRecord {
    /// The exact catalog code and cause pair.
    pub catalog: CatalogCode,
    /// Named input fields and their original diagnostic values.
    pub fields: BTreeMap<&'static str, String>,
    /// Typed steps from the checked value to the nested defect.
    pub path: Vec<PathStep>,
    /// The originating kernel cause, when construction was attempted.
    pub kernel: Option<ConstructionCause>,
}

/// The result of admitting an observation, retaining its original diagnostic
/// or invariant fault when no checked authority was admitted (FR-130).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationAdmission<T> {
    /// The admitted observation authority.
    Admitted(T),
    /// The observation input was refused.
    Refused(AdmissionRecord),
    /// Admission lacked evidence needed to decide the observation.
    Incomplete(AdmissionRecord),
    /// An admission invariant failed.
    Fault(InternalFault),
}

impl<T> ObservationAdmission<T> {
    /// Return the admitted authority, or the original non-admitted result.
    pub fn authority(&self) -> Result<&T, &Self> {
        match self {
            Self::Admitted(authority) => Ok(authority),
            Self::Refused(_) | Self::Incomplete(_) | Self::Fault(_) => Err(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::vec;

    use super::{AdmissionRecord, ObservationAdmission, PathStep};
    use crate::catalog::{CatalogCode, InternalFault};
    use quire_exact::ConstructionCause;

    /// Trace: FR-130-AC-1.
    #[test]
    fn admission_record_preserves_catalog_fields_typed_path_and_kernel_cause() {
        let fields = BTreeMap::from([
            ("document", String::from("orders")),
            ("population", String::from("customers")),
            ("object", String::from("customer-7")),
            ("field", String::from("address")),
        ]);
        let record = AdmissionRecord {
            catalog: CatalogCode::new("invalid_runtime_input", "invalid-value"),
            fields: fields.clone(),
            path: vec![
                PathStep::Field(String::from("0")),
                PathStep::OptionValue,
                PathStep::Index(2),
            ],
            kernel: Some(ConstructionCause::TypeMismatch),
        };

        assert_eq!(record.catalog.code(), "invalid_runtime_input");
        assert_eq!(record.catalog.cause(), "invalid-value");
        assert_eq!(record.fields, fields);
        assert_eq!(
            record.path,
            vec![
                PathStep::Field(String::from("0")),
                PathStep::OptionValue,
                PathStep::Index(2),
            ]
        );
        assert_ne!(record.path[0], PathStep::Index(0));
        assert_eq!(record.kernel, Some(ConstructionCause::TypeMismatch));
    }

    /// Trace: FR-130-AC-4, FR-130-AC-5.
    #[test]
    fn observation_authority_query_preserves_each_original_result() {
        let record = AdmissionRecord {
            catalog: CatalogCode::new("invalid_runtime_input", "invalid-value"),
            fields: BTreeMap::from([("field", String::from("address"))]),
            path: vec![PathStep::Field(String::from("0")), PathStep::Index(2)],
            kernel: Some(ConstructionCause::TypeMismatch),
        };
        let admitted = ObservationAdmission::Admitted(17_u8);
        assert_eq!(admitted.authority(), Ok(&17));

        let refused: ObservationAdmission<u8> = ObservationAdmission::Refused(record.clone());
        assert!(
            matches!(refused.authority(), Err(ObservationAdmission::Refused(original))
            if original == &record)
        );

        let incomplete: ObservationAdmission<u8> = ObservationAdmission::Incomplete(record.clone());
        assert!(
            matches!(incomplete.authority(), Err(ObservationAdmission::Incomplete(original))
            if original == &record)
        );

        let fault = InternalFault::new("admission", "missing-observation-authority");
        let failed: ObservationAdmission<u8> = ObservationAdmission::Fault(fault.clone());
        assert!(
            matches!(failed.authority(), Err(ObservationAdmission::Fault(original))
            if original == &fault)
        );
    }
}
