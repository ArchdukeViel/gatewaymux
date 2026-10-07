# Rule: Codex Bridge

## Purpose
Govern the Windows-specific Codex child-subagent interception architecture and safety boundaries.

## Invariants
1. **Targeted Subagent Interception Only**: Only confirmed Codex child subagent requests may be intercepted. The parent conversation session and official ChatGPT authentication paths MUST remain untouched.
2. **Fail-Closed Passthrough**: If Codex Bridge encounters an unrecognized Codex binary version, altered payload schema, or ambiguous request type, it MUST fall back to `official_passthrough` rather than corrupting the session.
3. **Core Invocation via Contract**: Codex Bridge communicates directly with GatewayMux Core routing interfaces. It MUST NOT reach directly into provider adapters (`gatewaymux-providers`).
4. **Risk Classification R4**: Any modification touching the Codex Bridge (`crates/gatewaymux-codex-bridge/`) carries a path minimum risk of R4.
5. **Platform Boundary**: All Windows API interactions and process inspection code belong exclusively in `gatewaymux-codex-bridge` or `gatewaymux-app`.
