# Rule: Testing

## Purpose
Govern test suite organization, quality standards, and risk-tier verification expectations.

## Invariants
1. **Never Weaken Existing Tests**: Do not delete, ignore (`#[ignore]`), or alter existing tests to mask regressions. Failing tests indicate defects that must be resolved.
2. **Comprehensive Test Tiers**:
   - `tests/integration/`: Cross-crate component workflows (routing fallback, adapter execution).
   - `tests/e2e/`: Full pipeline validation including server startup and simulated client requests.
   - `tests/fault/`: Resiliency under network partitions, upstream 5xx errors, rate limits, and slow streams.
   - `tests/security/`: Secret containment, loopback binding, and auth failure verification.
   - `tests/fixtures/`: Sanitized request/response goldens and configurations.
3. **Execution Commands**:
   - `cargo test --workspace`
   - `npm test`
   - `cargo xtask test --risk <R0..R4>`
4. **Boundary & Edge Case Testing**: Test zero inputs, empty collections, malformed JSON, and cancellation during active streaming.
