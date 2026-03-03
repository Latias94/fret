use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::script_tooling::{canonicalize_json_value, read_script_json_resolving_redirects};

const SUITE_MANIFEST_FILENAMES: [&str; 2] = ["suite.json", "_suite.json"];

fn is_script_redirect(v: &Value) -> bool {
    v.get("kind").and_then(|v| v.as_str()) == Some("script_redirect")
}

fn is_suite_manifest(v: &Value) -> bool {
    v.get("kind").and_then(|v| v.as_str()) == Some("diag_script_suite_manifest")
}

fn read_json_value(path: &Path) -> Result<Value, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("failed to read JSON {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| format!("failed to parse JSON {}: {e}", path.display()))
}

fn normalize_string_list(v: Option<&Value>) -> Vec<String> {
    let Some(arr) = v.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn repo_rel_slash(workspace_root: &Path, p: &Path) -> String {
    let rel = p.strip_prefix(workspace_root).unwrap_or(p);
    let mut out = String::new();
    for (idx, part) in rel.components().enumerate() {
        if idx > 0 {
            out.push('/');
        }
        out.push_str(&part.as_os_str().to_string_lossy());
    }
    out
}

fn collect_json_files_recursive(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let read_dir = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
        for entry in read_dir {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            out.push(path);
        }
        Ok(())
    }

    let mut out: Vec<PathBuf> = Vec::new();
    if root.is_dir() {
        walk(root, &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn suite_manifest_script_paths(manifest_path: &Path) -> Result<Vec<String>, String> {
    let v = read_json_value(manifest_path)?;
    if !is_suite_manifest(&v) {
        return Err(format!(
            "suite manifest must have kind=diag_script_suite_manifest: {}",
            manifest_path.display()
        ));
    }
    let Some(scripts) = v.get("scripts").and_then(|v| v.as_array()) else {
        return Err(format!(
            "invalid suite manifest (expected scripts[]): {}",
            manifest_path.display()
        ));
    };
    let mut out: Vec<String> = Vec::with_capacity(scripts.len());
    for item in scripts {
        let Some(s) = item.as_str() else {
            return Err(format!(
                "invalid suite manifest (scripts entries must be strings): {}",
                manifest_path.display()
            ));
        };
        let s = s.trim();
        if s.is_empty() {
            return Err(format!(
                "invalid suite manifest (scripts entries must be non-empty strings): {}",
                manifest_path.display()
            ));
        }
        out.push(s.to_string());
    }
    if out.is_empty() {
        return Err(format!(
            "suite manifest contains no scripts: {}",
            manifest_path.display()
        ));
    }
    Ok(out)
}

fn read_redirect_to(stub_path: &Path) -> Result<String, String> {
    let v = read_json_value(stub_path)?;
    if !is_script_redirect(&v) {
        return Err(format!(
            "suite entry is expected to be a script_redirect stub: {}",
            stub_path.display()
        ));
    }
    let Some(to) = v.get("to").and_then(|v| v.as_str()) else {
        return Err(format!(
            "invalid script_redirect stub (missing string field: to): {}",
            stub_path.display()
        ));
    };
    let to = to.trim();
    if to.is_empty() {
        return Err(format!(
            "invalid script_redirect stub (empty to): {}",
            stub_path.display()
        ));
    }
    Ok(to.to_string())
}

pub(crate) fn build_promoted_registry_payload(workspace_root: &Path) -> Result<Value, String> {
    let scripts_root = workspace_root.join("tools").join("diag-scripts");
    let suites_dir = scripts_root.join("suites");
    let prelude_dir = scripts_root.join("_prelude");

    if !suites_dir.is_dir() {
        return Err(format!("suites dir not found: {}", suites_dir.display()));
    }

    let mut canonical_to_suites: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();

    // 1) Suites: either legacy stubs under tools/diag-scripts/suites/<suite>/**/*.json,
    // or a single suite manifest under tools/diag-scripts/suites/<suite>/suite.json.
    let suites_read_dir = std::fs::read_dir(&suites_dir).map_err(|e| e.to_string())?;
    let mut suite_dirs: Vec<PathBuf> = Vec::new();
    for entry in suites_read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            suite_dirs.push(path);
        }
    }
    suite_dirs.sort();

    for suite_dir in suite_dirs {
        let Some(suite_name) = suite_dir.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let suite_name = suite_name.to_string();

        let manifest_path = SUITE_MANIFEST_FILENAMES
            .iter()
            .map(|name| suite_dir.join(name))
            .find(|p| p.is_file());

        if let Some(manifest_path) = manifest_path {
            // Disallow mixing suite manifest + legacy stubs (ambiguous membership ownership).
            let all_json = collect_json_files_recursive(&suite_dir)?;
            let mut other_json: Vec<PathBuf> = Vec::new();
            for p in all_json {
                if p == manifest_path {
                    continue;
                }
                other_json.push(p);
                if other_json.len() >= 10 {
                    break;
                }
            }
            if !other_json.is_empty() {
                let shown = other_json
                    .into_iter()
                    .map(|p| format!("- {}", repo_rel_slash(workspace_root, &p)))
                    .collect::<Vec<_>>()
                    .join("\n");
                return Err(format!(
                    "suite directory mixes suite manifest with legacy *.json stubs:\n\
- suite: {suite_name}\n\
- manifest: {}\n\
- other json files (first 10):\n{shown}\n\
hint: delete legacy stubs or remove the manifest",
                    repo_rel_slash(workspace_root, &manifest_path)
                ));
            }

            let script_paths = suite_manifest_script_paths(&manifest_path)?;
            for to in script_paths {
                let resolved = crate::paths::resolve_path(workspace_root, PathBuf::from(to));
                let resolved = read_script_json_resolving_redirects(&resolved)?;
                let canonical = resolved.write_path;
                canonical_to_suites
                    .entry(canonical)
                    .or_default()
                    .insert(suite_name.clone());
            }
            continue;
        }

        let stubs = collect_json_files_recursive(&suite_dir)?;
        for stub in stubs {
            let to = read_redirect_to(&stub)?;
            let resolved = crate::paths::resolve_path(workspace_root, PathBuf::from(to));
            let resolved = read_script_json_resolving_redirects(&resolved)?;
            let canonical = resolved.write_path;
            canonical_to_suites
                .entry(canonical)
                .or_default()
                .insert(suite_name.clone());
        }
    }

    // 2) Preludes: canonical scripts under tools/diag-scripts/_prelude/*.json.
    if prelude_dir.is_dir() {
        let read_dir = std::fs::read_dir(&prelude_dir).map_err(|e| e.to_string())?;
        let mut preludes: Vec<PathBuf> = Vec::new();
        for entry in read_dir {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            preludes.push(path);
        }
        preludes.sort();

        for p in preludes {
            let v = read_json_value(&p)?;
            if is_script_redirect(&v) {
                continue;
            }
            canonical_to_suites
                .entry(p)
                .or_default()
                .insert("_prelude".to_string());
        }
    }

    let mut entries: Vec<Value> = Vec::new();
    let mut seen_ids: BTreeMap<String, PathBuf> = BTreeMap::new();

    for (script_path, suites) in canonical_to_suites {
        let obj = read_json_value(&script_path)?;
        if is_script_redirect(&obj) {
            return Err(format!(
                "canonical set includes a redirect stub: {}",
                script_path.display()
            ));
        }

        let meta = obj.get("meta").and_then(|v| v.as_object());
        let entry_id = meta
            .and_then(|m| m.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                script_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });

        if let Some(prev) = seen_ids.insert(entry_id.clone(), script_path.clone()) {
            return Err(format!(
                "duplicate registry id detected (file stem collision).\n\
id={entry_id}\n\
a={}\n\
b={}",
                repo_rel_slash(workspace_root, &prev),
                repo_rel_slash(workspace_root, &script_path)
            ));
        }

        let tags = {
            let set: BTreeSet<String> = normalize_string_list(meta.and_then(|m| m.get("tags")))
                .into_iter()
                .collect();
            set.into_iter().collect::<Vec<_>>()
        };
        let required_capabilities = {
            let set: BTreeSet<String> =
                normalize_string_list(meta.and_then(|m| m.get("required_capabilities")))
                    .into_iter()
                    .collect();
            set.into_iter().collect::<Vec<_>>()
        };
        let target_hints = normalize_string_list(meta.and_then(|m| m.get("target_hints")));

        let mut suite_memberships = suites.into_iter().collect::<Vec<_>>();
        suite_memberships.sort();

        let rel = repo_rel_slash(workspace_root, &script_path);
        entries.push(json!({
            "id": entry_id,
            "path": rel,
            "required_capabilities": required_capabilities,
            "suite_memberships": suite_memberships,
            "tags": tags,
            "target_hints": target_hints,
        }));
    }

    entries.sort_by(|a, b| {
        let a_id = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let b_id = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        a_id.cmp(b_id)
    });

    let mut payload = json!({
        "kind": "diag_script_registry",
        "schema_version": 1,
        "scope": "suites+prelude",
        "scripts": entries,
    });
    canonicalize_json_value(&mut payload);
    Ok(payload)
}

pub(crate) fn promoted_registry_expected_bytes(workspace_root: &Path) -> Result<Vec<u8>, String> {
    let payload = build_promoted_registry_payload(workspace_root)?;
    let mut pretty = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    pretty.push('\n');
    Ok(pretty.into_bytes())
}
