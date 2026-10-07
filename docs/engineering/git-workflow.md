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
4. **Declare Risk Class**: Fill out the risk classification (R0 to R4). CI will verify against path minima.
5. **CI Gating**: All checks must pass:
   - `repo-hygiene` (`cargo xtask repo-check`, `architecture-check`, `compat`, `generate --check`)
   - `rust-validation` (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`)
   - `npm-validation` (`npm run check`, `npm test`)
   - `risk-assessment` (`cargo xtask risk-check`)
   - `owner-approval` (metadata-only owner approval check)
6. **Merge**: Once checks pass and owner approval is verified, squash merge into `main`. The feature branch is automatically deleted upon merge.

## 4. Single-Owner Approval Protocol

Repository ownership is held by `@ArchdukeViel`:
- **PRs authored by `@ArchdukeViel`**: Automatically satisfy the approval gate once all technical CI checks pass.
- **PRs authored by external contributors**: Require an explicit review and approval from `@ArchdukeViel`.
- Enforced via the isolated metadata-only workflow `.github/workflows/owner-approval.yml`.
