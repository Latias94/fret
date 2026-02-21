use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryMode {
    Contains,
    Prefix,
    Glob,
}

impl QueryMode {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "contains" => Some(Self::Contains),
            "prefix" => Some(Self::Prefix),
            "glob" => Some(Self::Glob),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::Prefix => "prefix",
            Self::Glob => "glob",
        }
    }
}

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
}

fn resolve_latest_bundle_json_path(out_dir: &Path) -> Result<PathBuf, String> {
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(crate::resolve_bundle_json_path(&latest))
}

fn try_read_test_ids_index_json(path: &Path) -> Option<serde_json::Value> {
    let bytes = std::fs::read(path).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("test_ids_index") {
        return None;
    }
    Some(v)
}

fn resolve_test_ids_index_from_src(
    src: &Path,
    warmup_frames: u64,
) -> Result<(String, PathBuf, serde_json::Value), String> {
    if src.is_dir() {
        let direct = src.join("test_ids.index.json");
        if direct.is_file() {
            if let Some(v) = try_read_test_ids_index_json(&direct) {
                let bundle = v
                    .get("bundle")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| direct.display().to_string());
                return Ok((bundle, direct, v));
            }
        }

        let bundle_path = crate::resolve_bundle_json_path(src);
        let index_path =
            crate::bundle_index::ensure_test_ids_index_json(&bundle_path, warmup_frames)?;
        let v = try_read_test_ids_index_json(&index_path)
            .ok_or_else(|| "invalid test_ids.index.json".to_string())?;
        return Ok((bundle_path.display().to_string(), index_path, v));
    }

    if src.is_file()
        && src
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "test_ids.index.json")
    {
        let v = try_read_test_ids_index_json(src)
            .ok_or_else(|| format!("invalid test_ids.index.json: {}", src.display()))?;
        let bundle = v
            .get("bundle")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| src.display().to_string());
        return Ok((bundle, src.to_path_buf(), v));
    }

    let bundle_path = crate::resolve_bundle_json_path(src);
    let index_path = crate::bundle_index::ensure_test_ids_index_json(&bundle_path, warmup_frames)?;
    let v = try_read_test_ids_index_json(&index_path)
        .ok_or_else(|| "invalid test_ids.index.json".to_string())?;
    Ok((bundle_path.display().to_string(), index_path, v))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_query(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    query_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let Some(kind) = rest.first().map(|s| s.as_str()) else {
        return Err("missing query kind (try: fretboard diag query test-id <pattern>)".to_string());
    };

    match kind {
        "test-id" | "test_ids" => cmd_query_test_id(
            &rest[1..],
            workspace_root,
            out_dir,
            query_out,
            warmup_frames,
            stats_json,
        ),
        other => Err(format!("unknown query kind: {other}")),
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_query_test_id(
    rest: &[String],
    workspace_root: &Path,
    out_dir: &Path,
    query_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    let mut mode: QueryMode = QueryMode::Contains;
    let mut top: usize = 50;
    let mut case_sensitive: bool = false;

    let mut positionals: Vec<String> = Vec::new();
    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--mode" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --mode".to_string());
                };
                mode = QueryMode::from_str(v.as_str()).ok_or_else(|| {
                    "invalid value for --mode (expected contains|prefix|glob)".to_string()
                })?;
                i += 1;
            }
            "--top" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --top".to_string());
                };
                top = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --top (expected usize)".to_string())?;
                i += 1;
            }
            "--case-sensitive" => {
                case_sensitive = true;
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for query test-id: {other}"));
            }
            other => {
                positionals.push(other.to_string());
                i += 1;
            }
        }
    }

    if positionals.is_empty() {
        return Err("missing pattern (try: fretboard diag query test-id <pattern>)".to_string());
    }
    if positionals.len() > 2 {
        return Err(format!(
            "unexpected arguments: {}",
            positionals[2..].join(" ")
        ));
    }

    let (bundle_path, pattern) = match positionals.as_slice() {
        [bundle_src, pattern] => {
            let bundle_src = crate::resolve_path(workspace_root, PathBuf::from(bundle_src));
            (
                crate::resolve_bundle_json_path(&bundle_src),
                pattern.to_string(),
            )
        }
        [pattern] => {
            let maybe_path = crate::resolve_path(workspace_root, PathBuf::from(pattern));
            if looks_like_path(pattern) && (maybe_path.is_file() || maybe_path.is_dir()) {
                return Err(
                    "missing pattern (try: fretboard diag query test-id <bundle_dir|bundle.json> <pattern>)"
                        .to_string(),
                );
            }
            (
                resolve_latest_bundle_json_path(out_dir)?,
                pattern.to_string(),
            )
        }
        _ => unreachable!(),
    };

    let (bundle_label, index_path, index) = if positionals.len() == 2 {
        let src = crate::resolve_path(workspace_root, PathBuf::from(&positionals[0]));
        resolve_test_ids_index_from_src(&src, warmup_frames)?
    } else {
        let bundle_path = bundle_path;
        let index_path =
            crate::bundle_index::ensure_test_ids_index_json(&bundle_path, warmup_frames)?;
        let index = try_read_test_ids_index_json(&index_path)
            .ok_or_else(|| "invalid test_ids.index.json".to_string())?;
        (bundle_path.display().to_string(), index_path, index)
    };

    let truncated = index
        .get("truncated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let max_unique_test_ids_budget = index
        .get("max_unique_test_ids_budget")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let empty = Vec::new();
    let windows = index
        .get("windows")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);

    #[derive(Debug, Clone)]
    struct Agg {
        total: u64,
        windows_present: u32,
    }

    let mut by_id: HashMap<String, Agg> = HashMap::new();
    for w in windows {
        let items_empty = Vec::new();
        let items = w
            .get("items")
            .and_then(|v| v.as_array())
            .unwrap_or(&items_empty);

        let mut seen_in_window: HashSet<&str> = HashSet::new();
        for it in items {
            let Some(test_id) = it.get("test_id").and_then(|v| v.as_str()) else {
                continue;
            };
            let count = it.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            let entry = by_id.entry(test_id.to_string()).or_insert(Agg {
                total: 0,
                windows_present: 0,
            });
            entry.total = entry.total.saturating_add(count);
            seen_in_window.insert(test_id);
        }
        for test_id in seen_in_window {
            let entry = by_id.entry(test_id.to_string()).or_insert(Agg {
                total: 0,
                windows_present: 0,
            });
            entry.windows_present = entry.windows_present.saturating_add(1);
        }
    }

    let pat_norm = if case_sensitive {
        pattern.clone()
    } else {
        pattern.to_lowercase()
    };
    let glob_pat = if mode == QueryMode::Glob {
        Some(glob::Pattern::new(&pattern).map_err(|e| e.to_string())?)
    } else {
        None
    };

    let mut matches: Vec<(String, Agg)> = by_id
        .into_iter()
        .filter(|(test_id, _agg)| match mode {
            QueryMode::Contains => {
                if case_sensitive {
                    test_id.contains(&pattern)
                } else {
                    test_id.to_lowercase().contains(&pat_norm)
                }
            }
            QueryMode::Prefix => {
                if case_sensitive {
                    test_id.starts_with(&pattern)
                } else {
                    test_id.to_lowercase().starts_with(&pat_norm)
                }
            }
            QueryMode::Glob => glob_pat
                .as_ref()
                .is_some_and(|p| p.matches(test_id.as_str())),
        })
        .collect();

    matches.sort_by(|(a_id, a), (b_id, b)| {
        b.total
            .cmp(&a.total)
            .then_with(|| b.windows_present.cmp(&a.windows_present))
            .then_with(|| a_id.cmp(b_id))
    });
    if top > 0 && matches.len() > top {
        matches.truncate(top);
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "query.test_id",
        "bundle": bundle_label,
        "index": index_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "mode": mode.as_str(),
        "pattern": pattern,
        "case_sensitive": case_sensitive,
        "top": top,
        "truncated_index": truncated,
        "max_unique_test_ids_budget": max_unique_test_ids_budget,
        "results": matches.iter().map(|(test_id, agg)| serde_json::json!({
            "test_id": test_id,
            "count_total": agg.total,
            "windows_present": agg.windows_present,
        })).collect::<Vec<_>>(),
    });

    if let Some(out) = query_out.map(|p| crate::resolve_path(workspace_root, p)) {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
        std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;
        if !stats_json {
            println!("{}", out.display());
            return Ok(());
        }
    }

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(());
    }

    if truncated {
        eprintln!(
            "warning: test_ids.index.json truncated at max_unique_test_ids_budget={}",
            max_unique_test_ids_budget
        );
    }
    for (test_id, agg) in matches {
        println!(
            "{test_id} count_total={} windows_present={}",
            agg.total, agg.windows_present
        );
    }
    Ok(())
}
