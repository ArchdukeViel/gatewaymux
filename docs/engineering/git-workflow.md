# Git Workflow & Pull Request Policy

This document defines the Git lifecycle, branch management, and merge controls enforced on `ArchdukeViel/gatewaymux`.

## 1. Branch Strategy

- **`main` is Canonical**: Production-ready, passing all CI checks, always deployable.
- **No Long-Lived Branches**: There is no persistent `develop` or `staging` branch.
- **Short-Lived Feature Branches**:
  - Name feature branches descriptively: `feat/<name>`, `fix/<name>`, `docs/<name>`.
  - Rebase or merge `main` into your feature branch before requesting review.
- **No Direct Pushes**: Direct pushes to `main` are blocked by branch protection.

## 2. Commit Guidelines

- **Conventional Commits**: Every commit message must follow Conventional Commits:
  ```
  <type>(<scope>): <summary>

  [optional body]

  [optional footer]
  ```
- **Squash Merge**: PRs are squash-merged into `main`. The PR title becomes the squash commit summary, and the PR description (or curated commit history) forms the commit body.

## 3. Pull Request Process

1. **Create Branch**: `git checkout -b feat/combo-fallback`
2. **Implement & Test**: Run local validation:
   ```bash
   cargo xtask check
   cargo xtask test
   ```
3. **Open Pull Request**: Use `.github/PULL_REQUEST_TEMPLATE.md`.
4. **Declare Risk Class**: Select EXACTLY ONE risk tier (R0 to R4). CI verifies against path minima: zero selected tiers, multiple selected tiers, and under-declarations all FAIL the `Risk Classification Verification` check. Trusted Dependabot PRs are exempt and use path-derived risk automatically.
5. **CI Gating**: All checks must pass:
   - `repo-hygiene` (`cargo xtask repo-check`, `architecture-check`, `compat`, `generate --check`)
   - `rust-validation` (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`)
   - `npm-validation` (`npm run check`, `npm test`)
   - `risk-assessment` (`cargo xtask risk-check`)
   - `owner-approval` (metadata-only owner approval gate, required for external/bot PRs)
   - `Security Gate` (required aggregation of the Security workflow's secret & pattern scan, cargo-deny, and dependency review)
   - CI installs exactly the Rust toolchain declared in `rust-toolchain.toml`; workflows must not declare an independent toolchain pin.
6. **Merge**: Once checks pass and the review/approval requirements below are satisfied, squash merge into `main`. The feature branch is automatically deleted upon merge.

## 4. Mandatory Review vs. Owner Approval

GitHub does not permit a PR author to approve their own PR. The repository therefore enforces two distinct requirements:

### 4.1 GitHub approval — external contributor or bot authored PRs

- Requires an actual GitHub `APPROVED` review from `@ArchdukeViel`.
- The approval must be submitted against the PR's current head SHA; approving an outdated revision does not satisfy the gate, and a new push invalidates prior approvals.
- All required status checks must pass, and unresolved review findings block the merge.
- Enforced by the required metadata-only `Owner Approval Gate` status check; after a valid owner approval for the current head SHA, previously FAILED gate runs are re-evaluated automatically (and only failed runs — never previously successful ones; see `docs/engineering/github-setup.md`), so no manual workflow re-run is needed. Dismissal of an owner approval is handled by the normal Owner Approval Gate, which re-blocks the merge.
- Dependabot patch/minor PRs may additionally be auto-merged (squash) once the owner approves the current revision, official Dependabot metadata proves a patch/minor update type, and all required checks pass, per the Dependabot policy in `docs/engineering/github-setup.md`. Major updates are manually merged.

### 4.2 Mandatory review — every PR, including owner-authored PRs

A PR authored by `@ArchdukeViel`:
- MUST still undergo a deliberate review before merge;
- does NOT require or attempt an impossible GitHub self-approval, and MUST NOT be described as "owner-approved";
- requires a full diff review by the lead agent, with an independent read-only reviewer subagent delegated where available;
- the reviewer must inspect correctness, scope, security implications, architecture compliance, test evidence, and documentation consistency;
- all findings must be resolved before merge;
- all required GitHub checks must pass;
- only then may the PR be squash-merged.

An owner-authored PR is therefore mergeable when: the independent reviewer review is complete, the lead-agent diff review is complete, all valid material findings are resolved, all required CI checks are green, the PR remains mergeable, and the reviewed head SHA is the head being merged.
