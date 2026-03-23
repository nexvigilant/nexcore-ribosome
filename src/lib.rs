// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # NexVigilant Core — ribosome — Schema Contract Registry with Drift Detection
//!
//! Translates inferred schemas into enforceable contracts, detects drift,
//! generates synthetic test data, and emits Guardian-compatible DAMP signals.
//!
//! ## Biological Analogy
//!
//! In biology, ribosomes translate mRNA → protein.
//! This ribosome translates schemas → contracts → enforcement.
//!
//! ## Pipeline
//!
//! ```text
//! transcriptase (foundation) → ribosome (domain) → Guardian (orchestration)
//!      Schema                    Contract + Drift       Signal<DriftPattern>
//! ```
//!
//! ## Drift Score Formula
//!
//! ```text
//! drift = (type_drift × 0.5) + (range_drift × 0.3) + (structure_drift × 0.2)
//! ```
//!
//! ## Tier: T2-C (κ + σ + μ + ∂ + N)

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::allow_attributes_without_reason,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::disallowed_types,
    clippy::indexing_slicing,
    clippy::iter_over_hash_type,
    clippy::wildcard_enum_match_arm,
    clippy::too_many_arguments,
    clippy::let_underscore_must_use,
    reason = "Ribosome contract translation code favors explicit schema math and compatibility with prior persisted contracts"
)]

pub mod grounding;
pub mod pv_contracts;
pub mod stall_monitor;

use std::collections::HashMap;
use std::fmt;

use nexcore_chrono::DateTime;
use nexcore_transcriptase::{Schema, SchemaKind};
use serde::{Deserialize, Serialize};

// ─── Error Types ────────────────────────────────────────────────────────────

/// Ribosome error type.
#[derive(Debug, nexcore_error::Error)]
pub enum RibosomeError {
    #[error("∂[json]: {0}")]
    Json(#[from] serde_json::Error),

    #[error("∂[transcriptase]: {0}")]
    Transcriptase(#[from] nexcore_transcriptase::TranscriptaseError),

    #[error("∂[internal]: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, RibosomeError>;

// ─── Configuration ──────────────────────────────────────────────────────────

/// Ribosome configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RibosomeConfig {
    /// Drift threshold — score >= this triggers a signal (default: 0.25).
    pub drift_threshold: f64,
    /// Whether to auto-update contracts on re-store (default: false).
    pub auto_update: bool,
}

impl Default for RibosomeConfig {
    fn default() -> Self {
        Self {
            drift_threshold: 0.25,
            auto_update: false,
        }
    }
}

// ─── Core Types ─────────────────────────────────────────────────────────────

/// Stored schema contract with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Unique contract identifier.
    pub id: String,
    /// The baseline schema.
    pub schema: Schema,
    /// When this contract was created.
    pub created_at: DateTime,
    /// When this contract was last updated.
    pub updated_at: DateTime,
    /// Number of observations that built this contract.
    pub observation_count: usize,
    /// Arbitrary metadata.
    pub metadata: HashMap<String, String>,
}

/// Drift detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftResult {
    /// Which contract was validated against.
    pub contract_id: String,
    /// Overall drift score: 0.0 = identical, 1.0 = completely different.
    pub drift_score: f64,
    /// Whether drift exceeds threshold.
    pub drift_detected: bool,
    /// Per-field drift violations.
    pub violations: Vec<SchemaDrift>,
    /// When validation occurred.
    pub validated_at: DateTime,
}

/// Specific schema drift violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDrift {
    /// Field path (dot-separated).
    pub field: String,
    /// What kind of drift.
    pub drift_type: DriftType,
    /// What was expected (from baseline).
    pub expected: String,
    /// What was observed (from data).
    pub observed: String,
    /// How serious is this drift.
    pub severity: DriftSeverity,
}

/// Types of schema drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftType {
    /// Int → String, etc.
    TypeMismatch,
    /// Field disappeared from data.
    MissingField,
    /// New field appeared in data.
    ExtraField,
    /// Values outside baseline range.
    RangeExpansion,
    /// Range narrowed from baseline.
    RangeContraction,
    /// String length shifted.
    LengthChange,
    /// Array bounds shifted.
    ArraySizeChange,
}

/// Severity of a drift violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DriftSeverity {
    /// Informational — no action needed.
    Info,
    /// Warning — investigate.
    Warning,
    /// Critical — enforce.
    Critical,
}

/// Guardian-agnostic drift signal.
///
/// Caller wraps into `Signal<DriftPattern>` + DAMP source at the Guardian layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSignal {
    /// Which contract triggered this signal.
    pub contract_id: String,
    /// Overall drift score.
    pub drift_score: f64,
    /// Per-field violations.
    pub violations: Vec<SchemaDrift>,
    /// Confidence: 1.0 - (drift_score × 0.2).
    pub confidence: f64,
}

// ─── Display Traits ─────────────────────────────────────────────────────────

impl fmt::Display for DriftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch => write!(f, "TYPE_MISMATCH"),
            Self::MissingField => write!(f, "MISSING_FIELD"),
            Self::ExtraField => write!(f, "EXTRA_FIELD"),
            Self::RangeExpansion => write!(f, "RANGE_EXPANSION"),
            Self::RangeContraction => write!(f, "RANGE_CONTRACTION"),
            Self::LengthChange => write!(f, "LENGTH_CHANGE"),
            Self::ArraySizeChange => write!(f, "ARRAY_SIZE_CHANGE"),
        }
    }
}

impl fmt::Display for DriftSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl fmt::Display for DriftSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DriftSignal({}: score={:.3}, violations={}, confidence={:.3})",
            self.contract_id,
            self.drift_score,
            self.violations.len(),
            self.confidence
        )
    }
}

impl fmt::Display for DriftResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.drift_detected {
            "DRIFT_DETECTED"
        } else {
            "STABLE"
        };
        write!(
            f,
            "[{}] {} — score={:.3}, violations={}",
            status,
            self.contract_id,
            self.drift_score,
            self.violations.len()
        )
    }
}

// ─── Drift Algorithm ────────────────────────────────────────────────────────

/// Weight for type drift component.
const TYPE_WEIGHT: f64 = 0.5;
/// Weight for range drift component.
const RANGE_WEIGHT: f64 = 0.3;
/// Weight for structure drift component.
const STRUCTURE_WEIGHT: f64 = 0.2;

/// Compute the overall drift score between a baseline schema and observed data.
///
/// Returns a value in [0.0, 1.0]:
/// - 0.0 = identical
/// - 1.0 = completely different
fn compute_drift_score(baseline: &Schema, observed: &Schema) -> f64 {
    let type_score = type_drift_score(&baseline.kind, &observed.kind);
    let range_score = range_drift_score(&baseline.kind, &observed.kind);
    let structure_score = structure_drift_score(&baseline.kind, &observed.kind);

    let raw = structure_score.mul_add(
        STRUCTURE_WEIGHT,
        type_score.mul_add(TYPE_WEIGHT, range_score * RANGE_WEIGHT),
    );

    raw.clamp(0.0, 1.0)
}

/// Type drift: how different are the SchemaKinds?
fn type_drift_score(baseline: &SchemaKind, observed: &SchemaKind) -> f64 {
    match (baseline, observed) {
        // Same kind → no type drift
        (SchemaKind::Null, SchemaKind::Null)
        | (SchemaKind::Bool { .. }, SchemaKind::Bool { .. })
        | (SchemaKind::Int { .. }, SchemaKind::Int { .. })
        | (SchemaKind::Float { .. }, SchemaKind::Float { .. })
        | (SchemaKind::Str { .. }, SchemaKind::Str { .. })
        | (SchemaKind::Array { .. }, SchemaKind::Array { .. })
        | (SchemaKind::Record(_), SchemaKind::Record(_)) => 0.0,

        // Int ↔ Float: compatible widening
        (SchemaKind::Int { .. }, SchemaKind::Float { .. })
        | (SchemaKind::Float { .. }, SchemaKind::Int { .. }) => 0.3,

        // Anything involving Mixed
        (SchemaKind::Mixed, _) | (_, SchemaKind::Mixed) => 0.5,

        // Complete type mismatch
        _ => 1.0,
    }
}

/// Range drift: how much have numeric/string ranges shifted?
#[allow(clippy::cast_precision_loss)] // Acceptable: range differences are typically small; exact precision not required for drift scoring
fn range_drift_score(baseline: &SchemaKind, observed: &SchemaKind) -> f64 {
    match (baseline, observed) {
        (
            SchemaKind::Int {
                min: b_min,
                max: b_max,
                ..
            },
            SchemaKind::Int {
                min: o_min,
                max: o_max,
                ..
            },
        ) => {
            let base_range = (*b_max - *b_min).max(1) as f64;
            let obs_range = (*o_max - *o_min).max(1) as f64;
            ((obs_range - base_range).abs() / base_range).min(1.0)
        }

        (
            SchemaKind::Float {
                min: b_min,
                max: b_max,
                ..
            },
            SchemaKind::Float {
                min: o_min,
                max: o_max,
                ..
            },
        ) => {
            let base_range = (b_max - b_min).max(f64::EPSILON);
            let obs_range = (o_max - o_min).max(f64::EPSILON);
            ((obs_range - base_range).abs() / base_range).min(1.0)
        }

        (
            SchemaKind::Str {
                min_len: b_min,
                max_len: b_max,
                ..
            },
            SchemaKind::Str {
                min_len: o_min,
                max_len: o_max,
                ..
            },
        ) => {
            #[allow(clippy::cast_precision_loss)]
            // String lengths are small; exact precision not needed
            let base_range = (*b_max - *b_min).max(1) as f64;
            #[allow(clippy::cast_precision_loss)]
            let obs_range = (*o_max - *o_min).max(1) as f64;
            ((obs_range - base_range).abs() / base_range).min(1.0)
        }

        // Non-range types → no range drift
        _ => 0.0,
    }
}

/// Structure drift: field additions/removals in records, array size changes,
/// and recursive drift within shared fields.
#[allow(clippy::cast_precision_loss)] // Field counts and array sizes are small; exact precision not required for drift scoring
fn structure_drift_score(baseline: &SchemaKind, observed: &SchemaKind) -> f64 {
    match (baseline, observed) {
        (SchemaKind::Record(b_fields), SchemaKind::Record(o_fields)) => {
            let total = b_fields.len().max(o_fields.len()).max(1);
            let missing = b_fields
                .keys()
                .filter(|k| !o_fields.contains_key(*k))
                .count();
            let extra = o_fields
                .keys()
                .filter(|k| !b_fields.contains_key(*k))
                .count();
            let field_drift = (missing + extra) as f64 / total as f64;

            // Recurse into shared fields — average their drift scores
            let shared: Vec<f64> = b_fields
                .iter()
                .filter_map(|(k, b_schema)| {
                    o_fields
                        .get(k)
                        .map(|o_schema| compute_drift_score(b_schema, o_schema))
                })
                .collect();
            let avg_inner = if shared.is_empty() {
                0.0
            } else {
                shared.iter().sum::<f64>() / shared.len() as f64
            };

            (field_drift + avg_inner).min(1.0)
        }

        (
            SchemaKind::Array {
                element: b_elem,
                min_len: b_min,
                max_len: b_max,
            },
            SchemaKind::Array {
                element: o_elem,
                min_len: o_min,
                max_len: o_max,
            },
        ) => {
            // Size drift
            let base_range = (*b_max - *b_min).max(1) as f64;
            let obs_range = (*o_max - *o_min).max(1) as f64;
            let size_drift = ((obs_range - base_range).abs() / base_range).min(1.0);

            // Element drift (recursive, attenuated 0.5×)
            let elem_drift = compute_drift_score(b_elem, o_elem) * 0.5;

            (size_drift + elem_drift).min(1.0)
        }

        _ => 0.0,
    }
}

/// Collect per-field drift violations between baseline and observed schemas.
#[allow(clippy::too_many_lines)] // Exhaustive match on all SchemaKind pairs requires many arms; splitting would hurt readability
fn collect_drift_violations(
    baseline: &Schema,
    observed: &Schema,
    prefix: &str,
) -> Vec<SchemaDrift> {
    let mut violations = Vec::new();
    let path = match (&baseline.name, prefix.is_empty()) {
        (Some(n), true) => n.clone(),
        (Some(n), false) => format!("{prefix}.{n}"),
        (None, _) => prefix.to_string(),
    };

    match (&baseline.kind, &observed.kind) {
        // Type mismatch (excluding compatible widening)
        (SchemaKind::Int { .. }, SchemaKind::Float { .. })
        | (SchemaKind::Float { .. }, SchemaKind::Int { .. }) => {
            violations.push(SchemaDrift {
                field: path,
                drift_type: DriftType::TypeMismatch,
                expected: kind_name(&baseline.kind),
                observed: kind_name(&observed.kind),
                severity: DriftSeverity::Warning,
            });
        }

        // Same kind — check ranges
        (
            SchemaKind::Int {
                min: b_min,
                max: b_max,
                ..
            },
            SchemaKind::Int {
                min: o_min,
                max: o_max,
                ..
            },
        ) => {
            let expanded = o_min < b_min || o_max > b_max;
            let contracted = o_min > b_min || o_max < b_max;
            if expanded {
                violations.push(SchemaDrift {
                    field: path,
                    drift_type: DriftType::RangeExpansion,
                    expected: format!("[{b_min}, {b_max}]"),
                    observed: format!("[{o_min}, {o_max}]"),
                    severity: DriftSeverity::Warning,
                });
            } else if contracted {
                violations.push(SchemaDrift {
                    field: path,
                    drift_type: DriftType::RangeContraction,
                    expected: format!("[{b_min}, {b_max}]"),
                    observed: format!("[{o_min}, {o_max}]"),
                    severity: DriftSeverity::Info,
                });
            }
        }

        (
            SchemaKind::Float {
                min: b_min,
                max: b_max,
                ..
            },
            SchemaKind::Float {
                min: o_min,
                max: o_max,
                ..
            },
        ) => {
            let expanded = o_min < b_min || o_max > b_max;
            let contracted = o_min > b_min || o_max < b_max;
            if expanded {
                violations.push(SchemaDrift {
                    field: path,
                    drift_type: DriftType::RangeExpansion,
                    expected: format!("[{b_min}, {b_max}]"),
                    observed: format!("[{o_min}, {o_max}]"),
                    severity: DriftSeverity::Warning,
                });
            } else if contracted {
                violations.push(SchemaDrift {
                    field: path,
                    drift_type: DriftType::RangeContraction,
                    expected: format!("[{b_min}, {b_max}]"),
                    observed: format!("[{o_min}, {o_max}]"),
                    severity: DriftSeverity::Info,
                });
            }
        }

        (
            SchemaKind::Str {
                min_len: b_min,
                max_len: b_max,
                ..
            },
            SchemaKind::Str {
                min_len: o_min,
                max_len: o_max,
                ..
            },
        ) => {
            if o_min != b_min || o_max != b_max {
                violations.push(SchemaDrift {
                    field: path,
                    drift_type: DriftType::LengthChange,
                    expected: format!("len[{b_min}, {b_max}]"),
                    observed: format!("len[{o_min}, {o_max}]"),
                    severity: DriftSeverity::Info,
                });
            }
        }

        (
            SchemaKind::Array {
                element: b_elem,
                min_len: b_min,
                max_len: b_max,
            },
            SchemaKind::Array {
                element: o_elem,
                min_len: o_min,
                max_len: o_max,
            },
        ) => {
            if o_min != b_min || o_max != b_max {
                violations.push(SchemaDrift {
                    field: path.clone(),
                    drift_type: DriftType::ArraySizeChange,
                    expected: format!("size[{b_min}, {b_max}]"),
                    observed: format!("size[{o_min}, {o_max}]"),
                    severity: DriftSeverity::Warning,
                });
            }
            let elem_violations = collect_drift_violations(b_elem, o_elem, &format!("{path}[]"));
            violations.extend(elem_violations);
        }

        (SchemaKind::Record(b_fields), SchemaKind::Record(o_fields)) => {
            // Missing fields
            for key in b_fields.keys() {
                if !o_fields.contains_key(key) {
                    violations.push(SchemaDrift {
                        field: if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{path}.{key}")
                        },
                        drift_type: DriftType::MissingField,
                        expected: "present".to_string(),
                        observed: "absent".to_string(),
                        severity: DriftSeverity::Critical,
                    });
                }
            }
            // Extra fields
            for key in o_fields.keys() {
                if !b_fields.contains_key(key) {
                    violations.push(SchemaDrift {
                        field: if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{path}.{key}")
                        },
                        drift_type: DriftType::ExtraField,
                        expected: "absent".to_string(),
                        observed: "present".to_string(),
                        severity: DriftSeverity::Warning,
                    });
                }
            }
            // Recurse into shared fields
            for (key, b_schema) in b_fields {
                if let Some(o_schema) = o_fields.get(key) {
                    let field_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    let field_violations =
                        collect_drift_violations(b_schema, o_schema, &field_path);
                    violations.extend(field_violations);
                }
            }
        }

        // Complete type mismatch
        _ if !matches!(
            (&baseline.kind, &observed.kind),
            (SchemaKind::Null, SchemaKind::Null)
                | (SchemaKind::Bool { .. }, SchemaKind::Bool { .. })
                | (SchemaKind::Mixed, SchemaKind::Mixed)
        ) =>
        {
            violations.push(SchemaDrift {
                field: path,
                drift_type: DriftType::TypeMismatch,
                expected: kind_name(&baseline.kind),
                observed: kind_name(&observed.kind),
                severity: DriftSeverity::Critical,
            });
        }

        _ => {}
    }

    violations
}

/// Human-readable name for a `SchemaKind`.
fn kind_name(kind: &SchemaKind) -> String {
    match kind {
        SchemaKind::Null => "Null".to_string(),
        SchemaKind::Bool { .. } => "Bool".to_string(),
        SchemaKind::Int { .. } => "Int".to_string(),
        SchemaKind::Float { .. } => "Float".to_string(),
        SchemaKind::Str { .. } => "Str".to_string(),
        SchemaKind::Array { .. } => "Array".to_string(),
        SchemaKind::Record(_) => "Record".to_string(),
        SchemaKind::Mixed => "Mixed".to_string(),
    }
}

// ─── Ribosome Registry ──────────────────────────────────────────────────────

/// The Ribosome — schema contract registry with drift detection.
#[derive(Debug)]
pub struct Ribosome {
    config: RibosomeConfig,
    contracts: HashMap<String, Contract>,
}

impl Ribosome {
    /// Create a new ribosome with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: RibosomeConfig::default(),
            contracts: HashMap::new(),
        }
    }

    /// Create a new ribosome with custom config.
    #[must_use]
    pub fn with_config(config: RibosomeConfig) -> Self {
        Self {
            config,
            contracts: HashMap::new(),
        }
    }

    /// Get the current configuration.
    #[must_use]
    pub fn config(&self) -> &RibosomeConfig {
        &self.config
    }

    // ── Contract Management ─────────────────────────────────────────────

    /// Store a contract from a baseline schema.
    ///
    /// If the ID already exists:
    /// - With `auto_update=true`: merges the new schema into the existing contract
    /// - With `auto_update=false`: returns the existing contract unchanged
    pub fn store_contract(&mut self, id: impl Into<String>, schema: Schema) -> Result<Contract> {
        let id = id.into();
        let now = DateTime::now();

        if let Some(existing) = self.contracts.get_mut(&id) {
            if self.config.auto_update {
                existing.schema = nexcore_transcriptase::merge(&existing.schema, &schema);
                existing.updated_at = now;
                existing.observation_count += schema.observations;
                return Ok(existing.clone());
            }
            return Ok(existing.clone());
        }

        let contract = Contract {
            id: id.clone(),
            schema,
            created_at: now,
            updated_at: now,
            observation_count: 1,
            metadata: HashMap::new(),
        };

        self.contracts.insert(id, contract.clone());
        Ok(contract)
    }

    /// Get a contract by ID.
    #[must_use]
    pub fn get_contract(&self, id: &str) -> Option<&Contract> {
        self.contracts.get(id)
    }

    /// List all contract IDs.
    #[must_use]
    pub fn list_contracts(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.contracts.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Delete a contract by ID.
    pub fn delete_contract(&mut self, id: &str) -> Option<Contract> {
        self.contracts.remove(id)
    }

    /// Number of stored contracts.
    #[must_use]
    pub fn contract_count(&self) -> usize {
        self.contracts.len()
    }

    // ── Validation ──────────────────────────────────────────────────────

    /// Validate data against a stored contract.
    ///
    /// Returns `None` if the contract doesn't exist (fail-open).
    #[must_use]
    pub fn validate(&self, contract_id: &str, data: &serde_json::Value) -> Option<DriftResult> {
        let contract = self.contracts.get(contract_id)?;
        let observed = nexcore_transcriptase::infer(data);

        let drift_score = compute_drift_score(&contract.schema, &observed);
        let violations = collect_drift_violations(&contract.schema, &observed, "");

        Some(DriftResult {
            contract_id: contract_id.to_string(),
            drift_score,
            drift_detected: drift_score >= self.config.drift_threshold,
            violations,
            validated_at: DateTime::now(),
        })
    }

    // ── Generation ──────────────────────────────────────────────────────

    /// Generate synthetic data from a stored contract's schema.
    ///
    /// Returns `None` if the contract doesn't exist (fail-open).
    #[must_use]
    pub fn generate(&self, contract_id: &str) -> Option<serde_json::Value> {
        let contract = self.contracts.get(contract_id)?;
        Some(nexcore_transcriptase::generate(&contract.schema))
    }

    /// Generate a batch of synthetic records.
    ///
    /// Returns `None` if the contract doesn't exist (fail-open).
    #[must_use]
    pub fn generate_batch(
        &self,
        contract_id: &str,
        count: usize,
    ) -> Option<Vec<serde_json::Value>> {
        let contract = self.contracts.get(contract_id)?;
        Some(
            (0..count)
                .map(|_| nexcore_transcriptase::generate(&contract.schema))
                .collect(),
        )
    }

    // ── Batch Drift Detection ───────────────────────────────────────────

    /// Validate multiple data samples against their contracts.
    ///
    /// Only emits `DriftSignal` when `drift_score >= threshold`.
    /// Silently skips unknown contract IDs (fail-open).
    #[must_use]
    pub fn detect_drift(&self, data: &HashMap<String, serde_json::Value>) -> Vec<DriftSignal> {
        let mut signals = Vec::new();

        for (contract_id, value) in data {
            if let Some(result) = self.validate(contract_id, value) {
                if result.drift_detected {
                    signals.push(DriftSignal {
                        contract_id: contract_id.clone(),
                        drift_score: result.drift_score,
                        violations: result.violations,
                        confidence: result.drift_score.mul_add(-0.2, 1.0),
                    });
                }
            }
        }

        signals
    }
}

impl Default for Ribosome {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_schema(json: &serde_json::Value) -> Schema {
        nexcore_transcriptase::infer(json)
    }

    // ── Contract CRUD ───────────────────────────────────────────────────

    #[test]
    fn test_store_contract() {
        let mut rb = Ribosome::new();
        let schema = make_schema(&json!({"drug": "aspirin", "cases": 42}));
        let result = rb.store_contract("icsr-v1", schema);
        assert!(result.is_ok());
        assert_eq!(rb.contract_count(), 1);
    }

    #[test]
    fn test_store_existing_no_update() {
        let mut rb = Ribosome::new();
        let s1 = make_schema(&json!({"cases": 10}));
        let s2 = make_schema(&json!({"cases": 90}));
        let _ = rb.store_contract("c1", s1);
        let _ = rb.store_contract("c1", s2);
        // Without auto_update, should still have original
        assert_eq!(rb.contract_count(), 1);
        let c = rb.get_contract("c1");
        assert!(c.is_some());
        if let Some(contract) = c {
            if let SchemaKind::Record(fields) = &contract.schema.kind {
                if let Some(cases) = fields.get("cases") {
                    if let SchemaKind::Int { max, .. } = &cases.kind {
                        assert_eq!(*max, 10); // original, not updated
                    }
                }
            }
        }
    }

    #[test]
    fn test_store_existing_with_auto_update() {
        let config = RibosomeConfig {
            auto_update: true,
            ..RibosomeConfig::default()
        };
        let mut rb = Ribosome::with_config(config);
        let s1 = make_schema(&json!({"cases": 10}));
        let s2 = make_schema(&json!({"cases": 90}));
        let _ = rb.store_contract("c1", s1);
        let _ = rb.store_contract("c1", s2);
        assert_eq!(rb.contract_count(), 1);
        let c = rb.get_contract("c1");
        assert!(c.is_some());
        if let Some(contract) = c {
            if let SchemaKind::Record(fields) = &contract.schema.kind {
                if let Some(cases) = fields.get("cases") {
                    if let SchemaKind::Int { min, max, .. } = &cases.kind {
                        assert_eq!(*min, 10);
                        assert_eq!(*max, 90); // merged!
                    }
                }
            }
        }
    }

    #[test]
    fn test_get_nonexistent() {
        let rb = Ribosome::new();
        assert!(rb.get_contract("nope").is_none());
    }

    #[test]
    fn test_list_contracts() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("beta", make_schema(&json!(1)));
        let _ = rb.store_contract("alpha", make_schema(&json!(2)));
        let ids = rb.list_contracts();
        assert_eq!(ids, vec!["alpha", "beta"]); // sorted
    }

    #[test]
    fn test_delete_contract() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!(42)));
        assert_eq!(rb.contract_count(), 1);
        let deleted = rb.delete_contract("c1");
        assert!(deleted.is_some());
        assert_eq!(rb.contract_count(), 0);
        assert!(rb.delete_contract("c1").is_none());
    }

    // ── Drift Calculation ───────────────────────────────────────────────

    #[test]
    fn test_drift_identical_is_zero() {
        let schema = make_schema(&json!({"score": 50}));
        let score = compute_drift_score(&schema, &schema);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_drift_type_mismatch_high() {
        let baseline = make_schema(&json!(42));
        let observed = make_schema(&json!("hello"));
        let score = compute_drift_score(&baseline, &observed);
        assert!(score >= 0.5); // type mismatch dominates
    }

    #[test]
    fn test_drift_int_float_low() {
        let baseline = make_schema(&json!(42));
        let observed = make_schema(&json!(42.0));
        let score = compute_drift_score(&baseline, &observed);
        assert!(score > 0.0); // some drift
        assert!(score < 0.5); // but not catastrophic
    }

    #[test]
    fn test_drift_range_expansion() {
        let baseline = make_schema(&json!(50));
        // Observe with wider range
        let mut engine = nexcore_transcriptase::Engine::new();
        engine.observe(&json!(0));
        engine.observe(&json!(100));
        let wide_schema = engine.schema().cloned();
        assert!(wide_schema.is_some());
        if let Some(ws) = &wide_schema {
            let score = compute_drift_score(&baseline, ws);
            assert!(score > 0.0);
        }
    }

    #[test]
    fn test_drift_missing_extra_fields() {
        let baseline = make_schema(&json!({"a": 1, "b": 2}));
        let observed = make_schema(&json!({"a": 1, "c": 3}));
        let score = compute_drift_score(&baseline, &observed);
        // Missing "b" + extra "c" → structure drift > 0
        assert!(score > 0.0);
    }

    #[test]
    fn test_drift_nested_record() {
        let baseline = make_schema(&json!({"inner": {"x": 10}}));
        let observed = make_schema(&json!({"inner": {"x": 10, "y": 20}}));
        let violations = collect_drift_violations(&baseline, &observed, "");
        assert!(!violations.is_empty());
        assert!(
            violations
                .iter()
                .any(|v| v.drift_type == DriftType::ExtraField)
        );
    }

    #[test]
    fn test_drift_score_bounded() {
        // Even extreme mismatch stays in [0, 1]
        let baseline = make_schema(&json!({"a": 1, "b": [1, 2], "c": "hello"}));
        let observed = make_schema(&json!(true));
        let score = compute_drift_score(&baseline, &observed);
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_drift_array_size_change() {
        let baseline = make_schema(&json!([1, 2, 3]));
        let observed = make_schema(&json!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
        let violations = collect_drift_violations(&baseline, &observed, "");
        assert!(
            violations
                .iter()
                .any(|v| v.drift_type == DriftType::ArraySizeChange)
        );
    }

    // ── Validation ──────────────────────────────────────────────────────

    #[test]
    fn test_validate_no_drift() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"score": 50})));
        let result = rb.validate("c1", &json!({"score": 50}));
        assert!(result.is_some());
        if let Some(dr) = result {
            assert!(!dr.drift_detected);
            assert!((dr.drift_score - 0.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_validate_type_drift() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"score": 50})));
        let result = rb.validate("c1", &json!({"score": "fifty"}));
        assert!(result.is_some());
        if let Some(dr) = result {
            assert!(dr.drift_score > 0.0);
            assert!(!dr.violations.is_empty());
        }
    }

    #[test]
    fn test_validate_field_drift() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"a": 1, "b": 2})));
        let result = rb.validate("c1", &json!({"a": 1, "c": 3}));
        assert!(result.is_some());
        if let Some(dr) = result {
            assert!(!dr.violations.is_empty());
            assert!(
                dr.violations
                    .iter()
                    .any(|v| v.drift_type == DriftType::MissingField)
            );
            assert!(
                dr.violations
                    .iter()
                    .any(|v| v.drift_type == DriftType::ExtraField)
            );
        }
    }

    #[test]
    fn test_validate_nonexistent_contract() {
        let rb = Ribosome::new();
        assert!(rb.validate("nope", &json!(42)).is_none());
    }

    #[test]
    fn test_validate_threshold_boundary() {
        let config = RibosomeConfig {
            drift_threshold: 0.01, // very sensitive
            ..RibosomeConfig::default()
        };
        let mut rb = Ribosome::with_config(config);
        let _ = rb.store_contract("c1", make_schema(&json!({"x": 10})));
        // Even a small range change should trigger at 0.01 threshold
        let result = rb.validate("c1", &json!({"x": 10, "y": 20}));
        assert!(result.is_some());
        if let Some(dr) = result {
            assert!(dr.drift_detected);
        }
    }

    #[test]
    fn test_validate_range_drift() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"score": 50})));
        let result = rb.validate("c1", &json!({"score": 500}));
        assert!(result.is_some());
        if let Some(dr) = result {
            assert!(!dr.violations.is_empty());
        }
    }

    // ── Generation ──────────────────────────────────────────────────────

    #[test]
    fn test_generate_from_contract() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"drug": "aspirin", "cases": 42})));
        let generated = rb.generate("c1");
        assert!(generated.is_some());
        if let Some(val) = generated {
            assert!(val.is_object());
            let obj = val.as_object();
            assert!(obj.is_some());
            if let Some(map) = obj {
                assert!(map.contains_key("drug"));
                assert!(map.contains_key("cases"));
            }
        }
    }

    #[test]
    fn test_generate_batch_count() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"x": 10})));
        let batch = rb.generate_batch("c1", 5);
        assert!(batch.is_some());
        if let Some(records) = batch {
            assert_eq!(records.len(), 5);
        }
    }

    #[test]
    fn test_generate_nonexistent() {
        let rb = Ribosome::new();
        assert!(rb.generate("nope").is_none());
        assert!(rb.generate_batch("nope", 3).is_none());
    }

    // ── Drift Signals ───────────────────────────────────────────────────

    #[test]
    fn test_drift_signal_above_threshold() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"a": 1})));
        let mut data = HashMap::new();
        // Complete type mismatch: Record vs Bool → drift >> 0.25
        data.insert("c1".to_string(), json!(true));
        let signals = rb.detect_drift(&data);
        assert!(!signals.is_empty());
    }

    #[test]
    fn test_drift_signal_below_threshold() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"a": 1})));
        let mut data = HashMap::new();
        data.insert("c1".to_string(), json!({"a": 1})); // identical
        let signals = rb.detect_drift(&data);
        assert!(signals.is_empty());
    }

    #[test]
    fn test_drift_signal_multiple_contracts() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"a": 1})));
        let _ = rb.store_contract("c2", make_schema(&json!({"b": "hello"})));
        let mut data = HashMap::new();
        data.insert("c1".to_string(), json!({"a": 1})); // identical → no signal
        data.insert("c2".to_string(), json!(42)); // type mismatch → signal
        let signals = rb.detect_drift(&data);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].contract_id, "c2");
    }

    #[test]
    fn test_drift_signal_confidence_decreases() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"a": 1})));
        let mut data = HashMap::new();
        data.insert("c1".to_string(), json!(true)); // total type mismatch
        let signals = rb.detect_drift(&data);
        assert!(!signals.is_empty());
        // confidence = 1.0 - (drift_score * 0.2), drift_score > 0 → confidence < 1.0
        assert!(signals[0].confidence < 1.0);
        assert!(signals[0].confidence > 0.0);
    }

    // ── Serialization ───────────────────────────────────────────────────

    #[test]
    fn test_contract_serializes() {
        let mut rb = Ribosome::new();
        let result = rb.store_contract("c1", make_schema(&json!({"x": 42})));
        assert!(result.is_ok());
        if let Ok(contract) = result {
            let json = serde_json::to_string(&contract);
            assert!(json.is_ok());
        }
    }

    #[test]
    fn test_drift_result_serializes() {
        let mut rb = Ribosome::new();
        let _ = rb.store_contract("c1", make_schema(&json!({"x": 42})));
        let result = rb.validate("c1", &json!({"x": 100}));
        assert!(result.is_some());
        if let Some(dr) = result {
            let json = serde_json::to_string(&dr);
            assert!(json.is_ok());
        }
    }

    #[test]
    fn test_schema_drift_serializes() {
        let drift = SchemaDrift {
            field: "score".to_string(),
            drift_type: DriftType::RangeExpansion,
            expected: "[0, 100]".to_string(),
            observed: "[0, 500]".to_string(),
            severity: DriftSeverity::Warning,
        };
        let json = serde_json::to_string(&drift);
        assert!(json.is_ok());
    }

    #[test]
    fn test_drift_signal_serializes() {
        let signal = DriftSignal {
            contract_id: "c1".to_string(),
            drift_score: 0.5,
            violations: vec![],
            confidence: 0.9,
        };
        let json = serde_json::to_string(&signal);
        assert!(json.is_ok());
    }

    // ── Display ─────────────────────────────────────────────────────────

    #[test]
    fn test_drift_type_display() {
        assert_eq!(format!("{}", DriftType::TypeMismatch), "TYPE_MISMATCH");
        assert_eq!(format!("{}", DriftType::MissingField), "MISSING_FIELD");
        assert_eq!(format!("{}", DriftType::ExtraField), "EXTRA_FIELD");
        assert_eq!(format!("{}", DriftType::RangeExpansion), "RANGE_EXPANSION");
    }

    #[test]
    fn test_drift_severity_display() {
        assert_eq!(format!("{}", DriftSeverity::Info), "INFO");
        assert_eq!(format!("{}", DriftSeverity::Warning), "WARNING");
        assert_eq!(format!("{}", DriftSeverity::Critical), "CRITICAL");
    }

    #[test]
    fn test_drift_signal_display() {
        let signal = DriftSignal {
            contract_id: "test".to_string(),
            drift_score: 0.42,
            violations: vec![],
            confidence: 0.916,
        };
        let s = format!("{signal}");
        assert!(s.contains("test"));
        assert!(s.contains("0.420"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(DriftSeverity::Info < DriftSeverity::Warning);
        assert!(DriftSeverity::Warning < DriftSeverity::Critical);
    }
}
