# Security Policy

GatewayMux is a security-critical component that manages sensitive upstream API keys, credentials, and outbound network traffic for developer tools and coding subagents. We take security vulnerabilities seriously.

## Supported Versions

Only the latest release version on the `main` branch receives active security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.0-pre | :white_check_mark: |
| < 0.1.0 | :x:                |

## Reporting a Vulnerability

**DO NOT report security vulnerabilities via public GitHub issues, discussions, or pull requests.**

Public issue trackers must never contain API keys, credentials, tokens, or details of exploitable vulnerabilities.

If you discover a security vulnerability in GatewayMux:

1. **GitHub Security Advisory**: Use the [Private Security Advisory](https://github.com/ArchdukeViel/gatewaymux/security/advisories/new) feature on GitHub.
2. **Direct Contact**: If GitHub advisories are unavailable, contact the project maintainer directly at:
   `security@gatewaymux.invalid` (or via private communication to `@ArchdukeViel`).

### What to Include in a Report
- A detailed description of the vulnerability.
- Steps to reproduce or proof-of-concept code.
- The affected component (`core`, `codex-bridge`, `sync`, `server`, etc.).
- The potential impact and attack vectors.

You should receive an acknowledgment within 48 hours. We will coordinate remediation and a disclosure timeline prior to public release.

## Security Architecture & Invariants

1. **Loopback Isolation**: By default, GatewayMux data and control planes bind strictly to `127.0.0.1`. Remote access is forbidden unless explicitly enabled by the user with mutual authentication.
2. **Secret Store Protection**: Upstream API keys and OAuth tokens are never stored in plain text configuration files. They are managed through OS-native secret stores (Windows Credential Manager / DPAPI) or environment variables.
3. **Egress Boundaries**: All provider deployments default to the `general_cloud` trust tier. High-trust requests cannot route to unapproved external endpoints.
4. **Codex Subagent Isolation**: The Codex Bridge strictly limits interception to confirmed child subagents. Parent session credentials and official ChatGPT telemetry paths remain protected.
5. **No Secret Telemetry**: GatewayMux analytics and audit logs capture operational metadata only. Request bodies, prompt contents, and credentials are never logged or exported.

## Security-Sensitive Changes (R4)

Modifications affecting authentication, secrets, credential stores, cloud sync encryption, or Codex process interception are classified as **R4 (Critical Risk)**. These changes require:
- Explicit threat modeling and security rationale in the PR description;
- Isolated verification proving no credential leakage or privilege escalation;
- Mandatory review and sign-off by `@ArchdukeViel`.
