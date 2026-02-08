use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Best-effort discovery of JSON Pointers that are convenient to edit in Script Studio.
///
/// This is intentionally heuristic and tolerant of invalid/partial scripts.
pub fn collect_common_json_pointers(script_text: &str) -> Vec<String> {
    let v: serde_json::Value = match serde_json::from_str(script_text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut push = |p: String| {
        if seen.insert(p.clone()) {
            out.push(p);
        }
    };

    // Common root-level pointers.
    if v.get("schema_version").is_some() {
        push("/schema_version".to_string());
    }
    if v.get("name").is_some() {
        push("/name".to_string());
    }
    if v.get("steps").is_some() {
        push("/steps".to_string());
    }

    let steps = v.get("steps").and_then(|x| x.as_array());
    let Some(steps) = steps else { return out };

    for (i, step) in steps.iter().enumerate() {
        let Some(obj) = step.as_object() else {
            continue;
        };

        let prefix = format!("/steps/{i}");
        push(prefix.clone());

        // Common step fields.
        for key in [
            "type",
            "target",
            "predicate",
            "container",
            "from",
            "to",
            "menu",
            "item",
            "a",
            "b",
        ] {
            if obj.contains_key(key) {
                push(format!("{prefix}/{key}"));
            }
        }

        // Common nested values.
        if obj.contains_key("path") {
            push(format!("{prefix}/path"));
            push(format!("{prefix}/path/0"));
        }
    }

    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptOrigin {
    WorkspaceTools,
    UserLocal,
}

impl ScriptOrigin {
    pub fn label(&self) -> &'static str {
        match self {
            Self::WorkspaceTools => "tools",
            Self::UserLocal => "user",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptPaths {
    pub workspace_tools_dir: PathBuf,
    pub user_scripts_dir: PathBuf,
}

impl ScriptPaths {
    pub fn from_repo_root(repo_root: PathBuf) -> Self {
        Self {
            workspace_tools_dir: repo_root.join("tools").join("diag-scripts"),
            user_scripts_dir: repo_root.join(".fret").join("diag").join("scripts"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptItem {
    pub origin: ScriptOrigin,
    pub file_name: Arc<str>,
    pub path: PathBuf,
}

pub fn repo_root_from_manifest_dir() -> Option<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let apps_dir = manifest_dir.parent()?;
    apps_dir.parent().map(|p| p.to_path_buf())
}

pub fn scan_script_library(paths: &ScriptPaths) -> Vec<ScriptItem> {
    let mut out = Vec::new();

    scan_dir(
        &mut out,
        ScriptOrigin::WorkspaceTools,
        &paths.workspace_tools_dir,
    );
    scan_dir(&mut out, ScriptOrigin::UserLocal, &paths.user_scripts_dir);

    out.sort_by(|a, b| {
        a.origin
            .label()
            .cmp(b.origin.label())
            .then_with(|| a.file_name.cmp(&b.file_name))
            .then_with(|| a.path.cmp(&b.path))
    });

    out
}

fn scan_dir(out: &mut Vec<ScriptItem>, origin: ScriptOrigin, dir: &Path) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        out.push(ScriptItem {
            origin,
            file_name: Arc::from(file_name),
            path,
        });
    }
}

pub fn load_script_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

pub fn ensure_user_scripts_dir(paths: &ScriptPaths) -> Result<(), String> {
    std::fs::create_dir_all(&paths.user_scripts_dir).map_err(|e| e.to_string())
}

pub fn fork_script_to_user(paths: &ScriptPaths, item: &ScriptItem) -> Result<ScriptItem, String> {
    if item.origin != ScriptOrigin::WorkspaceTools {
        return Err("only workspace scripts can be forked".to_string());
    }

    ensure_user_scripts_dir(paths)?;

    let file_name = item.file_name.as_ref();
    let dest = unique_dest_path(&paths.user_scripts_dir, file_name);
    std::fs::copy(&item.path, &dest).map_err(|e| e.to_string())?;

    Ok(ScriptItem {
        origin: ScriptOrigin::UserLocal,
        file_name: Arc::from(
            dest.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(file_name),
        ),
        path: dest,
    })
}

pub fn save_user_script(paths: &ScriptPaths, path: &Path, text: &str) -> Result<(), String> {
    if !is_descendant_of(path, &paths.user_scripts_dir) {
        return Err("refusing to write: not under .fret/diag/scripts".to_string());
    }
    ensure_user_scripts_dir(paths)?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn apply_pick_to_json_pointer(
    script_text: &str,
    pointer: &str,
    pick_result_payload_text: &str,
) -> Result<String, String> {
    let mut script: serde_json::Value =
        serde_json::from_str(script_text).map_err(|e| format!("script json parse failed: {e}"))?;

    if pointer.trim().is_empty() {
        return Err("json pointer is empty".to_string());
    }
    if !pointer.starts_with('/') {
        return Err("json pointer must start with '/'".to_string());
    }

    let pick_payload: serde_json::Value = serde_json::from_str(pick_result_payload_text)
        .map_err(|e| format!("pick json parse failed: {e}"))?;
    let selector = best_selector_value_from_pick_payload(&pick_payload)
        .ok_or_else(|| "pick has no selectors".to_string())?;

    let Some(slot) = script.pointer_mut(pointer) else {
        return Err("json pointer not found in script".to_string());
    };
    *slot = selector;

    serde_json::to_string_pretty(&script).map_err(|e| e.to_string())
}

pub fn apply_json_value_to_json_pointer(
    script_text: &str,
    pointer: &str,
    value: serde_json::Value,
) -> Result<String, String> {
    let mut script: serde_json::Value =
        serde_json::from_str(script_text).map_err(|e| format!("script json parse failed: {e}"))?;

    if pointer.trim().is_empty() {
        return Err("json pointer is empty".to_string());
    }
    if !pointer.starts_with('/') {
        return Err("json pointer must start with '/'".to_string());
    }

    let Some(slot) = script.pointer_mut(pointer) else {
        return Err("json pointer not found in script".to_string());
    };
    *slot = value;

    serde_json::to_string_pretty(&script).map_err(|e| e.to_string())
}

pub fn append_step_json(script_text: &str, step: serde_json::Value) -> Result<String, String> {
    insert_step_json(script_text, usize::MAX, step)
}

pub fn insert_step_json(
    script_text: &str,
    index: usize,
    step: serde_json::Value,
) -> Result<String, String> {
    let mut script: serde_json::Value =
        serde_json::from_str(script_text).map_err(|e| format!("script json parse failed: {e}"))?;

    let Some(obj) = script.as_object_mut() else {
        return Err("script root must be an object".to_string());
    };
    if !obj.contains_key("steps") {
        obj.insert("steps".to_string(), serde_json::Value::Array(Vec::new()));
    }

    let steps = obj
        .get_mut("steps")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "script.steps must be an array".to_string())?;

    if index >= steps.len() {
        steps.push(step);
    } else {
        steps.insert(index, step);
    }

    serde_json::to_string_pretty(&script).map_err(|e| e.to_string())
}

fn best_selector_value_from_pick_payload(
    pick_payload: &serde_json::Value,
) -> Option<serde_json::Value> {
    let selectors = pick_payload
        .pointer("/selection/selectors")
        .and_then(|v| v.as_array())?;

    selectors.iter().cloned().min_by_key(|v| selector_rank(v))
}

fn selector_rank(v: &serde_json::Value) -> u8 {
    let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    match kind {
        "test_id" => 0,
        "role_and_path" => 1,
        "role_and_name" => 2,
        "node_id" => 3,
        "global_element_id" => 4,
        _ => 10,
    }
}

fn unique_dest_path(dir: &Path, file_name: &str) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("script");
    let ext = Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("json");

    for i in 1..=999u32 {
        let name = format!("{stem}-fork-{i:03}.{ext}");
        let candidate = dir.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }

    dir.join(format!("{stem}-fork.{ext}"))
}

fn is_descendant_of(path: &Path, dir: &Path) -> bool {
    let Ok(path) = path.canonicalize() else {
        return false;
    };
    let Ok(dir) = dir.canonicalize() else {
        return false;
    };
    path.starts_with(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_pick_prefers_test_id_selector() {
        let script = serde_json::json!({
            "schema_version": 2,
            "steps": [
                {"type":"click","target":{"kind":"node_id","node": 1}, "button":"left"}
            ]
        });
        let pick = serde_json::json!({
            "schema_version": 1,
            "run_id": 1,
            "updated_unix_ms": 0,
            "window": 1,
            "stage": "picked",
            "selection": {
                "node": {"id": 1, "role": "button", "name": "OK"},
                "selectors": [
                    {"kind":"role_and_name","role":"button","name":"OK"},
                    {"kind":"test_id","id":"ok_button"}
                ]
            }
        });

        let updated = apply_pick_to_json_pointer(
            &serde_json::to_string_pretty(&script).unwrap(),
            "/steps/0/target",
            &serde_json::to_string_pretty(&pick).unwrap(),
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(
            v.pointer("/steps/0/target/kind").and_then(|x| x.as_str()),
            Some("test_id")
        );
    }

    #[test]
    fn collect_common_json_pointers_discovers_step_slots() {
        let script = serde_json::json!({
            "schema_version": 2,
            "steps": [
                {"type":"click","target":{"kind":"node_id","node": 1}, "button":"left"},
                {"type":"assert","predicate":{"kind":"exists","target":{"kind":"test_id","id":"x"}}},
                {"type":"drag_to","from":{"kind":"test_id","id":"a"},"to":{"kind":"test_id","id":"b"}},
                {"type":"assert","predicate":{"kind":"bounds_overlapping_x","a":{"kind":"test_id","id":"a"},"b":{"kind":"test_id","id":"b"},"eps_px": 0.0}},
                {"type":"click_path","path":[{"kind":"test_id","id":"x"}]},
            ]
        });

        let pointers =
            collect_common_json_pointers(&serde_json::to_string_pretty(&script).unwrap());
        assert!(pointers.contains(&"/schema_version".to_string()));
        assert!(pointers.contains(&"/steps".to_string()));
        assert!(pointers.contains(&"/steps/0/target".to_string()));
        assert!(pointers.contains(&"/steps/1/predicate".to_string()));
        assert!(pointers.contains(&"/steps/2/from".to_string()));
        assert!(pointers.contains(&"/steps/2/to".to_string()));
        assert!(pointers.contains(&"/steps/3/a".to_string()));
        assert!(pointers.contains(&"/steps/3/b".to_string()));
        assert!(pointers.contains(&"/steps/4/path/0".to_string()));
    }
}
