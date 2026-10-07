# Rule: Generated Files

## Purpose
Govern definition, generation, and drift detection for mechanically generated repository artifacts.

## Invariants
1. **Never Hand-Edit Generated Files**: Any file registered in `.generated-manifest.json` is generated code. Never edit it manually.
2. **Authoritative Source is King**: Modify the underlying generator script or authoritative schema file, then re-run generation.
3. **Reproducibility**: Generation must be deterministic. Running generation on the same source must yield bit-identical output.
4. **Manifest Registration**: Every tracked generated file must be declared in `.generated-manifest.json` with its generator and source path.
5. **Drift Detection**: Verified via `cargo xtask generate --check` in local checks and CI.
