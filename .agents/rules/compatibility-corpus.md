# Rule: Compatibility Corpus

## Purpose
Preserve durable protocol transcripts and test vectors to prevent regressions with AI clients and upstream providers.

## Invariants
1. **Regression Capture**: Any reported bug regarding client compatibility (OpenAI SDK, Claude Code, Cursor, Codex, etc.) or upstream provider behavior must result in a new fixture under `compat/`.
2. **Strict Sanitization**:
   - Zero real credentials, authorization headers with bearer tokens, API keys (`sk-*`, `ghp_*`), or private data.
   - All sensitive values must be replaced with dummy values like `test-key-mock` or `[REDACTED]`.
3. **Manifest Tracking**: New fixtures must be registered in `compat/corpus-manifest.json` with client version and description.
4. **Validation**: Run `cargo xtask compat` to verify fixture structure and automated sanitization checks.
