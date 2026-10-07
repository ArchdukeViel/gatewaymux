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

## 2. Active Branch Protection Ruleset (Ruleset ID: 24628464)

The repository uses GitHub Rulesets (`main-protection`), enforced actively on `refs/heads/main`:
- **Branch Protection & Enforcement**: `active`
- **Protected Target**: `refs/heads/main`
- **Direct Pushes**: Prohibited (pull request required before merging)
- **Force Pushes**: Prohibited (`non_fast_forward`)
- **Branch Deletion**: Prohibited (`deletion`)
- **Allowed Merge Methods**: Strictly `squash`
- **Required Status Checks**:
  - `Owner Approval Gate`
  - `Repository Hygiene & Architecture`
  - `Rust Check, Clippy & Tests`
  - `npm Workspaces Validation`
  - `Risk Classification Verification`
  - `Release Readiness & Version Alignment`

### Single-Owner Approval Protocol
Under native GitHub Branch Protection, configuring `required_approving_review_count = 1` prevents PR authors from approving their own PRs. For a single-owner repository owned by `@ArchdukeViel`, this would make owner-authored PRs completely unmergeable without creating a dummy second GitHub account.

**Resolution**:
1. The ruleset sets `required_approving_review_count = 0` at the native ruleset level, but mandates the `Owner Approval Gate` status check.
2. An isolated, metadata-only GitHub Actions check (`.github/workflows/owner-approval.yml`) acts as the gate:
   - If PR author is `@ArchdukeViel`: status check passes automatically.
   - If PR author is an external contributor: status check queries PR reviews via GitHub API and requires an `APPROVED` review from `@ArchdukeViel`.
3. This achieves exact single-owner security without unmergeable deadlocks.

## 4. Dependabot

Configured in `.github/dependabot.yml`:
- Package ecosystems: `cargo`, `npm`, `github-actions`.
- Update interval: Weekly.
- Grouping: Grouped updates enabled across each ecosystem to prevent PR spam.
