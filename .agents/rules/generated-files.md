# Rule: Generated Files

## Purpose
Govern definition, generation, and drift detection for mechanically generated repository artifacts.

## Invariants
1. **Never Hand-Edit Generated Files**: Any file registered in `.generated-manifest.json` is generated code. Never edit it manually.
2. **Authoritative Source is King**: Modify the underlying generator or authoritative source file, then re-run `cargo xtask generate`.
3. **Real Reproducibility Verification**: `cargo xtask generate --check` regenerates every registered output deterministically in an isolated in-memory state and byte-compares it with the tracked artifact. Identical output passes; any difference (including formatting-only drift) fails. `--check` never writes to the working tree.
4. **Registered Generator Allowlist**: Each manifest entry's `generator` field must reference a generator compiled into `xtask` (`copy`, `json-canonicalize`). Manifest data can never declare arbitrary generators or invoke shell commands.
5. **Manifest Schema v2**: Entries accept exactly `path`, `generator`, and `authoritative_source`. Unknown keys, unsupported manifest versions, missing fields, duplicate targets, and self-referencing entries are rejected.
6. **Path Safety**: Manifest paths must be repository-relative; absolute paths, drive-qualified paths, and `..` traversal components are rejected. Generated targets stay inside the repository.
7. **Failure Never Mutates**: Any verification or generation failure must leave source files and tracked artifacts untouched; temporary state is always cleaned up.
8. **Drift Detection**: Verified via `cargo xtask generate --check` in local checks and CI.
