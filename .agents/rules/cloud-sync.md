# Rule: Cloud Sync

## Purpose
Govern state synchronization, distributed coordination, and multi-device consistency rules.

## Invariants
1. **Device-Local Isolation**: Device-local overrides (e.g. local loopback ports, device names, machine-specific secret stores) are NEVER overwritten by remote sync payloads.
2. **Secrets Stay Local by Default**: Secrets and credentials are not synchronized unless the user explicitly configures the separate End-to-End Encrypted (E2EE) Secret Vault.
3. **No Silent Last-Write-Wins**: When remote and local configuration revisions conflict, the sync engine must create a Safe Draft and notify the user for manual merge/review.
4. **Strict Quota Coordination**: Distributed quota coordination across devices must fail safe; if backend coordination capabilities are unavailable or degrade, strict quota policies must decline requests rather than allowing upstream quota breaches.
5. **Risk Classification R4**: Sync engine modifications (`crates/gatewaymux-sync/**`) and reference Sync Server modifications (`sync-server/**`) carry a path minimum risk of R4.
6. **Rust Reference Server**: The reference Sync Server is the Rust workspace member `gatewaymux-sync-server` (`sync-server/`), NOT a Node.js/TypeScript application. It depends only on `gatewaymux-core` and `gatewaymux-sync`, and must not depend on the gateway runtime crates.
