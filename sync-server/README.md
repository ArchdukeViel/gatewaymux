# GatewayMux Reference Sync Server

Self-hostable reference implementation of the GatewayMux sync backend
capabilities (S3-level coordination).

This is a **Rust binary/service scaffold** and a member of the root Cargo
workspace (`gatewaymux-sync-server`). Pre-implementation behavior-free
skeleton: no synchronization protocol, HTTP endpoints, leases, counters,
device enrollment, encrypted vault storage, or database persistence is
implemented in this phase.

## Architectural Direction

The reference server consumes shared GatewayMux contracts only:

- `gatewaymux-sync-server` -> `gatewaymux-core`
- `gatewaymux-sync-server` -> `gatewaymux-sync`
- `gatewaymux-sync` -> `gatewaymux-core`

Directional dependencies are enforced by `cargo xtask architecture-check`.
