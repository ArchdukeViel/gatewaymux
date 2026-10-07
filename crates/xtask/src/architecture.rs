//! Workspace directional dependency boundary enforcement.
//!
//! The normative allowed internal-dependency matrix is defined here and
//! enforced mechanically by `cargo xtask architecture-check` on every
//! commit and pull request. The matrix implements PRD Section 28.2 and
//! ADR-0002:
//!
//! - `gatewaymux-routing` depends only on `gatewaymux-core`. It MUST NOT
//!   depend on `gatewaymux-providers` (or any concrete provider
//!   implementation); routing operates on provider-neutral contracts.
//! - `gatewaymux-sync-server` (the Rust reference Sync Server under
//!   `sync-server/`) depends only on `gatewaymux-core` and
//!   `gatewaymux-sync`.

use std::collections::BTreeMap;

use crate::run_cmd_output;

/// The normative allowed internal workspace dependency matrix.
///
/// Keys are workspace crate names; values are the internal workspace
/// crates each crate is permitted to depend on. Any internal dependency
/// not listed here is an architectural violation.
pub fn allowed_workspace_dependencies() -> BTreeMap<&'static str, Vec<&'static str>> {
    [
        ("gatewaymux-core", vec![]),
        ("gatewaymux-protocols", vec!["gatewaymux-core"]),
        (
            "gatewaymux-providers",
            vec!["gatewaymux-core", "gatewaymux-protocols"],
        ),
        ("gatewaymux-routing", vec!["gatewaymux-core"]),
        ("gatewaymux-sync", vec!["gatewaymux-core"]),
        ("gatewaymux-codex-bridge", vec!["gatewaymux-core"]),
        (
            "gatewaymux-server",
            vec![
                "gatewaymux-core",
                "gatewaymux-protocols",
                "gatewaymux-providers",
                "gatewaymux-routing",
            ],
        ),
        (
            "gatewaymux-cli",
            vec![
                "gatewaymux-core",
                "gatewaymux-protocols",
                "gatewaymux-providers",
                "gatewaymux-routing",
                "gatewaymux-server",
                "gatewaymux-sync",
                "gatewaymux-codex-bridge",
            ],
        ),
        (
            "gatewaymux-app",
            vec![
                "gatewaymux-core",
                "gatewaymux-protocols",
                "gatewaymux-providers",
                "gatewaymux-routing",
                "gatewaymux-server",
                "gatewaymux-cli",
                "gatewaymux-sync",
                "gatewaymux-codex-bridge",
            ],
        ),
        (
            "gatewaymux-sync-server",
            vec!["gatewaymux-core", "gatewaymux-sync"],
        ),
        ("xtask", vec![]),
    ]
    .into_iter()
    .collect()
}

/// Detect all architectural boundary violations in a workspace dependency
/// map (crate name -> direct dependency names, as reported by
/// `cargo metadata`).
pub fn detect_architecture_violations(pkg_deps: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    let mut violations = Vec::new();
    let allowed_rules = allowed_workspace_dependencies();

    for (crate_name, allowed) in &allowed_rules {
        if let Some(actual_deps) = pkg_deps.get(*crate_name) {
            for dep in actual_deps {
                // Check internal workspace dependencies
                if (dep.starts_with("gatewaymux-") || dep == "xtask")
                    && !allowed.contains(&dep.as_str())
                {
                    violations.push(format!(
                        "Architectural violation: crate '{crate_name}' is not permitted to depend on workspace crate '{dep}'"
                    ));
                }
            }
        } else {
            violations.push(format!(
                "Expected workspace crate '{crate_name}' missing from cargo metadata"
            ));
        }
    }

    // Explicit Policy Rule: gatewaymux-core must be OS-neutral
    if let Some(core_deps) = pkg_deps.get("gatewaymux-core") {
        for d in core_deps {
            if d == "windows" || d == "winapi" || d == "windows-sys" {
                violations.push(format!(
                    "gatewaymux-core must be OS-neutral and cannot depend on '{d}'"
                ));
            }
        }
    }

    // Explicit Policy Rule: no Rust crate may depend on dashboard/UI code
    for (name, deps) in pkg_deps {
        for d in deps {
            if d.contains("dashboard") || d.contains("ui") {
                violations.push(format!(
                    "Rust crate '{name}' cannot depend on UI/dashboard '{d}'"
                ));
            }
        }
    }

    violations
}

/// `cargo xtask architecture-check` entry point.
pub fn cmd_architecture_check(root: &std::path::Path) -> Result<(), String> {
    println!("=== Running Architectural Dependency Direction Check ===");

    let metadata_output = run_cmd_output(
        "cargo",
        &["metadata", "--format-version", "1", "--no-deps"],
        root,
    )?;
    let meta: serde_json::Value = serde_json::from_str(&metadata_output)
        .map_err(|e| format!("Failed to parse cargo metadata JSON: {e}"))?;

    let packages = meta
        .get("packages")
        .and_then(|p| p.as_array())
        .ok_or_else(|| "No packages array in cargo metadata".to_string())?;

    let mut pkg_deps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for pkg in packages {
        let name = pkg
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let mut deps = Vec::new();
        if let Some(dep_arr) = pkg.get("dependencies").and_then(|d| d.as_array()) {
            for dep in dep_arr {
                if let Some(dep_name) = dep.get("name").and_then(|n| n.as_str()) {
                    deps.push(dep_name.to_string());
                }
            }
        }
        pkg_deps.insert(name, deps);
    }

    println!("Detected workspace members ({}):", pkg_deps.len());
    for (name, deps) in &pkg_deps {
        println!("  \u{2022} {name} -> [{}]", deps.join(", "));
    }

    let violations = detect_architecture_violations(&pkg_deps);

    if !violations.is_empty() {
        eprintln!(
            "\nArchitectural boundary violations found ({}):",
            violations.len()
        );
        for v in violations {
            eprintln!("  - {v}");
        }
        return Err("Architectural boundary check failed.".to_string());
    }

    println!("[architecture-check PASS] Architectural dependency direction strictly adhered to across all workspace members.\n");
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn deps(map: &[(&'static str, &[&'static str])]) -> BTreeMap<String, Vec<String>> {
        map.iter()
            .map(|(k, v)| ((*k).to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    /// The complete expected workspace with the normative dependency set
    /// must produce zero violations.
    #[test]
    fn test_normative_workspace_has_no_violations() {
        let mut pkgs = deps(&[
            ("gatewaymux-core", &[]),
            ("gatewaymux-protocols", &["serde", "gatewaymux-core"]),
            (
                "gatewaymux-providers",
                &["gatewaymux-core", "gatewaymux-protocols"],
            ),
            ("gatewaymux-routing", &["gatewaymux-core"]),
            ("gatewaymux-sync", &["gatewaymux-core"]),
            ("gatewaymux-codex-bridge", &["gatewaymux-core"]),
            (
                "gatewaymux-server",
                &[
                    "gatewaymux-core",
                    "gatewaymux-protocols",
                    "gatewaymux-providers",
                    "gatewaymux-routing",
                ],
            ),
            (
                "gatewaymux-cli",
                &[
                    "gatewaymux-core",
                    "gatewaymux-protocols",
                    "gatewaymux-providers",
                    "gatewaymux-routing",
                    "gatewaymux-server",
                    "gatewaymux-sync",
                    "gatewaymux-codex-bridge",
                ],
            ),
            (
                "gatewaymux-app",
                &[
                    "gatewaymux-core",
                    "gatewaymux-protocols",
                    "gatewaymux-providers",
                    "gatewaymux-routing",
                    "gatewaymux-server",
                    "gatewaymux-cli",
                    "gatewaymux-sync",
                    "gatewaymux-codex-bridge",
                ],
            ),
            (
                "gatewaymux-sync-server",
                &["gatewaymux-core", "gatewaymux-sync"],
            ),
            ("xtask", &["serde", "serde_json"]),
        ]);
        // Exercise the full matrix by including every expected member.
        let violations = detect_architecture_violations(&pkgs);
        assert!(
            violations.is_empty(),
            "unexpected violations: {violations:?}"
        );
        // Sanity: all expected members are covered by the matrix.
        for name in allowed_workspace_dependencies().keys() {
            assert!(pkgs.contains_key(*name), "matrix key {name} not exercised");
        }
        let _ = &mut pkgs;
    }

    #[test]
    fn test_routing_must_not_depend_on_providers() {
        let pkgs = deps(&[
            ("gatewaymux-core", &[]),
            ("gatewaymux-protocols", &["gatewaymux-core"]),
            (
                "gatewaymux-providers",
                &["gatewaymux-core", "gatewaymux-protocols"],
            ),
            (
                "gatewaymux-routing",
                &["gatewaymux-core", "gatewaymux-providers"],
            ),
            ("gatewaymux-sync", &["gatewaymux-core"]),
            ("gatewaymux-codex-bridge", &["gatewaymux-core"]),
            (
                "gatewaymux-server",
                &[
                    "gatewaymux-core",
                    "gatewaymux-protocols",
                    "gatewaymux-providers",
                    "gatewaymux-routing",
                ],
            ),
            ("gatewaymux-cli", &[]),
            ("gatewaymux-app", &[]),
            ("gatewaymux-sync-server", &[]),
            ("xtask", &[]),
        ]);
        let violations = detect_architecture_violations(&pkgs);
        assert!(
            violations
                .iter()
                .any(|v| v.contains("gatewaymux-routing") && v.contains("gatewaymux-providers")),
            "routing -> providers must be rejected: {violations:?}"
        );
    }

    #[test]
    fn test_routing_must_not_depend_on_protocols_under_current_contract() {
        let pkgs = deps(&[
            ("gatewaymux-core", &[]),
            ("gatewaymux-protocols", &["gatewaymux-core"]),
            (
                "gatewaymux-providers",
                &["gatewaymux-core", "gatewaymux-protocols"],
            ),
            (
                "gatewaymux-routing",
                &["gatewaymux-core", "gatewaymux-protocols"],
            ),
            ("gatewaymux-sync", &["gatewaymux-core"]),
            ("gatewaymux-codex-bridge", &["gatewaymux-core"]),
            (
                "gatewaymux-server",
                &[
                    "gatewaymux-core",
                    "gatewaymux-protocols",
                    "gatewaymux-providers",
                    "gatewaymux-routing",
                ],
            ),
            ("gatewaymux-cli", &[]),
            ("gatewaymux-app", &[]),
            ("gatewaymux-sync-server", &[]),
            ("xtask", &[]),
        ]);
        let violations = detect_architecture_violations(&pkgs);
        assert!(
            violations
                .iter()
                .any(|v| v.contains("gatewaymux-routing") && v.contains("gatewaymux-protocols")),
            "routing -> protocols must be rejected under the current contract: {violations:?}"
        );
    }

    #[test]
    fn test_sync_server_allowed_dependencies() {
        let base: &[(&'static str, &[&'static str])] = &[
            ("gatewaymux-core", &[]),
            ("gatewaymux-protocols", &["gatewaymux-core"]),
            (
                "gatewaymux-providers",
                &["gatewaymux-core", "gatewaymux-protocols"],
            ),
            ("gatewaymux-routing", &["gatewaymux-core"]),
            ("gatewaymux-sync", &["gatewaymux-core"]),
            ("gatewaymux-codex-bridge", &["gatewaymux-core"]),
            (
                "gatewaymux-server",
                &[
                    "gatewaymux-core",
                    "gatewaymux-protocols",
                    "gatewaymux-providers",
                    "gatewaymux-routing",
                ],
            ),
            ("gatewaymux-cli", &[]),
            ("gatewaymux-app", &[]),
            ("xtask", &[]),
        ];

        // core + sync are permitted
        let mut ok = deps(base);
        ok.insert(
            "gatewaymux-sync-server".to_string(),
            vec!["gatewaymux-core".to_string(), "gatewaymux-sync".to_string()],
        );
        assert!(detect_architecture_violations(&ok).is_empty());

        // server / providers / routing are not permitted
        for forbidden in [
            "gatewaymux-server",
            "gatewaymux-providers",
            "gatewaymux-routing",
        ] {
            let mut bad = deps(base);
            bad.insert(
                "gatewaymux-sync-server".to_string(),
                vec!["gatewaymux-core".to_string(), forbidden.to_string()],
            );
            let violations = detect_architecture_violations(&bad);
            assert!(
                violations
                    .iter()
                    .any(|v| v.contains("gatewaymux-sync-server") && v.contains(forbidden)),
                "sync-server -> {forbidden} must be rejected: {violations:?}"
            );
        }
    }

    #[test]
    fn test_missing_expected_member_is_a_violation() {
        let pkgs = deps(&[("gatewaymux-core", &[])]);
        let violations = detect_architecture_violations(&pkgs);
        assert!(
            violations
                .iter()
                .any(|v| v.contains("missing from cargo metadata")),
            "missing members must be reported: {violations:?}"
        );
    }

    #[test]
    fn test_core_os_neutrality() {
        let mut pkgs = deps(&[
            ("gatewaymux-core", &["serde", "windows-sys"]),
            ("gatewaymux-protocols", &["gatewaymux-core"]),
            (
                "gatewaymux-providers",
                &["gatewaymux-core", "gatewaymux-protocols"],
            ),
            ("gatewaymux-routing", &["gatewaymux-core"]),
            ("gatewaymux-sync", &["gatewaymux-core"]),
            ("gatewaymux-codex-bridge", &["gatewaymux-core"]),
            (
                "gatewaymux-server",
                &[
                    "gatewaymux-core",
                    "gatewaymux-protocols",
                    "gatewaymux-providers",
                    "gatewaymux-routing",
                ],
            ),
            ("gatewaymux-cli", &[]),
            ("gatewaymux-app", &[]),
            ("gatewaymux-sync-server", &[]),
            ("xtask", &[]),
        ]);
        let violations = detect_architecture_violations(&pkgs);
        assert!(
            violations.iter().any(|v| v.contains("OS-neutral")),
            "windows dependency in core must be rejected: {violations:?}"
        );
        let _ = &mut pkgs;
    }

    #[test]
    fn test_no_crate_depends_on_dashboard() {
        let pkgs = deps(&[
            ("gatewaymux-core", &[]),
            ("gatewaymux-protocols", &["gatewaymux-core"]),
            (
                "gatewaymux-providers",
                &["gatewaymux-core", "gatewaymux-protocols"],
            ),
            ("gatewaymux-routing", &["gatewaymux-core"]),
            ("gatewaymux-sync", &["gatewaymux-core"]),
            ("gatewaymux-codex-bridge", &["gatewaymux-core"]),
            (
                "gatewaymux-server",
                &[
                    "gatewaymux-core",
                    "gatewaymux-protocols",
                    "gatewaymux-providers",
                    "gatewaymux-routing",
                ],
            ),
            ("gatewaymux-cli", &[]),
            ("gatewaymux-app", &[]),
            ("gatewaymux-sync-server", &[]),
            ("xtask", &[]),
            ("some-crate", &["@gatewaymux/dashboard"]),
        ]);
        let violations = detect_architecture_violations(&pkgs);
        assert!(
            violations
                .iter()
                .any(|v| v.contains("cannot depend on UI/dashboard")),
            "dashboard dependency must be rejected: {violations:?}"
        );
    }
}
