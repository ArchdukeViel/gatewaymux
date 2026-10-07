# Rule: Release Artifacts

## Purpose
Govern release packaging, coordinated versioning, and distribution safety.

## Invariants
1. **Coordinated Versioning**: The native executable (`gatewaymux-app`), npm distribution wrapper (`gatewaymux`), dashboard (`@gatewaymux/dashboard`), sync server (`@gatewaymux/sync-server`), and Windows installer share a single, unified semver version string.
2. **Independent Protocol Revisions**: Protocol and schema version numbers (sync protocol, config schema, corpus revision) increment independently according to wire/schema compatibility rules.
3. **npm Distribution Safety**: The `gatewaymux` npm package is strictly a delivery channel for the pre-built native Windows binary. Postinstall binary downloading or execution of arbitrary remote scripts is strictly prohibited.
4. **Release Manifest**: Official builds require a machine-readable release manifest detailing version, commit, SHA-256 checksums, and Authenticode signatures.
5. **Validation**: Verified locally and in CI via `cargo xtask release-check`.
