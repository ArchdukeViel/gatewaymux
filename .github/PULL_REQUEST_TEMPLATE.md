## Summary
<!-- Concise description of what this PR changes -->

## Why
<!-- What problem does this solve or what requirement does this fulfill? -->

## Declared Risk Class
<!-- Select exactly one declared risk tier. Note: CI enforces path-derived minimums. -->
- [ ] **R0**: Docs / comments / non-runtime fixtures
- [ ] **R1**: Isolated implementation / unit tests / CLI
- [ ] **R2**: Provider / protocol / model-profile behavior
- [ ] **R3**: Routing / persistence / public API / configuration
- [ ] **R4**: Security / auth / trust / cloud sync / Codex interception

## Affected Contracts & Areas
- [ ] `gatewaymux-core`
- [ ] `gatewaymux-protocols`
- [ ] `gatewaymux-providers`
- [ ] `gatewaymux-routing`
- [ ] `gatewaymux-sync`
- [ ] `gatewaymux-codex-bridge`
- [ ] `gatewaymux-server`
- [ ] `gatewaymux-cli` / `gatewaymux-app`
- [ ] `dashboard` / `sync-server`
- [ ] Profiles / Schemas / Documentation

## Correctness Impact
<!-- Does this alter any semantic interpretation, streaming invariant, or error code? -->

## Security & Egress Impact
<!-- Does this change credential handling, SecretStore, network egress, or trust gates? -->

## Migration Impact
<!-- Does this alter configuration schema or database persistence? If so, are migrations included? -->

## Compatibility Corpus Status
<!-- Does this address a client compatibility regression? If yes, is a sanitized fixture included under compat/? -->
- [ ] New sanitized fixture added to `compat/` and registered in `compat/corpus-manifest.json`
- [ ] Not applicable (no client/provider compatibility changes)

## Tests & Verification Record
<!-- List all tests run and their results -->
- [ ] `cargo xtask check` passed
- [ ] `cargo xtask test` passed
- [ ] `cargo xtask architecture-check` passed
- [ ] `cargo xtask repo-check` passed

## Documentation, PRD & ADR Changes
<!-- Did this require updates to the PRD, engineering docs, or a new ADR? -->
- [ ] PRD updated / No changes required
- [ ] ADR added / No ADR required

## Rollback Considerations
<!-- How can this change be safely rolled back in production if an unexpected defect occurs? -->
