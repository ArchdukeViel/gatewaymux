//! GatewayMux xtask developer automation and governance enforcement.
//!
//! Provides canonical verification commands:
//! - cargo xtask check
//! - cargo xtask test [--risk R0|R1|R2|R3|R4]
//! - cargo xtask architecture-check
//! - cargo xtask repo-check
//! - cargo xtask generate [--check]
//! - cargo xtask compat
//! - cargo xtask provider-check <provider>
//! - cargo xtask risk-check [--risk R0|R1|R2|R3|R4] [--base <ref>]
//! - cargo xtask release-check

mod architecture;
mod generate;
mod risk;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use risk::RiskClass;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = args[1].as_str();
    let subargs = &args[2..];

    let result = match command {
        "check" => cmd_check(subargs),
        "test" => cmd_test(subargs),
        "architecture-check" => {
            get_repo_root().and_then(|root| architecture::cmd_architecture_check(&root))
        }
        "repo-check" => cmd_repo_check(subargs),
        "generate" => get_repo_root().and_then(|root| generate::cmd_generate(subargs, &root)),
        "compat" => cmd_compat(subargs),
        "provider-check" => cmd_provider_check(subargs),
        "risk-check" => get_repo_root().and_then(|root| risk::cmd_risk_check(subargs, &root)),
        "release-check" => cmd_release_check(subargs),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        unknown => {
            eprintln!("Unknown xtask command: {unknown}\n");
            print_usage();
            Err(format!("Unknown command: {unknown}"))
        }
    };

    if let Err(err) = result {
        eprintln!("\n[xtask ERROR] {err}");
        std::process::exit(1);
    }
}

fn print_usage() {
    println!(
        r#"GatewayMux Workspace Task Runner (cargo xtask)

Available Commands:
  check                                  Run full workspace validation (repo, arch, clippy, check, fmt, npm)
  test [--risk R0..R4]                   Run workspace tests with risk-level scope and test harness status
  architecture-check                     Enforce workspace directional dependency rules across all crates
  repo-check                             Enforce root hygiene, structure, workflow action pins, and toolchain authority
  generate [--check]                     Regenerate-and-compare tracked generated files (--check never writes)
  compat                                 Validate Compatibility Corpus fixtures and sanitization
  provider-check <provider>              Evaluate provider conformance, lifecycle state, and checklist
  risk-check [--risk <R>] [--base <ref>] Derive path minimum risk class and verify against declared risk
  release-check                          Validate release readiness, version alignment, and docs
"#
    );
}

fn get_repo_root() -> Result<PathBuf, String> {
    let cwd = env::current_dir().map_err(|e| format!("Failed to get current dir: {e}"))?;
    if cwd.join("Cargo.toml").exists() && cwd.join("crates").exists() {
        return Ok(cwd);
    }
    // Check parent if executed from within crates/xtask
    if let Some(parent) = cwd.parent() {
        if parent.join("Cargo.toml").exists() && parent.join("crates").exists() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(cwd)
}

pub(crate) fn resolve_program(program: &str) -> String {
    if cfg!(windows) && program == "npm" {
        "npm.cmd".to_string()
    } else {
        program.to_string()
    }
}

pub(crate) fn run_cmd(program: &str, args: &[&str], cwd: &Path) -> Result<ExitStatus, String> {
    let resolved = resolve_program(program);
    println!("> {} {}", program, args.join(" "));
    let status = Command::new(&resolved)
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|e| format!("Failed to execute '{program}': {e}"))?;
    Ok(status)
}

pub(crate) fn run_cmd_output(program: &str, args: &[&str], cwd: &Path) -> Result<String, String> {
    let resolved = resolve_program(program);
    let output = Command::new(&resolved)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Failed to execute '{program}': {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Command '{program} {}' failed: {stderr}",
            args.join(" ")
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ---------------------------------------------------------------------------
// 1. REPO-CHECK: Root Hygiene and Directory Structure
// ---------------------------------------------------------------------------
fn cmd_repo_check(_args: &[String]) -> Result<(), String> {
    println!("=== Running Repository Root Hygiene Check ===");
    let root = get_repo_root()?;

    let allowed_root_files: HashSet<&str> = [
        ".editorconfig",
        ".gitattributes",
        ".gitignore",
        ".generated-manifest.json",
        "AGENTS.md",
        "Cargo.lock",
        "Cargo.toml",
        "CHANGELOG.md",
        "CONTRIBUTING.md",
        "deny.toml",
        "LICENSE",
        "package-lock.json",
        "package.json",
        "README.md",
        "rust-toolchain.toml",
        "SECURITY.md",
    ]
    .into_iter()
    .collect();

    let allowed_root_dirs: HashSet<&str> = [
        ".agents",
        ".cargo",
        ".git",
        ".github",
        "compat",
        "config",
        "crates",
        "dashboard",
        "docs",
        "packaging",
        "profiles",
        "scripts",
        "sync-server",
        "tests",
        "target",
        "node_modules",
    ]
    .into_iter()
    .collect();

    let required_dirs = [
        "crates",
        "crates/gatewaymux-core",
        "crates/gatewaymux-routing",
        "crates/gatewaymux-protocols",
        "crates/gatewaymux-providers",
        "crates/gatewaymux-server",
        "crates/gatewaymux-codex-bridge",
        "crates/gatewaymux-sync",
        "crates/gatewaymux-cli",
        "crates/gatewaymux-app",
        "crates/xtask",
        "dashboard",
        "sync-server",
        "compat/codex",
        "compat/providers",
        "compat/protocols",
        "compat/transports",
        "compat/operations",
        "profiles/providers",
        "profiles/models",
        "profiles/schema-dialects",
        "profiles/transports",
        "tests/integration",
        "tests/e2e",
        "tests/fault",
        "tests/security",
        "tests/fixtures",
        "config/examples",
        "config/schemas",
        "docs/architecture/adr",
        "docs/engineering",
        "packaging/windows",
        "packaging/npm",
        "scripts",
        ".agents/rules",
        ".github/workflows",
        ".github/ISSUE_TEMPLATE",
    ];

    let mut violations = Vec::new();

    // Verify root entries against strict allowlist
    for entry in fs::read_dir(&root).map_err(|e| format!("Failed to read root dir: {e}"))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to get file type: {e}"))?;

        if file_type.is_dir() {
            if !allowed_root_dirs.contains(name.as_ref()) {
                violations.push(format!("Disallowed directory in root: {name}"));
            }
        } else if !allowed_root_files.contains(name.as_ref()) {
            violations.push(format!("Disallowed file in root: {name}"));
        }
    }

    // Verify required directories exist
    for req in required_dirs {
        let p = root.join(req);
        if !p.exists() || !p.is_dir() {
            violations.push(format!("Missing required repository directory: {req}"));
        }
    }

    // Verify CI toolchain authority: rust-toolchain.toml is the single
    // compiler source of truth; workflows must not declare their own.
    if let Err(err) = verify_ci_toolchain_pins(&root) {
        violations.push(err);
    }

    // Verify workflow actions are pinned to immutable commit SHAs.
    if let Err(err) = verify_workflow_action_pins(&root) {
        violations.push(err);
    }

    if !violations.is_empty() {
        eprintln!(
            "\nRepository hygiene violations found ({}):",
            violations.len()
        );
        for v in violations {
            eprintln!("  - {v}");
        }
        return Err("Repository root hygiene check failed.".to_string());
    }

    println!("[repo-check PASS] Repository root hygiene and structural layout verified.\n");
    Ok(())
}

// ---------------------------------------------------------------------------
// 1b. WORKFLOW TOOLCHAIN AUTHORITY & ACTION-PIN STATIC CHECKS
// ---------------------------------------------------------------------------
/// Parse the `channel` and `components` declared in rust-toolchain.toml,
/// the single authoritative Rust compiler declaration.
fn read_rust_toolchain(root: &Path) -> Result<(String, Vec<String>), String> {
    let path = root.join("rust-toolchain.toml");
    let content = fs::read_to_string(&path).map_err(|e| {
        format!(
            "Missing or unreadable rust-toolchain.toml (the single Rust compiler authority): {e}"
        )
    })?;

    let mut channel: Option<String> = None;
    let mut components: Vec<String> = Vec::new();

    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t
            .strip_prefix("channel")
            .and_then(|r| r.trim_start().strip_prefix('='))
        {
            let value = rest.trim().trim_matches('"');
            if !value.is_empty() {
                channel = Some(value.to_string());
            }
        } else if let Some(rest) = t
            .strip_prefix("components")
            .and_then(|r| r.trim_start().strip_prefix('='))
        {
            let inner = rest.trim().trim_start_matches('[').trim_end_matches(']');
            components = inner
                .split(',')
                .map(|c| c.trim().trim_matches('"').to_string())
                .filter(|c| !c.is_empty())
                .collect();
        }
    }

    let channel = channel.ok_or_else(|| {
        "rust-toolchain.toml must declare channel = \"<version>\"; it is the single authoritative Rust compiler version for CI".to_string()
    })?;
    Ok((channel, components))
}

/// List every GitHub Actions workflow file under `.github/workflows`.
fn workflow_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = root.join(".github").join("workflows");
    if !dir.is_dir() {
        return Err("Missing required repository directory: .github/workflows".to_string());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("Failed to read .github/workflows: {e}"))? {
        let entry = entry.map_err(|e| format!("Failed to read workflow entry: {e}"))?;
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("yml") || e.eq_ignore_ascii_case("yaml"))
                .unwrap_or(false)
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Enforce that CI never declares an independent Rust toolchain pin:
/// `rust-toolchain.toml` is the single source of truth, so no workflow
/// may pass a `toolchain:` input (for example `toolchain: stable`) that
/// could drift from the repository pin. CI must install exactly the
/// channel declared in the file. The file must also declare the
/// rustfmt and clippy components CI requires.
pub(crate) fn verify_ci_toolchain_pins(root: &Path) -> Result<(), String> {
    let (channel, components) = read_rust_toolchain(root)?;

    for component in ["rustfmt", "clippy"] {
        if !components.iter().any(|c| c == component) {
            return Err(format!(
                "rust-toolchain.toml must declare the '{component}' component: CI requires rustfmt and clippy from the same authoritative toolchain file (channel = \"{channel}\")"
            ));
        }
    }

    for path in workflow_files(root)? {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("toolchain:") {
                return Err(format!(
                    "{}:{}: CI workflows must not declare an independent '{}' input. rust-toolchain.toml (channel = \"{}\") is the single authoritative Rust compiler version; never create a second version constant such as 'toolchain: stable'.",
                    path.display(),
                    idx + 1,
                    trimmed,
                    channel
                ));
            }
        }
    }
    Ok(())
}

/// Enforce immutable full-length commit SHA pinning for every action
/// referenced from `.github/workflows/**`. Floating tags (`@v4`,
/// `@stable`, `@master`) and unpinned references are rejected: CI
/// workflows are part of the repository security and software
/// supply-chain boundary.
pub(crate) fn verify_workflow_action_pins(root: &Path) -> Result<(), String> {
    for path in workflow_files(root)? {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        for (idx, line) in content.lines().enumerate() {
            let mut trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("- ") {
                trimmed = rest.trim();
            }
            let Some(spec) = trimmed.strip_prefix("uses:") else {
                continue;
            };
            let reference = spec.split_whitespace().next().unwrap_or("");
            if reference.is_empty() {
                return Err(format!(
                    "{}:{}: 'uses:' must reference an action pinned by a full-length commit SHA; found an empty reference.",
                    path.display(),
                    idx + 1
                ));
            }
            let Some((_, action_ref)) = reference.rsplit_once('@') else {
                return Err(format!(
                    "{}:{}: action '{reference}' is not pinned to a commit SHA. Pin every workflow action to an immutable full-length commit SHA (for example 'owner/repo@<40-hex-sha> # vX.Y.Z').",
                    path.display(),
                    idx + 1
                ));
            };
            let sha_pinned =
                action_ref.len() == 40 && action_ref.chars().all(|c| c.is_ascii_hexdigit());
            if !sha_pinned {
                return Err(format!(
                    "{}:{}: action '{reference}' must be pinned to an immutable full-length commit SHA; the floating reference '{action_ref}' is prohibited.",
                    path.display(),
                    idx + 1
                ));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. COMPAT: Compatibility Corpus Manifest and Sanitization
// ---------------------------------------------------------------------------
fn cmd_compat(_args: &[String]) -> Result<(), String> {
    println!("=== Running Compatibility Corpus Verification ===");
    let root = get_repo_root()?;
    let manifest_path = root.join("compat/corpus-manifest.json");

    if !manifest_path.exists() {
        return Err("Missing compat/corpus-manifest.json".to_string());
    }

    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read corpus manifest: {e}"))?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Invalid JSON in corpus manifest: {e}"))?;

    let rev = manifest
        .get("revision")
        .and_then(|r| r.as_i64())
        .unwrap_or(0);
    println!("Compatibility Corpus revision: {rev}");

    // Verify required compat directories exist
    let subdirs = [
        "codex",
        "providers",
        "protocols",
        "transports",
        "operations",
    ];
    for sub in subdirs {
        let dir_path = root.join("compat").join(sub);
        if !dir_path.exists() {
            return Err(format!("Missing compat subdirectory: compat/{sub}"));
        }
    }

    // Sanitization check: Ensure no unmasked secrets or auth tokens in fixtures
    let compat_root = root.join("compat");
    let mut files_checked = 0;
    scan_compat_dir(&compat_root, &mut files_checked)?;

    println!("Scanned {files_checked} files under compat/ - no unredacted credentials found.");
    println!("[compat PASS] Compatibility Corpus structure and sanitization verified.\n");
    Ok(())
}

fn scan_compat_dir(dir: &Path, count: &mut usize) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read dir {dir:?}: {e}"))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let p = entry.path();
        if p.is_dir() {
            scan_compat_dir(&p, count)?;
        } else if p.is_file() {
            *count += 1;
            let content = fs::read_to_string(&p).unwrap_or_default();
            let forbidden = ["ghp_", "sk-", "Bearer ey", "AIzaSy"];
            for token in forbidden {
                if content.contains(token) {
                    return Err(format!(
                        "Unsanitized secret pattern '{token}' detected in {p:?}"
                    ));
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. PROVIDER-CHECK: Conformance Lab & Promotion Checklist
// ---------------------------------------------------------------------------
fn cmd_provider_check(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: cargo xtask provider-check <provider-id>\nSupported providers: nvidia, antigravity, poolside, openrouter, ollama, custom".to_string());
    }
    let provider = args[0].to_lowercase();
    let root = get_repo_root()?;

    // Validate provider identifier syntax (lowercase alphanumeric + hyphen)
    if !provider
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(format!(
            "Invalid provider identifier '{provider}'. Must be lowercase alphanumeric kebab-case."
        ));
    }

    println!("=== Evaluating Provider Conformance & Lifecycle: '{provider}' ===");

    let first_class_targets = [
        ("nvidia", "NVIDIA NIM"),
        ("antigravity", "Antigravity Connector"),
        ("poolside", "Poolside Inference"),
        ("openrouter", "OpenRouter"),
        ("ollama", "Ollama Cloud"),
        ("custom", "Custom Endpoint"),
    ];

    let found_target = first_class_targets.iter().find(|(id, _)| *id == provider);

    let lifecycle_state = match found_target {
        Some(_) => "TARGET: FIRST_CLASS (Pre-Implementation Baseline)",
        None => "EXPERIMENTAL",
    };

    println!("Provider ID:             {provider}");
    println!("Lifecycle State:         {lifecycle_state}");

    // Inspect physical artifacts on disk
    let profile_json = root
        .join("profiles/providers")
        .join(format!("{provider}.json"));
    let profile_toml = root
        .join("profiles/providers")
        .join(format!("{provider}.toml"));
    let has_profile = profile_json.exists() || profile_toml.exists();

    let adapter_mod = root
        .join("crates/gatewaymux-providers/src")
        .join(format!("{}.rs", provider.replace('-', "_")));
    let has_adapter = adapter_mod.exists();

    println!("\nRepository Artifact Status:");
    println!(
        "  - Profile specification (profiles/providers/{provider}.json): {}",
        if has_profile {
            "PRESENT"
        } else {
            "NOT YET CREATED (Pre-implementation)"
        }
    );
    println!(
        "  - Adapter module (crates/gatewaymux-providers/src/): {}",
        if has_adapter {
            "PRESENT"
        } else {
            "NOT YET IMPLEMENTED (Pre-implementation)"
        }
    );

    println!("\nNormative FIRST_CLASS Promotion Checklist (PRD Section 28.5 & Appendix B):");
    println!("  [PENDING - PRE-IMPLEMENTATION] 1. Auth Safety: SecretStore integration verified; zero plaintext keys in config.");
    println!("  [PENDING - PRE-IMPLEMENTATION] 2. Egress & Trust: Default general_cloud; strictly scoped network egress.");
    println!("  [PENDING - PRE-IMPLEMENTATION] 3. Canonical IR: Operation mapping preservation verified across supported families.");
    println!("  [PENDING - PRE-IMPLEMENTATION] 4. Retry & Ledger: Idempotency check before retry; AttemptLedger recorded.");
    println!("  [PENDING - PRE-IMPLEMENTATION] 5. Streaming Invariants: Zero provider splice after visible token yield.");
    println!(
        "  [PENDING - PRE-IMPLEMENTATION] 6. Error Taxonomy: Mapped to standard GMX_* error codes."
    );
    println!("  [PENDING - PRE-IMPLEMENTATION] 7. Profile Registered: Provider & model profile specifications in profiles/providers/.");
    println!("  [PENDING - PRE-IMPLEMENTATION] 8. Compatibility Corpus: Sanitized golden transcripts in compat/providers/{provider}/.");

    if lifecycle_state == "EXPERIMENTAL" {
        println!("\nRules for EXPERIMENTAL providers:");
        println!("  - MUST be explicit opt-in in user configuration.");
        println!("  - MUST NOT be included in default routing combos.");
        println!("  - MUST NOT weaken trust, secrets, or streaming invariants.");
    }

    println!("\n[provider-check INFO] Provider specification evaluated successfully. Runtime checklist validation will become active once provider adapter and profile implementations begin.\n");
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. TEST: Test Execution with Risk-Tier Awareness
// ---------------------------------------------------------------------------
fn cmd_test(args: &[String]) -> Result<(), String> {
    println!("=== Running Workspace Test Suite ===");
    let root = get_repo_root()?;

    let mut risk_filter: Option<RiskClass> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--risk" && i + 1 < args.len() {
            risk_filter = RiskClass::parse_opt(&args[i + 1]);
            i += 2;
        } else {
            i += 1;
        }
    }

    if let Some(r) = risk_filter {
        println!("Testing targeted for risk tier: {}", r.as_str());
        match r {
            RiskClass::R0 => {
                println!("  -> R0 scope: Non-runtime documentation and metadata.");
                println!("     Executing repo hygiene check and formatting validation...");
                cmd_repo_check(&[])?;
                let fmt_status = run_cmd("cargo", &["fmt", "--check"], &root)?;
                if !fmt_status.success() {
                    return Err("cargo fmt --check failed".to_string());
                }
                println!("[test PASS] R0 verification suite completed.\n");
                return Ok(());
            }
            RiskClass::R1 => {
                println!("  -> R1 scope: Isolated implementation. Running workspace unit tests.");
            }
            RiskClass::R2 => {
                println!("  -> R2 scope: Provider & protocol behavior. Running unit tests and compat checks.");
                cmd_compat(&[])?;
            }
            RiskClass::R3 => {
                println!("  -> R3 scope: Routing & core APIs. Running unit tests, compat checks, and integration suite.");
                cmd_compat(&[])?;
            }
            RiskClass::R4 => {
                println!("  -> R4 scope: Critical trust, sync, bridge, security. Running full suite with security checks.");
                cmd_compat(&[])?;
            }
        }
    }

    let status = run_cmd("cargo", &["test", "--workspace"], &root)?;
    if !status.success() {
        return Err("cargo test --workspace failed".to_string());
    }

    let npm_status = run_cmd("npm", &["test"], &root)?;
    if !npm_status.success() {
        return Err("npm test failed".to_string());
    }

    // Report pre-implementation test harness directory status
    let test_dirs = [
        ("tests/integration", "Integration test suite"),
        ("tests/e2e", "End-to-end full server test suite"),
        ("tests/fault", "Fault injection and recovery test suite"),
        (
            "tests/security",
            "Security containment and boundary test suite",
        ),
        ("tests/fixtures", "Test fixtures"),
    ];
    println!("\nPre-Implementation Test Harness Status:");
    for (dir, desc) in test_dirs {
        let p = root.join(dir);
        let exists = p.exists();
        println!(
            "  - {dir: <20} ({desc}): {}",
            if exists {
                "INITIALIZED (Pre-implementation scaffold)"
            } else {
                "MISSING"
            }
        );
    }

    println!("\n[test PASS] All applicable test suites completed successfully.\n");
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. CHECK: Full Integrated Verification
// ---------------------------------------------------------------------------
fn cmd_check(_args: &[String]) -> Result<(), String> {
    println!("=== Running Full Workspace Integrated Validation ===");
    let root = get_repo_root()?;

    cmd_repo_check(&[])?;
    architecture::cmd_architecture_check(&root)?;
    generate::cmd_generate(&["--check".to_string()], &root)?;
    cmd_compat(&[])?;

    println!("Checking Rust formatting...");
    let fmt_status = run_cmd("cargo", &["fmt", "--check"], &root)?;
    if !fmt_status.success() {
        return Err("cargo fmt --check failed".to_string());
    }

    println!("Checking Rust compilation...");
    let check_status = run_cmd("cargo", &["check", "--workspace"], &root)?;
    if !check_status.success() {
        return Err("cargo check --workspace failed".to_string());
    }

    println!("Running Rust Clippy...");
    let clippy_status = run_cmd(
        "cargo",
        &["clippy", "--workspace", "--", "-D", "warnings"],
        &root,
    )?;
    if !clippy_status.success() {
        return Err("cargo clippy failed".to_string());
    }

    println!("Running npm workspaces check...");
    let npm_status = run_cmd("npm", &["run", "check"], &root)?;
    if !npm_status.success() {
        return Err("npm run check failed".to_string());
    }

    println!("[check PASS] Full workspace check succeeded without warnings or errors.\n");
    Ok(())
}

// ---------------------------------------------------------------------------
// 6. RELEASE-CHECK: Release Readiness & Coordinated Version Synchronization
// ---------------------------------------------------------------------------
fn cmd_release_check(_args: &[String]) -> Result<(), String> {
    println!("=== Running Release Readiness & Version Alignment Check ===");
    let root = get_repo_root()?;

    // Read root Cargo.toml version
    let cargo_content = fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|e| format!("Failed to read Cargo.toml: {e}"))?;
    let mut cargo_version = String::new();
    for line in cargo_content.lines() {
        if line.trim().starts_with("version = ") {
            cargo_version = line.trim()["version = ".len()..]
                .trim_matches('"')
                .to_string();
            break;
        }
    }

    if cargo_version.is_empty() {
        return Err("Failed to determine version from Cargo.toml".to_string());
    }

    println!("Cargo workspace version: {cargo_version}");

    // Coordinated release versioning: the npm manifests and the Rust
    // reference Sync Server must match the root Cargo workspace version.
    let package_manifests = [
        ("Root package.json", root.join("package.json")),
        (
            "Dashboard package.json",
            root.join("dashboard/package.json"),
        ),
        (
            "Packaging/npm package.json",
            root.join("packaging/npm/package.json"),
        ),
    ];

    for (label, path) in &package_manifests {
        if !path.exists() {
            return Err(format!("Missing manifest: {label} at {path:?}"));
        }
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read {label}: {e}"))?;
        let json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {label}: {e}"))?;
        let ver = json.get("version").and_then(|v| v.as_str()).unwrap_or("");
        println!("{label: <28} version: {ver}");
        if ver != cargo_version {
            return Err(format!(
                "Version mismatch in {label}: expected '{cargo_version}', found '{ver}'"
            ));
        }
    }

    // The Rust reference Sync Server (sync-server/) is a workspace member
    // and MUST derive its version from the workspace manifest so that the
    // coordinated GatewayMux semantic version stays unified.
    let sync_cargo_path = root.join("sync-server/Cargo.toml");
    if !sync_cargo_path.exists() {
        return Err("Missing Rust Sync Server manifest: sync-server/Cargo.toml".to_string());
    }
    let sync_cargo = fs::read_to_string(&sync_cargo_path)
        .map_err(|e| format!("Failed to read sync-server/Cargo.toml: {e}"))?;
    let version_from_workspace = sync_cargo
        .lines()
        .any(|l| l.trim() == "version.workspace = true");
    if !version_from_workspace {
        return Err(
            "sync-server/Cargo.toml must derive its version from the workspace manifest (version.workspace = true) to keep the unified GatewayMux semantic version".to_string(),
        );
    }
    println!("Sync Server (Rust)          version: {cargo_version} (workspace-derived)");

    // Check lockfiles
    if !root.join("Cargo.lock").exists() {
        return Err("Cargo.lock is missing".to_string());
    }
    if !root.join("package-lock.json").exists() {
        return Err("package-lock.json is missing".to_string());
    }

    // Check required governance & legal documents
    for doc in [
        "LICENSE",
        "README.md",
        "CHANGELOG.md",
        "SECURITY.md",
        "CONTRIBUTING.md",
        "AGENTS.md",
    ] {
        if !root.join(doc).exists() {
            return Err(format!("Required release document missing: {doc}"));
        }
    }

    println!("\nRelease Manifest Specification (PRD Section 28.6):");
    println!("  - product_version:        {cargo_version}");
    println!("  - sync_protocol_version:  1");
    println!("  - schema_version:         1");
    println!("  - control_api_version:    1");
    println!("  - corpus_revision:        1");

    println!("\n[release-check PASS] Repository is release-check compliant and all coordinated manifests are synchronized.\n");
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests for the workflow toolchain-authority and action-pin checks
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// A disposable test repository root under the system temp dir.
    struct TempRepo(PathBuf);

    impl TempRepo {
        fn new() -> Self {
            let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "gatewaymux-xtask-repo-check-test-{}-{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&dir).expect("failed to create temp dir");
            TempRepo(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write(&self, relative: &str, content: &str) {
            let p = self.0.join(relative);
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).expect("failed to create parent dir");
            }
            fs::write(&p, content).expect("failed to write file");
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const VALID_TOOLCHAIN_FILE: &str = "[toolchain]\nchannel = \"1.98.1\"\ncomponents = [\"rustfmt\", \"clippy\"]\nprofile = \"minimal\"\n";

    const SHA_PINNED_WORKFLOW: &str = "name: CI\non:\n  push:\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1\n      - name: Setup Rust toolchain\n        uses: actions-rust-lang/setup-rust-toolchain@ecabd13d1c56bd1345c230e542e9144811ad706f # v2.0.0\n";

    #[test]
    fn test_toolchain_channel_parsing() {
        let repo = TempRepo::new();
        repo.write("rust-toolchain.toml", VALID_TOOLCHAIN_FILE);
        let (channel, components) =
            read_rust_toolchain(repo.path()).expect("valid file must parse");
        assert_eq!(channel, "1.98.1");
        assert!(components.contains(&"rustfmt".to_string()));
        assert!(components.contains(&"clippy".to_string()));
    }

    #[test]
    fn test_toolchain_check_rejects_missing_channel_or_components() {
        let repo = TempRepo::new();
        repo.write(
            "rust-toolchain.toml",
            "[toolchain]\ncomponents = [\"rustfmt\"]\n",
        );
        let err = read_rust_toolchain(repo.path()).unwrap_err();
        assert!(err.contains("channel"), "unexpected error: {err}");

        let repo = TempRepo::new();
        repo.write(
            "rust-toolchain.toml",
            "[toolchain]\nchannel = \"1.98.1\"\ncomponents = [\"rustfmt\"]\n",
        );
        let err = verify_ci_toolchain_pins(repo.path()).unwrap_err();
        assert!(err.contains("clippy"), "unexpected error: {err}");
    }

    #[test]
    fn test_toolchain_check_rejects_workflow_toolchain_input() {
        let repo = TempRepo::new();
        repo.write("rust-toolchain.toml", VALID_TOOLCHAIN_FILE);
        repo.write(
            ".github/workflows/ci.yml",
            "jobs:\n  build:\n    steps:\n      - uses: dtolnay/rust-toolchain@7e38f4b43b4db5c8dd498af069a4f6196df1d067\n        with:\n          toolchain: stable\n",
        );
        let err = verify_ci_toolchain_pins(repo.path()).unwrap_err();
        assert!(err.contains("toolchain"), "unexpected error: {err}");
        assert!(
            err.contains("1.98.1"),
            "error must name the pinned channel: {err}"
        );
        assert!(
            err.contains("ci.yml"),
            "error must name the offending workflow: {err}"
        );
    }

    #[test]
    fn test_toolchain_check_accepts_sha_pinned_file_authoritative_workflow() {
        let repo = TempRepo::new();
        repo.write("rust-toolchain.toml", VALID_TOOLCHAIN_FILE);
        repo.write(".github/workflows/ci.yml", SHA_PINNED_WORKFLOW);
        assert!(verify_ci_toolchain_pins(repo.path()).is_ok());
    }

    #[test]
    fn test_toolchain_check_ignores_comments_mentioning_toolchain() {
        // Comments mentioning `toolchain` must never trip the check.
        let repo = TempRepo::new();
        repo.write("rust-toolchain.toml", VALID_TOOLCHAIN_FILE);
        repo.write(
            ".github/workflows/ci.yml",
            "# No `toolchain` input: the action installs the file's channel.\njobs: {}\n",
        );
        assert!(verify_ci_toolchain_pins(repo.path()).is_ok());
    }

    #[test]
    fn test_action_pins_accept_sha_pinned_workflows() {
        let repo = TempRepo::new();
        repo.write(".github/workflows/ci.yml", SHA_PINNED_WORKFLOW);
        assert!(verify_workflow_action_pins(repo.path()).is_ok());
    }

    #[test]
    fn test_action_pins_reject_floating_tags() {
        for floating in [
            "uses: actions/checkout@v4\n",
            "uses: actions/checkout@stable\n",
            "uses: dtolnay/rust-toolchain@master\n",
            "uses: actions/checkout\n",
        ] {
            let repo = TempRepo::new();
            repo.write(
                ".github/workflows/ci.yml",
                &format!("jobs:\n  build:\n    steps:\n      - {floating}"),
            );
            let err = verify_workflow_action_pins(repo.path()).unwrap_err();
            assert!(
                err.contains("full-length commit SHA"),
                "floating ref must be rejected: {err}"
            );
        }
    }

    #[test]
    fn test_action_pins_reject_short_shas_and_list_form() {
        // Short (non-40-hex) refs are not immutable pins.
        let repo = TempRepo::new();
        repo.write(
            ".github/workflows/ci.yml",
            "jobs:\n  build:\n    steps:\n      - uses: owner/repo@deadbeef\n",
        );
        assert!(verify_workflow_action_pins(repo.path()).is_err());

        // Local composite actions are not covered by the SHA-pin rule
        // and are rejected pending a separate governance decision.
        let repo = TempRepo::new();
        repo.write(
            ".github/workflows/ci.yml",
            "jobs:\n  build:\n    steps:\n      - uses: ./.github/actions/local\n",
        );
        assert!(verify_workflow_action_pins(repo.path()).is_err());
    }

    #[test]
    fn test_action_pins_ignores_non_uses_lines_and_comments() {
        let repo = TempRepo::new();
        repo.write(
            ".github/workflows/ci.yml",
            "# see owner/repo@v4 mentioned in prose\nname: CI\nrun: |\n  echo \"uses: not-a-key\"\n",
        );
        assert!(verify_workflow_action_pins(repo.path()).is_ok());
    }
}
