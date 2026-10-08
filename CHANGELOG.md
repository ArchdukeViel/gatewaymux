# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- PRD updated from Rev 6 to Rev 7: product repositioned as a local-first, multi-provider AI gateway (LLM/AI-coding workloads primary, with canonical routing for embeddings, image, audio, video, search, and fetch); Dependabot auto-merge now uses official `dependabot/fetch-metadata` update types; `Security Gate` added as a required aggregated security status check; `rust-toolchain.toml` established as the single authoritative Rust compiler declaration; owner-approval re-run scope narrowed to the failed-before-approval lifecycle with dismissal handled by the normal gate; risk declarations now require EXACTLY ONE selected tier.
- `cargo xtask risk-check` now requires EXACTLY ONE declared risk class for human/agent PRs: zero selected checkboxes, multiple selected checkboxes, and under-declarations all fail. The parser recognizes only the bold tier labels (`**R0**`..`**R4**`) on checked checkboxes, so unrelated occurrences of `R0`..`R4` in prose never become declarations; the first checked box no longer silently wins.
- `.github/workflows/dependabot-auto-merge.yml` redesigned: the custom inline diff/version parser was removed in favor of GitHub's official Dependabot metadata (`dependabot/fetch-metadata@25dd0e34f4fe68f24cc83900b1fe3fe149efef98`, pinned by commit SHA). Only `version-update:semver-patch` and `version-update:semver-minor` are auto-merge eligible; major or unprovable updates fail closed to manual merge. Auto-merge (`gh pr merge --auto --squash`) is enabled only after a valid owner APPROVED review for the current head revision. The workflow remains metadata-only.
- `.github/workflows/owner-approval-rerun.yml` simplified: it now reacts only to owner APPROVED reviews for the current head SHA and re-runs only previously FAILED gate runs; dismissal handling was removed (the normal Owner Approval Gate re-blocks merges on `pull_request_review: dismissed` events). Previously successful gate runs are never re-run.
- CI Rust setup switched from `dtolnay/rust-toolchain` with `toolchain: stable` to `actions-rust-lang/setup-rust-toolchain` (pinned by commit SHA) with no `toolchain` input, so `rust-toolchain.toml` is the single compiler source of truth.
- `.github/workflows/security.yml` now aggregates Secret & Pattern Scan, Cargo Deny, and Dependency Review into the final `Security Gate` job, which fails when any applicable prerequisite fails and tolerates legitimate event-based skips.
- PRD updated from Rev 5 to Rev 6: mandatory review vs. owner GitHub approval distinction, corrected owner-approval lifecycle, strict risk-declaration enforcement with the trusted Dependabot exemption, workflows classified R4, real generated-artifact reproducibility, routing/provider dependency decoupling, Rust reference Sync Server, and the refined Dependabot policy.
- `gatewaymux-routing` no longer depends on `gatewaymux-providers` (or `gatewaymux-protocols`); routing depends only on `gatewaymux-core` and operates on provider-neutral contracts.
- `cargo xtask generate --check` now performs real deterministic regenerate-and-compare verification (manifest schema v2; registered generator allowlist; traversal-rejecting path validation; never mutates the working tree in check mode).
- `cargo xtask architecture-check` now covers the Rust Sync Server workspace member and the routing decoupling contract.

### Added
- `cargo xtask repo-check` now mechanically verifies that `.github/workflows/**` never declare an independent `toolchain:` input (rust-toolchain.toml is the single compiler authority) and that every workflow action is pinned to an immutable full-length commit SHA; both rules carry unit tests.
- `Security Gate` added to the required status checks of the active `main-protection` ruleset.
- `gatewaymux-sync-server`: Rust reference Sync Server scaffold (`sync-server/`) replacing the npm/TypeScript workspace member; depends only on `gatewaymux-core` and `gatewaymux-sync`.
- `.github/workflows/owner-approval-rerun.yml`: metadata-only approval re-evaluation that re-runs failed Owner Approval Gate runs after a verified owner approval, eliminating manual workflow re-runs.
- `.github/workflows/dependabot-auto-merge.yml`: metadata-only post-approval auto-merge enablement for verified, provably non-major Dependabot PRs.
- ADR 0002: routing/provider decoupling and Rust reference Sync Server.
- Governance documentation updates across `AGENTS.md`, `CONTRIBUTING.md`, `.agents/rules/`, and `docs/engineering/`.

### Removed
- The custom Dependabot version-diff parser (inline Python interpreting Cargo.toml/package.json diffs and workflow `uses:` pins) was removed from `.github/workflows/dependabot-auto-merge.yml` in favor of official Dependabot metadata.
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
