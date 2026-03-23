# AI Guidance — nexcore-ribosome

Schema contract registry and drift detection engine.

## Use When
- Establishing baseline schemas for reliable data exchange.
- Monitoring long-term stability of external API responses.
- Detecting breaking changes in internal data structures before they cause failures.
- Automating the transition from observed data (Transcriptase) to enforced contracts.

## Grounding Patterns
- **Drift Threshold**: The default threshold is `0.25`. For high-precision financial or medical data, consider lowering to `0.10`.
- **Auto-Update**: Be cautious with `auto_update: true`; it will merge observations into the baseline, potentially "learning" and accepting drift rather than flagging it.
- **T1 Primitives**:
  - `κ + N`: Root primitives for drift scoring.
  - `π + ∂`: Root primitives for contract storage and threshold enforcement.

## Maintenance SOPs
- **Contract ID**: Use semantic versioning in contract IDs (e.g., `user_profile_v2`) to manage breaking schema changes.
- **Drift Signals**: Always wrap `DriftResult` into a `DriftSignal` when reporting to the orchestrator layer (`nexcore-friday` or `Guardian`).
- **Data Generation**: Use the `generate()` method to verify that the stored contract still produces "sensible" mock data for testing.

## Key Entry Points
- `src/lib.rs`: `Ribosome`, `Contract`, and `DriftResult` definitions.
- `src/grounding.rs`: T1 grounding for ribosome types.
