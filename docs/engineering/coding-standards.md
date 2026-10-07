# Coding Standards & Naming Conventions

This document establishes the official coding standards, design conventions, and naming rules across the GatewayMux codebase.

## 1. Stable Naming Conventions

Stable machine identifiers MUST NOT be renamed merely for display purposes. Display labels are user-facing strings that can be modified independently of internal IDs.

| Domain | Convention | Example |
|---|---|---|
| Rust Crates | Kebab-case with `gatewaymux-` prefix | `gatewaymux-routing`, `gatewaymux-core` |
| Rust Modules | Snake_case | `retry_budget`, `attempt_ledger` |
| Rust Types / Structs | UpperCamelCase | `QuotaBucket`, `CanonicalOperation` |
| Rust Enums & Variants | UpperCamelCase | `PricingTier::FreeTier`, `QualityTier::TierS` |
| Rust Functions / Methods | Snake_case | `evaluate_route()`, `record_attempt()` |
| Provider IDs | Lowercase alphanumeric kebab-case | `nvidia`, `antigravity`, `openrouter-us` |
| Logical Model IDs | Lowercase alphanumeric kebab-case | `claude-sonnet-tier`, `fast-chat` |
| Deployment IDs | Provider + region/variant suffix | `nim-cloud`, `openrouter-eu` |
| React / TypeScript Components | PascalCase | `RouteSimulator.tsx`, `QuotaViewer.tsx` |
| TypeScript Utilities | CamelCase or kebab-case | `formatTokens.ts`, `api-client.ts` |
| SQL Migrations | `NNNN_descriptive_name.sql` | `0001_initial_schema.sql` |
| ADRs | `NNNN-short-kebab-title.md` | `0001-repository-workspace.md` |
| Compatibility Fixtures | `[client]-[operation]-[scenario].json` | `openai-chat-tools-call.json` |
| GitHub Workflows | Kebab-case `.yml` | `ci.yml`, `owner-approval.yml` |
| Generated Artifacts | Explicit `.generated.` infix | `config_schema.generated.json` |

## 2. Rust Engineering Standards

1. **Clippy & Formatting**: All Rust code must pass `cargo fmt --check` and `cargo clippy --workspace -- -D warnings`.
2. **Error Handling**:
   - Use `thiserror` for typed domain errors within crates.
   - Map all errors into standard `GMX_*` error taxonomy codes before client responses.
   - Never use `.unwrap()` or `.expect()` in runtime request paths. Return typed results.
3. **Safety & Panics**: `#![forbid(unsafe_code)]` is enforced across all core crates unless low-level Win32 FFI requires `unsafe` in `gatewaymux-codex-bridge`, where every unsafe block must include a `// SAFETY:` rationale.
4. **Cancellation & Async**: Asynchronous request handlers must respect `tokio::select!` cancellation tokens. When a client closes the socket, upstream requests must be cancelled promptly to conserve provider quotas.
5. **No Provider Splice**: Once a streaming response yields visible tokens to the client, the provider path is locked. Mid-stream failover is forbidden.

## 3. Shell & Cross-Platform Conventions

1. **Windows & PowerShell**: Development is primary on Windows. Never write Unix-specific bash commands (`export`, `cat`, `grep`, `rm -rf`, `touch`) in documentation, task scripts, or tooling.
2. **Path Separators**: When writing cross-platform code, use `Path::join` or normalize paths with forward slashes.
3. **BOM-Free UTF-8**: All files written by scripts or agents must use UTF-8 without Byte Order Marks (`\uFEFF`).
