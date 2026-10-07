# Contributing to GatewayMux

Thank you for your interest in contributing to GatewayMux!

GatewayMux is a high-reliability, local-first LLM gateway. To maintain safety, security, and architectural coherence, all contributions must adhere to the standards outlined in this guide.

## Branching & Workflow

1. **`main` Branch**: `main` is the sole authoritative branch. There is no long-lived `develop` or release-staging branch.
2. **Short-Lived Feature Branches**: All work must occur on short-lived branches created from up-to-date `main` (e.g., `feat/provider-conformance`, `fix/quota-leak`).
3. **No Direct Pushes**: Direct pushes to `main` are strictly prohibited by GitHub branch protection and rulesets.
4. **Pull Requests & Squash Merge**: All contributions must enter `main` via pull requests. Squash merging is canonical to maintain a clean, linear Git history.
5. **Small & Reviewable PRs**: Keep PRs focused on a single logical change. While we avoid arbitrary line-count caps, PRs should be easily reviewable by one person in a single sitting. Avoid mixing unrelated refactors with functional changes.

## Commit Message Conventions

We strictly enforce **Conventional Commits**:

`<type>(<optional scope>): <description>`

Common types:
- `feat`: A new user- or client-facing capability.
- `fix`: A bug fix.
- `docs`: Documentation-only changes.
- `test`: Adding or correcting tests without runtime code changes.
- `chore`: Tooling, dependencies, or repository maintenance.
- `refactor`: Code reorganization that neither fixes a bug nor adds a feature.
- `ci`: CI/CD workflows and automation scripts.

Examples:
- `feat(routing): implement exponential backoff retry budget`
- `fix(providers/nim): map 429 quota exhaustion to GMX_QUOTA_EXHAUSTED`
- `docs(adr): record ADR 0001 workspace scaffolding decisions`

## Risk Classification (R0–R4)

Every PR MUST declare its risk class in the pull request template:

- **R0 (Metadata & Docs)**: Docs, comments, non-runtime fixtures, formatting.
- **R1 (Isolated Implementation)**: Leaf utilities, unit tests, CLI formatting.
- **R2 (Provider & Protocol Behavior)**: Upstream adapters, model profiles, dialect conversions.
- **R3 (Core Routing & APIs)**: Combo fallback, quota engines, persistence schemas, public endpoints.
- **R4 (Critical Trust & Platform)**: Security, authentication, secrets, cloud sync, Codex interception.

CI automatically computes a path-derived minimum risk from the modified files. Contributors may raise the declared risk, but cannot lower it below the path-derived minimum.

## Architectural Boundaries

GatewayMux enforces a strict directional dependency rule:
`gatewaymux-app` → `server` / `cli` / `codex-bridge` / `sync` → `routing` / `providers` / `protocols` → `gatewaymux-core`

- `gatewaymux-core` is strictly OS-neutral and depends on no workspace crates.
- Provider code must not depend on the dashboard.
- Dashboard consumes control-plane REST/WebSocket contracts and does not reach into Rust internals.
- Codex Bridge calls Core directly and must not reach into provider internals.

Violations are caught by `cargo xtask architecture-check`.

## Generated Files & Manifest

Any mechanically generated files (e.g. schemas, manifests) belong to the `GENERATED` class. They must never be manually edited. Changes must be made to the authoritative source and regenerated via `cargo xtask generate`. CI checks for drift using `cargo xtask generate --check`.

## Compatibility Corpus Obligations

When fixing compatibility regressions with AI clients (e.g. OpenAI SDK, Claude Code, Cursor, Codex), contributors MUST create a sanitized test fixture under `compat/` capturing the upstream interaction. Fixtures MUST be completely sanitized of all API keys, bearer tokens, and private data.

## Proposing and Promoting Providers

Providers progress through three lifecycle stages: `EXPERIMENTAL` → `FIRST_CLASS` → `DEPRECATED`.

- **Proposing Experimental Providers**: External contributions adding new upstream providers start as `EXPERIMENTAL`. They must be explicit opt-in, labeled clearly, and excluded from default combos.
- **Promotion to First-Class**: Promotion requires passing the full Provider Conformance Lab checklist via `cargo xtask provider-check <provider>`, registering model profiles, and demonstrating robust error taxonomy mapping.
- **Security Invariants**: Experimental status never exempts a provider from secret safety, egress controls, or streaming invariants.

## Architecture Decision Records (ADRs)

Substantial architectural changes, technology selections, or wire-protocol modifications not already settled by the PRD require an accepted Architecture Decision Record in `docs/architecture/adr/`. Use `docs/architecture/adr/template.md`.

## Required Pre-PR Verification

Before submitting a pull request, run the canonical verification runner:

```bash
cargo xtask check
cargo xtask test
```

Ensure `git status` is clean and all tests, linters, and repository checks pass.
