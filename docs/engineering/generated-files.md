# Repository Artifact Classes & Generated Files

This document formalizes the four repository artifact classes and specifies the management policy for mechanically generated files.

## Four Repository Artifact Classes

1. **`SOURCE`**:
   - Authoritative human- or agent-written code, documentation, templates, and configurations.
   - Subject to strict peer review, formatting rules, and static analysis.
   - Examples: Rust source files in `crates/`, Markdown documentation in `docs/`, `Cargo.toml`.

2. **`GENERATED`**:
   - Mechanically produced from an authoritative `SOURCE` artifact (e.g. JSON schemas derived from Rust types, generated manifests).
   - Tracked in Git ONLY when necessary for distribution or quick inspection.
   - **Strictly read-only**: Never edit a generated file by hand.
   - Registered in `.generated-manifest.json`.

3. **`FIXTURE`**:
   - Durable test vectors, golden transcripts, and Compatibility Corpus captures.
   - Tracked in Git under `compat/` or `tests/fixtures/`.
   - **Strictly sanitized**: Zero real credentials, tokens, or PII.

4. **`EPHEMERAL`**:
   - Build outputs, temporary logs, coverage reports, test output, agent scratchpads.
   - Strictly forbidden from Git. Must be covered by `.gitignore`.
   - Examples: `target/`, `node_modules/`, `dist/`, `.scratch/`.

## Generated File Manifest (`.generated-manifest.json`)

All tracked generated files must be registered in `.generated-manifest.json`:

```json
{
  "version": 1,
  "description": "Authoritative registry of mechanically generated files in the GatewayMux repository",
  "generated_files": [
    {
      "path": "config/schemas/gatewaymux.generated.json",
      "generator": "cargo xtask generate-schema",
      "authoritative_source": "crates/gatewaymux-core/src/config.rs"
    }
  ]
}
```

## Drift Detection & Verification

CI runs `cargo xtask generate --check` on every pull request:
- Verifies that all registered generated files exist.
- Verifies that running the generator produces bit-identical output.
- Rejects any PR where generated files were modified without updating the authoritative source, or where generator output drifted.
