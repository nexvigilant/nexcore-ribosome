# nexcore-ribosome

Schema Contract Registry with Drift Detection for the NexVigilant Core kernel. It implements the **Ribosome** model, translating inferred schemas (from `nexcore-transcriptase`) into enforceable contracts and detecting statistical or structural drift in data over time.

## Intent
To provide a persistent storage and enforcement layer for data schemas. It allows the system to establish "baselines" for API responses or data records and automatically flags when future data deviates significantly from those baselines, ensuring system stability.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **κ (Comparison)**: The primary primitive for computing drift scores between baseline and observed schemas.
- **σ (Sequence)**: Management of the translation pipeline from `Schema` to `Contract`.
- **π (Persistence)**: Durable storage of schema contracts and their version history.
- **∂ (Boundary)**: Enforces drift thresholds (default: 0.25) to trigger safety signals.
- **N (Quantity)**: Quantified drift scores based on type, range, and structural shifts.

## Drift Score Formula
`drift = (type_drift × 0.5) + (range_drift × 0.3) + (structure_drift × 0.2)`
- **Type Drift**: Mismatches between kinds (e.g., Int → Str).
- **Range Drift**: Shifts in observed numeric ranges or string lengths.
- **Structure Drift**: Additions or removals of fields in records.

## SOPs for Use
### Registering a Contract
```rust
use nexcore_ribosome::Ribosome;

let mut rb = Ribosome::new();
let schema = nexcore_transcriptase::infer(&json_sample);
rb.store_contract("icsr-v1", schema)?;
```

### Validating for Drift
```rust
if let Some(result) = rb.validate("icsr-v1", &new_json_data) {
    if result.drift_detected {
        println!("Drift Score: {:.3}", result.drift_score);
        // result.violations contains specific field-level drift info
    }
}
```

## Key Components
- **Contract**: The baseline schema along with observation counts and metadata.
- **DriftResult**: The outcome of a validation run, containing scores and violations.
- **DriftSignal**: A Guardian-compatible summary of a drift event for orchestration.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
