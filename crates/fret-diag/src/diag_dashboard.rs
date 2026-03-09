use std::path::{Path, PathBuf};

use crate::regression_summary::DIAG_REGRESSION_INDEX_FILENAME_V1;

#[derive(Debug, Clone)]
pub(crate) struct DashboardCmdContext {
    pub rest: Vec<String>,
    pub workspace_root: PathBuf,
    pub resolved_out_dir: PathBuf,
    pub stats_json: bool,
}

#[derive(Debug, Clone, Copy)]
struct DashboardOptions {
    top: usize,
}

pub(crate) fn cmd_dashboard(ctx: DashboardCmdContext) -> Result<(), String> {
    let DashboardCmdContext {
        rest,
        workspace_root,
        resolved_out_dir,
        stats_json,
    } = ctx;

    let (opts, positionals) = parse_dashboard_options(&rest)?;
    let index_path =
        resolve_dashboard_index_path(&workspace_root, &resolved_out_dir, &positionals)?;
    let payload = load_dashboard_index(&index_path)?;

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    print_dashboard_human(&index_path, &payload, opts.top);
    Ok(())
}

fn parse_dashboard_options(rest: &[String]) -> Result<(DashboardOptions, Vec<String>), String> {
    let mut out = DashboardOptions { top: 5 };
    let mut positionals = Vec::new();
    let mut idx = 0usize;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "--top" => {
                idx += 1;
                let Some(raw) = rest.get(idx) else {
                    return Err("missing value for --top".to_string());
                };
                out.top = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid value for --top: {raw}"))?
                    .max(1);
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown dashboard flag: {value}"));
            }
            other => positionals.push(other.to_string()),
        }
        idx += 1;
    }
    if positionals.len() > 1 {
        return Err(format!(
            "unexpected extra arguments for dashboard: {}",
            positionals[1..].join(" ")
        ));
    }
    Ok((out, positionals))
}

fn resolve_dashboard_index_path(
    workspace_root: &Path,
    resolved_out_dir: &Path,
    positionals: &[String],
) -> Result<PathBuf, String> {
    let base = match positionals.first() {
        Some(raw) => {
            let candidate = PathBuf::from(raw);
            if candidate.is_absolute() {
                candidate
            } else {
                workspace_root.join(candidate)
            }
        }
        None => resolved_out_dir.to_path_buf(),
    };

    let index_path = if base.is_dir() {
        base.join(DIAG_REGRESSION_INDEX_FILENAME_V1)
    } else {
        base
    };

    if index_path.is_file() {
        return Ok(index_path);
    }

    let summary_hint_path = if index_path.file_name().and_then(|v| v.to_str())
        == Some(DIAG_REGRESSION_INDEX_FILENAME_V1)
    {
        index_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        index_path.clone()
    };
    let summary_hint = summary_hint_path.join("regression.summary.json");
    if summary_hint.is_file() {
        return Err(format!(
            "dashboard index is missing: {}\nhint: run `fretboard diag summarize --dir {}` first",
            index_path.display(),
            summary_hint_path.display()
        ));
    }

    Err(format!(
        "dashboard index not found: {}",
        index_path.display()
    ))
}

fn load_dashboard_index(index_path: &Path) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(index_path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes)
        .map_err(|e| format!("invalid dashboard index {}: {}", index_path.display(), e))
}

fn print_counter_map(payload: &serde_json::Value, pointer: &str, title: &str) {
    let Some(obj) = payload.pointer(pointer).and_then(|v| v.as_object()) else {
        return;
    };
    if obj.is_empty() {
        return;
    }
    println!("{title}:");
    for (key, value) in obj {
        if let Some(count) = value.as_u64() {
            println!("  {key}: {count}");
        }
    }
}

fn print_dashboard_human(index_path: &Path, payload: &serde_json::Value, top: usize) {
    println!("dashboard index: {}", index_path.display());
    if let Some(kind) = payload.get("kind").and_then(|v| v.as_str()) {
        println!("kind: {kind}");
    }
    if let Some(out_dir) = payload.get("out_dir").and_then(|v| v.as_str()) {
        println!("out_dir: {out_dir}");
    }

    let summaries_total = payload
        .get("summaries")
        .and_then(|v| v.as_array())
        .map(|rows| rows.len())
        .unwrap_or(0);
    let items_total = payload
        .pointer("/summaries")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .map(|row| row.get("items_total").and_then(|v| v.as_u64()).unwrap_or(0))
                .sum::<u64>()
        })
        .unwrap_or(0);
    println!("summaries_total: {summaries_total}");
    println!("items_total: {items_total}");

    print_counter_map(payload, "/counters/by_status", "status counters");
    print_counter_map(payload, "/counters/by_lane", "lane counters");
    print_counter_map(payload, "/counters/by_tool", "tool counters");

    if let Some(rows) = payload.get("top_reason_codes").and_then(|v| v.as_array())
        && !rows.is_empty()
    {
        println!("top reason codes:");
        for row in rows.iter().take(top) {
            let reason_code = row
                .get("reason_code")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            let count = row.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            println!("  {reason_code}: {count}");
        }
    }

    if let Some(rows) = payload.get("failing_summaries").and_then(|v| v.as_array())
        && !rows.is_empty()
    {
        println!("failing summaries:");
        for row in rows.iter().take(top) {
            let path = row
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            let failures = row.get("failures").and_then(|v| v.as_u64()).unwrap_or(0);
            let items_total = row.get("items_total").and_then(|v| v.as_u64()).unwrap_or(0);
            let lane = row
                .get("lane")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            println!("  {path} | lane={lane} failures={failures} items={items_total}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dashboard_options_accepts_top_and_one_positional() {
        let (opts, positionals) = parse_dashboard_options(&[
            "--top".to_string(),
            "7".to_string(),
            "target/out".to_string(),
        ])
        .expect("parse options");
        assert_eq!(opts.top, 7);
        assert_eq!(positionals, vec!["target/out".to_string()]);
    }

    #[test]
    fn resolve_dashboard_index_path_uses_dir_default_filename() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-dashboard-{}-{}",
            std::process::id(),
            crate::util::now_unix_ms()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let index_path = root.join(DIAG_REGRESSION_INDEX_FILENAME_V1);
        std::fs::write(&index_path, b"{}").unwrap();

        let resolved = resolve_dashboard_index_path(Path::new("F:/repo"), &root, &[]).unwrap();
        assert_eq!(resolved, index_path);

        let _ = std::fs::remove_dir_all(&root);
    }
}
