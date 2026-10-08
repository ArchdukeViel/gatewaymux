//! Change-risk classification and PR risk-declaration enforcement.
//!
//! Every human- or agent-authored pull request MUST declare EXACTLY ONE
//! R0-R4 risk class; CI rejects missing, ambiguous (multiple), and
//! under-declarations. Trusted GitHub Dependabot pull requests are
//! exempt from manual declaration and use a path-derived risk class
//! automatically. Dependabot identity is accepted only from
//! CI-provided event metadata (e.g. the `PR_AUTHOR` environment
//! variable set from the `pull_request` event payload), never from
//! pull request body text.

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

/// The risk-declaration outcome parsed from a pull request body that
/// follows the PR template checkbox format.
///
/// A checked checkbox line contributes a selection for each tier whose
/// BOLD label (`**Rn**`, case-insensitive) appears on the line. Only
/// the bold tier labels of the PR template's `Declared Risk Class`
/// section match this form, so unrelated occurrences of `R0`..`R4` in
/// prose (or in other checkbox lists) never become declarations.
#[derive(Debug, PartialEq, Eq)]
pub enum RiskDeclaration {
    /// No checked box carries a bold tier label.
    Undeclared,
    /// Exactly one tier was selected across all checked boxes.
    Single(RiskClass),
    /// Two or more tiers were selected across checked boxes; the
    /// declaration is ambiguous and fails risk governance.
    Ambiguous(Vec<RiskClass>),
}

/// Parse the risk-declaration outcome out of a pull request body that
/// follows the PR template checkbox format.
///
/// Policy: zero selected boxes means no declaration; exactly one
/// selected box is the declaration; two or more selected boxes are
/// ambiguous and rejected. The first checked box never silently wins.
pub fn parse_risk_declaration(body: &str) -> RiskDeclaration {
    let mut selected: Vec<RiskClass> = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim_start();
        let lower = trimmed.to_lowercase();
        if !lower.starts_with("- [x]") {
            continue;
        }
        for tier in [
            RiskClass::R0,
            RiskClass::R1,
            RiskClass::R2,
            RiskClass::R3,
            RiskClass::R4,
        ] {
            let marker = format!("**{}**", tier.as_str().to_lowercase());
            if lower.contains(&marker) {
                selected.push(tier);
            }
        }
    }

    match selected.len() {
        0 => RiskDeclaration::Undeclared,
        1 => RiskDeclaration::Single(selected[0]),
        _ => RiskDeclaration::Ambiguous(selected),
    }
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
/// - Human/agent PRs: EXACTLY ONE declaration is mandatory. Zero selected
///   tiers, multiple selected tiers, or a declaration lower than the
///   path-derived minimum all fail.
/// - Non-PR contexts (local runs, push events): a declaration is optional
///   and informational, but an explicit under-declaration or an ambiguous
///   (multiple) declaration still fails.
pub fn evaluate_risk_decision(
    declaration: RiskDeclaration,
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

        return match declaration {
            RiskDeclaration::Single(d) if d >= derived => RiskDecision::Pass {
                declared: Some(d),
                derived,
                dependabot_exempt: false,
            },
            RiskDeclaration::Single(d) => RiskDecision::Fail(format!(
                "Declared risk class {} is lower than the path-derived minimum {}. Contributors may elevate risk, but never lower it.",
                d.as_str(),
                derived.as_str()
            )),
            RiskDeclaration::Undeclared => RiskDecision::Fail(format!(
                "No declared risk class found. Every human- or agent-authored pull request MUST declare exactly one R0-R4 risk class in the PR template; the path-derived minimum for this change set is {}.",
                derived.as_str()
            )),
            RiskDeclaration::Ambiguous(tiers) => RiskDecision::Fail(format!(
                "Multiple risk classes declared ({}). Exactly one R0-R4 declaration is required: zero or multiple selected declarations both fail risk classification. The path-derived minimum for this change set is {}.",
                tiers
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                derived.as_str()
            )),
        };
    }

    match declaration {
        RiskDeclaration::Single(d) if d >= derived => RiskDecision::Pass {
            declared: Some(d),
            derived,
            dependabot_exempt: false,
        },
        RiskDeclaration::Single(d) => RiskDecision::Fail(format!(
            "Declared risk class {} is lower than the path-derived minimum {}. Contributors may elevate risk, but never lower it.",
            d.as_str(),
            derived.as_str()
        )),
        RiskDeclaration::Undeclared => RiskDecision::Pass {
            declared: None,
            derived,
            dependabot_exempt: false,
        },
        RiskDeclaration::Ambiguous(tiers) => RiskDecision::Fail(format!(
            "Multiple risk classes declared ({}). Exactly one R0-R4 declaration is required: zero or multiple selected declarations both fail risk classification. The path-derived minimum for this change set is {}.",
            tiers
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            derived.as_str()
        )),
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

    let mut declaration = RiskDeclaration::Undeclared;
    let mut base_ref: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--risk" && i + 1 < args.len() {
            if let Some(r) = RiskClass::parse_opt(&args[i + 1]) {
                declaration = RiskDeclaration::Single(r);
            }
            i += 2;
        } else if args[i] == "--base" && i + 1 < args.len() {
            base_ref = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    // If not supplied via CLI flag, check environment variables
    if matches!(declaration, RiskDeclaration::Undeclared) {
        if let Ok(env_risk) = env::var("DECLARED_RISK") {
            if let Some(r) = RiskClass::parse_opt(&env_risk) {
                declaration = RiskDeclaration::Single(r);
            }
        }
    }
    if matches!(declaration, RiskDeclaration::Undeclared) {
        if let Ok(pr_body) = env::var("PR_BODY") {
            declaration = parse_risk_declaration(&pr_body);
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
    match &declaration {
        RiskDeclaration::Single(d) => println!("Declared risk class:             {}", d.as_str()),
        RiskDeclaration::Undeclared => match &pr_actor {
            Some(a) if is_trusted_dependabot(a) => {
                println!(
                    "Declared risk class:             (none; trusted Dependabot exemption applies)"
                );
            }
            Some(a) => {
                println!(
                    "Declared risk class:             (none; PR actor '{a}' requires declaration)"
                );
            }
            None => println!("Declared risk class:             (none; non-PR context)"),
        },
        RiskDeclaration::Ambiguous(tiers) => println!(
            "Declared risk class:             AMBIGUOUS ({})",
            tiers
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }

    let decision = evaluate_risk_decision(declaration, &modified_paths, pr_actor.as_deref());

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
    fn test_parse_risk_declaration_single_selection() {
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
        assert_eq!(
            parse_risk_declaration(body),
            RiskDeclaration::Single(RiskClass::R2)
        );

        let body_caps = r#"
- [X] **R4**: Critical Trust
"#;
        assert_eq!(
            parse_risk_declaration(body_caps),
            RiskDeclaration::Single(RiskClass::R4)
        );
    }

    #[test]
    fn test_parse_risk_declaration_one_r0_passes_for_docs_paths() {
        let body = "- [x] **R0**: Docs / comments / non-runtime fixtures";
        assert_eq!(
            parse_risk_declaration(body),
            RiskDeclaration::Single(RiskClass::R0)
        );
        let decision = evaluate_risk_decision(
            parse_risk_declaration(body),
            &["docs/engineering/git-workflow.md".to_string()],
            Some("contributor"),
        );
        assert_eq!(
            decision,
            RiskDecision::Pass {
                declared: Some(RiskClass::R0),
                derived: RiskClass::R0,
                dependabot_exempt: false
            }
        );
    }

    #[test]
    fn test_parse_risk_declaration_lowercase_and_format_edges() {
        // Lowercase bold tier label.
        assert_eq!(
            parse_risk_declaration("- [x] **r3**: routing change"),
            RiskDeclaration::Single(RiskClass::R3)
        );
        // Uppercase checkbox marker.
        assert_eq!(
            parse_risk_declaration("- [X] **r1**: isolated"),
            RiskDeclaration::Single(RiskClass::R1)
        );
        // Indented checkbox lines still parse.
        assert_eq!(
            parse_risk_declaration("   - [x] **R2**: provider"),
            RiskDeclaration::Single(RiskClass::R2)
        );
        // Unchecked boxes never declare anything.
        assert_eq!(
            parse_risk_declaration("- [ ] **R4**\n- [ ] **R1**"),
            RiskDeclaration::Undeclared
        );
    }

    #[test]
    fn test_parse_risk_declaration_prose_does_not_become_declaration() {
        // Random body prose containing bare tier tokens: no checkbox, no
        // bold marker -> never a declaration.
        let prose = r#"
## Summary
This fixes the R4 boundary issue. We considered R3 and even R0 paths.
The `.github/workflows/**` area is R4 by policy.
"#;
        assert_eq!(parse_risk_declaration(prose), RiskDeclaration::Undeclared);
        let decision = evaluate_risk_decision(
            parse_risk_declaration(prose),
            &["docs/README.md".to_string()],
            Some("contributor"),
        );
        assert!(matches!(decision, RiskDecision::Fail(_)));
        if let RiskDecision::Fail(reason) = decision {
            assert!(
                reason.contains("exactly one"),
                "failure must explain the exactly-one rule: {reason}"
            );
        }

        // A checked box whose prose merely mentions a tier (not bold)
        // is also not a declaration: other template checkbox lists
        // (e.g. Affected Contracts) may legitimately mention R4.
        let checked_prose = r#"
## Affected Contracts & Areas
- [x] `.github/workflows/**` (R4 supply-chain boundary)
"#;
        assert_eq!(
            parse_risk_declaration(checked_prose),
            RiskDeclaration::Undeclared
        );

        // No checkboxes at all.
        assert_eq!(
            parse_risk_declaration("No checkboxes here."),
            RiskDeclaration::Undeclared
        );
    }

    #[test]
    fn test_parse_risk_declaration_two_selected_boxes_is_ambiguous() {
        let body = r#"
- [x] **R2**: Provider behavior
- [x] **R4**: Security
"#;
        assert_eq!(
            parse_risk_declaration(body),
            RiskDeclaration::Ambiguous(vec![RiskClass::R2, RiskClass::R4])
        );
        let decision = evaluate_risk_decision(
            parse_risk_declaration(body),
            &workflow_paths(),
            Some("agent"),
        );
        assert!(matches!(decision, RiskDecision::Fail(_)));
        if let RiskDecision::Fail(reason) = decision {
            assert!(
                reason.contains("Multiple risk classes declared (R2, R4)"),
                "failure must name the ambiguous selections: {reason}"
            );
        }
    }

    #[test]
    fn test_parse_risk_declaration_all_selected_boxes_is_ambiguous() {
        let body = r#"
- [x] **R0**: Docs
- [x] **R1**: Isolated
- [x] **R2**: Provider behavior
- [x] **R3**: Routing
- [x] **R4**: Security
"#;
        assert_eq!(
            parse_risk_declaration(body),
            RiskDeclaration::Ambiguous(vec![
                RiskClass::R0,
                RiskClass::R1,
                RiskClass::R2,
                RiskClass::R3,
                RiskClass::R4,
            ])
        );
        let decision = evaluate_risk_decision(
            parse_risk_declaration(body),
            &["docs/README.md".to_string()],
            Some("contributor"),
        );
        assert!(matches!(decision, RiskDecision::Fail(_)));
    }

    #[test]
    fn test_parse_risk_declaration_multiple_bold_tiers_on_one_line_is_ambiguous() {
        // Two bold tier labels on a single checked line count as two
        // selections: the declaration is ambiguous, never "first wins".
        let body = "- [x] **R2** and also **R4** somewhere else";
        assert_eq!(
            parse_risk_declaration(body),
            RiskDeclaration::Ambiguous(vec![RiskClass::R2, RiskClass::R4])
        );
    }

    #[test]
    fn test_parse_risk_declaration_first_checked_box_never_wins() {
        // The historical defect: "first checked box wins" silently
        // accepted multi-selected bodies. It must fail instead.
        let body = r#"
- [x] **R0**: Docs
- [x] **R4**: Security
"#;
        let decision = evaluate_risk_decision(
            parse_risk_declaration(body),
            &[".github/workflows/ci.yml".to_string()],
            Some("agent"),
        );
        assert!(matches!(decision, RiskDecision::Fail(_)));
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
        let decision = evaluate_risk_decision(
            RiskDeclaration::Undeclared,
            &workflow_paths(),
            Some("some-contributor"),
        );
        assert!(matches!(decision, RiskDecision::Fail(_)));
        if let RiskDecision::Fail(reason) = decision {
            assert!(
                reason.contains("exactly one"),
                "failure must explain the exactly-one declaration rule: {reason}"
            );
        }
    }

    #[test]
    fn test_human_pr_multiple_declaration_fails() {
        let decision = evaluate_risk_decision(
            RiskDeclaration::Ambiguous(vec![RiskClass::R1, RiskClass::R4]),
            &workflow_paths(),
            Some("some-contributor"),
        );
        assert!(matches!(decision, RiskDecision::Fail(_)));
        if let RiskDecision::Fail(reason) = decision {
            assert!(reason.contains("Multiple risk classes declared"));
        }
    }

    #[test]
    fn test_human_pr_under_declaration_fails() {
        let decision = evaluate_risk_decision(
            RiskDeclaration::Single(RiskClass::R1),
            &workflow_paths(),
            Some("agent"),
        );
        assert!(matches!(decision, RiskDecision::Fail(_)));
        if let RiskDecision::Fail(reason) = decision {
            assert!(reason.contains("lower than the path-derived minimum"));
        }
    }

    #[test]
    fn test_human_declaration_equal_or_higher_passes() {
        let equal = evaluate_risk_decision(
            RiskDeclaration::Single(RiskClass::R4),
            &workflow_paths(),
            Some("ArchdukeViel"),
        );
        assert_eq!(
            equal,
            RiskDecision::Pass {
                declared: Some(RiskClass::R4),
                derived: RiskClass::R4,
                dependabot_exempt: false
            }
        );

        let elevated = evaluate_risk_decision(
            RiskDeclaration::Single(RiskClass::R4),
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
        let decision = evaluate_risk_decision(
            RiskDeclaration::Undeclared,
            &workflow_paths(),
            Some("dependabot[bot]"),
        );
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
            RiskDeclaration::Single(RiskClass::R0),
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
            RiskDeclaration::Single(RiskClass::R4),
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

        // Even an ambiguous body declaration never overrides the
        // Dependabot path-derived exemption.
        let ambiguous = evaluate_risk_decision(
            RiskDeclaration::Ambiguous(vec![RiskClass::R0, RiskClass::R4]),
            &workflow_paths(),
            Some("dependabot[bot]"),
        );
        assert!(matches!(
            ambiguous,
            RiskDecision::Pass {
                dependabot_exempt: true,
                ..
            }
        ));
    }

    #[test]
    fn test_dependabot_empty_change_set_fails() {
        let decision =
            evaluate_risk_decision(RiskDeclaration::Undeclared, &[], Some("dependabot[bot]"));
        assert!(matches!(decision, RiskDecision::Fail(_)));
    }

    #[test]
    fn test_non_pr_context_is_informational_without_declaration() {
        let decision = evaluate_risk_decision(RiskDeclaration::Undeclared, &workflow_paths(), None);
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
        let decision = evaluate_risk_decision(
            RiskDeclaration::Single(RiskClass::R2),
            &workflow_paths(),
            None,
        );
        assert!(matches!(decision, RiskDecision::Fail(_)));
    }

    #[test]
    fn test_non_pr_context_multiple_declaration_still_fails() {
        let decision = evaluate_risk_decision(
            RiskDeclaration::Ambiguous(vec![RiskClass::R3, RiskClass::R4]),
            &workflow_paths(),
            None,
        );
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
            let decision =
                evaluate_risk_decision(RiskDeclaration::Undeclared, &workflow_paths(), Some(actor));
            assert!(
                matches!(decision, RiskDecision::Fail(_)),
                "actor '{actor}' must not receive the Dependabot exemption"
            );
        }
    }
}
