//! Change-risk classification and PR risk-declaration enforcement.
//!
//! Every human- or agent-authored pull request MUST declare an R0-R4 risk
//! class; CI rejects missing declarations and under-declarations. Trusted
//! GitHub Dependabot pull requests are exempt from manual declaration and
//! use a path-derived risk class automatically. Dependabot identity is
//! accepted only from CI-provided event metadata (e.g. the `PR_AUTHOR`
//! environment variable set from the `pull_request` event payload), never
//! from pull request body text.

use std::env;
use std::path::Path;

use crate::run_cmd_output;

/// The only actor identity exempt from manual risk declaration.
/// Deliberately not a generic "bot" exemption: it matches the exact
/// GitHub Dependabot account.
pub const TRUSTED_DEPENDABOT_ACTOR: &str = "dependabot[bot]";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum RiskClass {
    R0,
    R1,
    R2,
    R3,
    R4,
}

impl std::str::FromStr for RiskClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "R0" => Ok(RiskClass::R0),
            "R1" => Ok(RiskClass::R1),
            "R2" => Ok(RiskClass::R2),
            "R3" => Ok(RiskClass::R3),
            "R4" => Ok(RiskClass::R4),
            other => Err(format!("Unknown risk class: {other}")),
        }
    }
}

impl RiskClass {
    pub fn parse_opt(s: &str) -> Option<Self> {
        s.parse::<RiskClass>().ok()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RiskClass::R0 => "R0",
            RiskClass::R1 => "R1",
            RiskClass::R2 => "R2",
            RiskClass::R3 => "R3",
            RiskClass::R4 => "R4",
        }
    }
}

/// Classify a single repository path into its minimum risk class.
///
/// Path separators are normalized (backslash and forward slash) so the
/// classification is identical on Windows and Unix checkouts.
pub fn path_to_risk(path: &str) -> RiskClass {
    let p = path.replace('\\', "/");
    let p_lower = p.to_lowercase();

    // R4: Critical trust, auth, secrets, Codex interception, sync, and the
    // CI workflow security / software-supply-chain boundary.
    if p.starts_with(".github/workflows/")
        || p.contains("gatewaymux-codex-bridge")
        || p.contains("gatewaymux-sync")
        || p.starts_with("sync-server/")
        || p_lower.contains("secret")
        || p_lower.contains("auth")
        || p_lower.contains("owner-approval")
        || p_lower.contains("security")
        || p == "deny.toml"
        || p == "SECURITY.md"
    {
        RiskClass::R4
    }
    // R3: Core routing, public API, persistence, configuration, workspace
    // dependency manifests.
    else if p.contains("gatewaymux-routing")
        || p.contains("gatewaymux-server")
        || p.contains("gatewaymux-core")
        || p.starts_with("config/")
        || p == "Cargo.toml"
        || p == "Cargo.lock"
    {
        RiskClass::R3
    }
    // R2: Provider & protocol adapters, model profiles, dialects,
    // compatibility corpus.
    else if p.contains("gatewaymux-providers")
        || p.contains("gatewaymux-protocols")
        || p.starts_with("profiles/")
        || p.starts_with("compat/")
    {
        RiskClass::R2
    }
    // R1: Isolated implementation, leaf crates, dashboard, packaging,
    // tests, scripts, non-workflow GitHub configuration.
    else if p.contains("gatewaymux-cli")
        || p.contains("gatewaymux-app")
        || p.contains("xtask")
        || p.starts_with("dashboard/")
        || p.starts_with("packaging/")
        || p.starts_with("tests/")
        || p.starts_with("scripts/")
        || p.starts_with(".github/")
        || p == "package.json"
        || p == "package-lock.json"
        || p == "rust-toolchain.toml"
    {
        RiskClass::R1
    }
    // R0: Non-runtime documentation, comments, git configuration.
    else {
        RiskClass::R0
    }
}

/// Derive the minimum risk class across a set of modified paths.
pub fn derive_minimum_risk(paths: &[String]) -> RiskClass {
    paths
        .iter()
        .map(|p| path_to_risk(p))
        .max()
        .unwrap_or(RiskClass::R0)
}

/// True only for the exact trusted Dependabot actor identity.
pub fn is_trusted_dependabot(actor: &str) -> bool {
    actor.trim().eq_ignore_ascii_case(TRUSTED_DEPENDABOT_ACTOR)
}

/// Parse the declared risk class out of a pull request body that follows
/// the PR template checkbox format.
pub fn extract_declared_risk_from_body(body: &str) -> Option<RiskClass> {
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            for r in ["R4", "R3", "R2", "R1", "R0"] {
                if trimmed.contains(&format!("**{r}**"))
                    || trimmed.contains(&format!("**{r}:"))
                    || trimmed.contains(r)
                {
                    return RiskClass::parse_opt(r);
                }
            }
        }
    }
    None
}

/// The outcome of evaluating a change set against declared risk and the
/// (trusted) pull request actor.
#[derive(Debug, PartialEq, Eq)]
pub enum RiskDecision {
    /// The change set satisfies risk governance.
    Pass {
        /// The declared risk class (absent for Dependabot-exempt PRs).
        declared: Option<RiskClass>,
        /// The path-derived minimum risk class.
        derived: RiskClass,
        /// Whether the trusted Dependabot exemption was applied.
        dependabot_exempt: bool,
    },
    /// The change set violates risk governance; carries the reason.
    Fail(String),
}

/// Evaluate risk governance for a change set.
///
/// - Trusted Dependabot PRs (actor identity supplied by CI event metadata,
///   never PR text): declaration is not required; the path-derived minimum
///   is authoritative and must be derivable from a non-empty change set.
/// - Human/agent PRs: a declaration is mandatory and must be at least the
///   path-derived minimum.
/// - Non-PR contexts (local runs, push events): a declaration is optional
///   and informational, but an explicit under-declaration still fails.
pub fn evaluate_risk_decision(
    declared: Option<RiskClass>,
    modified_paths: &[String],
    pr_actor: Option<&str>,
) -> RiskDecision {
    let derived = derive_minimum_risk(modified_paths);

    if let Some(actor) = pr_actor {
        if is_trusted_dependabot(actor) {
            if modified_paths.is_empty() {
                return RiskDecision::Fail(format!(
                    "Trusted Dependabot PR exemption requires a path-derived risk class, but no modified paths could be determined; cannot exempt an unresolvable change set (path-derived minimum would be {}).",
                    derived.as_str()
                ));
            }
            return RiskDecision::Pass {
                declared: None,
                derived,
                dependabot_exempt: true,
            };
        }

        return match declared {
            Some(d) if d >= derived => RiskDecision::Pass {
                declared,
                derived,
                dependabot_exempt: false,
            },
            Some(d) => RiskDecision::Fail(format!(
                "Declared risk class {} is lower than the path-derived minimum {}. Contributors may elevate risk, but never lower it.",
                d.as_str(),
                derived.as_str()
            )),
            None => RiskDecision::Fail(format!(
                "No declared risk class found. Every human- or agent-authored pull request MUST declare an R0-R4 risk class in the PR template; the path-derived minimum for this change set is {}.",
                derived.as_str()
            )),
        };
    }

    match declared {
        Some(d) if d >= derived => RiskDecision::Pass {
            declared,
            derived,
            dependabot_exempt: false,
        },
        Some(d) => RiskDecision::Fail(format!(
            "Declared risk class {} is lower than the path-derived minimum {}. Contributors may elevate risk, but never lower it.",
            d.as_str(),
            derived.as_str()
        )),
        None => RiskDecision::Pass {
            declared: None,
            derived,
            dependabot_exempt: false,
        },
    }
}

/// Collect the set of modified paths: uncommitted working-tree changes,
/// plus the diff against an explicit or environment-provided base ref.
pub fn get_modified_paths(root: &Path, base: Option<&str>) -> Vec<String> {
    use std::collections::HashSet;

    let mut paths = HashSet::new();

    // 1. Uncommitted changes in working tree
    if let Ok(output) = run_cmd_output("git", &["status", "--porcelain"], root) {
        for line in output.lines() {
            if line.len() > 3 {
                let file = line[3..].trim();
                paths.insert(file.to_string());
            }
        }
    }

    // 2. Base diff: explicit base or environment variable
    let effective_base = base
        .map(|b| b.to_string())
        .or_else(|| {
            env::var("GITHUB_BASE_REF")
                .ok()
                .map(|b| format!("origin/{b}"))
        })
        .or_else(|| env::var("BASE_REF").ok().map(|b| format!("origin/{b}")));

    if let Some(ref base_ref) = effective_base {
        let triple_dot = format!("{base_ref}...HEAD");
        if let Ok(diff_output) = run_cmd_output("git", &["diff", "--name-only", &triple_dot], root)
        {
            for line in diff_output.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    paths.insert(trimmed.to_string());
                }
            }
        } else if let Ok(diff_output) =
            run_cmd_output("git", &["diff", "--name-only", base_ref], root)
        {
            for line in diff_output.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    paths.insert(trimmed.to_string());
                }
            }
        }
    } else if paths.is_empty() {
        // Fallback: if working tree is clean and no base is available,
        // compare against HEAD~1 if possible.
        if let Ok(diff_output) = run_cmd_output("git", &["diff", "--name-only", "HEAD~1"], root) {
            for line in diff_output.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    paths.insert(trimmed.to_string());
                }
            }
        }
    }

    let mut result: Vec<String> = paths.into_iter().collect();
    result.sort();
    result
}

/// `cargo xtask risk-check` entry point.
pub fn cmd_risk_check(args: &[String], root: &Path) -> Result<(), String> {
    println!("=== Running Risk Classification Check ===");

    let mut declared_risk: Option<RiskClass> = None;
    let mut base_ref: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--risk" && i + 1 < args.len() {
            declared_risk = RiskClass::parse_opt(&args[i + 1]);
            i += 2;
        } else if args[i] == "--base" && i + 1 < args.len() {
            base_ref = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    // If not supplied via CLI flag, check environment variables
    if declared_risk.is_none() {
        if let Ok(env_risk) = env::var("DECLARED_RISK") {
            declared_risk = RiskClass::parse_opt(&env_risk);
        } else if let Ok(pr_body) = env::var("PR_BODY") {
            declared_risk = extract_declared_risk_from_body(&pr_body);
        }
    }

    // Trusted actor identity: supplied by CI from the pull_request event
    // payload. Never derived from PR body text.
    let pr_actor: Option<String> = env::var("PR_AUTHOR")
        .ok()
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty());

    let modified_paths = get_modified_paths(root, base_ref.as_deref());
    let derived = derive_minimum_risk(&modified_paths);

    println!("Detected modified paths ({}):", modified_paths.len());
    for p in &modified_paths {
        println!("  - {p} -> {}", path_to_risk(p).as_str());
    }
    println!("Path-derived minimum risk class: {}", derived.as_str());
    match (&declared_risk, &pr_actor) {
        (Some(d), _) => println!("Declared risk class:             {}", d.as_str()),
        (None, Some(a)) => {
            if is_trusted_dependabot(a) {
                println!(
                    "Declared risk class:             (none; trusted Dependabot exemption applies)"
                );
            } else {
                println!(
                    "Declared risk class:             (none; PR actor '{a}' requires declaration)"
                );
            }
        }
        (None, None) => println!("Declared risk class:             (none; non-PR context)"),
    }

    let decision = evaluate_risk_decision(declared_risk, &modified_paths, pr_actor.as_deref());

    match decision {
        RiskDecision::Pass {
            declared,
            derived,
            dependabot_exempt,
        } => {
            if dependabot_exempt {
                println!(
                    "[risk-check PASS] Trusted Dependabot PR: manual declaration exempt; path-derived risk class {} is authoritative.\n",
                    derived.as_str()
                );
            } else if let Some(d) = declared {
                println!(
                    "[risk-check PASS] Declared risk ({}) meets or exceeds path-derived minimum ({}).\n",
                    d.as_str(),
                    derived.as_str()
                );
            } else {
                println!(
                    "[risk-check PASS] Non-PR context with no declaration: path-derived minimum is {}.\n",
                    derived.as_str()
                );
            }
            Ok(())
        }
        RiskDecision::Fail(reason) => Err(reason),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_class_ordering() {
        assert!(RiskClass::R0 < RiskClass::R1);
        assert!(RiskClass::R1 < RiskClass::R2);
        assert!(RiskClass::R2 < RiskClass::R3);
        assert!(RiskClass::R3 < RiskClass::R4);
    }

    #[test]
    fn test_risk_class_parsing() {
        assert_eq!(RiskClass::parse_opt("R0"), Some(RiskClass::R0));
        assert_eq!(RiskClass::parse_opt("r4"), Some(RiskClass::R4));
        assert_eq!(RiskClass::parse_opt(" R2 "), Some(RiskClass::R2));
        assert_eq!(RiskClass::parse_opt("R5"), None);
        assert_eq!(RiskClass::parse_opt(""), None);
        assert_eq!(RiskClass::parse_opt("rx"), None);
    }

    #[test]
    fn test_path_to_risk_mappings() {
        // R0: ordinary documentation and non-runtime metadata
        assert_eq!(path_to_risk("README.md"), RiskClass::R0);
        assert_eq!(
            path_to_risk("docs/engineering/git-workflow.md"),
            RiskClass::R0
        );

        // R1: isolated implementation and tooling
        assert_eq!(
            path_to_risk("crates/gatewaymux-cli/src/main.rs"),
            RiskClass::R1
        );
        assert_eq!(path_to_risk("dashboard/package.json"), RiskClass::R1);
        assert_eq!(path_to_risk("tests/integration/test.rs"), RiskClass::R1);
        assert_eq!(path_to_risk(".github/CODEOWNERS"), RiskClass::R1);
        assert_eq!(path_to_risk(".github/dependabot.yml"), RiskClass::R1);

        // R2: provider & protocol behavior
        assert_eq!(
            path_to_risk("crates/gatewaymux-providers/src/lib.rs"),
            RiskClass::R2
        );
        assert_eq!(
            path_to_risk("crates/gatewaymux-protocols/src/lib.rs"),
            RiskClass::R2
        );
        assert_eq!(path_to_risk("compat/codex/sample.json"), RiskClass::R2);
        assert_eq!(path_to_risk("profiles/models/test.json"), RiskClass::R2);

        // R3: core routing, server, persistence, configuration
        assert_eq!(
            path_to_risk("crates/gatewaymux-routing/src/lib.rs"),
            RiskClass::R3
        );
        assert_eq!(
            path_to_risk("crates/gatewaymux-server/src/lib.rs"),
            RiskClass::R3
        );
        assert_eq!(
            path_to_risk("crates/gatewaymux-core/src/lib.rs"),
            RiskClass::R3
        );
        assert_eq!(path_to_risk("config/examples/test.toml"), RiskClass::R3);
        assert_eq!(path_to_risk("Cargo.toml"), RiskClass::R3);

        // R4: critical trust / supply-chain / sync boundary
        assert_eq!(
            path_to_risk("crates/gatewaymux-codex-bridge/src/lib.rs"),
            RiskClass::R4
        );
        assert_eq!(
            path_to_risk("crates/gatewaymux-sync/src/lib.rs"),
            RiskClass::R4
        );
        assert_eq!(path_to_risk("docs/security/threat-model.md"), RiskClass::R4);
        assert_eq!(path_to_risk("deny.toml"), RiskClass::R4);
    }

    #[test]
    fn test_workflows_are_r4() {
        // CI workflows are part of the repository security and
        // software-supply-chain boundary: minimum R4.
        assert_eq!(path_to_risk(".github/workflows/ci.yml"), RiskClass::R4);
        assert_eq!(
            path_to_risk(".github/workflows/owner-approval.yml"),
            RiskClass::R4
        );
        assert_eq!(
            path_to_risk(".github/workflows/security.yml"),
            RiskClass::R4
        );
    }

    #[test]
    fn test_sync_server_is_r4() {
        assert_eq!(path_to_risk("sync-server/Cargo.toml"), RiskClass::R4);
        assert_eq!(path_to_risk("sync-server/src/main.rs"), RiskClass::R4);
        assert_eq!(path_to_risk("sync-server/README.md"), RiskClass::R4);
    }

    #[test]
    fn test_cross_platform_path_normalization() {
        assert_eq!(
            path_to_risk("crates\\gatewaymux-sync\\src\\lib.rs"),
            RiskClass::R4
        );
        assert_eq!(
            path_to_risk("crates\\gatewaymux-routing\\src\\lib.rs"),
            RiskClass::R3
        );
        assert_eq!(path_to_risk(".github\\workflows\\ci.yml"), RiskClass::R4);
        assert_eq!(
            path_to_risk("docs\\engineering\\risk-classification.md"),
            RiskClass::R0
        );
    }

    #[test]
    fn test_extract_declared_risk() {
        let body = r#"
## Summary
Some change

## Declared Risk Class
- [ ] **R0**: Docs
- [ ] **R1**: Isolated
- [x] **R2**: Provider behavior
- [ ] **R3**: Routing
- [ ] **R4**: Security
"#;
        assert_eq!(extract_declared_risk_from_body(body), Some(RiskClass::R2));

        let body_caps = r#"
- [X] **R4**: Critical Trust
"#;
        assert_eq!(
            extract_declared_risk_from_body(body_caps),
            Some(RiskClass::R4)
        );

        let no_declaration = "No checkboxes here.";
        assert_eq!(extract_declared_risk_from_body(no_declaration), None);
    }

    #[test]
    fn test_dependabot_identity_is_exact() {
        assert!(is_trusted_dependabot("dependabot[bot]"));
        assert!(is_trusted_dependabot("Dependabot[bot]"));
        assert!(!is_trusted_dependabot("renovate[bot]"));
        assert!(!is_trusted_dependabot("dependabot"));
        assert!(!is_trusted_dependabot("dependabot-preview[bot]"));
        assert!(!is_trusted_dependabot("github-actions[bot]"));
        assert!(!is_trusted_dependabot("someuser[bot]"));
        assert!(!is_trusted_dependabot(""));
    }

    fn workflow_paths() -> Vec<String> {
        vec![".github/workflows/ci.yml".to_string()]
    }

    #[test]
    fn test_human_pr_missing_declaration_fails() {
        let decision = evaluate_risk_decision(None, &workflow_paths(), Some("some-contributor"));
        assert!(matches!(decision, RiskDecision::Fail(_)));
        if let RiskDecision::Fail(reason) = decision {
            assert!(
                reason.contains("MUST declare"),
                "failure reason must explain the missing declaration: {reason}"
            );
        }
    }

    #[test]
    fn test_human_pr_under_declaration_fails() {
        let decision =
            evaluate_risk_decision(Some(RiskClass::R1), &workflow_paths(), Some("agent"));
        assert!(matches!(decision, RiskDecision::Fail(_)));
        if let RiskDecision::Fail(reason) = decision {
            assert!(reason.contains("lower than the path-derived minimum"));
        }
    }

    #[test]
    fn test_human_declaration_equal_or_higher_passes() {
        let equal =
            evaluate_risk_decision(Some(RiskClass::R4), &workflow_paths(), Some("ArchdukeViel"));
        assert_eq!(
            equal,
            RiskDecision::Pass {
                declared: Some(RiskClass::R4),
                derived: RiskClass::R4,
                dependabot_exempt: false
            }
        );

        let elevated = evaluate_risk_decision(
            Some(RiskClass::R4),
            &["docs/README.md".to_string()],
            Some("ArchdukeViel"),
        );
        assert_eq!(
            elevated,
            RiskDecision::Pass {
                declared: Some(RiskClass::R4),
                derived: RiskClass::R0,
                dependabot_exempt: false
            }
        );
    }

    #[test]
    fn test_dependabot_exemption_requires_no_declaration() {
        let decision = evaluate_risk_decision(None, &workflow_paths(), Some("dependabot[bot]"));
        assert_eq!(
            decision,
            RiskDecision::Pass {
                declared: None,
                derived: RiskClass::R4,
                dependabot_exempt: true
            }
        );
    }

    #[test]
    fn test_dependabot_exemption_ignores_declared_value() {
        // Path-derived risk is authoritative for Dependabot PRs; a stray
        // declared value (higher or lower) never overrides the derived
        // minimum.
        let lower = evaluate_risk_decision(
            Some(RiskClass::R0),
            &workflow_paths(),
            Some("dependabot[bot]"),
        );
        assert!(matches!(
            lower,
            RiskDecision::Pass {
                dependabot_exempt: true,
                ..
            }
        ));

        let higher = evaluate_risk_decision(
            Some(RiskClass::R4),
            &workflow_paths(),
            Some("dependabot[bot]"),
        );
        assert!(matches!(
            higher,
            RiskDecision::Pass {
                dependabot_exempt: true,
                ..
            }
        ));
    }

    #[test]
    fn test_dependabot_empty_change_set_fails() {
        let decision = evaluate_risk_decision(None, &[], Some("dependabot[bot]"));
        assert!(matches!(decision, RiskDecision::Fail(_)));
    }

    #[test]
    fn test_non_pr_context_is_informational_without_declaration() {
        let decision = evaluate_risk_decision(None, &workflow_paths(), None);
        assert!(matches!(
            decision,
            RiskDecision::Pass {
                dependabot_exempt: false,
                ..
            }
        ));
    }

    #[test]
    fn test_non_pr_context_under_declaration_still_fails() {
        let decision = evaluate_risk_decision(Some(RiskClass::R2), &workflow_paths(), None);
        assert!(matches!(decision, RiskDecision::Fail(_)));
    }

    #[test]
    fn test_derive_minimum_risk_takes_maximum() {
        let paths = vec![
            "docs/README.md".to_string(),
            "crates/gatewaymux-providers/src/lib.rs".to_string(),
            ".github/workflows/ci.yml".to_string(),
        ];
        assert_eq!(derive_minimum_risk(&paths), RiskClass::R4);
        assert_eq!(derive_minimum_risk(&[]), RiskClass::R0);
    }

    #[test]
    fn test_bot_lookalike_actors_get_no_exemption() {
        // PR body text or a lookalike login must never grant the
        // Dependabot exemption.
        for actor in ["dependabot", "dependabot-bot", "fake[bot]", "Dependabot "] {
            let decision = evaluate_risk_decision(None, &workflow_paths(), Some(actor));
            assert!(
                matches!(decision, RiskDecision::Fail(_)),
                "actor '{actor}' must not receive the Dependabot exemption"
            );
        }
    }
}
