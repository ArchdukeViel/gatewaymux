# GitHub Configuration & Governance Setup

This document records the repository configuration, rulesets, branch protections, security settings, and automation policy for `ArchdukeViel/gatewaymux`. It describes what is actually configured, not aspirations.

## 1. Automated Repository Settings (Applied via GitHub API)

The following settings are configured on the repository:

- **Merge Strategies**:
  - `allow_squash_merge`: `true` (Canonical merge method).
  - `allow_merge_commit`: `false` (Disabled).
  - `allow_rebase_merge`: `false` (Disabled).
  - `allow_auto_merge`: `true` (GitHub-native auto-merge enabled; used ONLY by the Dependabot post-approval auto-merge policy in Section 6).
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
- **Required Approving Reviews**: `0` at the native ruleset level (see Section 3 for why; the owner-approval requirement is enforced by the required status check instead)
- **Allowed Merge Methods**: Strictly `squash`
- **Required Status Checks**:
  - `Owner Approval Gate`
  - `Repository Hygiene & Architecture`
  - `Rust Check, Clippy & Tests`
  - `npm Workspaces Validation`
  - `Risk Classification Verification`
  - `Release Readiness & Version Alignment`

All modifications to `.github/workflows/**` carry a minimum risk class of **R4** because CI workflows are part of the repository security and software-supply-chain boundary.

## 3. Mandatory Review vs. Owner GitHub Approval

GitHub does not permit a PR author to submit an approving review on their own PR. A single-owner repository therefore cannot rely on `required_approving_review_count = 1`, which would make every owner-authored PR permanently unmergeable. The repository instead enforces two distinct requirements:

| Requirement | Applies to | Mechanism |
|---|---|---|
| **GitHub approval** (an actual `APPROVED` review from `@ArchdukeViel`) | Every PR authored by an external contributor or a bot (including Dependabot) | Required status check `Owner Approval Gate` |
| **Mandatory review** (a deliberate review of the complete diff) | **Every** PR, including PRs authored by `@ArchdukeViel` | Repository policy (see `docs/engineering/git-workflow.md`); owner-authored PRs never claim or require a GitHub self-approval |

### 3.1 Owner Approval Gate (`.github/workflows/owner-approval.yml`)

An isolated, metadata-only required status check (job name `Owner Approval Gate`) that never checks out pull request code and holds read-only permissions:

- If the PR author is `@ArchdukeViel`: the gate passes automatically (the mandatory-review policy still applies).
- Otherwise, the gate queries the PR reviews through the GitHub API and requires an `APPROVED` review from `@ArchdukeViel` whose `commit_id` equals the PR's **current head SHA**. Approvals submitted against outdated revisions do not satisfy the gate, and dismissing the owner approval re-fails the gate (blocking the merge again).

### 3.2 Approval Lifecycle Fix (`.github/workflows/owner-approval-rerun.yml`)

Previous defect: an external/Dependabot PR opened with a failing required `Owner Approval Gate`; after the owner approved, a new review-triggered gate run succeeded, but the original failed required check remained attached to the PR and GitHub kept blocking the merge until someone manually re-ran the original workflow.

Fix: the metadata-only re-evaluation workflow reacts to `pull_request_review` events. It acts only after verifying, **from the GitHub event payload** (never from PR text):
- the event is an owner review;
- the reviewer identity is exactly `ArchdukeViel`;
- the review state is `APPROVED` (or the owner approval was `DISMISSED`);
- the reviewed commit (`review.commit_id`) matches the PR's current head SHA.

On a valid approval it re-runs the failed `Owner Approval Gate` runs for that head SHA through the Actions API (`rerun-failed-jobs`), so the original check-run conclusions transition in place to success and the merge unblocks **without any manual workflow re-run**. On dismissal it re-evaluates all completed gate runs so the required check re-blocks the merge. The external-owner-approval requirement itself is never weakened.

## 4. Security Scanning & Dependency Review

Configured in `.github/workflows/security.yml`:
- **Secret & Pattern Scan**: Rejects accidental credential leaks (`ghp_*`, `sk-*`, bearer tokens).
- **Cargo Deny**: Validates Cargo dependencies against advisories, bans, and licenses via `EmbarkStudios/cargo-deny-action` using `deny.toml`.
- **Dependency Review**: Runs `actions/dependency-review-action` on PRs to prevent vulnerable dependency introduction.

## 5. Dependabot Update Policy

Configured in `.github/dependabot.yml`:

- **Ecosystems**: Cargo (`/`), npm root workspace (`/`), npm packaging tooling (`/packaging/npm`), and GitHub Actions (`/`).
- **Grouping**: Patch and minor version updates MAY be grouped per ecosystem. Each group is restricted to `update-types: [minor, patch]` and `applies-to: version-updates`, so:
  - **Major updates are excluded from groups** and MUST be opened as separate, per-dependency pull requests.
  - **Security updates are never grouped** and remain individually visible.
- **Major updates require explicit owner review and a manual squash merge.**

## 6. Dependabot Post-Approval Auto-Merge

GitHub-native auto-merge (`allow_auto_merge: true`) is enabled at the repository level, and `.github/workflows/dependabot-auto-merge.yml` enables it per-PR for qualifying Dependabot PRs only. The exact behavior:

1. **Identity**: the workflow runs from the base branch (`pull_request_target`) and verifies the PR author is exactly `dependabot[bot]` using the GitHub event payload. PR title/body text is never trusted for identity. The workflow never checks out or executes PR code despite holding write permissions.
2. **Major-version gate**: the update's version impact is proven from the PR file patches fetched via the GitHub REST API (Cargo.toml requirements, package.json ranges, and workflow action pins with their `# vN` version comments). A PR is treated as major if any dependency's major version changes, a new dependency is introduced, or the version impact cannot be proven. Detection **fails closed**: unprovable updates are never auto-merged. (SHA-pinned workflow actions without a `# vN` comment fall into this category.)
3. **Enablement**: only for verified, provably non-major Dependabot PRs, the workflow enables GitHub-native auto-merge with `merge_method=squash`. GitHub itself performs the merge only when **every** ruleset requirement is satisfied — including the required `Owner Approval Gate` check (i.e. a valid `APPROVED` review from `@ArchdukeViel` for the current head SHA), all other required checks, mergeability, and absence of blocking review states.
4. **Never weakened**: the automation never submits approvals, never merges major updates, and never bypasses the risk-classification check (Dependabot PRs use the path-derived risk automatically via `cargo xtask risk-check`).

Result: a patch/minor Dependabot PR is merged automatically **only after the owner's explicit approval and full required-check success**; major updates always wait for a manual merge.

## 7. CI Workflows

Configured in `.github/workflows/ci.yml` (required checks) — `Repository Hygiene & Architecture`, `Risk Classification Verification`, `Rust Check, Clippy & Tests`, `npm Workspaces Validation`, `Release Readiness & Version Alignment` — all delegating validation to `cargo xtask` commands. The risk job passes the PR author identity from the event payload to `cargo xtask risk-check` via the `PR_AUTHOR` environment variable so the trusted Dependabot exemption is evaluated from trustworthy metadata.
