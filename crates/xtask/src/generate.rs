//! Deterministic generated-file regeneration and drift verification.
//!
//! Contract implemented here:
//!
//! ```text
//! authoritative SOURCE
//!     -> declared (registered, allowlisted) generator
//!     -> generated output in isolated in-memory state
//!     -> deterministic comparison with the tracked GENERATED artifact
//!     -> identical = PASS, different = FAIL
//! ```
//!
//! `cargo xtask generate --check` never writes to the working tree; it
//! regenerates each registered output in memory and compares bytes with
//! the tracked artifact. Plain `cargo xtask generate` deliberately
//! rewrites registered generated outputs.
//!
//! Security properties:
//! - Generators are compiled-in allowlisted variants; manifest data can
//!   never invoke arbitrary shell commands.
//! - Manifest paths are validated against traversal, absolute paths, and
//!   drive letters; every source and target stays inside the repository.
//! - Failures never mutate source files; generation output is produced
//!   in memory before any write decision is made.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Manifest schema version understood by this engine.
pub const MANIFEST_VERSION: u64 = 2;

/// A registered generator. The set of generators is compiled into xtask;
/// contributors cannot register new generators by editing the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorKind {
    /// Byte-for-byte copy of the authoritative source.
    Copy,
    /// Canonicalized JSON of the authoritative source: parsed, keys
    /// sorted, stable pretty formatting, trailing newline.
    JsonCanonicalize,
}

impl GeneratorKind {
    /// Resolve a registered generator from its manifest identifier.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "copy" => Some(GeneratorKind::Copy),
            "json-canonicalize" => Some(GeneratorKind::JsonCanonicalize),
            _ => None,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            GeneratorKind::Copy => "copy",
            GeneratorKind::JsonCanonicalize => "json-canonicalize",
        }
    }

    /// Deterministically produce the generated output bytes from the
    /// authoritative source bytes. This is the isolated generation
    /// state: nothing is written to disk here.
    pub fn generate(&self, source_bytes: &[u8]) -> Result<Vec<u8>, String> {
        match self {
            GeneratorKind::Copy => Ok(source_bytes.to_vec()),
            GeneratorKind::JsonCanonicalize => {
                let value: serde_json::Value = serde_json::from_slice(source_bytes)
                    .map_err(|e| format!("Authoritative source is not valid JSON: {e}"))?;
                let mut out = serde_json::to_string_pretty(&value)
                    .map_err(|e| format!("Failed to serialize canonical JSON: {e}"))?;
                out.push('\n');
                Ok(out.into_bytes())
            }
        }
    }
}

/// A single registered generated-file entry from the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedEntry {
    /// Repository-relative path of the tracked generated artifact.
    pub path: String,
    /// Registered generator identifier.
    pub generator: GeneratorKind,
    /// Repository-relative path of the authoritative source.
    pub authoritative_source: String,
}

/// Parse and strictly validate the generated-file manifest.
///
/// Unknown manifest versions, unknown generators, missing fields, and
/// unknown entry keys are all rejected: the manifest schema is
/// machine-verifiable, not decorative text.
pub fn parse_manifest(content: &str) -> Result<Vec<GeneratedEntry>, String> {
    let manifest: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("Invalid JSON in manifest: {e}"))?;

    let version = manifest
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "Manifest must declare an integer 'version' field".to_string())?;
    if version != MANIFEST_VERSION {
        return Err(format!(
            "Unsupported manifest schema version {version}; expected {MANIFEST_VERSION}. Regeneration requires the current xtask engine schema."
        ));
    }

    let files = manifest
        .get("generated_files")
        .and_then(|f| f.as_array())
        .ok_or_else(|| "Manifest must declare a 'generated_files' array".to_string())?;

    let allowed_keys = ["path", "generator", "authoritative_source"];
    let mut entries = Vec::new();
    let mut seen_targets: HashSet<String> = HashSet::new();

    for item in files {
        let obj = item
            .as_object()
            .ok_or_else(|| "Each generated_files entry must be a JSON object".to_string())?;

        for key in obj.keys() {
            if !allowed_keys.contains(&key.as_str()) {
                return Err(format!(
                    "Unknown manifest entry key '{key}'; allowed keys are {allowed_keys:?}"
                ));
            }
        }

        let path = obj
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'path' in generated_files entry".to_string())?;
        if path.trim().is_empty() {
            return Err("'path' must be a non-empty repository-relative path".to_string());
        }

        let generator_id = obj
            .get("generator")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'generator' in generated_files entry".to_string())?;
        let generator = GeneratorKind::from_id(generator_id).ok_or_else(|| {
            format!(
                "Unknown generator id '{generator_id}'. Generators are registered in xtask; manifest data cannot declare arbitrary generators."
            )
        })?;

        let source = obj
            .get("authoritative_source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'authoritative_source' in generated_files entry".to_string())?;
        if source.trim().is_empty() {
            return Err(
                "'authoritative_source' must be a non-empty repository-relative path".to_string(),
            );
        }

        if path == source {
            return Err(format!(
                "Generated target '{path}' must differ from its authoritative source"
            ));
        }

        if !seen_targets.insert(path.to_string()) {
            return Err(format!("Duplicate generated target '{path}' in manifest"));
        }

        entries.push(GeneratedEntry {
            path: path.to_string(),
            generator,
            authoritative_source: source.to_string(),
        });
    }

    Ok(entries)
}

/// Validate and resolve a repository-relative manifest path.
///
/// Rejects empty paths, absolute paths (Unix or Windows drive letters),
/// path traversal (`..`), and dot components. The resolved path is
/// guaranteed to remain within the repository root.
pub fn resolve_manifest_path(root: &Path, relative: &str, field: &str) -> Result<PathBuf, String> {
    let normalized = relative.trim().replace('\\', "/");
    if normalized.is_empty() {
        return Err(format!("Manifest field '{field}' must not be empty"));
    }
    if normalized.starts_with('/') {
        return Err(format!(
            "Manifest field '{field}' must be repository-relative; absolute path '{relative}' rejected"
        ));
    }
    if normalized.len() >= 2 && normalized.as_bytes()[1] == b':' {
        return Err(format!(
            "Manifest field '{field}' must be repository-relative; drive-qualified path '{relative}' rejected"
        ));
    }
    for component in normalized.split('/') {
        if component.is_empty() {
            return Err(format!(
                "Manifest field '{field}' contains an empty path component: '{relative}'"
            ));
        }
        if component == "." || component == ".." {
            return Err(format!(
                "Manifest field '{field}' rejected: path traversal component '{component}' in '{relative}'"
            ));
        }
    }
    let resolved = root.join(&normalized);
    if !resolved.starts_with(root) {
        return Err(format!(
            "Manifest field '{field}' resolves outside the repository root: '{relative}'"
        ));
    }
    Ok(resolved)
}

/// Regenerate-and-compare / regenerate-and-write engine.
///
/// `check_mode == true`: regenerate in memory and byte-compare against
/// the tracked artifact. Never mutates the working tree.
///
/// `check_mode == false`: deliberately rewrite registered generated
/// outputs with freshly generated bytes.
pub fn run_generate(root: &Path, check_mode: bool) -> Result<(), String> {
    let manifest_path = root.join(".generated-manifest.json");
    if !manifest_path.exists() {
        return Err("Missing .generated-manifest.json manifest file.".to_string());
    }

    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read .generated-manifest.json: {e}"))?;
    let entries = parse_manifest(&manifest_content)?;

    println!("=== Running Generated Files Verification (check_mode={check_mode}) ===");

    if entries.is_empty() {
        if check_mode {
            println!("No generated files registered in .generated-manifest.json.");
            println!("[generate --check PASS] Manifest is valid; 0 tracked generated files (clean drift check).\n");
        } else {
            println!("No generated files registered in .generated-manifest.json.");
            println!(
                "[generate INFO] 0 generator tasks to execute (pre-implementation scaffold).\n"
            );
        }
        return Ok(());
    }

    println!("Registered generated files: {}", entries.len());

    for entry in &entries {
        let source_path =
            resolve_manifest_path(root, &entry.authoritative_source, "authoritative_source")?;
        let target_path = resolve_manifest_path(root, &entry.path, "path")?;

        let source_bytes = fs::read(&source_path).map_err(|e| {
            format!(
                "Authoritative source '{}' missing or unreadable: {e}",
                entry.authoritative_source
            )
        })?;

        // Isolated deterministic generation: bytes are produced in memory
        // and never touch the working tree during verification.
        let produced = entry.generator.generate(&source_bytes)?;

        if check_mode {
            let tracked = fs::read(&target_path).map_err(|e| {
                format!(
                    "Registered generated file '{}' missing on disk: {e}",
                    entry.path
                )
            })?;
            if tracked != produced {
                return Err(format!(
                    "Generated file '{}' has drifted from its authoritative source '{}' (generator '{}'). Regenerate with `cargo xtask generate` and commit the result; hand-editing generated files is prohibited.",
                    entry.path,
                    entry.authoritative_source,
                    entry.generator.id()
                ));
            }
            println!(
                "  \u{2713} {} (generator: {}, source: {}) - regenerated output is identical",
                entry.path,
                entry.generator.id(),
                entry.authoritative_source
            );
        } else {
            let up_to_date = fs::read(&target_path)
                .map(|tracked| tracked == produced)
                .unwrap_or(false);
            if up_to_date {
                println!(
                    "  \u{2713} {} already up to date (generator: {})",
                    entry.path,
                    entry.generator.id()
                );
            } else {
                // Deliberate update of a registered generated output only.
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        format!(
                            "Failed to create parent directory for '{}': {e}",
                            entry.path
                        )
                    })?;
                }
                fs::write(&target_path, &produced)
                    .map_err(|e| format!("Failed to write generated file '{}': {e}", entry.path))?;
                println!(
                    "  [RUN] {} regenerated from {} (generator: {})",
                    entry.path,
                    entry.authoritative_source,
                    entry.generator.id()
                );
            }
        }
    }

    if check_mode {
        println!("[generate --check PASS] All registered generated files regenerate deterministically and match their tracked artifacts.\n");
    } else {
        println!("[generate PASS] Registered generated files synchronized with their authoritative sources.\n");
    }
    Ok(())
}

/// `cargo xtask generate [--check]` entry point.
pub fn cmd_generate(args: &[String], root: &Path) -> Result<(), String> {
    let check_mode = args.iter().any(|a| a == "--check");
    run_generate(root, check_mode)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// A disposable test root under the system temp directory. The engine
    /// under test treats it exactly like a repository root.
    struct TempRepo(PathBuf);

    impl TempRepo {
        fn new() -> Self {
            let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "gatewaymux-xtask-generate-test-{}-{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&dir).expect("failed to create temp dir");
            TempRepo(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write(&self, relative: &str, content: &str) -> PathBuf {
            let p = self.0.join(relative);
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).expect("failed to create parent dir");
            }
            fs::write(&p, content).expect("failed to write file");
            p
        }

        fn read(&self, relative: &str) -> String {
            fs::read_to_string(self.0.join(relative)).expect("failed to read file")
        }

        fn manifest(&self, files_json: &str) -> PathBuf {
            let content = format!(
                "{{\n  \"version\": {MANIFEST_VERSION},\n  \"description\": \"test manifest\",\n  \"generated_files\": {files_json}\n}}\n"
            );
            self.write(".generated-manifest.json", &content)
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            // Temporary test state is always cleaned up.
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn copy_entry() -> String {
        r#"[{"path": "out/generated.txt", "generator": "copy", "authoritative_source": "src/source.txt"}]"#.to_string()
    }

    #[test]
    fn test_check_passes_on_identical_generation() {
        let repo = TempRepo::new();
        repo.manifest(&copy_entry());
        repo.write("src/source.txt", "hello world");
        repo.write("out/generated.txt", "hello world");

        assert!(run_generate(repo.path(), true).is_ok());
    }

    #[test]
    fn test_check_fails_on_drift() {
        let repo = TempRepo::new();
        repo.manifest(&copy_entry());
        repo.write("src/source.txt", "hello world v2");
        repo.write("out/generated.txt", "hello world v1");

        let err = run_generate(repo.path(), true).unwrap_err();
        assert!(err.contains("drifted"), "unexpected error: {err}");
        assert!(err.contains("out/generated.txt"));
    }

    #[test]
    fn test_check_does_not_mutate_drifted_target() {
        // Verification must never rewrite the working tree, even when
        // drift is detected or the target is missing.
        let repo = TempRepo::new();
        repo.manifest(&copy_entry());
        repo.write("src/source.txt", "authoritative");
        repo.write("out/generated.txt", "stale content");

        assert!(run_generate(repo.path(), true).is_err());
        assert_eq!(repo.read("out/generated.txt"), "stale content");

        // Missing target in check mode: error, and no file is created.
        let repo_missing = TempRepo::new();
        repo_missing.manifest(&copy_entry());
        repo_missing.write("src/source.txt", "authoritative");
        assert!(run_generate(repo_missing.path(), true).is_err());
        assert!(!repo_missing.path().join("out/generated.txt").exists());
    }

    #[test]
    fn test_sync_mode_rewrites_registered_output_only() {
        let repo = TempRepo::new();
        repo.manifest(&copy_entry());
        repo.write("src/source.txt", "authoritative v1");
        repo.write("unrelated.txt", "untouched");
        assert!(run_generate(repo.path(), false).is_ok());
        assert_eq!(repo.read("out/generated.txt"), "authoritative v1");
        assert_eq!(repo.read("unrelated.txt"), "untouched");

        // Regeneration after source change updates the target; the source
        // itself is never modified.
        repo.write("src/source.txt", "authoritative v2");
        assert!(run_generate(repo.path(), false).is_ok());
        assert_eq!(repo.read("out/generated.txt"), "authoritative v2");
        assert_eq!(repo.read("src/source.txt"), "authoritative v2");
        assert!(run_generate(repo.path(), true).is_ok());
    }

    #[test]
    fn test_rejects_path_traversal() {
        let repo = TempRepo::new();
        let traversal = r#"[{"path": "../escape.txt", "generator": "copy", "authoritative_source": "src/source.txt"}]"#;
        repo.manifest(traversal);
        repo.write("src/source.txt", "data");

        let err = run_generate(repo.path(), true).unwrap_err();
        assert!(err.contains("traversal"), "unexpected error: {err}");
        assert!(!repo.path().parent().unwrap().join("escape.txt").exists());
    }

    #[test]
    fn test_rejects_traversal_in_source_field() {
        let repo = TempRepo::new();
        let traversal = r#"[{"path": "out/generated.txt", "generator": "copy", "authoritative_source": "../../etc/passwd"}]"#;
        repo.manifest(traversal);

        let err = run_generate(repo.path(), true).unwrap_err();
        assert!(err.contains("traversal"), "unexpected error: {err}");
    }

    #[test]
    fn test_rejects_absolute_paths() {
        for field in ["path", "authoritative_source"] {
            let value = if field == "path" {
                "/etc/passwd"
            } else {
                "C:/Windows/evil.txt"
            };
            let entries = format!(
                r#"[{{"path": "out/generated.txt", "generator": "copy", "authoritative_source": "src/source.txt"}}]"#
            );
            if field == "path" {
                let entries = entries.replace("out/generated.txt", value);
                let repo = TempRepo::new();
                repo.manifest(&entries);
                let err = run_generate(repo.path(), true).unwrap_err();
                assert!(
                    err.contains("repository-relative") || err.contains("root"),
                    "unexpected error: {err}"
                );
            } else {
                let entries = entries.replace("src/source.txt", value);
                let repo = TempRepo::new();
                repo.manifest(&entries);
                let err = run_generate(repo.path(), true).unwrap_err();
                assert!(
                    err.contains("repository-relative") || err.contains("root"),
                    "unexpected error: {err}"
                );
            }
        }
    }

    #[test]
    fn test_rejects_drive_qualified_path() {
        let repo = TempRepo::new();
        let entries = r#"[{"path": "C:/evil.txt", "generator": "copy", "authoritative_source": "src/source.txt"}]"#;
        repo.manifest(entries);

        let err = run_generate(repo.path(), true).unwrap_err();
        assert!(
            err.contains("drive-qualified") || err.contains("repository-relative"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_rejects_unknown_generator() {
        let repo = TempRepo::new();
        let entries = r#"[{"path": "out/generated.txt", "generator": "shell-exec", "authoritative_source": "src/source.txt"}]"#;
        repo.manifest(entries);
        repo.write("src/source.txt", "data");

        let err = run_generate(repo.path(), true).unwrap_err();
        assert!(err.contains("Unknown generator"), "unexpected error: {err}");

        // Failure never mutates anything: no target written.
        assert!(!repo.path().join("out/generated.txt").exists());
    }

    #[test]
    fn test_rejects_unsupported_manifest_version() {
        let repo = TempRepo::new();
        repo.write(
            ".generated-manifest.json",
            r#"{"version": 1, "generated_files": []}"#,
        );
        let err = run_generate(repo.path(), true).unwrap_err();
        assert!(
            err.contains("Unsupported manifest schema version"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_rejects_unknown_entry_keys() {
        let repo = TempRepo::new();
        let entries = r#"[{"path": "out/generated.txt", "generator": "copy", "authoritative_source": "src/source.txt", "command": "rm -rf /"}]"#;
        repo.manifest(entries);
        let err = run_generate(repo.path(), true).unwrap_err();
        assert!(
            err.contains("Unknown manifest entry key 'command'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_rejects_missing_fields_and_self_reference() {
        let repo = TempRepo::new();
        repo.manifest(r#"[{"generator": "copy", "authoritative_source": "src/source.txt"}]"#);
        assert!(run_generate(repo.path(), true)
            .unwrap_err()
            .contains("Missing 'path'"));

        let repo = TempRepo::new();
        repo.manifest(
            r#"[{"path": "same.txt", "generator": "copy", "authoritative_source": "same.txt"}]"#,
        );
        assert!(run_generate(repo.path(), true)
            .unwrap_err()
            .contains("must differ from its authoritative source"));

        let repo = TempRepo::new();
        repo.manifest(
            r#"[{"path": "a.txt", "generator": "copy", "authoritative_source": "s.txt"},
                {"path": "a.txt", "generator": "copy", "authoritative_source": "s.txt"}]"#,
        );
        assert!(run_generate(repo.path(), true)
            .unwrap_err()
            .contains("Duplicate generated target"));
    }

    #[test]
    fn test_empty_manifest_passes() {
        let repo = TempRepo::new();
        repo.manifest("[]");
        assert!(run_generate(repo.path(), true).is_ok());
        assert!(run_generate(repo.path(), false).is_ok());
    }

    #[test]
    fn test_json_canonicalize_is_deterministic_and_detects_drift() {
        let repo = TempRepo::new();
        let entries = r#"[{"path": "out/canonical.json", "generator": "json-canonicalize", "authoritative_source": "src/source.json"}]"#;
        repo.manifest(entries);
        repo.write("src/source.json", r#"{"b": 1, "a": 2}"#);
        repo.write("out/canonical.json", "{\n  \"a\": 2,\n  \"b\": 1\n}\n");
        assert!(run_generate(repo.path(), true).is_ok());

        // Semantic drift: tracked artifact no longer matches regenerated
        // canonical form.
        repo.write("out/canonical.json", "{\n  \"a\": 2,\n  \"b\": 0\n}\n");
        assert!(run_generate(repo.path(), true).is_err());

        // Reformatting-only drift is still drift: determinism is byte-exact.
        repo.write("out/canonical.json", "{\"a\": 2, \"b\": 1}");
        assert!(run_generate(repo.path(), true).is_err());

        // Sync mode restores byte-identical canonical output.
        assert!(run_generate(repo.path(), false).is_ok());
        assert_eq!(
            repo.read("out/canonical.json"),
            "{\n  \"a\": 2,\n  \"b\": 1\n}\n"
        );
        assert!(run_generate(repo.path(), true).is_ok());
    }

    #[test]
    fn test_json_canonicalize_rejects_invalid_source() {
        let repo = TempRepo::new();
        let entries = r#"[{"path": "out/canonical.json", "generator": "json-canonicalize", "authoritative_source": "src/source.json"}]"#;
        repo.manifest(entries);
        repo.write("src/source.json", "not json");
        repo.write("out/canonical.json", "{}");
        let err = run_generate(repo.path(), true).unwrap_err();
        assert!(err.contains("not valid JSON"), "unexpected error: {err}");
        // Failed generation never mutates the tracked target.
        assert_eq!(repo.read("out/canonical.json"), "{}");
    }

    #[test]
    fn test_generator_ids_roundtrip() {
        for kind in [GeneratorKind::Copy, GeneratorKind::JsonCanonicalize] {
            assert_eq!(GeneratorKind::from_id(kind.id()), Some(kind));
        }
        assert_eq!(GeneratorKind::from_id("cargo xtask generate-schema"), None);
        assert_eq!(GeneratorKind::from_id(""), None);
    }
}
