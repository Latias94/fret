use super::*;

use crate::regression_summary::{
    DIAG_REGRESSION_INDEX_FILENAME_V1, DIAG_REGRESSION_SUMMARY_FILENAME_V1, RegressionArtifactsV1,
    RegressionCampaignSummaryV1, RegressionHighlightsV1, RegressionItemSummaryV1, RegressionLaneV1,
    RegressionRunSummaryV1, RegressionSourceV1, RegressionSummaryV1, RegressionTotalsV1,
};

const DIAG_REGRESSION_INDEX_KIND_V1: &str = "diag_regression_index";

#[derive(Debug, Clone)]
pub(crate) struct SummarizeCmdContext {
    pub rest: Vec<String>,
    pub workspace_root: PathBuf,
    pub resolved_out_dir: PathBuf,
    pub stats_json: bool,
}

#[derive(Debug, Clone)]
struct LoadedRegressionSummary {
    path: PathBuf,
    summary: RegressionSummaryV1,
}

pub(crate) fn cmd_summarize(ctx: SummarizeCmdContext) -> Result<(), String> {
    let SummarizeCmdContext {
        rest,
        workspace_root,
        resolved_out_dir,
        stats_json,
    } = ctx;

    let summary_paths = resolve_summary_inputs(&workspace_root, &resolved_out_dir, &rest)?;
    if summary_paths.is_empty() {
        return Err(format!(
            "no regression summaries found under {}",
            resolved_out_dir.display()
        ));
    }

    let loaded = load_regression_summaries(&summary_paths)?;
    let generated_unix_ms = now_unix_ms();
    let index_payload = regression_index_json(
        &workspace_root,
        &resolved_out_dir,
        generated_unix_ms,
        &loaded,
    );
    let index_path = resolved_out_dir.join(DIAG_REGRESSION_INDEX_FILENAME_V1);
    write_json_value(&index_path, &index_payload)?;

    let aggregate = aggregate_regression_summaries(
        &workspace_root,
        &resolved_out_dir,
        generated_unix_ms,
        &loaded,
        &index_path,
    );
    let summary_path = resolved_out_dir.join(DIAG_REGRESSION_SUMMARY_FILENAME_V1);
    write_json_value(
        &summary_path,
        &serde_json::to_value(&aggregate).unwrap_or_else(|_| serde_json::json!({})),
    )?;

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&aggregate).unwrap_or_else(|_| "{}".to_string())
        );
    } else {
        println!(
            "summarize: ok (summaries={}, items={}, failed={}, out_dir={})",
            loaded.len(),
            aggregate.totals.items_total,
            aggregate.totals.failed_deterministic
                + aggregate.totals.failed_flaky
                + aggregate.totals.failed_tooling
                + aggregate.totals.failed_timeout,
            resolved_out_dir.display()
        );
    }

    Ok(())
}

fn resolve_summary_inputs(
    workspace_root: &Path,
    resolved_out_dir: &Path,
    rest: &[String],
) -> Result<Vec<PathBuf>, String> {
    let mut summary_paths: Vec<PathBuf> = Vec::new();
    let skip_output_summary = resolved_out_dir.join(DIAG_REGRESSION_SUMMARY_FILENAME_V1);

    if rest.is_empty() {
        collect_regression_summaries_recursively(
            resolved_out_dir,
            &skip_output_summary,
            &mut summary_paths,
        )?;
    } else {
        for raw in rest {
            let path = resolve_summary_input_path(workspace_root, raw);
            if path.is_file() {
                if path.file_name().and_then(|v| v.to_str())
                    != Some(DIAG_REGRESSION_SUMMARY_FILENAME_V1)
                {
                    return Err(format!(
                        "expected a regression summary file named {}: {}",
                        DIAG_REGRESSION_SUMMARY_FILENAME_V1,
                        path.display()
                    ));
                }
                if path != skip_output_summary {
                    summary_paths.push(path);
                }
                continue;
            }
            if path.is_dir() {
                collect_regression_summaries_recursively(
                    &path,
                    &skip_output_summary,
                    &mut summary_paths,
                )?;
                continue;
            }
            return Err(format!("summary input not found: {}", path.display()));
        }
    }

    summary_paths.sort();
    summary_paths.dedup();
    Ok(summary_paths)
}

fn resolve_summary_input_path(workspace_root: &Path, raw: &str) -> PathBuf {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        candidate
    } else {
        workspace_root.join(candidate)
    }
}

fn collect_regression_summaries_recursively(
    dir: &Path,
    skip_output_summary: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            collect_regression_summaries_recursively(&path, skip_output_summary, out)?;
            continue;
        }
        if ty.is_file()
            && path.file_name().and_then(|v| v.to_str())
                == Some(DIAG_REGRESSION_SUMMARY_FILENAME_V1)
            && path != skip_output_summary
        {
            out.push(path);
        }
    }
    Ok(())
}

fn load_regression_summaries(paths: &[PathBuf]) -> Result<Vec<LoadedRegressionSummary>, String> {
    paths
        .iter()
        .map(|path| {
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let summary = serde_json::from_slice::<RegressionSummaryV1>(&bytes)
                .map_err(|e| format!("invalid regression summary {}: {}", path.display(), e))?;
            Ok(LoadedRegressionSummary {
                path: path.clone(),
                summary,
            })
        })
        .collect()
}

fn merge_source_metadata(
    existing: Option<serde_json::Value>,
    summary_path: &str,
    campaign_name: &str,
    campaign_lane: RegressionLaneV1,
) -> serde_json::Value {
    let mut obj = match existing {
        Some(serde_json::Value::Object(map)) => map,
        Some(other) => {
            let mut map = serde_json::Map::new();
            map.insert("inner".to_string(), other);
            map
        }
        None => serde_json::Map::new(),
    };
    obj.insert(
        "summary_path".to_string(),
        serde_json::Value::String(summary_path.to_string()),
    );
    obj.insert(
        "summary_campaign_name".to_string(),
        serde_json::Value::String(campaign_name.to_string()),
    );
    obj.insert(
        "summary_campaign_lane".to_string(),
        serde_json::Value::String(
            serde_json::to_value(campaign_lane)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "full".to_string()),
        ),
    );
    serde_json::Value::Object(obj)
}

fn aggregate_regression_summaries(
    workspace_root: &Path,
    resolved_out_dir: &Path,
    generated_unix_ms: u64,
    loaded: &[LoadedRegressionSummary],
    index_path: &Path,
) -> RegressionSummaryV1 {
    let mut totals = RegressionTotalsV1::default();
    let mut items: Vec<RegressionItemSummaryV1> = Vec::new();

    for loaded_summary in loaded {
        let summary_path = normalize_repo_relative_path(workspace_root, &loaded_summary.path);
        for item in &loaded_summary.summary.items {
            let mut item = item.clone();
            item.item_id = format!("{summary_path}::{}", item.item_id);
            let source = item.source.get_or_insert(RegressionSourceV1 {
                script: None,
                suite: None,
                campaign_case: None,
                metadata: None,
            });
            source.metadata = Some(merge_source_metadata(
                source.metadata.take(),
                &summary_path,
                &loaded_summary.summary.campaign.name,
                loaded_summary.summary.campaign.lane,
            ));
            totals.record_status(item.status);
            items.push(item);
        }
    }

    let mut summary = RegressionSummaryV1::new(
        RegressionCampaignSummaryV1 {
            name: "summary-index".to_string(),
            lane: RegressionLaneV1::Full,
            profile: Some("aggregate".to_string()),
            schema_version: Some(1),
            requested_by: Some("diag summarize".to_string()),
            filters: Some(serde_json::json!({
                "summaries": loaded.len(),
            })),
        },
        RegressionRunSummaryV1 {
            run_id: generated_unix_ms.to_string(),
            created_unix_ms: generated_unix_ms,
            started_unix_ms: None,
            finished_unix_ms: None,
            duration_ms: None,
            workspace_root: Some(workspace_root.display().to_string()),
            out_dir: Some(resolved_out_dir.display().to_string()),
            tool: "fretboard diag summarize".to_string(),
            tool_version: None,
            git_commit: None,
            git_branch: None,
            host: None,
        },
        totals,
    );
    summary.items = items;
    summary.highlights = RegressionHighlightsV1::from_items(&summary.items);
    summary.artifacts = Some(RegressionArtifactsV1 {
        summary_dir: Some(resolved_out_dir.display().to_string()),
        packed_report: None,
        index_json: Some(index_path.display().to_string()),
        html_report: None,
    });
    summary
}

fn regression_index_json(
    workspace_root: &Path,
    resolved_out_dir: &Path,
    generated_unix_ms: u64,
    loaded: &[LoadedRegressionSummary],
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": generated_unix_ms,
        "kind": DIAG_REGRESSION_INDEX_KIND_V1,
        "out_dir": resolved_out_dir.display().to_string(),
        "summaries": loaded.iter().map(|loaded_summary| {
            serde_json::json!({
                "path": normalize_repo_relative_path(workspace_root, &loaded_summary.path),
                "campaign": loaded_summary.summary.campaign,
                "run": {
                    "run_id": loaded_summary.summary.run.run_id,
                    "created_unix_ms": loaded_summary.summary.run.created_unix_ms,
                    "tool": loaded_summary.summary.run.tool,
                },
                "totals": loaded_summary.summary.totals,
                "items_total": loaded_summary.summary.items.len(),
                "artifacts": loaded_summary.summary.artifacts,
            })
        }).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fret-diag-{prefix}-{}-{}",
            std::process::id(),
            now_unix_ms()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_summary(name: &str, lane: RegressionLaneV1, item_id: &str) -> RegressionSummaryV1 {
        let mut totals = RegressionTotalsV1::default();
        totals.record_status(crate::regression_summary::RegressionStatusV1::Passed);
        let mut summary = RegressionSummaryV1::new(
            RegressionCampaignSummaryV1 {
                name: name.to_string(),
                lane,
                profile: None,
                schema_version: Some(1),
                requested_by: Some("test".to_string()),
                filters: None,
            },
            RegressionRunSummaryV1 {
                run_id: format!("run-{name}"),
                created_unix_ms: 1,
                started_unix_ms: None,
                finished_unix_ms: None,
                duration_ms: None,
                workspace_root: None,
                out_dir: None,
                tool: "fretboard diag suite".to_string(),
                tool_version: None,
                git_commit: None,
                git_branch: None,
                host: None,
            },
            totals,
        );
        summary.items.push(RegressionItemSummaryV1 {
            item_id: item_id.to_string(),
            kind: crate::regression_summary::RegressionItemKindV1::Script,
            name: item_id.to_string(),
            status: crate::regression_summary::RegressionStatusV1::Passed,
            reason_code: None,
            source_reason_code: None,
            lane,
            owner: None,
            feature_tags: Vec::new(),
            timing: None,
            attempts: None,
            evidence: None,
            source: Some(RegressionSourceV1 {
                script: Some(format!("scripts/{item_id}.json")),
                suite: None,
                campaign_case: None,
                metadata: None,
            }),
            notes: None,
        });
        summary
    }

    #[test]
    fn aggregate_regression_summaries_namespaces_item_ids() {
        let workspace_root = Path::new("F:/repo");
        let resolved_out_dir = Path::new("F:/repo/out");
        let loaded = vec![
            LoadedRegressionSummary {
                path: PathBuf::from(format!(
                    "F:/repo/out/suite/{}",
                    DIAG_REGRESSION_SUMMARY_FILENAME_V1
                )),
                summary: sample_summary("suite", RegressionLaneV1::Correctness, "case-a"),
            },
            LoadedRegressionSummary {
                path: PathBuf::from(format!(
                    "F:/repo/out/perf/{}",
                    DIAG_REGRESSION_SUMMARY_FILENAME_V1
                )),
                summary: sample_summary("perf", RegressionLaneV1::Perf, "case-a"),
            },
        ];

        let aggregate = aggregate_regression_summaries(
            workspace_root,
            resolved_out_dir,
            42,
            &loaded,
            Path::new("F:/repo/out/regression.index.json"),
        );

        assert_eq!(aggregate.totals.items_total, 2);
        assert_eq!(aggregate.items.len(), 2);
        assert_ne!(aggregate.items[0].item_id, aggregate.items[1].item_id);
        assert!(
            aggregate.items[0]
                .item_id
                .contains("suite/regression.summary.json::")
        );
        assert!(
            aggregate.items[1]
                .item_id
                .contains("perf/regression.summary.json::")
        );
        assert_eq!(
            aggregate
                .artifacts
                .as_ref()
                .and_then(|artifacts| artifacts.index_json.as_deref()),
            Some("F:/repo/out/regression.index.json")
        );
    }

    #[test]
    fn resolve_summary_inputs_discovers_nested_summaries_and_skips_root_output() {
        let root = temp_test_dir("summarize");
        let nested = root.join("suite-a");
        std::fs::create_dir_all(&nested).unwrap();
        write_json_value(
            &root.join(DIAG_REGRESSION_SUMMARY_FILENAME_V1),
            &serde_json::json!({"schema_version":1}),
        )
        .unwrap();
        write_json_value(
            &nested.join(DIAG_REGRESSION_SUMMARY_FILENAME_V1),
            &serde_json::json!({"schema_version":1}),
        )
        .unwrap();

        let found = resolve_summary_inputs(&root, &root, &[]).unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0], nested.join(DIAG_REGRESSION_SUMMARY_FILENAME_V1));

        let _ = std::fs::remove_dir_all(&root);
    }
}
