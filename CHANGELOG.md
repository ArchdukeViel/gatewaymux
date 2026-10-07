# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- PRD updated from Rev 5 to Rev 6: mandatory review vs. owner GitHub approval distinction, corrected owner-approval lifecycle, strict risk-declaration enforcement with the trusted Dependabot exemption, workflows classified R4, real generated-artifact reproducibility, routing/provider dependency decoupling, Rust reference Sync Server, and the refined Dependabot policy.
- `gatewaymux-routing` no longer depends on `gatewaymux-providers` (or `gatewaymux-protocols`); routing depends only on `gatewaymux-core` and operates on provider-neutral contracts.
- `cargo xtask generate --check` now performs real deterministic regenerate-and-compare verification (manifest schema v2; registered generator allowlist; traversal-rejecting path validation; never mutates the working tree in check mode).
- `cargo xtask risk-check` now rejects missing/under-declared risk classes for human/agent PRs and applies the trusted Dependabot path-derived exemption; `.github/workflows/**` and `sync-server/**` are minimum R4.
- `cargo xtask architecture-check` now covers the Rust Sync Server workspace member and the routing decoupling contract.

### Added
- `gatewaymux-sync-server`: Rust reference Sync Server scaffold (`sync-server/`) replacing the npm/TypeScript workspace member; depends only on `gatewaymux-core` and `gatewaymux-sync`.
- `.github/workflows/owner-approval-rerun.yml`: metadata-only approval re-evaluation that re-runs failed Owner Approval Gate runs after a verified owner approval (or dismissal), eliminating manual workflow re-runs.
- `.github/workflows/dependabot-auto-merge.yml`: metadata-only post-approval auto-merge enablement for verified, provably non-major Dependabot PRs.
- ADR 0002: routing/provider decoupling and Rust reference Sync Server.
- Governance documentation updates across `AGENTS.md`, `CONTRIBUTING.md`, `.agents/rules/`, and `docs/engineering/`.

### Removed
- `sync-server` npm workspace package (`@gatewaymux/sync-server`); the reference server is now a Rust workspace member only.

## [0.1.0-pre] - 2026-10-07

### Added
- Initial Greenfield architecture baseline and PRD Rev 5.
- Multi-crate Rust workspace skeleton (`crates/`):
  - `gatewaymux-core`: Domain models, canonical IR, and security invariants.
  - `gatewaymux-protocols`: Wire protocols, schema dialects, transport profiles.
  - `gatewaymux-providers`: Upstream provider adapters and credential protocols.
  - `gatewaymux-routing`: Combo fallback engine, quota management, adaptive selection.
  - `gatewaymux-sync`: Cloud State Sync client and distributed coordination.
  - `gatewaymux-codex-bridge`: Windows Codex subagent interception bridge.
  - `gatewaymux-server`: Data plane and control plane HTTP listeners.
  - `gatewaymux-cli`: Administrative CLI and daemon commands.
  - `gatewaymux-app`: Unified native application entrypoint.
  - `xtask`: Developer automation and governance verification task runner.
- npm workspaces skeleton:
  - `@gatewaymux/dashboard`: Control plane web interface.
  - `@gatewaymux/sync-server`: Reference sync server implementation.
  - `gatewaymux`: npm Windows binary distribution wrapper.
- Engineering governance documents and rules:
  - `AGENTS.md` repository constitution and 11 scoped `.agents/rules/` policies.
  - `CONTRIBUTING.md`, `SECURITY.md`, and Architecture Decision Records (ADR) system.
  - Engineering specifications under `docs/engineering/`.
- CI/CD automation and enforcement:
  - GitHub Actions workflows for lint, test, architecture check, and risk evaluation.
  - Single-owner approval check workflow for `@ArchdukeViel`.
  - Dependabot grouped version update configuration.
  - Issue and pull request templates.
