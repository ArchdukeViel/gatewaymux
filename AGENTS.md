# GatewayMux AI Agent Constitution

This document is the normative repository constitution governing AI agents and autonomous coding assistants working within `ArchdukeViel/gatewaymux`.

## 1. Authority Hierarchy

All agents MUST resolve decisions and ambiguities using this strict precedence order:

1. **Security Invariants & Safety**: Non-negotiable protection of secrets, process isolation, and egress trust policies.
2. **GatewayMux PRD (`docs/GatewayMux_PRD_v1.0.0.md`)**: The authoritative product and architecture specification.
3. **Accepted Architecture Decision Records (`docs/architecture/adr/`)**: Binding architectural precedents.
4. **Repository Constitution (`AGENTS.md`)**: This operational constitution.
5. **Scoped Rules (`.agents/rules/*.md`)**: Subsystem-specific contributor directives.
6. **Task / Issue Description**: Scoped execution instructions.
7. **Code & Tests**: The concrete implementation.

**Discrepancy Protocol**: If a lower authority appears to conflict with a higher authority, you MUST NOT silently change architecture. Surface the discrepancy to the user or repository owner (`ArchdukeViel`) and ask for guidance.

## 2. Inviolable Core Principles

1. **Never Weaken Security or Correctness**: Never lower security thresholds, disable trust filters, or bypass error checks to make a test pass. If a test fails, fix the underlying defect.
2. **Core is OS-Neutral & Provider-Agnostic; Routing is Provider-Decoupled**: `gatewaymux-core` must never contain provider-specific logic, Win32 APIs, or Codex Bridge dependencies. `gatewaymux-routing` depends only on `gatewaymux-core` and must never depend on `gatewaymux-providers` or branch on provider names (ADR-0002).
3. **Honor Canonical Operation Boundaries**: Do not bypass canonical Operation IR or protocol mappings for convenience. Every client request maps to canonical IR before provider dispatch.
4. **Zero Secrets in Git**: Never print, log, or commit API keys, auth tokens, private keys, or passwords.
5. **Generated Files Are Read-Only and Reproducibility-Verified**: Never edit files marked `GENERATED` directly. Edit their authoritative generator source and regenerate using `cargo xtask generate`. `cargo xtask generate --check` deterministically regenerates registered outputs and byte-compares them with the tracked artifacts; it never writes to the working tree.
6. **Persistence Requires Migrations**: Once database persistence is introduced, all schema alterations require reversible, versioned SQL migrations.
7. **Preserve Compatibility Evidence**: Any production bug or compatibility fix must be accompanied by a sanitized test fixture in `compat/`.
8. **No Unrelated Refactors**: Keep PRs and commits tightly focused on the scoped assignment. Do not refactor surrounding code or modernize formatting outside the assigned scope.
9. **Lead Agent Integration Ownership**: Subagents may execute bounded audit or implementation tasks, but the lead agent personally reviews, integrates, and verifies all results.
10. **Pre-Handoff Verification**: You MUST run `cargo xtask check` and `cargo xtask test` and verify clean repository status before declaring completion.
11. **CI Workflows are the Security and Supply-Chain Boundary**: All `.github/workflows/**` modifications carry minimum risk class R4. Privileged workflows must be metadata-only and must never check out or execute pull request code.
12. **Mandatory Review vs. Owner GitHub Approval**: Every PR requires a deliberate review of the complete diff, including PRs authored by `ArchdukeViel` (which never claim a GitHub self-approval). An actual GitHub `APPROVED` review from `ArchdukeViel` is additionally required for every external-contributor or bot PR, enforced by the metadata-only `Owner Approval Gate` required check.

## 3. Scoped Contributor Rules

Detailed subsystem directives are organized under `.agents/rules/`:

- [Architecture Boundaries](.agents/rules/architecture-boundaries.md)
- [Security & Secrets](.agents/rules/security-and-secrets.md)
- [Database Migrations](.agents/rules/database-migrations.md)
- [Provider Adapters](.agents/rules/provider-adapters.md)
- [Compatibility Corpus](.agents/rules/compatibility-corpus.md)
- [Dashboard](.agents/rules/dashboard.md)
- [Codex Bridge](.agents/rules/codex-bridge.md)
- [Cloud Sync](.agents/rules/cloud-sync.md)
- [Testing](.agents/rules/testing.md)
- [Generated Files](.agents/rules/generated-files.md)
- [Release Artifacts](.agents/rules/release-artifacts.md)

Do NOT create a separate `.agents/specs/` hierarchy; the PRD and ADRs remain the durable specifications.
