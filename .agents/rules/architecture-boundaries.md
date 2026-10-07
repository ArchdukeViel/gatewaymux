# Rule: Architecture Boundaries

## Purpose
Enforce the directional dependencies and modular boundaries defined in PRD Section 28 and ADR 0001.

## Directional Flow
```
gatewaymux-app -> [server, cli, codex-bridge, sync] -> [routing, providers, protocols] -> gatewaymux-core
```

## Invariants
1. **OS Neutrality**: `gatewaymux-core` must remain portable and compile cleanly across platforms. No Win32 API calls, no Windows-specific crate dependencies (`windows`, `winapi`, `windows-sys`).
2. **Core Isolation**: `gatewaymux-core` defines domain models, Canonical IR, and security types. It MUST NOT depend on any internal workspace crate.
3. **No Provider Code in Core**: Provider-specific concepts (e.g. NVIDIA NIM account structure, Antigravity OAuth endpoints) belong exclusively in `gatewaymux-providers`.
4. **Codex Bridge Isolation**: `gatewaymux-codex-bridge` intercepts Windows Codex subagent traffic and invokes GatewayMux Core. It MUST NOT reach directly into `gatewaymux-providers` internals.
5. **Dashboard Isolation**: The web dashboard consumes control-plane REST/WebSocket endpoints. No Rust crate may depend on JavaScript/TypeScript code or web assets.
6. **Enforcement**: Run `cargo xtask architecture-check` before submitting any PR.
