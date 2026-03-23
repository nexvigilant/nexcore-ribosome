//! # GroundsTo implementations for nexcore-ribosome types
//!
//! Connects schema contract registry and drift detection types to the
//! Lex Primitiva type system.
//!
//! ## Biological Analogy
//!
//! In biology, ribosomes translate mRNA into protein.
//! This ribosome translates schemas into enforceable contracts,
//! then detects drift when data deviates from those contracts.
//!
//! ## Key Primitive Mapping
//!
//! - Contract storage: pi (Persistence) -- stored schema contracts
//! - Drift detection: kappa (Comparison) -- baseline vs observed comparison
//! - Drift scoring: N (Quantity) -- numeric drift scores
//! - Schema translation: mu (Mapping) -- schema -> contract

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    Contract, DriftResult, DriftSeverity, DriftSignal, DriftType, Ribosome, RibosomeConfig,
    RibosomeError, SchemaDrift,
};

// ---------------------------------------------------------------------------
// Classification enums -- Sigma (Sum) dominant
// ---------------------------------------------------------------------------

/// DriftType: T2-P (Sigma + kappa), dominant Sigma
///
/// Seven-variant enum classifying drift types: TypeMismatch, MissingField, etc.
/// Sum-dominant: the type IS a categorical alternation of drift categories.
/// Comparison is secondary (each drift type IS a comparison result).
impl GroundsTo for DriftType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- drift type variant
            LexPrimitiva::Comparison, // kappa -- baseline vs observed
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// DriftSeverity: T2-P (Sigma + kappa), dominant Sigma
///
/// Ordinal severity: Info < Warning < Critical.
impl GroundsTo for DriftSeverity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- severity variant
            LexPrimitiva::Comparison, // kappa -- ordinal comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Drift analysis types -- kappa (Comparison) dominant
// ---------------------------------------------------------------------------

/// SchemaDrift: T2-C (kappa + Sigma + lambda + N), dominant kappa
///
/// A specific per-field drift violation.
/// Comparison-dominant: it compares expected vs observed.
/// Sum is secondary (drift type classification).
/// Location is tertiary (field path).
/// Quantity is quaternary (implicit in severity ordering).
impl GroundsTo for SchemaDrift {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- expected vs observed
            LexPrimitiva::Sum,        // Sigma -- drift type
            LexPrimitiva::Location,   // lambda -- field path
            LexPrimitiva::Quantity,   // N -- severity level
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// DriftResult: T2-C (kappa + N + sigma + partial), dominant kappa
///
/// Complete drift validation result with score and violations.
/// Comparison-dominant: the result IS a comparison of contract vs data.
/// Quantity is secondary (drift score [0, 1]).
/// Sequence is tertiary (ordered violations).
/// Boundary is quaternary (threshold-based detection).
impl GroundsTo for DriftResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- contract vs data comparison
            LexPrimitiva::Quantity,   // N -- drift score
            LexPrimitiva::Sequence,   // sigma -- violation ordering
            LexPrimitiva::Boundary,   // partial -- threshold boundary
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// DriftSignal: T2-C (causality + kappa + N + sigma), dominant causality
///
/// Guardian-compatible signal emitted when drift exceeds threshold.
/// Causality-dominant: the signal IS a causal alert (drift caused signal).
/// Comparison is secondary (drift score evaluation).
/// Quantity is tertiary (drift score, confidence).
/// Sequence is quaternary (violation list).
impl GroundsTo for DriftSignal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // causality -- drift causes signal
            LexPrimitiva::Comparison, // kappa -- threshold evaluation
            LexPrimitiva::Quantity,   // N -- drift score, confidence
            LexPrimitiva::Sequence,   // sigma -- violations
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Contract types -- pi (Persistence) dominant
// ---------------------------------------------------------------------------

/// Contract: T2-C (pi + mu + N + varsigma), dominant pi
///
/// A stored schema contract with metadata.
/// Persistence-dominant: a contract IS persistent schema knowledge.
/// Mapping is secondary (schema maps structure to expectations).
/// Quantity is tertiary (observation count).
/// State is quaternary (contract metadata state).
impl GroundsTo for Contract {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- stored contract
            LexPrimitiva::Mapping,     // mu -- schema mapping
            LexPrimitiva::Quantity,    // N -- observation count
            LexPrimitiva::State,       // varsigma -- metadata state
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

/// RibosomeConfig: T2-P (varsigma + partial), dominant varsigma
///
/// Configuration: drift_threshold, auto_update flag.
impl GroundsTo for RibosomeConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- configuration state
            LexPrimitiva::Boundary, // partial -- threshold boundary
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Engine type -- T3
// ---------------------------------------------------------------------------

/// Ribosome: T3 (pi + kappa + mu + varsigma + partial + causality), dominant pi
///
/// The full schema contract registry with drift detection.
/// Persistence-dominant: the ribosome IS a persistent contract store.
/// Comparison is secondary (drift detection).
/// Mapping is tertiary (schema translation).
/// State is quaternary (registry state).
/// Boundary is quinary (threshold enforcement).
/// Causality is senary (drift signals).
impl GroundsTo for Ribosome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- persistent contract store
            LexPrimitiva::Comparison,  // kappa -- drift detection
            LexPrimitiva::Mapping,     // mu -- schema translation
            LexPrimitiva::State,       // varsigma -- registry state
            LexPrimitiva::Boundary,    // partial -- threshold enforcement
            LexPrimitiva::Causality,   // causality -- drift signals
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// RibosomeError: T2-P (partial + Sigma), dominant partial
impl GroundsTo for RibosomeError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error boundary
            LexPrimitiva::Sum,      // Sigma -- error variant
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn drift_type_is_t2p() {
        assert_eq!(DriftType::tier(), Tier::T2Primitive);
        assert_eq!(DriftType::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn schema_drift_is_t2c() {
        assert_eq!(SchemaDrift::tier(), Tier::T2Composite);
        assert_eq!(
            SchemaDrift::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn drift_signal_causality_dominant() {
        assert_eq!(
            DriftSignal::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn contract_persistence_dominant() {
        assert_eq!(
            Contract::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
        assert_eq!(Contract::tier(), Tier::T2Composite);
    }

    #[test]
    fn ribosome_is_t3() {
        assert_eq!(Ribosome::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            Ribosome::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            DriftType::primitive_composition(),
            DriftSeverity::primitive_composition(),
            SchemaDrift::primitive_composition(),
            DriftResult::primitive_composition(),
            DriftSignal::primitive_composition(),
            Contract::primitive_composition(),
            RibosomeConfig::primitive_composition(),
            Ribosome::primitive_composition(),
            RibosomeError::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
