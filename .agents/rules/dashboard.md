# Rule: Dashboard

## Purpose
Govern control-plane user interface architecture, contract boundaries, and frontend hygiene.

## Invariants
1. **Contract-Driven**: The dashboard must interact exclusively with GatewayMux Core via documented Control-Plane REST APIs and WebSocket endpoints.
2. **No Rust Coupling**: Dashboard frontend code (`dashboard/`) must never depend on or attempt to read internal Rust source files.
3. **No Embedded Secrets**: The dashboard must never request or display plaintext provider secrets; secrets are represented by `secret_ref` identifiers.
4. **Local Host Security**: The dashboard serves on loopback `127.0.0.1` and authenticates via one-time bootstrap token transitioning to session cookies.
5. **Tooling & Validation**: Validate the workspace using `npm run check` and `npm test`.
