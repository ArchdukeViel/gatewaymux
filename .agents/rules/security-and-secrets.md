# Rule: Security and Secrets

## Purpose
Enforce zero-trust credential safety and isolation standards across the repository.

## Invariants
1. **Never Commit Secrets**: Never commit, print, echo, or log API keys, bearer tokens, OAuth refresh tokens, private keys, or passwords.
2. **Prohibited File Patterns**: Never read, print, display, log, or commit `.env*` files, `~/.ssh/*`, `~/.aws/*`, `*.pem`, `*.key`, `id_rsa`, or `id_ed25519`.
3. **SecretStore Trait**: Provider credentials and GatewayMux client API keys must be retrieved via the `SecretStore` abstraction (Windows Credential Manager / DPAPI or environment variables).
4. **Masking Obligation**: All API keys, tokens (e.g. `ghp_*`, `sk-*`, Cloudflare, Gemini, Antigravity OAuth tokens), and JWT secrets must be masked (`***` or `[REDACTED]`) in all logs and outputs.
5. **Egress Boundaries**: All provider deployments default to `general_cloud` trust tier. Requests requiring high trust or data protection MUST NOT route to unverified external endpoints.
6. **Local-First Listeners**: Data and control planes bind to `127.0.0.1` by default. Remote binding requires explicit user opt-in and authentication.
