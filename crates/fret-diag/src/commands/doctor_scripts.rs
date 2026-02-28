use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::paths::resolve_path;
use crate::script_registry::{PromotedScriptRegistry, promoted_registry_default_path};
use crate::util::read_json_value;

#[derive(Debug, Default)]
struct ScriptsDoctorCounts {
    root_json_total: u64,
    root_redirect_total: u64,
    root_non_redirect_total: u64,
    root_invalid_json_total: u64,

    redirects_missing_to_total: u64,
    redirects_target_missing_total: u64,
    redirects_target_outside_library_total: u64,
    redirects_chain_depth_gt2_total: u64,
    redirects_chain_depth_max: u64,

    suites_json_total: u64,
    suites_redirect_total: u64,
    suites_non_redirect_total: u64,
    suites_invalid_json_total: u64,
    suites_target_missing_total: u64,
    suites_redirect_target_is_redirect_total: u64,
    suites_redirect_chain_depth_gt2_total: u64,
    suites_redirect_chain_depth_max: u64,

    registry_entries_total: u64,
    registry_duplicate_id_total: u64,
    registry_path_missing_total: u64,
    registry_path_outside_library_total: u64,
    registry_path_is_redirect_total: u64,
    registry_schema_v1_total: u64,
}

#[derive(Debug, Default)]
struct ScriptsDoctorExamples {
    root_non_redirect: Vec<String>,
    root_invalid_json: Vec<String>,
    root_redirect_missing_to: Vec<String>,
    root_redirect_target_missing: Vec<String>,
    root_redirect_target_outside_library: Vec<String>,
    root_redirect_chain_depth_gt2: Vec<String>,

    suite_non_redirect: Vec<String>,
    suite_invalid_json: Vec<String>,
    suite_target_missing: Vec<String>,
    suite_redirect_chain_depth_gt2: Vec<String>,

    registry_duplicate_ids: Vec<String>,
    registry_path_missing: Vec<String>,
    registry_path_outside_library: Vec<String>,
    registry_path_is_redirect: Vec<String>,
    registry_schema_v1: Vec<String>,
}

fn push_bounded(out: &mut Vec<String>, max_examples: usize, item: String) {
    if out.len() < max_examples {
        out.push(item);
    }
}

fn is_script_redirect(v: &Value) -> bool {
    v.get("kind").and_then(|v| v.as_str()) == Some("script_redirect")
}

fn read_redirect_to(v: &Value) -> Option<String> {
    v.get("to").and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn redirect_chain_depth(
    workspace_root: &Path,
    start_path: &Path,
    max_depth: u64,
) -> Result<u64, String> {
    let mut depth: u64 = 0;
    let mut cur = start_path.to_path_buf();
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();

    loop {
        if !seen.insert(cur.clone()) {
            return Err(format!(
                "script redirect loop detected (revisiting): {}",
                cur.display()
            ));
        }
        let Some(v) = read_json_value(&cur) else {
            return Err(format!(
                "invalid JSON while resolving redirect chain: {}",
                cur.display()
            ));
        };
        if !is_script_redirect(&v) {
            return Ok(depth);
        }
        let Some(to) = read_redirect_to(&v) else {
            return Err(format!(
                "invalid script_redirect (missing string field: to): {}",
                cur.display()
            ));
        };
        depth = depth.saturating_add(1);
        if depth > max_depth {
            return Err(format!(
                "script redirect depth exceeded (max_depth={max_depth})"
            ));
        }
        cur = resolve_path(workspace_root, PathBuf::from(to));
    }
}

fn json_glob_recursive(dir: &Path, pattern_suffix: &str) -> Result<Vec<PathBuf>, String> {
    let mut pattern = dir.to_string_lossy().to_string().replace('\\', "/");
    if !pattern.ends_with('/') {
        pattern.push('/');
    }
    pattern.push_str(pattern_suffix);

    let mut out: BTreeSet<PathBuf> = BTreeSet::new();
    for entry in glob::glob(&pattern).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?;
        #[cfg(windows)]
        let path = PathBuf::from(path.to_string_lossy().replace('/', "\\"));
        out.insert(path);
    }
    Ok(out.into_iter().collect())
}

pub(crate) fn cmd_doctor_scripts(
    rest: &[String],
    workspace_root: &Path,
    stats_json: bool,
) -> Result<(), String> {
    let mut max_examples: usize = 20;
    let mut strict: bool = false;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--help" | "-h" => {
                println!(
                    "Usage: fretboard diag doctor scripts [--max-examples N] [--strict] [--json]\n\
                     \n\
                     Checks the diag script library for common drift issues:\n\
                     - canonical scripts accidentally committed under tools/diag-scripts/*.json\n\
                     - broken script_redirect stubs (missing/invalid targets)\n\
                     - promoted registry (index.json) drift (missing paths, duplicate ids)\n\
                     \n\
                     Notes:\n\
                     - This command is read-only; it prints suggested repair commands.\n\
                     - Use --json (top-level flag) for structured output."
                );
                return Ok(());
            }
            "--strict" => {
                strict = true;
                i += 1;
            }
            "--max-examples" | "--examples" | "--top" => {
                let Some(raw) = rest.get(i + 1) else {
                    return Err(format!("missing value for {}", rest[i]));
                };
                max_examples = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid --max-examples value: {raw}"))?;
                i += 2;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for doctor scripts: {other}"));
            }
            other => {
                return Err(format!("unexpected argument for doctor scripts: {other}"));
            }
        }
    }

    let scripts_root = workspace_root.join("tools").join("diag-scripts");
    if !scripts_root.is_dir() {
        return Err(format!(
            "diag script library directory not found: {}",
            scripts_root.display()
        ));
    }

    let index_path = promoted_registry_default_path(workspace_root);

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut repairs: Vec<Value> = Vec::new();

    let mut counts = ScriptsDoctorCounts::default();
    let mut examples = ScriptsDoctorExamples::default();

    const MAX_REDIRECT_DEPTH: u64 = 8;

    // Check top-level `tools/diag-scripts/*.json` are redirect stubs (except index.json).
    let read_dir = std::fs::read_dir(&scripts_root).map_err(|e| e.to_string())?;
    for entry in read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Some(file_name) = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
        else {
            continue;
        };
        if file_name == "index.json" {
            continue;
        }

        counts.root_json_total = counts.root_json_total.saturating_add(1);
        let Some(v) = read_json_value(&path) else {
            counts.root_invalid_json_total = counts.root_invalid_json_total.saturating_add(1);
            push_bounded(
                &mut examples.root_invalid_json,
                max_examples,
                path.display().to_string(),
            );
            continue;
        };

        if is_script_redirect(&v) {
            counts.root_redirect_total = counts.root_redirect_total.saturating_add(1);
            let Some(to) = read_redirect_to(&v) else {
                counts.redirects_missing_to_total =
                    counts.redirects_missing_to_total.saturating_add(1);
                push_bounded(
                    &mut examples.root_redirect_missing_to,
                    max_examples,
                    path.display().to_string(),
                );
                continue;
            };
            let target = resolve_path(workspace_root, PathBuf::from(&to));
            if !target.exists() {
                counts.redirects_target_missing_total =
                    counts.redirects_target_missing_total.saturating_add(1);
                push_bounded(
                    &mut examples.root_redirect_target_missing,
                    max_examples,
                    format!("{} -> {}", path.display(), to),
                );
                continue;
            }
            if !target.starts_with(&scripts_root) {
                counts.redirects_target_outside_library_total = counts
                    .redirects_target_outside_library_total
                    .saturating_add(1);
                push_bounded(
                    &mut examples.root_redirect_target_outside_library,
                    max_examples,
                    format!("{} -> {}", path.display(), to),
                );
            }
            match redirect_chain_depth(workspace_root, &path, MAX_REDIRECT_DEPTH) {
                Ok(depth) => {
                    counts.redirects_chain_depth_max = counts.redirects_chain_depth_max.max(depth);
                    if depth > 2 {
                        counts.redirects_chain_depth_gt2_total =
                            counts.redirects_chain_depth_gt2_total.saturating_add(1);
                        push_bounded(
                            &mut examples.root_redirect_chain_depth_gt2,
                            max_examples,
                            path.display().to_string(),
                        );
                    }
                }
                Err(err) => {
                    errors.push(format!(
                        "failed to resolve redirect chain for {}: {err}",
                        path.display()
                    ));
                }
            }
        } else {
            counts.root_non_redirect_total = counts.root_non_redirect_total.saturating_add(1);
            push_bounded(
                &mut examples.root_non_redirect,
                max_examples,
                path.display().to_string(),
            );
        }
    }

    if counts.root_invalid_json_total > 0 {
        errors.push(format!(
            "tools/diag-scripts/*.json contains invalid JSON files: {}",
            counts.root_invalid_json_total
        ));
    }
    if counts.redirects_missing_to_total > 0 {
        errors.push(format!(
            "tools/diag-scripts/*.json contains script_redirect stubs missing 'to': {}",
            counts.redirects_missing_to_total
        ));
    }
    if counts.redirects_target_missing_total > 0 {
        errors.push(format!(
            "tools/diag-scripts/*.json contains script_redirect stubs whose targets are missing: {}",
            counts.redirects_target_missing_total
        ));
    }
    if counts.root_non_redirect_total > 0 {
        errors.push(format!(
            "canonical scripts committed at tools/diag-scripts/*.json (expected only redirect stubs): {}",
            counts.root_non_redirect_total
        ));
        repairs.push(json!({
            "code": "diag-scripts.migrate-taxonomy",
            "note": "Move top-level scripts into taxonomy directories and leave redirect stubs behind.",
            "command": "python tools/diag-scripts/migrate-script-library.py --check-root && python tools/diag-scripts/migrate-script-library.py --apply --write-redirects",
        }));
    }
    if counts.redirects_target_outside_library_total > 0 {
        warnings.push(format!(
            "some script_redirect stubs point outside tools/diag-scripts/: {}",
            counts.redirects_target_outside_library_total
        ));
    }
    if counts.redirects_chain_depth_gt2_total > 0 {
        warnings.push(format!(
            "some script_redirect chains are deeper than 2 (consider flattening to reduce drift risk): {}",
            counts.redirects_chain_depth_gt2_total
        ));
    }

    // Check suites (`tools/diag-scripts/suites/**/*.json`) are redirect stubs with valid targets.
    let suites_root = scripts_root.join("suites");
    if suites_root.is_dir() {
        let suite_paths = json_glob_recursive(&suites_root, "**/*.json")?;
        for path in suite_paths {
            counts.suites_json_total = counts.suites_json_total.saturating_add(1);
            let Some(v) = read_json_value(&path) else {
                counts.suites_invalid_json_total =
                    counts.suites_invalid_json_total.saturating_add(1);
                push_bounded(
                    &mut examples.suite_invalid_json,
                    max_examples,
                    path.display().to_string(),
                );
                continue;
            };
            if !is_script_redirect(&v) {
                counts.suites_non_redirect_total =
                    counts.suites_non_redirect_total.saturating_add(1);
                push_bounded(
                    &mut examples.suite_non_redirect,
                    max_examples,
                    path.display().to_string(),
                );
                continue;
            }
            counts.suites_redirect_total = counts.suites_redirect_total.saturating_add(1);
            let Some(to) = read_redirect_to(&v) else {
                counts.suites_target_missing_total =
                    counts.suites_target_missing_total.saturating_add(1);
                push_bounded(
                    &mut examples.suite_target_missing,
                    max_examples,
                    format!("{} -> (missing to)", path.display()),
                );
                continue;
            };
            let target = resolve_path(workspace_root, PathBuf::from(&to));
            if !target.exists() {
                counts.suites_target_missing_total =
                    counts.suites_target_missing_total.saturating_add(1);
                push_bounded(
                    &mut examples.suite_target_missing,
                    max_examples,
                    format!("{} -> {}", path.display(), to),
                );
                continue;
            }

            if let Some(tv) = read_json_value(&target) {
                if is_script_redirect(&tv) {
                    counts.suites_redirect_target_is_redirect_total = counts
                        .suites_redirect_target_is_redirect_total
                        .saturating_add(1);
                }
            }
            match redirect_chain_depth(workspace_root, &path, MAX_REDIRECT_DEPTH) {
                Ok(depth) => {
                    counts.suites_redirect_chain_depth_max =
                        counts.suites_redirect_chain_depth_max.max(depth);
                    if depth > 2 {
                        counts.suites_redirect_chain_depth_gt2_total = counts
                            .suites_redirect_chain_depth_gt2_total
                            .saturating_add(1);
                        push_bounded(
                            &mut examples.suite_redirect_chain_depth_gt2,
                            max_examples,
                            path.display().to_string(),
                        );
                    }
                }
                Err(err) => {
                    errors.push(format!(
                        "failed to resolve suite redirect chain for {}: {err}",
                        path.display()
                    ));
                }
            }
        }
    } else {
        warnings.push("tools/diag-scripts/suites/ directory is missing".to_string());
    }

    if counts.suites_invalid_json_total > 0 {
        errors.push(format!(
            "tools/diag-scripts/suites/**/*.json contains invalid JSON files: {}",
            counts.suites_invalid_json_total
        ));
    }
    if counts.suites_non_redirect_total > 0 {
        errors.push(format!(
            "tools/diag-scripts/suites/**/*.json contains non-redirect scripts (expected script_redirect stubs): {}",
            counts.suites_non_redirect_total
        ));
    }
    if counts.suites_target_missing_total > 0 {
        errors.push(format!(
            "tools/diag-scripts/suites/**/*.json contains redirect stubs whose targets are missing: {}",
            counts.suites_target_missing_total
        ));
    }
    if counts.suites_redirect_chain_depth_gt2_total > 0 {
        warnings.push(format!(
            "some suite script_redirect chains are deeper than 2 (consider updating suite stubs): {}",
            counts.suites_redirect_chain_depth_gt2_total
        ));
    }

    // Check promoted registry (tools/diag-scripts/index.json).
    if !index_path.is_file() {
        errors.push(format!(
            "promoted scripts registry is missing: {}",
            index_path.display()
        ));
        repairs.push(json!({
            "code": "diag-scripts.registry-write",
            "note": "Regenerate tools/diag-scripts/index.json",
            "command": "python tools/check_diag_scripts_registry.py --write",
        }));
    } else {
        let registry = PromotedScriptRegistry::load_from_path(&index_path);
        match registry {
            Ok(registry) => {
                let mut seen_ids: BTreeSet<String> = BTreeSet::new();
                let mut duplicates: BTreeSet<String> = BTreeSet::new();
                let mut schema_v1_paths: BTreeSet<String> = BTreeSet::new();

                for entry in registry.entries() {
                    counts.registry_entries_total = counts.registry_entries_total.saturating_add(1);

                    if !seen_ids.insert(entry.id.clone()) {
                        duplicates.insert(entry.id.clone());
                        continue;
                    }

                    let script_path = resolve_path(workspace_root, PathBuf::from(&entry.path));
                    if !script_path.is_file() {
                        counts.registry_path_missing_total =
                            counts.registry_path_missing_total.saturating_add(1);
                        push_bounded(
                            &mut examples.registry_path_missing,
                            max_examples,
                            format!("{} -> {}", entry.id, entry.path),
                        );
                        continue;
                    }
                    if !script_path.starts_with(&scripts_root) {
                        counts.registry_path_outside_library_total =
                            counts.registry_path_outside_library_total.saturating_add(1);
                        push_bounded(
                            &mut examples.registry_path_outside_library,
                            max_examples,
                            format!("{} -> {}", entry.id, entry.path),
                        );
                    }
                    if let Some(v) = read_json_value(&script_path) {
                        if is_script_redirect(&v) {
                            counts.registry_path_is_redirect_total =
                                counts.registry_path_is_redirect_total.saturating_add(1);
                            push_bounded(
                                &mut examples.registry_path_is_redirect,
                                max_examples,
                                format!("{} -> {}", entry.id, entry.path),
                            );
                        }

                        let schema_version = v
                            .get("schema_version")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        if schema_version == 1 {
                            counts.registry_schema_v1_total =
                                counts.registry_schema_v1_total.saturating_add(1);
                            schema_v1_paths.insert(entry.path.clone());
                            push_bounded(
                                &mut examples.registry_schema_v1,
                                max_examples,
                                format!("{} -> {}", entry.id, entry.path),
                            );
                        }
                    }
                }

                if !duplicates.is_empty() {
                    counts.registry_duplicate_id_total = counts
                        .registry_duplicate_id_total
                        .saturating_add(duplicates.len() as u64);
                    for id in duplicates {
                        push_bounded(&mut examples.registry_duplicate_ids, max_examples, id);
                    }
                }

                if !schema_v1_paths.is_empty() {
                    warnings.push(format!(
                        "some promoted scripts are still schema_version=1 (tooling can upgrade them, but v2-only is preferred): {}",
                        schema_v1_paths.len()
                    ));
                    if strict {
                        errors.push(format!(
                            "strict: promoted scripts must be schema_version=2 (found schema v1): {}",
                            schema_v1_paths.len()
                        ));
                    }

                    let mut cmd =
                        String::from("cargo run -p fretboard -- diag script upgrade --write");
                    for p in &schema_v1_paths {
                        cmd.push(' ');
                        cmd.push_str(p);
                    }
                    repairs.push(json!({
                        "code": "diag-scripts.promoted-upgrade-v2",
                        "note": "Upgrade promoted scripts from schema v1 to schema v2 (writes in place).",
                        "command": cmd,
                    }));
                }
            }
            Err(e) => {
                errors.push(e);
                repairs.push(json!({
                    "code": "diag-scripts.registry-write",
                    "note": "Regenerate tools/diag-scripts/index.json",
                    "command": "python tools/check_diag_scripts_registry.py --write",
                }));
            }
        }
    }

    if counts.registry_duplicate_id_total > 0 {
        errors.push(format!(
            "promoted scripts registry contains duplicate ids: {}",
            counts.registry_duplicate_id_total
        ));
    }
    if counts.registry_path_missing_total > 0 {
        errors.push(format!(
            "promoted scripts registry contains missing paths: {}",
            counts.registry_path_missing_total
        ));
        repairs.push(json!({
            "code": "diag-scripts.registry-write",
            "note": "Regenerate tools/diag-scripts/index.json",
            "command": "python tools/check_diag_scripts_registry.py --write",
        }));
    }
    if counts.registry_path_outside_library_total > 0 {
        warnings.push(format!(
            "some promoted scripts resolve outside tools/diag-scripts/: {}",
            counts.registry_path_outside_library_total
        ));
    }
    if counts.registry_path_is_redirect_total > 0 {
        warnings.push(format!(
            "some promoted scripts point at script_redirect stubs (prefer canonical scripts): {}",
            counts.registry_path_is_redirect_total
        ));
    }

    // Stable ordering for JSON output.
    let mut counts_obj = BTreeMap::<String, Value>::new();
    counts_obj.insert("root_json_total".to_string(), json!(counts.root_json_total));
    counts_obj.insert(
        "root_redirect_total".to_string(),
        json!(counts.root_redirect_total),
    );
    counts_obj.insert(
        "root_non_redirect_total".to_string(),
        json!(counts.root_non_redirect_total),
    );
    counts_obj.insert(
        "root_invalid_json_total".to_string(),
        json!(counts.root_invalid_json_total),
    );
    counts_obj.insert(
        "redirects_missing_to_total".to_string(),
        json!(counts.redirects_missing_to_total),
    );
    counts_obj.insert(
        "redirects_target_missing_total".to_string(),
        json!(counts.redirects_target_missing_total),
    );
    counts_obj.insert(
        "redirects_target_outside_library_total".to_string(),
        json!(counts.redirects_target_outside_library_total),
    );
    counts_obj.insert(
        "redirects_chain_depth_gt2_total".to_string(),
        json!(counts.redirects_chain_depth_gt2_total),
    );
    counts_obj.insert(
        "redirects_chain_depth_max".to_string(),
        json!(counts.redirects_chain_depth_max),
    );
    counts_obj.insert(
        "suites_json_total".to_string(),
        json!(counts.suites_json_total),
    );
    counts_obj.insert(
        "suites_redirect_total".to_string(),
        json!(counts.suites_redirect_total),
    );
    counts_obj.insert(
        "suites_non_redirect_total".to_string(),
        json!(counts.suites_non_redirect_total),
    );
    counts_obj.insert(
        "suites_invalid_json_total".to_string(),
        json!(counts.suites_invalid_json_total),
    );
    counts_obj.insert(
        "suites_target_missing_total".to_string(),
        json!(counts.suites_target_missing_total),
    );
    counts_obj.insert(
        "suites_redirect_target_is_redirect_total".to_string(),
        json!(counts.suites_redirect_target_is_redirect_total),
    );
    counts_obj.insert(
        "suites_redirect_chain_depth_gt2_total".to_string(),
        json!(counts.suites_redirect_chain_depth_gt2_total),
    );
    counts_obj.insert(
        "suites_redirect_chain_depth_max".to_string(),
        json!(counts.suites_redirect_chain_depth_max),
    );
    counts_obj.insert(
        "registry_entries_total".to_string(),
        json!(counts.registry_entries_total),
    );
    counts_obj.insert(
        "registry_duplicate_id_total".to_string(),
        json!(counts.registry_duplicate_id_total),
    );
    counts_obj.insert(
        "registry_path_missing_total".to_string(),
        json!(counts.registry_path_missing_total),
    );
    counts_obj.insert(
        "registry_path_outside_library_total".to_string(),
        json!(counts.registry_path_outside_library_total),
    );
    counts_obj.insert(
        "registry_path_is_redirect_total".to_string(),
        json!(counts.registry_path_is_redirect_total),
    );
    counts_obj.insert(
        "registry_schema_v1_total".to_string(),
        json!(counts.registry_schema_v1_total),
    );

    let ok = errors.is_empty();

    let payload = json!({
        "ok": ok,
        "required_ok": ok,
        "scripts_root": scripts_root.to_string_lossy(),
        "registry_path": index_path.to_string_lossy(),
        "errors": errors,
        "warnings": warnings,
        "repairs": repairs,
        "counts": Value::Object(counts_obj.into_iter().collect()),
        "examples": {
            "root_non_redirect": examples.root_non_redirect,
            "root_invalid_json": examples.root_invalid_json,
            "root_redirect_missing_to": examples.root_redirect_missing_to,
            "root_redirect_target_missing": examples.root_redirect_target_missing,
            "root_redirect_target_outside_library": examples.root_redirect_target_outside_library,
            "root_redirect_chain_depth_gt2": examples.root_redirect_chain_depth_gt2,
            "suite_non_redirect": examples.suite_non_redirect,
            "suite_invalid_json": examples.suite_invalid_json,
            "suite_target_missing": examples.suite_target_missing,
            "suite_redirect_chain_depth_gt2": examples.suite_redirect_chain_depth_gt2,
            "registry_duplicate_ids": examples.registry_duplicate_ids,
            "registry_path_missing": examples.registry_path_missing,
            "registry_path_outside_library": examples.registry_path_outside_library,
            "registry_path_is_redirect": examples.registry_path_is_redirect,
            "registry_schema_v1": examples.registry_schema_v1,
        }
    });

    if stats_json {
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
        println!("{pretty}");
        if !ok {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("scripts_root: {}", scripts_root.display());
    println!("registry: {}", index_path.display());
    println!("ok: {ok}");

    let errors = payload
        .get("errors")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let warnings = payload
        .get("warnings")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    for e in errors {
        if let Some(s) = e.as_str() {
            println!("error: {s}");
        }
    }
    for w in warnings {
        if let Some(s) = w.as_str() {
            println!("warning: {s}");
        }
    }
    let repairs = payload
        .get("repairs")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    for r in repairs {
        let code = r.get("code").and_then(|v| v.as_str()).unwrap_or("");
        let command = r.get("command").and_then(|v| v.as_str());
        let note = r.get("note").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(command) = command {
            println!("repair: {code} ({note})");
            println!("  command: {command}");
        } else if !note.is_empty() {
            println!("repair: {code} ({note})");
        }
    }

    if !ok {
        println!("tip: re-run with `--json` for a structured report");
        std::process::exit(1);
    }

    Ok(())
}
