# Change-Risk Classification (R0–R4)

GatewayMux uses an explicit five-tier risk model to ensure that high-impact areas (security, process interception, cloud state sync) receive rigorous scrutiny, while low-risk changes (documentation, comments) proceed without unnecessary friction.

## Risk Tiers

### R0 — Non-Runtime & Documentation
- **Scope**: Documentation updates, code comments, editor configuration, non-runtime test vectors.
- **Paths**: `docs/**`, `*.md`, `.editorconfig`, `.gitattributes`, `.gitignore`.
- **Verification**: Formatting check, markdown link validation, repo hygiene.

### R1 — Isolated Implementation
- **Scope**: Internal utilities, unit tests, CLI formatters, packaging scripts, non-critical leaf changes.
- **Paths**: `crates/gatewaymux-cli/**`, `crates/gatewaymux-app/**`, `crates/xtask/**`, `dashboard/**`, `sync-server/**`, `packaging/**`, `tests/**`, `scripts/**`.
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
- **Scope**: Security, authentication, secret storage, cloud state sync, Codex process interception, Win32 hooks.
- **Paths**: `crates/gatewaymux-codex-bridge/**`, `crates/gatewaymux-sync/**`, `docs/security/**`, secret management modules.
- **Verification**: Threat model review, end-to-end sync consistency, memory safety audit, manual sign-off by `@ArchdukeViel`.

## Path Minima & Escalation Policy

1. **Path-Derived Minimum**: CI scans all files changed in a PR and determines the highest risk tier among the affected paths.
2. **Elevation Allowed**: Contributors may declare a higher risk class than the path-derived minimum if a change is conceptually sensitive despite touching fewer lines.
3. **Downgrade Forbidden**: CI automatically rejects any PR whose declared risk class is lower than the path-derived minimum.
4. **Cross-Cutting Changes**: If a PR touches multiple tiers (e.g. `gatewaymux-routing` and `gatewaymux-sync`), the entire PR escalates to the highest tier (R4).
