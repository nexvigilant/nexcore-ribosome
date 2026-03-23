//! # PV Schema Contracts — Pharmacovigilance-Specific Schema Enforcement
//!
//! Defines typed schema contracts for FAERS drug, indication, and adverse event
//! records with domain-specific drift thresholds and severity overrides.
//!
//! ## Innovation Scan 001 — Goal 1 (Score: 8.50)
//!
//! ```text
//! FAERS data → transcriptase (infer schema) → ribosome PV contracts → drift detection → Guardian
//! ```
//!
//! ## ToV Alignment: V3 Conservation
//! No signal data lost in pipeline transformation. Schema contracts enforce
//! structural integrity at the data boundary.
//!
//! ## Tier: T2-C (κ + σ + μ + ∂ + N + ν)

use crate::{DriftResult, DriftSeverity, DriftType, RibosomeConfig};
use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Contract Constants ────────────────────────────────────────────────────

/// Contract ID for FAERS drug records.
pub const DRUG_CONTRACT_ID: &str = "pv.drug_record.v1";
/// Contract ID for FAERS indication records.
pub const INDICATION_CONTRACT_ID: &str = "pv.indication_record.v1";
/// Contract ID for FAERS adverse event records.
pub const AE_CONTRACT_ID: &str = "pv.adverse_event_record.v1";

// ─── PV Contract Category ──────────────────────────────────────────────────

/// PV contract category for domain-specific drift thresholds.
///
/// Adverse event records have the tightest thresholds because
/// structural corruption in AE data directly impacts signal detection accuracy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PvContractCategory {
    /// Drug record schema — moderate sensitivity (threshold: 0.20).
    Drug,
    /// Indication record — moderate sensitivity (threshold: 0.20).
    Indication,
    /// Adverse event record — HIGH sensitivity (threshold: 0.10).
    /// Safety-critical: type mismatches are always Critical severity.
    AdverseEvent,
}

impl PvContractCategory {
    /// Domain-specific drift threshold.
    /// AE records use 0.10 (stricter than ribosome default of 0.25).
    #[must_use]
    pub const fn drift_threshold(&self) -> f64 {
        match self {
            Self::Drug | Self::Indication => 0.20,
            Self::AdverseEvent => 0.10,
        }
    }

    /// Contract identifier for this category.
    #[must_use]
    pub const fn contract_id(&self) -> &'static str {
        match self {
            Self::Drug => DRUG_CONTRACT_ID,
            Self::Indication => INDICATION_CONTRACT_ID,
            Self::AdverseEvent => AE_CONTRACT_ID,
        }
    }

    /// PV-specific severity override.
    ///
    /// Type mismatches and missing fields on AE records are ALWAYS Critical
    /// because they can cause silent signal corruption.
    #[must_use]
    pub const fn severity_override(&self, drift_type: DriftType) -> Option<DriftSeverity> {
        match (self, drift_type) {
            // AE records: type changes and missing fields are always critical
            (Self::AdverseEvent, DriftType::TypeMismatch | DriftType::MissingField) => {
                Some(DriftSeverity::Critical)
            }
            // Drug records: type changes are at least Warning
            (Self::Drug, DriftType::TypeMismatch) => Some(DriftSeverity::Warning),
            // All other combinations: use default severity
            _ => None,
        }
    }

    /// All PV contract categories.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Drug, Self::Indication, Self::AdverseEvent]
    }
}

impl fmt::Display for PvContractCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Drug => write!(f, "Drug Record (threshold: 0.20)"),
            Self::Indication => write!(f, "Indication Record (threshold: 0.20)"),
            Self::AdverseEvent => write!(f, "Adverse Event Record (threshold: 0.10)"),
        }
    }
}

// ─── Expected Fields ───────────────────────────────────────────────────────

/// Field descriptor for PV schema contracts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvFieldSpec {
    /// Field name (FAERS column name).
    pub name: &'static str,
    /// Expected JSON type.
    pub expected_type: &'static str,
    /// Whether this field is required (vs optional).
    pub required: bool,
    /// Description for documentation.
    pub description: &'static str,
}

/// Expected fields for a FAERS-compatible drug record.
#[must_use]
pub fn drug_record_fields() -> Vec<PvFieldSpec> {
    vec![
        PvFieldSpec {
            name: "drug_name",
            expected_type: "string",
            required: true,
            description: "Generic or brand drug name",
        },
        PvFieldSpec {
            name: "drug_characterization",
            expected_type: "integer",
            required: true,
            description: "1=Suspect, 2=Concomitant, 3=Interacting",
        },
        PvFieldSpec {
            name: "medicinal_product",
            expected_type: "string",
            required: false,
            description: "Medicinal product name as reported",
        },
        PvFieldSpec {
            name: "drug_indication",
            expected_type: "string",
            required: false,
            description: "Indication for drug use (MedDRA PT)",
        },
        PvFieldSpec {
            name: "route_of_administration",
            expected_type: "string",
            required: false,
            description: "Route of administration code",
        },
        PvFieldSpec {
            name: "drug_dosage_text",
            expected_type: "string",
            required: false,
            description: "Free-text dosage description",
        },
        PvFieldSpec {
            name: "drug_start_date",
            expected_type: "string",
            required: false,
            description: "Drug start date (YYYYMMDD format)",
        },
        PvFieldSpec {
            name: "drug_end_date",
            expected_type: "string",
            required: false,
            description: "Drug end date (YYYYMMDD format)",
        },
    ]
}

/// Expected fields for a FAERS-compatible adverse event record.
#[must_use]
pub fn adverse_event_fields() -> Vec<PvFieldSpec> {
    vec![
        PvFieldSpec {
            name: "reaction_meddra_pt",
            expected_type: "string",
            required: true,
            description: "MedDRA Preferred Term for the adverse reaction",
        },
        PvFieldSpec {
            name: "reaction_outcome",
            expected_type: "integer",
            required: false,
            description: "1=Recovered, 2=Recovering, 3=Not recovered, 4=Recovered with sequelae, 5=Fatal, 6=Unknown",
        },
        PvFieldSpec {
            name: "reaction_start_date",
            expected_type: "string",
            required: false,
            description: "Reaction onset date (YYYYMMDD format)",
        },
        PvFieldSpec {
            name: "serious",
            expected_type: "integer",
            required: true,
            description: "1=Serious, 2=Not serious",
        },
        PvFieldSpec {
            name: "seriousness_death",
            expected_type: "integer",
            required: false,
            description: "1=Yes if resulted in death",
        },
        PvFieldSpec {
            name: "seriousness_hospitalization",
            expected_type: "integer",
            required: false,
            description: "1=Yes if caused/prolonged hospitalization",
        },
        PvFieldSpec {
            name: "seriousness_lifethreatening",
            expected_type: "integer",
            required: false,
            description: "1=Yes if life-threatening",
        },
        PvFieldSpec {
            name: "seriousness_disabling",
            expected_type: "integer",
            required: false,
            description: "1=Yes if caused disability",
        },
    ]
}

/// Expected fields for an indication record.
#[must_use]
pub fn indication_fields() -> Vec<PvFieldSpec> {
    vec![
        PvFieldSpec {
            name: "indication_pt",
            expected_type: "string",
            required: true,
            description: "MedDRA Preferred Term for the indication",
        },
        PvFieldSpec {
            name: "indication_meddra_version",
            expected_type: "string",
            required: false,
            description: "MedDRA dictionary version used",
        },
    ]
}

/// Get field specs for a given PV contract category.
#[must_use]
pub fn fields_for(category: PvContractCategory) -> Vec<PvFieldSpec> {
    match category {
        PvContractCategory::Drug => drug_record_fields(),
        PvContractCategory::Indication => indication_fields(),
        PvContractCategory::AdverseEvent => adverse_event_fields(),
    }
}

// ─── PV Ribosome Configuration ─────────────────────────────────────────────

/// Create a PV-specific ribosome configuration with domain-appropriate thresholds.
///
/// Key difference from default: `auto_update` is ALWAYS false for PV contracts.
/// Safety-critical schemas must never silently drift.
#[must_use]
pub fn pv_config(category: PvContractCategory) -> RibosomeConfig {
    RibosomeConfig {
        drift_threshold: category.drift_threshold(),
        auto_update: false, // NEVER auto-update safety-critical schemas
    }
}

// ─── PV Drift Evaluation ───────────────────────────────────────────────────

/// Recommended action based on PV-specific drift evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PvDriftAction {
    /// No drift detected — continue processing.
    Pass,
    /// Minor drift — log and continue.
    LogAndContinue,
    /// Warning-level drift — flag for human review.
    FlagForReview,
    /// Critical drift — halt the pipeline immediately.
    HaltPipeline,
}

impl fmt::Display for PvDriftAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::LogAndContinue => write!(f, "LOG_AND_CONTINUE"),
            Self::FlagForReview => write!(f, "FLAG_FOR_REVIEW"),
            Self::HaltPipeline => write!(f, "HALT_PIPELINE"),
        }
    }
}

/// Result of evaluating a `DriftResult` with PV-specific rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvDriftEvaluation {
    /// Which PV contract category was evaluated.
    pub category: PvContractCategory,
    /// The contract ID.
    pub contract_id: String,
    /// Raw drift score from ribosome.
    pub drift_score: f64,
    /// Number of Critical-severity violations (after PV overrides).
    pub critical_count: u32,
    /// Number of Warning-severity violations (after PV overrides).
    pub warning_count: u32,
    /// Recommended action.
    pub action: PvDriftAction,
}

impl fmt::Display for PvDriftEvaluation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} — drift={:.3}, critical={}, warnings={}, action={}",
            self.category,
            self.contract_id,
            self.drift_score,
            self.critical_count,
            self.warning_count,
            self.action
        )
    }
}

/// Evaluate a `DriftResult` with PV-specific severity overrides.
///
/// Applies domain-specific rules:
/// - AE record type mismatches → always Critical
/// - AE record missing fields → always Critical
/// - Drug record type mismatches → at least Warning
/// - Critical violations → HALT_PIPELINE
/// - Warning violations → FLAG_FOR_REVIEW
///
/// # Example
///
/// ```ignore
/// let drift_result = ribosome.check_drift("pv.adverse_event_record.v1", &observed);
/// let eval = pv_contracts::evaluate_drift(&drift_result, PvContractCategory::AdverseEvent);
/// if eval.action == PvDriftAction::HaltPipeline {
///     // emit Guardian signal, stop processing
/// }
/// ```
pub fn evaluate_drift(result: &DriftResult, category: PvContractCategory) -> PvDriftEvaluation {
    let mut critical_count = 0u32;
    let mut warning_count = 0u32;

    for violation in &result.violations {
        let effective_severity = category
            .severity_override(violation.drift_type)
            .unwrap_or(violation.severity);

        match effective_severity {
            DriftSeverity::Critical => critical_count += 1,
            DriftSeverity::Warning => warning_count += 1,
            DriftSeverity::Info => {}
        }
    }

    let action = if critical_count > 0 {
        PvDriftAction::HaltPipeline
    } else if warning_count > 0 {
        PvDriftAction::FlagForReview
    } else if result.drift_detected {
        PvDriftAction::LogAndContinue
    } else {
        PvDriftAction::Pass
    };

    PvDriftEvaluation {
        category,
        contract_id: result.contract_id.clone(),
        drift_score: result.drift_score,
        critical_count,
        warning_count,
        action,
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DriftResult, SchemaDrift};
    use nexcore_chrono::DateTime;

    fn make_drift_result(
        contract_id: &str,
        score: f64,
        detected: bool,
        violations: Vec<SchemaDrift>,
    ) -> DriftResult {
        DriftResult {
            contract_id: contract_id.to_string(),
            drift_score: score,
            drift_detected: detected,
            violations,
            validated_at: DateTime::now(),
        }
    }

    fn make_violation(field: &str, drift_type: DriftType, severity: DriftSeverity) -> SchemaDrift {
        SchemaDrift {
            field: field.to_string(),
            drift_type,
            expected: "string".to_string(),
            observed: "integer".to_string(),
            severity,
        }
    }

    #[test]
    fn test_no_drift_passes() {
        let result = make_drift_result(AE_CONTRACT_ID, 0.0, false, vec![]);
        let eval = evaluate_drift(&result, PvContractCategory::AdverseEvent);
        assert_eq!(eval.action, PvDriftAction::Pass);
        assert_eq!(eval.critical_count, 0);
        assert_eq!(eval.warning_count, 0);
    }

    #[test]
    fn test_ae_type_mismatch_escalates_to_critical() {
        let violations = vec![make_violation(
            "reaction_meddra_pt",
            DriftType::TypeMismatch,
            DriftSeverity::Warning, // Would be Warning by default...
        )];
        let result = make_drift_result(AE_CONTRACT_ID, 0.5, true, violations);
        let eval = evaluate_drift(&result, PvContractCategory::AdverseEvent);

        // ...but PV override escalates to Critical
        assert_eq!(eval.critical_count, 1);
        assert_eq!(eval.action, PvDriftAction::HaltPipeline);
    }

    #[test]
    fn test_ae_missing_field_escalates_to_critical() {
        let violations = vec![make_violation(
            "serious",
            DriftType::MissingField,
            DriftSeverity::Info, // Would be Info by default...
        )];
        let result = make_drift_result(AE_CONTRACT_ID, 0.3, true, violations);
        let eval = evaluate_drift(&result, PvContractCategory::AdverseEvent);

        // ...but PV override escalates to Critical for AE missing fields
        assert_eq!(eval.critical_count, 1);
        assert_eq!(eval.action, PvDriftAction::HaltPipeline);
    }

    #[test]
    fn test_drug_type_mismatch_is_warning() {
        let violations = vec![make_violation(
            "drug_name",
            DriftType::TypeMismatch,
            DriftSeverity::Info,
        )];
        let result = make_drift_result(DRUG_CONTRACT_ID, 0.3, true, violations);
        let eval = evaluate_drift(&result, PvContractCategory::Drug);

        assert_eq!(eval.warning_count, 1);
        assert_eq!(eval.critical_count, 0);
        assert_eq!(eval.action, PvDriftAction::FlagForReview);
    }

    #[test]
    fn test_indication_uses_default_severity() {
        let violations = vec![make_violation(
            "indication_pt",
            DriftType::RangeExpansion,
            DriftSeverity::Info,
        )];
        let result = make_drift_result(INDICATION_CONTRACT_ID, 0.15, true, violations);
        let eval = evaluate_drift(&result, PvContractCategory::Indication);

        // No override for indication range expansion → stays Info → LogAndContinue
        assert_eq!(eval.critical_count, 0);
        assert_eq!(eval.warning_count, 0);
        assert_eq!(eval.action, PvDriftAction::LogAndContinue);
    }

    #[test]
    fn test_ae_threshold_is_strictest() {
        assert!(
            PvContractCategory::AdverseEvent.drift_threshold()
                < PvContractCategory::Drug.drift_threshold()
        );
    }

    #[test]
    fn test_pv_config_never_auto_updates() {
        for category in PvContractCategory::all() {
            let config = pv_config(category);
            assert!(!config.auto_update, "PV contracts must never auto-update");
        }
    }

    #[test]
    fn test_field_specs_have_required_fields() {
        let ae_fields = adverse_event_fields();
        let required: Vec<_> = ae_fields.iter().filter(|f| f.required).collect();
        assert!(
            required.len() >= 2,
            "AE records need at least reaction_meddra_pt and serious"
        );
    }

    #[test]
    fn test_drug_fields_completeness() {
        let fields = drug_record_fields();
        assert!(fields.len() >= 8);
        assert!(fields.iter().any(|f| f.name == "drug_name"));
    }

    #[test]
    fn test_all_categories() {
        let all = PvContractCategory::all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_display_format() {
        let eval = PvDriftEvaluation {
            category: PvContractCategory::AdverseEvent,
            contract_id: AE_CONTRACT_ID.to_string(),
            drift_score: 0.35,
            critical_count: 2,
            warning_count: 1,
            action: PvDriftAction::HaltPipeline,
        };
        let display = format!("{eval}");
        assert!(display.contains("HALT_PIPELINE"));
        assert!(display.contains("critical=2"));
    }
}
