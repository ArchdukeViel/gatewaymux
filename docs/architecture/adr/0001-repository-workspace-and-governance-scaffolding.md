# ADR-0001: Repository Workspace and Governance Scaffolding

## Status
Accepted

## Context
GatewayMux is a greenfield, multi-provider LLM gateway with strict requirements for OS portability (Core is OS-neutral while Codex Bridge is Windows-specific), architectural boundary enforcement, and contributor governance before product feature implementation begins. We need an enforceable repository baseline that prevents architectural erosion, credential leakage, and unauthorized dependency inversion.

## Decision
1. **Multi-Crate Rust Workspace**: Establish a 10-crate workspace partitioned by responsibility:
   - `gatewaymux-core`: OS-neutral Canonical IR and domain models.
   - `gatewaymux-protocols`: Wire serialization, schema dialects, transport profiles.
   - `gatewaymux-providers`: Upstream provider adapters.
   - `gatewaymux-routing`: Combo fallback, quota management, adaptive selection.
   - `gatewaymux-sync`: Cloud State Sync protocol client.
   - `gatewaymux-codex-bridge`: Windows Codex subagent interceptor.
   - `gatewaymux-server`: HTTP listeners.
   - `gatewaymux-cli`: CLI binary and commands.
   - `gatewaymux-app`: Unified native application entrypoint.
   - `xtask`: Independent automation and architecture enforcement runner.
2. **Behavior-Free npm Workspace**: Minimal package metadata for `dashboard`, `sync-server`, and `packaging/npm` without product logic or postinstall downloads.
3. **Directional Dependency Enforcement**: Enforce `app -> server/cli/bridge/sync -> routing/providers/protocols -> core` via `cargo xtask architecture-check`.
4. **Change-Risk Model (R0–R4)**: Enforce path-derived risk floors in CI.
5. **Strict Repository Hygiene**: Whitelist root files and enforce via `cargo xtask repo-check`.

## Alternatives Considered
- Single monolithic crate: Rejected due to inability to enforce architectural boundaries and OS-neutrality mechanically.
- Makefiles or Shell scripts for task runner: Rejected in favor of `cargo xtask` which works natively across Windows and Linux without bash dependencies.
- Renovate for dependency management: Rejected in favor of GitHub Dependabot with grouped updates to minimize noisy PRs.

## Consequences
- **Positive**: Clean separation of concerns, compile-time verified architecture, automated enforcement in CI, zero ambiguity for future contributors.
- **Negative / Trade-offs**: Initial workspace scaffolding overhead; requiring `cargo xtask` for repository validation.

## References
- GatewayMux PRD v1.0.0 (Rev 5), Section 28 (Repository & Engineering Governance).
- PRD Appendix B (Decision Register).
