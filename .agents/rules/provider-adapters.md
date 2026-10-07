# Rule: Provider Adapters

## Purpose
Govern upstream provider integration, adapter contracts, and the provider promotion lifecycle.

## Invariants
1. **Canonical IR Preservation**: Upstream adapters must translate client requests from Canonical Operation IR to the provider format and translate upstream responses back into Canonical IR. No bypass shortcuts.
2. **Lifecycle States**:
   - `EXPERIMENTAL`: Opt-in only, marked in logs/UI, excluded from default combos.
   - `FIRST_CLASS`: Supported in default combos, complete error mapping, passing Conformance Lab.
   - `DEPRECATED`: Emits deprecation warnings on startup, guides users to successor.
3. **Promotion Checklist**: Running `cargo xtask provider-check <provider>` must pass all 7 criteria before promotion to `FIRST_CLASS`.
4. **Retry & Ledger Rules**:
   - Upstream retry is bounded by `RetryBudget` and recorded in `AttemptLedger`.
   - Never retry on ambiguous acceptance without verified idempotency or proof the request was not processed.
5. **Streaming Contract**: Once a streaming response yields visible tokens to the client, the provider path is locked; mid-stream failover or splicing is strictly prohibited.
