# ADR-0002: Routing/Provider Decoupling and Rust Reference Sync Server

## Status
Accepted

## Context
Before product implementation begins, the pre-implementation governance hardening pass (PRD Rev 6) identified two structural weaknesses in the accepted workspace contract from ADR-0001:

1. **Routing coupled to providers**: `gatewaymux-routing` was permitted (and declared) to depend on `gatewaymux-providers`. This invited provider-name branching (`if provider == "openrouter" { ... }`) inside the routing layer and let provider-specific behavior leak into route selection, contradicting the PRD's requirement that routing operate on provider-neutral contracts and that provider quirks stay isolated in the provider adapter layer (PRD Section 6.2, Section 28.2).

2. **Reference Sync Server on the wrong runtime**: the behavior-free reference Sync Server scaffold was an npm/TypeScript workspace member (`@gatewaymux/sync-server`), which would have forced a Node.js runtime and a second toolchain into the self-hostable sync backend for no architectural benefit. The Sync Server consumes GatewayMux contracts; those contracts live in the Rust workspace (`gatewaymux-core`, `gatewaymux-sync`), and the product's own distribution never requires a Node runtime (PRD Sections 18.2, 22.3, 25.13).

## Decision
1. **Routing decoupling**: `gatewaymux-routing` depends only on `gatewaymux-core` and MUST NOT depend on `gatewaymux-providers` (or any concrete provider adapter implementation). Routing operates on provider-neutral contracts and candidate data. Provider-name branching for provider-specific behavior is prohibited in the routing layer; the composition/runtime layer (`gatewaymux-server`/`gatewaymux-app`) wires selected routes to concrete provider execution. `gatewaymux-routing` MAY adopt protocol-neutral contracts from `gatewaymux-protocols` only through a future accepted governance change that records the concrete requirement.
2. **Rust reference Sync Server**: the reference Sync Server is the Rust binary crate `gatewaymux-sync-server` (workspace member at `sync-server/`), not an npm/TypeScript package. It depends only on `gatewaymux-core` and `gatewaymux-sync`, and MUST NOT depend on the gateway runtime crates (`gatewaymux-server`, `gatewaymux-providers`, `gatewaymux-routing`). The npm workspaces now cover only the Dashboard and npm distribution packaging. The Sync Server shares the unified GatewayMux semantic version via the workspace manifest and participates in the standard Rust toolchain (build, clippy, fmt, cargo-deny) and `cargo xtask architecture-check` enforcement.
3. **Mechanical enforcement**: the `cargo xtask architecture-check` dependency matrix is updated to encode this contract exactly (routing -> core only; `gatewaymux-sync-server` -> core + sync) and is covered by unit tests.

## Alternatives Considered
- **Keep routing -> providers and rely on code review**: Rejected; review-only boundaries erode, and the violation was already encoded in the PRD and the enforcement matrix, making it normative.
- **Allow routing -> protocols as well**: Rejected for now; nothing in the current behavior-free scaffold requires it. The dependency can be added deliberately later if a concrete, documented requirement appears.
- **Keep the Sync Server in npm and wrap Rust via N-API/FFI later**: Rejected; it adds a Node runtime dependency and cross-language binding complexity to an optional self-hosted component with no product requirement.
- **Reference Sync Server as a new crate under `crates/`**: Rejected; the PRD reserves `crates/` for the gateway product surface. The top-level `sync-server/` placement keeps the optional reference deployment visibly separate while remaining a workspace member.

## Consequences
- **Positive**: Provider-specific behavior cannot leak into routing by construction; the sync backend shares one toolchain, one supply chain (cargo-deny), one version coordinate, and one architecture-check contract; npm workspaces shrink to genuinely Node.js concerns (dashboard, packaging).
- **Negative / Trade-offs**: `gatewaymux-routing` must express everything it needs from candidates through `gatewaymux-core` contracts (routing cannot call provider helpers); the sync backend gains Rust build/deploy characteristics instead of Node-based deployment familiarity.

## References
- GatewayMux PRD v1.0.0 (Rev 6), Section 22.3 (Reference Sync Server), Section 28.2 (Workspace Boundaries), Section 28.6 (Coordinated Release Versioning).
- ADR-0001 (amended: directional dependency contract).
- `cargo xtask architecture-check` and its dependency-matrix unit tests.
