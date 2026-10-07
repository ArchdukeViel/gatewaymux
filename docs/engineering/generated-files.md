# Repository Artifact Classes & Generated Files

This document formalizes the four repository artifact classes and specifies the management policy for mechanically generated files, including the exact contract enforced by `cargo xtask generate --check`.

## Four Repository Artifact Classes

1. **`SOURCE`**:
   - Authoritative human- or agent-written code, documentation, templates, and configurations.
   - Subject to strict peer review, formatting rules, and static analysis.
   - Examples: Rust source files in `crates/`, Markdown documentation in `docs/`, `Cargo.toml`.

2. **`GENERATED`**:
   - Mechanically produced from an authoritative `SOURCE` artifact.
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

All tracked generated files must be registered in `.generated-manifest.json` (schema version 2):

```json
{
  "version": 2,
  "description": "Authoritative registry of mechanically generated files...",
  "generated_files": [
    {
      "path": "config/schemas/gatewaymux.generated.json",
      "generator": "json-canonicalize",
      "authoritative_source": "config/schemas/gatewaymux.source.json"
    }
  ]
}
```

The schema is machine-verified, not decorative text:

- `version` MUST be `2`; other versions are rejected.
- Each entry accepts exactly the keys `path`, `generator`, and `authoritative_source`; unknown keys are rejected.
- `generator` MUST reference a generator registered (compiled) into `xtask`. The registered generators are:
  - `copy`: byte-for-byte copy of the authoritative source;
  - `json-canonicalize`: canonicalized JSON (parsed, keys sorted, stable pretty formatting, trailing newline).
- Manifest data can never invoke arbitrary commands: there is no shell execution, and new generators are added by modifying xtask code through a reviewed pull request, never by editing the manifest.
- `path` and `authoritative_source` must be non-empty, repository-relative paths. Absolute paths, drive-qualified paths, empty components, and `..`/`.` traversal components are rejected; every resolved path stays inside the repository.
- Entries must not point at their own source, and target paths must be unique.

## Drift Detection & Verification Contract

CI runs `cargo xtask generate --check` on every pull request. `--check` implements a real deterministic regenerate-and-compare:

```text
authoritative SOURCE
    -> declared (registered) generator
    -> generated output in isolated in-memory state
    -> byte-for-byte deterministic comparison with the tracked GENERATED artifact
    -> identical = PASS
    -> different  = FAIL (drift)
```

Guarantees:

- **`--check` never writes to the working tree.** Generation happens in memory; the tracked artifact is only read and compared. This holds even when drift is detected or the target is missing.
- **Determinism is byte-exact**, including formatting (a reformatting-only difference is drift).
- **Plain `cargo xtask generate`** (without `--check`) deliberately rewrites registered generated outputs from their authoritative sources; it never touches unregistered files.
- Failures (invalid JSON sources, unreadable files, rejected paths) never mutate source or target files.
- The engine is covered by unit tests using disposable temporary directories (identical generation passes; drift fails; invalid paths and unknown generators are rejected; check mode provably does not mutate the checked target; temporary state is cleaned up).

The repository currently registers **zero** generated files (pre-implementation scaffold); the empty manifest is a clean PASS. Product schemas must not be invented merely to populate the manifest — entries are added when real generated artifacts exist.
