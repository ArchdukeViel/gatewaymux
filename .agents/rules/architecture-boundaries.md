# Rule: Architecture Boundaries

## Purpose
Enforce the directional dependencies and modular boundaries defined in PRD Section 28, ADR 0001 (as amended by ADR 0002).

## Directional Flow
```
gatewaymux-app -> [server, cli, codex-bridge, sync] -> [routing, providers, protocols] -> gatewaymux-core
gatewaymux-routing -> gatewaymux-core only (provider-neutral contracts)
gatewaymux-sync-server -> [gatewaymux-core, gatewaymux-sync]
```

## Invariants
1. **OS Neutrality**: `gatewaymux-core` must remain portable and compile cleanly across platforms. No Win32 API calls, no Windows-specific crate dependencies (`windows`, `winapi`, `windows-sys`).
2. **Core Isolation**: `gatewaymux-core` defines domain models, Canonical IR, and security types. It MUST NOT depend on any internal workspace crate.
3. **Routing/Provider Decoupling**: `gatewaymux-routing` depends only on `gatewaymux-core`. It MUST NOT depend on `gatewaymux-providers` or any concrete provider implementation. Routing operates on provider-neutral contracts and candidate data; provider-name branching (e.g. `if provider == "openrouter"`) for provider-specific behavior is prohibited in the routing layer. The composition/runtime layer wires selected routes to concrete provider execution.
4. **No Provider Code in Core**: Provider-specific concepts (e.g. NVIDIA NIM account structure, Antigravity OAuth endpoints) belong exclusively in `gatewaymux-providers`.
5. **Codex Bridge Isolation**: `gatewaymux-codex-bridge` intercepts Windows Codex subagent traffic and invokes GatewayMux Core. It MUST NOT reach directly into `gatewaymux-providers` internals.
6. **Sync Server Isolation**: The Rust reference Sync Server (`gatewaymux-sync-server` at `sync-server/`) depends only on `gatewaymux-core` and `gatewaymux-sync`; it MUST NOT depend on the gateway runtime crates (`gatewaymux-server`, `gatewaymux-providers`, `gatewaymux-routing`).
7. **Dashboard Isolation**: The web dashboard consumes control-plane REST/WebSocket endpoints. No Rust crate may depend on JavaScript/TypeScript code or web assets.
8. **Enforcement**: Run `cargo xtask architecture-check` before submitting any PR. The dependency matrix is unit-tested in `crates/xtask/src/architecture.rs`.
