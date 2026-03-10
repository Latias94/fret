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

#[derive(Debug, PartialEq, Eq)]
enum DashboardOutputPresentation {
    Text(String),
    Lines(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardCountEntry {
    pub key: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardReasonCodeEntry {
    pub reason_code: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardFailingSummaryEntry {
    pub path: String,
    pub lane: String,
    pub failures: u64,
    pub items_total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardSummaryProjection {
    pub kind: Option<String>,
    pub out_dir: Option<String>,
    pub summaries_total: usize,
    pub items_total: u64,
    pub status_counters: Vec<DashboardCountEntry>,
    pub lane_counters: Vec<DashboardCountEntry>,
    pub tool_counters: Vec<DashboardCountEntry>,
    pub top_reason_codes: Vec<DashboardReasonCodeEntry>,
    pub failing_summaries: Vec<DashboardFailingSummaryEntry>,
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

    let presentation =
        build_dashboard_output_presentation(&index_path, &payload, opts.top, stats_json)?;
    emit_dashboard_output_presentation(presentation);
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

fn dashboard_counter_lines(entries: &[DashboardCountEntry], title: &str) -> Vec<String> {
    if entries.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![format!("{title}:")];
    for entry in entries {
        lines.push(format!("  {}: {}", entry.key, entry.count));
    }
    lines
}

fn dashboard_summaries_total(payload: &serde_json::Value) -> usize {
    payload
        .get("summaries")
        .and_then(|v| v.as_array())
        .map(|rows| rows.len())
        .unwrap_or(0)
}

fn dashboard_items_total(payload: &serde_json::Value) -> u64 {
    payload
        .pointer("/summaries")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .map(|row| row.get("items_total").and_then(|v| v.as_u64()).unwrap_or(0))
                .sum::<u64>()
        })
        .unwrap_or(0)
}

pub fn dashboard_counter_entries(
    payload: &serde_json::Value,
    pointer: &str,
) -> Vec<DashboardCountEntry> {
    let Some(obj) = payload.pointer(pointer).and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    let mut rows: Vec<DashboardCountEntry> = obj
        .iter()
        .filter_map(|(key, value)| {
            value.as_u64().map(|count| DashboardCountEntry {
                key: key.to_string(),
                count,
            })
        })
        .collect();
    rows.sort_by(|left, right| left.key.cmp(&right.key));
    rows
}

pub fn dashboard_reason_code_entries(
    payload: &serde_json::Value,
    top: usize,
) -> Vec<DashboardReasonCodeEntry> {
    let Some(rows) = payload.get("top_reason_codes").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    rows.iter()
        .take(top)
        .map(|row| DashboardReasonCodeEntry {
            reason_code: row
                .get("reason_code")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
                .to_string(),
            count: row.get("count").and_then(|v| v.as_u64()).unwrap_or(0),
        })
        .collect()
}

pub fn dashboard_failing_summary_entries(
    payload: &serde_json::Value,
    top: usize,
) -> Vec<DashboardFailingSummaryEntry> {
    let Some(rows) = payload.get("failing_summaries").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    rows.iter()
        .take(top)
        .map(|row| DashboardFailingSummaryEntry {
            path: row
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
                .to_string(),
            lane: row
                .get("lane")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>")
                .to_string(),
            failures: row.get("failures").and_then(|v| v.as_u64()).unwrap_or(0),
            items_total: row.get("items_total").and_then(|v| v.as_u64()).unwrap_or(0),
        })
        .collect()
}

pub fn project_dashboard_summary(
    payload: &serde_json::Value,
    top: usize,
) -> DashboardSummaryProjection {
    DashboardSummaryProjection {
        kind: payload
            .get("kind")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        out_dir: payload
            .get("out_dir")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        summaries_total: dashboard_summaries_total(payload),
        items_total: dashboard_items_total(payload),
        status_counters: dashboard_counter_entries(payload, "/counters/by_status"),
        lane_counters: dashboard_counter_entries(payload, "/counters/by_lane"),
        tool_counters: dashboard_counter_entries(payload, "/counters/by_tool"),
        top_reason_codes: dashboard_reason_code_entries(payload, top),
        failing_summaries: dashboard_failing_summary_entries(payload, top),
    }
}

pub fn dashboard_human_lines_from_projection(
    index_path: &Path,
    projection: &DashboardSummaryProjection,
) -> Vec<String> {
    let mut lines = vec![format!("regression index: {}", index_path.display())];
    if let Some(kind) = projection.kind.as_deref() {
        lines.push(format!("kind: {kind}"));
    }
    if let Some(out_dir) = projection.out_dir.as_deref() {
        lines.push(format!("out_dir: {out_dir}"));
    }

    lines.push(format!("summaries_total: {}", projection.summaries_total));
    lines.push(format!("items_total: {}", projection.items_total));

    lines.extend(dashboard_counter_lines(
        &projection.status_counters,
        "normalized status counters",
    ));
    lines.extend(dashboard_counter_lines(
        &projection.lane_counters,
        "canonical lane counters",
    ));
    lines.extend(dashboard_counter_lines(
        &projection.tool_counters,
        "tool counters",
    ));
    if !projection.top_reason_codes.is_empty() {
        lines.push("top reason codes:".to_string());
        for row in &projection.top_reason_codes {
            lines.push(format!("  {}: {}", row.reason_code, row.count));
        }
    }
    if !projection.failing_summaries.is_empty() {
        lines.push("non-passing summaries:".to_string());
        for row in &projection.failing_summaries {
            lines.push(format!(
                "  {} | lane={} failures={} items={}",
                row.path, row.lane, row.failures, row.items_total
            ));
        }
    }
    lines
}

fn dashboard_human_lines(
    index_path: &Path,
    payload: &serde_json::Value,
    top: usize,
) -> Vec<String> {
    let projection = project_dashboard_summary(payload, top);
    dashboard_human_lines_from_projection(index_path, &projection)
}

fn build_dashboard_output_presentation(
    index_path: &Path,
    payload: &serde_json::Value,
    top: usize,
    stats_json: bool,
) -> Result<DashboardOutputPresentation, String> {
    if stats_json {
        Ok(DashboardOutputPresentation::Text(
            serde_json::to_string_pretty(payload).map_err(|e| e.to_string())?,
        ))
    } else {
        Ok(DashboardOutputPresentation::Lines(dashboard_human_lines(
            index_path, payload, top,
        )))
    }
}

fn emit_dashboard_output_presentation(presentation: DashboardOutputPresentation) {
    match presentation {
        DashboardOutputPresentation::Text(text) => println!("{text}"),
        DashboardOutputPresentation::Lines(lines) => {
            for line in lines {
                println!("{line}");
            }
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

    #[test]
    fn dashboard_counter_lines_render_non_empty_map() {
        let payload = serde_json::json!({
            "counters": {
                "by_status": {
                    "passed": 3,
                    "failed_deterministic": 1
                }
            }
        });

        let entries = dashboard_counter_entries(&payload, "/counters/by_status");
        let lines = dashboard_counter_lines(&entries, "normalized status counters");

        assert_eq!(
            lines.first().map(String::as_str),
            Some("normalized status counters:")
        );
        assert!(lines.iter().any(|line| line == "  passed: 3"));
        assert!(lines.iter().any(|line| line == "  failed_deterministic: 1"));
    }

    #[test]
    fn dashboard_human_lines_include_summary_and_failure_sections() {
        let payload = serde_json::json!({
            "kind": "diag_regression_index",
            "out_dir": "F:/repo/out",
            "counters": {
                "by_status": { "failed_deterministic": 2 },
                "by_lane": { "smoke": 2 },
                "by_tool": { "fretboard diag suite": 2 }
            },
            "top_reason_codes": [
                { "reason_code": "assert.focus_restore.mismatch", "count": 2 }
            ],
            "failing_summaries": [
                { "path": "suite/regression.summary.json", "lane": "smoke", "failures": 2, "items_total": 4 }
            ],
            "summaries": [
                { "items_total": 4 }
            ]
        });

        let lines =
            dashboard_human_lines(Path::new("F:/repo/out/regression.index.json"), &payload, 5);

        assert_eq!(
            lines.first().map(String::as_str),
            Some("regression index: F:/repo/out/regression.index.json")
        );
        assert!(lines.iter().any(|line| line == "summaries_total: 1"));
        assert!(lines.iter().any(|line| line == "items_total: 4"));
        assert!(lines.iter().any(|line| line == "top reason codes:"));
        assert!(lines.iter().any(|line| line == "non-passing summaries:"));
        assert!(lines.iter().any(|line| {
            line == "  suite/regression.summary.json | lane=smoke failures=2 items=4"
        }));
    }

    #[test]
    fn project_dashboard_summary_and_human_lines_share_same_projection() {
        let payload = serde_json::json!({
            "kind": "diag_regression_index",
            "out_dir": "F:/repo/out",
            "counters": {
                "by_status": { "failed_deterministic": 2 },
                "by_lane": { "smoke": 2 },
                "by_tool": { "fretboard diag suite": 2 }
            },
            "top_reason_codes": [
                { "reason_code": "assert.focus_restore.mismatch", "count": 2 }
            ],
            "failing_summaries": [
                { "path": "suite/regression.summary.json", "lane": "smoke", "failures": 2, "items_total": 4 }
            ],
            "summaries": [
                { "items_total": 4 }
            ]
        });

        let projection = project_dashboard_summary(&payload, 5);
        let lines = dashboard_human_lines_from_projection(
            Path::new("F:/repo/out/regression.index.json"),
            &projection,
        );

        assert_eq!(projection.summaries_total, 1);
        assert_eq!(projection.items_total, 4);
        assert_eq!(projection.top_reason_codes.len(), 1);
        assert_eq!(projection.failing_summaries.len(), 1);
        assert!(lines.iter().any(|line| line == "top reason codes:"));
        assert!(lines.iter().any(|line| line == "non-passing summaries:"));
        assert!(lines.iter().any(|line| {
            line == "  suite/regression.summary.json | lane=smoke failures=2 items=4"
        }));
    }

    #[test]
    fn build_dashboard_output_presentation_uses_pretty_json_when_requested() {
        let payload = serde_json::json!({
            "kind": "diag_regression_index",
            "summaries": []
        });

        let presentation = build_dashboard_output_presentation(
            Path::new("F:/repo/out/regression.index.json"),
            &payload,
            5,
            true,
        )
        .expect("build dashboard presentation");

        match presentation {
            DashboardOutputPresentation::Text(text) => {
                assert!(text.contains("\"kind\": \"diag_regression_index\""));
            }
            DashboardOutputPresentation::Lines(_) => panic!("expected text presentation"),
        }
    }
}
