# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
