# Change-Risk Classification (R0–R4)

GatewayMux uses an explicit five-tier risk model to ensure that high-impact areas (security, process interception, cloud state sync) receive rigorous scrutiny, while low-risk changes (documentation, comments) proceed without unnecessary friction.

## Risk Tiers

### R0 — Non-Runtime & Documentation
- **Scope**: Documentation updates, code comments, editor configuration, non-runtime test vectors.
- **Paths**: `docs/**`, `*.md`, `.editorconfig`, `.gitattributes`, `.gitignore`.
- **Verification**: Formatting check, markdown link validation, repo hygiene.

### R1 — Isolated Implementation
- **Scope**: Internal utilities, unit tests, CLI formatters, packaging scripts, non-critical leaf changes.
- **Paths**: `crates/gatewaymux-cli/**`, `crates/gatewaymux-app/**`, `crates/xtask/**`, `dashboard/**`, `packaging/**`, `tests/**`, `scripts/**`.
- **Verification**: Unit tests pass, workspace compilation, clippy clean.

### R2 — Provider & Protocol Behavior
- **Scope**: Upstream provider adapters, model capability profiles, schema dialect translations, transport profiles, Compatibility Corpus updates.
- **Paths**: `crates/gatewaymux-providers/**`, `crates/gatewaymux-protocols/**`, `profiles/**`, `compat/**`.
- **Verification**: Provider Conformance Lab checks (`cargo xtask provider-check`), corpus sanitization, protocol serialization round-trip tests.

### R3 — Core Routing & APIs
- **Scope**: Combo fallback chains, quota enforcement, deployment selection, persistence migrations, public HTTP data plane, configuration schemas.
- **Paths**: `crates/gatewaymux-routing/**`, `crates/gatewaymux-server/**`, `crates/gatewaymux-core/**`, `config/**`.
- **Verification**: Integration tests, combo simulation, migration rollback testing, error taxonomy mapping validation.

### R4 — Critical Trust & Platform Boundaries
- **Scope**: Security, authentication, secret storage, cloud state sync, the reference Sync Server, Codex process interception, Win32 hooks, and the CI workflow security / software-supply-chain boundary.
- **Paths**: `crates/gatewaymux-codex-bridge/**`, `crates/gatewaymux-sync/**`, `sync-server/**`, `.github/workflows/**`, `docs/security/**`, secret management modules.
- **Verification**: Threat model review, end-to-end sync consistency, memory safety audit, manual sign-off by `@ArchdukeViel`.

All `.github/workflows/**` modifications are minimum R4 because CI workflows are part of the repository security and software-supply-chain boundary (they define how code is built, validated, and — in the case of privileged workflows — what automation may act on the repository).

## Path Minima & Escalation Policy

1. **Path-Derived Minimum**: CI scans all files changed in a PR and determines the highest risk tier among the affected paths.
2. **Exactly-One Mandatory Declaration (human/agent PRs)**: Every human- or agent-authored pull request MUST declare its risk class in the PR template, and EXACTLY ONE tier must be selected. Zero selected declarations FAIL the `Risk Classification Verification` check. Two or more selected declarations also FAIL — the first checked box never silently wins. A declared risk lower than the path-derived minimum also FAILS. The parser only recognizes the bold tier labels (`**R0**`..`**R4**`) on checked checkboxes, so unrelated occurrences of tier names in prose never become declarations.
3. **Trusted Dependabot Exemption**: Pull requests authored by GitHub Dependabot are exempt from manual declaration. Their governing risk class is derived automatically from the changed paths and must be derivable from a non-empty change set. The exemption is granted only to the verified Dependabot identity supplied by CI event metadata — never to PR body text, and never as a generic bot exemption.
4. **Elevation Allowed**: Contributors may declare a higher risk class than the path-derived minimum if a change is conceptually sensitive despite touching fewer lines.
5. **Downgrade Forbidden**: CI automatically rejects any PR whose declared risk class is lower than the path-derived minimum.
6. **Cross-Cutting Changes**: If a PR touches multiple tiers (e.g. `gatewaymux-routing` and `gatewaymux-sync`), the entire PR escalates to the highest tier (R4).
