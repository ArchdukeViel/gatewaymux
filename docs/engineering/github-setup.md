# GitHub Configuration & Governance Setup

This document records the repository configuration, rulesets, branch protections, and security settings for `ArchdukeViel/gatewaymux`.

## 1. Automated Repository Settings (Applied via GitHub API)

The following settings have been automatically configured via the authenticated `gh` CLI:

- **Merge Strategies**:
  - `allow_squash_merge`: `true` (Canonical merge method).
  - `allow_merge_commit`: `false` (Disabled).
  - `allow_rebase_merge`: `false` (Disabled).
  - `delete_branch_on_merge`: `true` (Merged branches automatically deleted).
  - `squash_merge_commit_title`: `PR_TITLE`.
  - `squash_merge_commit_message`: `PR_BODY`.
- **Security & Analysis**:
  - `vulnerability-alerts`: `enabled`.
  - `automated-security-fixes`: `enabled`.
  - `secret_scanning`: `enabled`.
  - `secret_scanning_push_protection`: `enabled`.

## 2. Branch Protection & The Single-Owner Exception

### Desired Policy
- Pull requests required before merging into `main`.
- Direct pushes to `main` prohibited.
- Force pushes and branch deletion prohibited.
- Strict CI status checks required (`repo-hygiene`, `rust-validation`, `npm-validation`, `risk-assessment`, `owner-approval`).
- Owner approval required for external contributions.

### GitHub Platform Limitation & Resolution
Under native GitHub Branch Protection, configuring `required_approving_review_count = 1` prevents PR authors from approving their own PRs. For a single-owner repository owned by `@ArchdukeViel`, this would make owner-authored PRs completely unmergeable without creating a dummy second GitHub account.

**Resolution**:
1. Branch protection enforces **Required Status Checks** rather than a native review count.
2. An isolated, metadata-only GitHub Actions check (`.github/workflows/owner-approval.yml`) acts as the gate:
   - If PR author is `@ArchdukeViel`: status check passes immediately.
   - If PR author is an external contributor: status check inspects PR reviews using GitHub API and requires an `APPROVED` review from `@ArchdukeViel`.
3. This achieves exact single-owner security without unmergeable deadlocks.

## 3. Manual Steps (Web UI Verification)

To enable branch protection on `main` via the GitHub Web UI or API:
1. Navigate to **Settings** -> **Branches** -> **Add branch protection rule**.
2. Set **Branch name pattern**: `main`.
3. Check **Require status checks to pass before merging**:
   - Require branches to be up to date before merging.
   - Select required status checks:
     - `repo-hygiene`
     - `rust-validation`
     - `npm-validation`
     - `risk-assessment`
     - `Owner Approval Gate`
4. Check **Do not allow bypassing the above settings**.
5. Save changes.

## 4. Dependabot

Configured in `.github/dependabot.yml`:
- Package ecosystems: `cargo`, `npm`, `github-actions`.
- Update interval: Weekly.
- Grouping: Grouped updates enabled across each ecosystem to prevent PR spam.
