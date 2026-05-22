use crate::util::shell_quote_arg;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const DIAG_REGRESSION_SUMMARY_KIND_V1: &str = "diag_regression_summary";
pub const DIAG_REGRESSION_SUMMARY_FILENAME_V1: &str = "regression.summary.json";
pub const DIAG_REGRESSION_INDEX_FILENAME_V1: &str = "regression.index.json";
pub const DIAG_MATRIX_SUMMARY_FILENAME_V1: &str = "matrix.summary.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionSummaryV1 {
    pub schema_version: u32,
    pub kind: String,
    pub campaign: RegressionCampaignSummaryV1,
    pub run: RegressionRunSummaryV1,
    pub totals: RegressionTotalsV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<RegressionItemSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlights: Option<RegressionHighlightsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<RegressionArtifactsV1>,
}

impl RegressionSummaryV1 {
    pub fn new(
        campaign: RegressionCampaignSummaryV1,
        run: RegressionRunSummaryV1,
        totals: RegressionTotalsV1,
    ) -> Self {
        Self {
            schema_version: 1,
            kind: DIAG_REGRESSION_SUMMARY_KIND_V1.to_string(),
            campaign,
            run,
            totals,
            items: Vec::new(),
            highlights: None,
            artifacts: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegressionSummaryDrilldownV1 {
    pub bundle_dirs: Vec<String>,
    pub capability_sources: Vec<String>,
    pub capabilities_check_paths: Vec<String>,
    pub perf_evidence_lines: Vec<String>,
    pub first_open_evidence_lines: Vec<String>,
    pub share_artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionBundleFollowupCommandV1 {
    pub id: String,
    pub label: String,
    pub command_line: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diag_args: Vec<String>,
    #[serde(default)]
    pub requires_baseline: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_bundle_dir: Option<String>,
}

impl RegressionBundleFollowupCommandV1 {
    pub fn display_line(&self) -> String {
        format!("{}: {}", self.label, self.command_line)
    }
}

pub const REGRESSION_PERF_DRILLDOWN_METRIC_KEYS: &[&str] = &[
    "top_total_time_us",
    "top_layout_time_us",
    "top_layout_engine_solve_time_us",
    "pointer_move_max_dispatch_time_us",
    "pointer_move_max_hit_test_time_us",
    "pointer_move_snapshots_with_global_changes",
    "top_renderer_encode_scene_us",
    "top_renderer_prepare_text_us",
    "top_renderer_draw_calls",
    "top_renderer_instance_bytes",
    "top_renderer_encode_scene_text_ops",
];

pub fn regression_summary_drilldown(summary: &RegressionSummaryV1) -> RegressionSummaryDrilldownV1 {
    let mut drilldown = RegressionSummaryDrilldownV1::default();
    for item in &summary.items {
        for line in regression_item_perf_evidence_lines(item) {
            push_unique_line(&mut drilldown.perf_evidence_lines, line);
        }
        if item.status == RegressionStatusV1::Passed {
            continue;
        }
        if let Some(source) = regression_item_capability_source_display(item) {
            push_unique_line(&mut drilldown.capability_sources, source);
        }
        if let Some(evidence) = item.evidence.as_ref() {
            for dir in regression_item_perf_threshold_bundle_dirs(evidence) {
                push_unique_line(&mut drilldown.bundle_dirs, dir);
            }
            if let Some(dir) = regression_evidence_bundle_dir(evidence) {
                push_unique_line(&mut drilldown.bundle_dirs, dir);
            }
            if let Some(path) = evidence
                .extra
                .as_ref()
                .and_then(|extra| extra.get("capabilities_check_path"))
                .and_then(|value| value.as_str())
                .filter(|path| !path.trim().is_empty())
            {
                push_unique_line(&mut drilldown.capabilities_check_paths, path.to_string());
            }
            for line in regression_item_first_open_evidence_lines(item) {
                push_unique_line(&mut drilldown.first_open_evidence_lines, line);
            }
            for path in regression_item_share_artifacts(evidence) {
                push_unique_line(&mut drilldown.share_artifacts, path);
            }
        }
    }
    drilldown
}

pub fn regression_bundle_followup_commands<'a>(
    bundle_dirs: impl IntoIterator<Item = &'a str>,
) -> Vec<RegressionBundleFollowupCommandV1> {
    bundle_dirs
        .into_iter()
        .map(str::trim)
        .filter(|dir| !dir.is_empty())
        .fold(Vec::<&str>::new(), |mut dirs, dir| {
            if !dirs.iter().any(|existing| *existing == dir) {
                dirs.push(dir);
            }
            dirs
        })
        .into_iter()
        .enumerate()
        .flat_map(|(index, bundle_dir)| regression_bundle_followup_commands_for(index, bundle_dir))
        .collect()
}

fn indexed_followup_id(base: &str, index: usize) -> String {
    if index == 0 {
        base.to_string()
    } else {
        format!("{base}-{}", index + 1)
    }
}

fn indexed_followup_label(base: &str, index: usize) -> String {
    if index == 0 {
        base.to_string()
    } else {
        format!("{base} [{}]", index + 1)
    }
}

fn regression_bundle_followup_commands_for(
    index: usize,
    bundle_dir: &str,
) -> Vec<RegressionBundleFollowupCommandV1> {
    let bundle_arg = shell_quote_arg(bundle_dir);
    vec![
        RegressionBundleFollowupCommandV1 {
            id: indexed_followup_id("stats", index),
            label: indexed_followup_label("diag stats", index),
            command_line: format!("cargo run -p fretboard-dev -- diag stats {bundle_arg} --json"),
            diag_args: vec![
                "stats".to_string(),
                bundle_dir.to_string(),
                "--json".to_string(),
            ],
            requires_baseline: false,
            target_bundle_dir: Some(bundle_dir.to_string()),
        },
        RegressionBundleFollowupCommandV1 {
            id: indexed_followup_id("layout-perf-summary", index),
            label: indexed_followup_label("layout perf summary", index),
            command_line: format!(
                "cargo run -p fretboard-dev -- diag layout-perf-summary {bundle_arg} --json"
            ),
            diag_args: vec![
                "layout-perf-summary".to_string(),
                bundle_dir.to_string(),
                "--json".to_string(),
            ],
            requires_baseline: false,
            target_bundle_dir: Some(bundle_dir.to_string()),
        },
        RegressionBundleFollowupCommandV1 {
            id: indexed_followup_id("memory-summary", index),
            label: indexed_followup_label("memory summary", index),
            command_line: format!(
                "cargo run -p fretboard-dev -- diag memory-summary {bundle_arg} --json"
            ),
            diag_args: vec![
                "memory-summary".to_string(),
                bundle_dir.to_string(),
                "--json".to_string(),
            ],
            requires_baseline: false,
            target_bundle_dir: Some(bundle_dir.to_string()),
        },
        RegressionBundleFollowupCommandV1 {
            id: indexed_followup_id("triage", index),
            label: indexed_followup_label("triage", index),
            command_line: format!("cargo run -p fretboard-dev -- diag triage {bundle_arg} --json"),
            diag_args: vec![
                "triage".to_string(),
                bundle_dir.to_string(),
                "--json".to_string(),
            ],
            requires_baseline: false,
            target_bundle_dir: Some(bundle_dir.to_string()),
        },
        RegressionBundleFollowupCommandV1 {
            id: indexed_followup_id("hotspots", index),
            label: indexed_followup_label("hotspots", index),
            command_line: format!(
                "cargo run -p fretboard-dev -- diag hotspots {bundle_arg} --json"
            ),
            diag_args: vec![
                "hotspots".to_string(),
                bundle_dir.to_string(),
                "--json".to_string(),
            ],
            requires_baseline: false,
            target_bundle_dir: Some(bundle_dir.to_string()),
        },
        RegressionBundleFollowupCommandV1 {
            id: indexed_followup_id("trace", index),
            label: indexed_followup_label("trace", index),
            command_line: format!("cargo run -p fretboard-dev -- diag trace {bundle_arg} --json"),
            diag_args: vec![
                "trace".to_string(),
                bundle_dir.to_string(),
                "--json".to_string(),
            ],
            requires_baseline: false,
            target_bundle_dir: Some(bundle_dir.to_string()),
        },
        RegressionBundleFollowupCommandV1 {
            id: indexed_followup_id("visual-compare", index),
            label: indexed_followup_label("visual compare", index),
            command_line: format!(
                "cargo run -p fretboard-dev -- diag compare <baseline-bundle-or-dir> {bundle_arg} --json"
            ),
            diag_args: Vec::new(),
            requires_baseline: true,
            target_bundle_dir: Some(bundle_dir.to_string()),
        },
        RegressionBundleFollowupCommandV1 {
            id: indexed_followup_id("footprint-compare", index),
            label: indexed_followup_label("footprint compare", index),
            command_line: format!(
                "cargo run -p fretboard-dev -- diag compare <baseline-session> {bundle_arg} --footprint --json"
            ),
            diag_args: Vec::new(),
            requires_baseline: true,
            target_bundle_dir: Some(bundle_dir.to_string()),
        },
    ]
}

pub fn regression_bundle_followup_command_lines<'a>(
    bundle_dirs: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    let bundle_dirs = bundle_dirs
        .into_iter()
        .map(str::trim)
        .filter(|dir| !dir.is_empty())
        .fold(Vec::<&str>::new(), |mut dirs, dir| {
            if !dirs.iter().any(|existing| *existing == dir) {
                dirs.push(dir);
            }
            dirs
        });
    if bundle_dirs.is_empty() {
        return Vec::new();
    }
    let mut lines = if bundle_dirs.len() == 1 {
        vec![format!("selected bundle: {}", bundle_dirs[0])]
    } else {
        bundle_dirs
            .iter()
            .enumerate()
            .map(|(index, dir)| format!("selected bundle[{}]: {dir}", index + 1))
            .collect()
    };
    lines.extend(
        regression_bundle_followup_commands(bundle_dirs)
            .into_iter()
            .map(|command| command.display_line()),
    );
    lines
}

fn push_unique_line(lines: &mut Vec<String>, line: String) {
    if !lines.iter().any(|existing| existing == &line) {
        lines.push(line);
    }
}

fn bundle_root_dir_string_from_artifact(path: &str) -> Option<String> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    crate::resolve_bundle_root_dir(std::path::Path::new(path))
        .ok()
        .map(|path| path.display().to_string())
}

fn regression_evidence_bundle_dir(evidence: &RegressionEvidenceV1) -> Option<String> {
    evidence
        .bundle_dir
        .as_deref()
        .filter(|dir| !dir.trim().is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            evidence
                .bundle_artifact
                .as_deref()
                .and_then(bundle_root_dir_string_from_artifact)
        })
}

fn regression_item_perf_threshold_bundle_dirs(evidence: &RegressionEvidenceV1) -> Vec<String> {
    let Some(failures) = evidence
        .extra
        .as_ref()
        .and_then(|extra| extra.get("threshold_failures"))
        .and_then(|value| value.as_array())
    else {
        return Vec::new();
    };

    let mut dirs = Vec::new();
    for failure in failures {
        if let Some(dir) = failure
            .get("evidence_bundle")
            .and_then(|value| value.as_str())
            .and_then(bundle_root_dir_string_from_artifact)
        {
            push_unique_line(&mut dirs, dir);
        }
    }
    dirs
}

fn capability_source_display_from_value(value: &serde_json::Value) -> Option<String> {
    if let Some(path) = value.get("path").and_then(|value| value.as_str())
        && !path.trim().is_empty()
    {
        return Some(path.to_string());
    }
    if let Some(label) = value.get("label").and_then(|value| value.as_str())
        && !label.trim().is_empty()
    {
        return Some(label.to_string());
    }
    let transport = value.get("transport").and_then(|value| value.as_str());
    let session_id = value.get("session_id").and_then(|value| value.as_str());
    match (transport, session_id) {
        (Some(transport), Some(session_id))
            if !transport.trim().is_empty() && !session_id.trim().is_empty() =>
        {
            Some(format!("{transport}:{session_id}"))
        }
        (Some(transport), _) if !transport.trim().is_empty() => Some(transport.to_string()),
        _ => None,
    }
}

fn regression_item_capability_source_display(item: &RegressionItemSummaryV1) -> Option<String> {
    item.evidence
        .as_ref()
        .and_then(|evidence| evidence.extra.as_ref())
        .and_then(|extra| extra.get("capability_source"))
        .and_then(capability_source_display_from_value)
        .or_else(|| {
            item.source
                .as_ref()
                .and_then(|source| source.metadata.as_ref())
                .and_then(|metadata| metadata.get("capability_source"))
                .and_then(capability_source_display_from_value)
        })
        .or_else(|| {
            item.evidence
                .as_ref()
                .and_then(|evidence| evidence.extra.as_ref())
                .and_then(|extra| extra.get("capabilities_source_path"))
                .and_then(|value| value.as_str())
                .filter(|value| !value.trim().is_empty())
                .map(ToString::to_string)
        })
}

fn regression_status_label(status: RegressionStatusV1) -> &'static str {
    match status {
        RegressionStatusV1::Passed => "passed",
        RegressionStatusV1::FailedDeterministic => "failed_deterministic",
        RegressionStatusV1::FailedFlaky => "failed_flaky",
        RegressionStatusV1::FailedTooling => "failed_tooling",
        RegressionStatusV1::FailedTimeout => "failed_timeout",
        RegressionStatusV1::SkippedPolicy => "skipped_policy",
        RegressionStatusV1::Quarantined => "quarantined",
    }
}

fn regression_item_perf_evidence_lines(item: &RegressionItemSummaryV1) -> Vec<String> {
    let Some(evidence) = item.evidence.as_ref() else {
        return Vec::new();
    };
    let label = regression_item_display_label(item);
    let prefix = format!("{} [{}]", label, regression_status_label(item.status));
    let mut lines = Vec::new();
    if let Some(path) = evidence
        .perf_summary_json
        .as_deref()
        .filter(|path| !path.trim().is_empty())
    {
        lines.push(format!("{prefix} perf_summary_json: {path}"));
    }
    if let Some(path) = evidence
        .compare_json
        .as_deref()
        .filter(|path| !path.trim().is_empty())
    {
        lines.push(format!("{prefix} compare_json: {path}"));
    }
    let Some(extra) = evidence.extra.as_ref() else {
        return lines;
    };
    if let Some(metrics) = extra.get("metrics").and_then(|value| value.as_object()) {
        for key in REGRESSION_PERF_DRILLDOWN_METRIC_KEYS {
            if let Some(value) = metrics.get(*key) {
                lines.push(format!("{prefix} metric {key}: {value}"));
            }
        }
        if let Some(stats) = metrics.get("stats") {
            lines.push(format!("{prefix} metrics.stats: {stats}"));
        }
    }
    if let Some(threshold_failures) = extra.get("threshold_failures") {
        let count = threshold_failures
            .as_array()
            .map(Vec::len)
            .unwrap_or_else(|| usize::from(!threshold_failures.is_null()));
        lines.push(format!("{prefix} threshold_failures: {count}"));
        if count > 0 {
            lines.push(format!(
                "{prefix} threshold_failures_json: {threshold_failures}"
            ));
        }
    }
    lines
}

fn regression_item_first_open_evidence_lines(item: &RegressionItemSummaryV1) -> Vec<String> {
    let Some(evidence) = item.evidence.as_ref() else {
        return Vec::new();
    };
    let label = regression_item_display_label(item);
    let prefix = format!("{} [{}]", label, regression_status_label(item.status));
    let mut lines = Vec::new();
    push_optional_evidence_line(
        &mut lines,
        &prefix,
        "bundle_artifact",
        &evidence.bundle_artifact,
    );
    push_optional_evidence_line(
        &mut lines,
        &prefix,
        "triage_artifact",
        &evidence.triage_json,
    );
    push_optional_evidence_line(
        &mut lines,
        &prefix,
        "script_result",
        &evidence.script_result_json,
    );
    push_optional_evidence_line(
        &mut lines,
        &prefix,
        "screenshots_manifest",
        &evidence.screenshots_manifest,
    );
    push_optional_evidence_line(
        &mut lines,
        &prefix,
        "share_artifact",
        &evidence.ai_packet_dir,
    );
    push_optional_evidence_line(&mut lines, &prefix, "packed_report", &evidence.pack_path);
    lines
}

fn push_optional_evidence_line(
    lines: &mut Vec<String>,
    prefix: &str,
    field: &str,
    path: &Option<String>,
) {
    if let Some(path) = path.as_deref().filter(|path| !path.trim().is_empty()) {
        lines.push(format!("{prefix} {field}: {path}"));
    }
}

fn regression_item_share_artifacts(evidence: &RegressionEvidenceV1) -> Vec<String> {
    let mut paths = Vec::new();
    if let Some(path) = evidence
        .ai_packet_dir
        .as_deref()
        .filter(|path| !path.trim().is_empty())
    {
        paths.push(path.to_string());
    }
    if let Some(path) = evidence
        .pack_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
    {
        paths.push(path.to_string());
    }
    paths
}

fn regression_item_display_label(item: &RegressionItemSummaryV1) -> &str {
    if item.name.trim().is_empty() {
        item.item_id.as_str()
    } else {
        item.name.as_str()
    }
}

impl RegressionTotalsV1 {
    pub fn record_status(&mut self, status: RegressionStatusV1) {
        self.items_total = self.items_total.saturating_add(1);
        match status {
            RegressionStatusV1::Passed => self.passed = self.passed.saturating_add(1),
            RegressionStatusV1::FailedDeterministic => {
                self.failed_deterministic = self.failed_deterministic.saturating_add(1)
            }
            RegressionStatusV1::FailedFlaky => {
                self.failed_flaky = self.failed_flaky.saturating_add(1)
            }
            RegressionStatusV1::FailedTooling => {
                self.failed_tooling = self.failed_tooling.saturating_add(1)
            }
            RegressionStatusV1::FailedTimeout => {
                self.failed_timeout = self.failed_timeout.saturating_add(1)
            }
            RegressionStatusV1::SkippedPolicy => {
                self.skipped_policy = self.skipped_policy.saturating_add(1)
            }
            RegressionStatusV1::Quarantined => {
                self.quarantined = self.quarantined.saturating_add(1)
            }
        }
    }
}

impl RegressionHighlightsV1 {
    pub fn from_items(items: &[RegressionItemSummaryV1]) -> Option<Self> {
        let first_failure = items
            .iter()
            .find(|item| item.status != RegressionStatusV1::Passed)
            .map(|item| RegressionHighlightRefV1 {
                item_id: item.item_id.clone(),
                reason_code: item.reason_code.clone(),
            });

        let mut reason_code_counts = std::collections::BTreeMap::<String, u32>::new();
        for item in items {
            if let Some(reason_code) = item.reason_code.as_deref()
                && !reason_code.trim().is_empty()
            {
                *reason_code_counts
                    .entry(reason_code.to_string())
                    .or_default() += 1;
            }
        }

        let mut top_reason_codes: Vec<RegressionReasonCodeCountV1> = reason_code_counts
            .into_iter()
            .map(|(reason_code, count)| RegressionReasonCodeCountV1 { reason_code, count })
            .collect();
        top_reason_codes.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.reason_code.cmp(&right.reason_code))
        });

        if first_failure.is_none() && top_reason_codes.is_empty() {
            return None;
        }

        Some(Self {
            first_failure,
            worst_perf_failure: None,
            flake_examples: Vec::new(),
            quarantine_examples: Vec::new(),
            top_reason_codes,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionCampaignSummaryV1 {
    pub name: String,
    pub lane: RegressionLaneV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionRunSummaryV1 {
    pub run_id: String,
    pub created_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out_dir: Option<String>,
    pub tool: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RegressionTotalsV1 {
    #[serde(default)]
    pub items_total: u32,
    #[serde(default)]
    pub passed: u32,
    #[serde(default)]
    pub failed_deterministic: u32,
    #[serde(default)]
    pub failed_flaky: u32,
    #[serde(default)]
    pub failed_tooling: u32,
    #[serde(default)]
    pub failed_timeout: u32,
    #[serde(default)]
    pub skipped_policy: u32,
    #[serde(default)]
    pub quarantined: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionHighlightsV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_failure: Option<RegressionHighlightRefV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worst_perf_failure: Option<RegressionHighlightRefV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flake_examples: Vec<RegressionHighlightRefV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quarantine_examples: Vec<RegressionHighlightRefV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_reason_codes: Vec<RegressionReasonCodeCountV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionHighlightRefV1 {
    pub item_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionReasonCodeCountV1 {
    pub reason_code: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionArtifactsV1 {
    // Derived summary root for convenience navigation; not a source-of-truth payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_dir: Option<String>,
    // Optional packaged handoff rooted at the summary layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packed_report: Option<String>,
    // Canonical path to the derived summary/index artifact for first-open routing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_json: Option<String>,
    // Presentation-facing static report projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_report: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionItemSummaryV1 {
    pub item_id: String,
    pub kind: RegressionItemKindV1,
    pub name: String,
    pub status: RegressionStatusV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_reason_code: Option<String>,
    pub lane: RegressionLaneV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub feature_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timing: Option<RegressionTimingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempts: Option<RegressionAttemptsV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<RegressionEvidenceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<RegressionSourceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<RegressionNotesV1>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegressionItemKindV1 {
    Script,
    Suite,
    MatrixCase,
    PerfCase,
    CampaignStep,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegressionStatusV1 {
    Passed,
    FailedDeterministic,
    FailedFlaky,
    FailedTooling,
    FailedTimeout,
    SkippedPolicy,
    Quarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegressionLaneV1 {
    Smoke,
    Correctness,
    Matrix,
    Perf,
    Nightly,
    Full,
}

impl RegressionLaneV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Correctness => "correctness",
            Self::Matrix => "matrix",
            Self::Perf => "perf",
            Self::Nightly | Self::Full => "nightly",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value.trim() {
            "smoke" => Some(Self::Smoke),
            "correctness" => Some(Self::Correctness),
            "matrix" => Some(Self::Matrix),
            "perf" => Some(Self::Perf),
            "nightly" => Some(Self::Nightly),
            "full" => Some(Self::Full),
            _ => None,
        }
    }
}

impl Serialize for RegressionLaneV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RegressionLaneV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).ok_or_else(|| {
            serde::de::Error::unknown_variant(
                &value,
                &["smoke", "correctness", "matrix", "perf", "nightly", "full"],
            )
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionTimingV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_delay_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionAttemptsV1 {
    pub attempts_total: u32,
    pub attempts_passed: u32,
    pub attempts_failed: u32,
    pub retried: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_summary_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shrink_summary_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionEvidenceV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_artifact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_dir: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "triage_artifact",
        alias = "triage_json"
    )]
    pub triage_json: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "script_result",
        alias = "script_result_json"
    )]
    pub script_result_json: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "share_artifact",
        alias = "ai_packet_dir"
    )]
    pub ai_packet_dir: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "packed_report",
        alias = "pack_path"
    )]
    pub pack_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshots_manifest: Option<String>,
    // Projection-only perf summary path. Useful for perf triage, but not canonical cross-surface
    // vocabulary for generic regression evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perf_summary_json: Option<String>,
    // Projection-only compare/check artifact path. Useful for matrix/perf drill-down, but not part
    // of the canonical artifact-path vocabulary shared across all regression rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegressionSourceV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign_case: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegressionNotesV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regression_summary_drilldown_projects_perf_evidence() {
        let summary: RegressionSummaryV1 = serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "kind": "diag_regression_summary",
            "campaign": { "name": "perf-docking", "lane": "perf" },
            "run": { "run_id": "run-1", "created_unix_ms": 1, "tool": "fretboard-dev diag perf" },
            "totals": { "items_total": 1, "passed": 0, "failed_deterministic": 1, "failed_flaky": 0, "failed_tooling": 0, "failed_timeout": 0, "skipped_policy": 0, "quarantined": 0 },
            "items": [
                {
                    "item_id": "perf-case",
                    "kind": "perf_case",
                    "name": "docking steady drag",
                    "status": "failed_deterministic",
                    "lane": "perf",
                    "evidence": {
                        "bundle_artifact": "target/fret-diag/perf-docking/run-a/bundle.schema2.json",
                        "triage_artifact": "target/fret-diag/perf-docking/run-a/triage.json",
                        "script_result": "target/fret-diag/perf-docking/run-a/script.result.json",
                        "screenshots_manifest": "target/fret-diag/perf-docking/run-a/screenshots.manifest.json",
                        "share_artifact": "target/fret-diag/perf-docking/share/perf-case.ai.zip",
                        "packed_report": "target/fret-diag/perf-docking/share/perf-case.report.zip",
                        "perf_summary_json": "target/fret-diag/perf-docking/layout.perf.summary.v1.json",
                        "compare_json": "target/fret-diag/perf-docking/check.perf_thresholds.json",
                        "extra": {
                            "capability_source": {
                                "kind": "filesystem",
                                "path": "target/fret-diag/capabilities.json",
                                "label": "filesystem:target/fret-diag/capabilities.json",
                                "transport": "filesystem",
                                "session_id": null
                            },
                            "capabilities_check_path": "target/fret-diag/perf-docking/check.capabilities.json",
                            "metrics": {
                                "top_total_time_us": 24000,
                                "top_renderer_encode_scene_us": 6000,
                                "top_renderer_instance_bytes": 700000,
                                "stats": {
                                    "total_time_us": 24000,
                                    "top_renderer_encode_scene_us": 6000
                                }
                            },
                            "threshold_failures": [
                                {
                                    "metric": "top_total_time_us",
                                    "observed": 24000,
                                    "threshold": 20000,
                                    "evidence_bundle": "target/fret-diag/perf-docking/run-threshold/bundle.schema2.json"
                                }
                            ]
                        }
                    }
                }
            ]
        }))
        .expect("summary should parse");

        let drilldown = regression_summary_drilldown(&summary);
        assert_eq!(
            drilldown.bundle_dirs,
            vec![
                "target/fret-diag/perf-docking/run-threshold".to_string(),
                "target/fret-diag/perf-docking/run-a".to_string(),
            ]
        );
        assert_eq!(
            drilldown.capability_sources,
            vec!["target/fret-diag/capabilities.json".to_string()]
        );
        assert_eq!(
            drilldown.capabilities_check_paths,
            vec!["target/fret-diag/perf-docking/check.capabilities.json".to_string()]
        );
        let text = drilldown.perf_evidence_lines.join("\n");
        assert!(text.contains(
            "docking steady drag [failed_deterministic] perf_summary_json: target/fret-diag/perf-docking/layout.perf.summary.v1.json"
        ));
        assert!(text.contains(
            "docking steady drag [failed_deterministic] compare_json: target/fret-diag/perf-docking/check.perf_thresholds.json"
        ));
        assert!(text.contains(
            "docking steady drag [failed_deterministic] metric top_total_time_us: 24000"
        ));
        assert!(text.contains(
            "docking steady drag [failed_deterministic] metric top_renderer_encode_scene_us: 6000"
        ));
        assert!(text.contains(
            "docking steady drag [failed_deterministic] metric top_renderer_instance_bytes: 700000"
        ));
        assert!(text.contains("docking steady drag [failed_deterministic] threshold_failures: 1"));
        assert!(text.contains("threshold_failures_json"));
        let first_open_text = drilldown.first_open_evidence_lines.join("\n");
        assert!(first_open_text.contains(
            "docking steady drag [failed_deterministic] bundle_artifact: target/fret-diag/perf-docking/run-a/bundle.schema2.json"
        ));
        assert!(first_open_text.contains(
            "docking steady drag [failed_deterministic] triage_artifact: target/fret-diag/perf-docking/run-a/triage.json"
        ));
        assert!(first_open_text.contains(
            "docking steady drag [failed_deterministic] script_result: target/fret-diag/perf-docking/run-a/script.result.json"
        ));
        assert!(first_open_text.contains(
            "docking steady drag [failed_deterministic] screenshots_manifest: target/fret-diag/perf-docking/run-a/screenshots.manifest.json"
        ));
        assert!(first_open_text.contains(
            "docking steady drag [failed_deterministic] share_artifact: target/fret-diag/perf-docking/share/perf-case.ai.zip"
        ));
        assert!(first_open_text.contains(
            "docking steady drag [failed_deterministic] packed_report: target/fret-diag/perf-docking/share/perf-case.report.zip"
        ));
        assert_eq!(
            drilldown.share_artifacts,
            vec![
                "target/fret-diag/perf-docking/share/perf-case.ai.zip".to_string(),
                "target/fret-diag/perf-docking/share/perf-case.report.zip".to_string(),
            ]
        );
    }

    #[test]
    fn regression_bundle_followup_command_lines_use_selected_bundle_dir() {
        let lines = regression_bundle_followup_command_lines([
            "target/fret-diag/perf-docking/run-a/bundle dir",
        ]);
        let text = lines.join("\n");
        assert!(text.contains("selected bundle: target/fret-diag/perf-docking/run-a/bundle dir"));
        assert!(text.contains(
            "diag stats: cargo run -p fretboard-dev -- diag stats 'target/fret-diag/perf-docking/run-a/bundle dir' --json"
        ));
        assert!(text.contains(
            "layout perf summary: cargo run -p fretboard-dev -- diag layout-perf-summary 'target/fret-diag/perf-docking/run-a/bundle dir' --json"
        ));
        assert!(text.contains(
            "memory summary: cargo run -p fretboard-dev -- diag memory-summary 'target/fret-diag/perf-docking/run-a/bundle dir' --json"
        ));
        assert!(text.contains(
            "triage: cargo run -p fretboard-dev -- diag triage 'target/fret-diag/perf-docking/run-a/bundle dir' --json"
        ));
        assert!(text.contains(
            "hotspots: cargo run -p fretboard-dev -- diag hotspots 'target/fret-diag/perf-docking/run-a/bundle dir' --json"
        ));
        assert!(text.contains(
            "trace: cargo run -p fretboard-dev -- diag trace 'target/fret-diag/perf-docking/run-a/bundle dir' --json"
        ));
        assert!(text.contains(
            "visual compare: cargo run -p fretboard-dev -- diag compare <baseline-bundle-or-dir> 'target/fret-diag/perf-docking/run-a/bundle dir' --json"
        ));
        assert!(text.contains(
            "footprint compare: cargo run -p fretboard-dev -- diag compare <baseline-session> 'target/fret-diag/perf-docking/run-a/bundle dir' --footprint --json"
        ));
    }

    #[test]
    fn regression_bundle_followup_commands_classify_runnable_and_baseline_required() {
        let commands = regression_bundle_followup_commands(["target/fret-diag/perf-docking/run-a"]);
        let runnable = commands
            .iter()
            .filter(|command| !command.requires_baseline)
            .collect::<Vec<_>>();
        let manual = commands
            .iter()
            .filter(|command| command.requires_baseline)
            .collect::<Vec<_>>();

        assert_eq!(runnable.len(), 6);
        assert_eq!(manual.len(), 2);
        assert!(runnable.iter().any(|command| {
            command.id == "stats"
                && command.target_bundle_dir.as_deref()
                    == Some("target/fret-diag/perf-docking/run-a")
                && command.diag_args
                    == vec![
                        "stats".to_string(),
                        "target/fret-diag/perf-docking/run-a".to_string(),
                        "--json".to_string(),
                    ]
        }));
        assert!(runnable.iter().any(|command| {
            command.id == "trace"
                && command.target_bundle_dir.as_deref()
                    == Some("target/fret-diag/perf-docking/run-a")
                && command.diag_args
                    == vec![
                        "trace".to_string(),
                        "target/fret-diag/perf-docking/run-a".to_string(),
                        "--json".to_string(),
                    ]
        }));
        assert!(manual.iter().all(|command| command.diag_args.is_empty()));
        assert!(
            manual
                .iter()
                .all(|command| command.target_bundle_dir.as_deref()
                    == Some("target/fret-diag/perf-docking/run-a"))
        );
        assert!(manual.iter().any(|command| command.id == "visual-compare"));
        assert!(
            manual
                .iter()
                .any(|command| command.id == "footprint-compare")
        );
    }

    #[test]
    fn regression_bundle_followup_commands_cover_each_selected_bundle() {
        let commands = regression_bundle_followup_commands([
            "target/fret-diag/perf-docking/run-threshold",
            "target/fret-diag/perf-docking/run-a",
            "target/fret-diag/perf-docking/run-threshold",
        ]);

        assert_eq!(commands.len(), 16);
        assert!(commands.iter().any(|command| {
            command.id == "stats"
                && command.label == "diag stats"
                && command.diag_args
                    == vec![
                        "stats".to_string(),
                        "target/fret-diag/perf-docking/run-threshold".to_string(),
                        "--json".to_string(),
                    ]
        }));
        assert!(commands.iter().any(|command| {
            command.id == "stats-2"
                && command.label == "diag stats [2]"
                && command.diag_args
                    == vec![
                        "stats".to_string(),
                        "target/fret-diag/perf-docking/run-a".to_string(),
                        "--json".to_string(),
                    ]
        }));
        assert!(commands.iter().any(|command| {
            command.id == "trace-2"
                && command.label == "trace [2]"
                && command.diag_args
                    == vec![
                        "trace".to_string(),
                        "target/fret-diag/perf-docking/run-a".to_string(),
                        "--json".to_string(),
                    ]
        }));
        assert!(commands.iter().any(|command| {
            command.id == "visual-compare-2"
                && command.requires_baseline
                && command
                    .command_line
                    .contains("target/fret-diag/perf-docking/run-a")
        }));

        let lines = regression_bundle_followup_command_lines([
            "target/fret-diag/perf-docking/run-threshold",
            "target/fret-diag/perf-docking/run-a",
        ]);
        let text = lines.join("\n");
        assert!(text.contains("selected bundle[1]: target/fret-diag/perf-docking/run-threshold"));
        assert!(text.contains("selected bundle[2]: target/fret-diag/perf-docking/run-a"));
        assert!(text.contains(
            "diag stats [2]: cargo run -p fretboard-dev -- diag stats target/fret-diag/perf-docking/run-a --json"
        ));
    }

    #[test]
    fn regression_summary_new_sets_kind_and_schema_version() {
        let summary = RegressionSummaryV1::new(
            RegressionCampaignSummaryV1 {
                name: "ui-gallery-pr".to_string(),
                lane: RegressionLaneV1::Smoke,
                profile: Some("default".to_string()),
                schema_version: None,
                requested_by: None,
                filters: None,
            },
            RegressionRunSummaryV1 {
                run_id: "20260306-001".to_string(),
                created_unix_ms: 1,
                started_unix_ms: None,
                finished_unix_ms: None,
                duration_ms: None,
                workspace_root: None,
                out_dir: None,
                tool: "fretboard-dev diag campaign".to_string(),
                tool_version: None,
                git_commit: None,
                git_branch: None,
                host: None,
            },
            RegressionTotalsV1::default(),
        );

        assert_eq!(summary.schema_version, 1);
        assert_eq!(summary.kind, DIAG_REGRESSION_SUMMARY_KIND_V1);
        assert!(summary.items.is_empty());
        assert!(summary.highlights.is_none());
        assert!(summary.artifacts.is_none());
    }

    #[test]
    fn regression_enums_serialize_as_expected() {
        let item = RegressionItemSummaryV1 {
            item_id: "item-1".to_string(),
            kind: RegressionItemKindV1::MatrixCase,
            name: "matrix check".to_string(),
            status: RegressionStatusV1::FailedDeterministic,
            reason_code: Some("assert.mismatch".to_string()),
            source_reason_code: None,
            lane: RegressionLaneV1::Perf,
            owner: None,
            feature_tags: vec!["overlay".to_string()],
            timing: None,
            attempts: None,
            evidence: None,
            source: None,
            notes: None,
        };

        let value = serde_json::to_value(&item).expect("serialize item");
        assert_eq!(
            value.get("kind").and_then(|v| v.as_str()),
            Some("matrix_case")
        );
        assert_eq!(
            value.get("status").and_then(|v| v.as_str()),
            Some("failed_deterministic")
        );
        assert_eq!(value.get("lane").and_then(|v| v.as_str()), Some("perf"));
        assert!(value.get("source_reason_code").is_none());
    }

    #[test]
    fn regression_lane_full_serializes_as_nightly_and_accepts_full_alias() {
        assert_eq!(
            serde_json::to_value(RegressionLaneV1::Full)
                .expect("serialize lane")
                .as_str(),
            Some("nightly")
        );
        assert_eq!(
            serde_json::from_value::<RegressionLaneV1>(serde_json::json!("full"))
                .expect("deserialize lane alias"),
            RegressionLaneV1::Full
        );
        assert_eq!(
            serde_json::from_value::<RegressionLaneV1>(serde_json::json!("nightly"))
                .expect("deserialize canonical lane"),
            RegressionLaneV1::Nightly
        );
    }

    #[test]
    fn regression_summary_serializes_bounded_minimal_shape() {
        let mut summary = RegressionSummaryV1::new(
            RegressionCampaignSummaryV1 {
                name: "ui-gallery-pr".to_string(),
                lane: RegressionLaneV1::Smoke,
                profile: None,
                schema_version: None,
                requested_by: None,
                filters: None,
            },
            RegressionRunSummaryV1 {
                run_id: "run-1".to_string(),
                created_unix_ms: 42,
                started_unix_ms: None,
                finished_unix_ms: None,
                duration_ms: None,
                workspace_root: None,
                out_dir: None,
                tool: "fretboard-dev diag campaign".to_string(),
                tool_version: None,
                git_commit: None,
                git_branch: None,
                host: None,
            },
            RegressionTotalsV1 {
                items_total: 1,
                passed: 0,
                failed_deterministic: 1,
                failed_flaky: 0,
                failed_tooling: 0,
                failed_timeout: 0,
                skipped_policy: 0,
                quarantined: 0,
            },
        );
        summary.items.push(RegressionItemSummaryV1 {
            item_id: "script-1".to_string(),
            kind: RegressionItemKindV1::Script,
            name: "dialog escape focus restore".to_string(),
            status: RegressionStatusV1::FailedDeterministic,
            reason_code: Some("assert.focus_restore.mismatch".to_string()),
            source_reason_code: None,
            lane: RegressionLaneV1::Smoke,
            owner: None,
            feature_tags: Vec::new(),
            timing: Some(RegressionTimingV1 {
                duration_ms: Some(1420),
                queue_delay_ms: None,
                started_unix_ms: None,
                finished_unix_ms: None,
            }),
            attempts: Some(RegressionAttemptsV1 {
                attempts_total: 1,
                attempts_passed: 0,
                attempts_failed: 1,
                retried: false,
                repeat_summary_path: None,
                shrink_summary_path: None,
            }),
            evidence: Some(RegressionEvidenceV1 {
                bundle_artifact: Some("target/fret-diag/bundle.schema2.json".to_string()),
                bundle_dir: Some("target/fret-diag".to_string()),
                triage_json: Some("target/fret-diag/triage.json".to_string()),
                script_result_json: Some("target/fret-diag/script.result.json".to_string()),
                ai_packet_dir: None,
                pack_path: None,
                screenshots_manifest: None,
                perf_summary_json: None,
                compare_json: None,
                extra: None,
            }),
            source: Some(RegressionSourceV1 {
                script: Some(
                    "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json".to_string(),
                ),
                suite: Some("ui-gallery".to_string()),
                campaign_case: None,
                metadata: None,
            }),
            notes: Some(RegressionNotesV1 {
                summary: Some("focus did not return to trigger".to_string()),
                details: Vec::new(),
            }),
        });

        let value = serde_json::to_value(&summary).expect("serialize summary");
        assert_eq!(
            value.get("schema_version").and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            value.get("kind").and_then(|v| v.as_str()),
            Some(DIAG_REGRESSION_SUMMARY_KIND_V1)
        );
        assert_eq!(
            value.pointer("/campaign/lane").and_then(|v| v.as_str()),
            Some("smoke")
        );
        assert_eq!(
            value.pointer("/items/0/status").and_then(|v| v.as_str()),
            Some("failed_deterministic")
        );
        assert_eq!(
            value
                .pointer("/items/0/evidence/triage_artifact")
                .and_then(|v| v.as_str()),
            Some("target/fret-diag/triage.json")
        );
        assert_eq!(
            value
                .pointer("/items/0/evidence/script_result")
                .and_then(|v| v.as_str()),
            Some("target/fret-diag/script.result.json")
        );
        assert!(value.get("highlights").is_none());
        assert!(value.get("artifacts").is_none());
    }

    #[test]
    fn regression_evidence_accepts_legacy_field_aliases() {
        let parsed: RegressionEvidenceV1 = serde_json::from_value(serde_json::json!({
            "bundle_artifact": "target/fret-diag/bundle.schema2.json",
            "triage_json": "target/fret-diag/triage.json",
            "script_result_json": "target/fret-diag/script.result.json",
            "ai_packet_dir": "target/fret-diag/ai.packet",
            "pack_path": "target/fret-diag/share.zip"
        }))
        .expect("deserialize evidence aliases");

        assert_eq!(
            parsed.triage_json.as_deref(),
            Some("target/fret-diag/triage.json")
        );
        assert_eq!(
            parsed.script_result_json.as_deref(),
            Some("target/fret-diag/script.result.json")
        );
        assert_eq!(
            parsed.ai_packet_dir.as_deref(),
            Some("target/fret-diag/ai.packet")
        );
        assert_eq!(
            parsed.pack_path.as_deref(),
            Some("target/fret-diag/share.zip")
        );
    }

    #[test]
    fn regression_projection_fields_stay_additive_and_optional() {
        let value = serde_json::to_value(RegressionItemSummaryV1 {
            item_id: "perf:ui-gallery".to_string(),
            kind: RegressionItemKindV1::PerfCase,
            name: "ui-gallery".to_string(),
            status: RegressionStatusV1::FailedDeterministic,
            reason_code: Some("diag.perf.threshold_failed".to_string()),
            source_reason_code: None,
            lane: RegressionLaneV1::Perf,
            owner: None,
            feature_tags: Vec::new(),
            timing: None,
            attempts: None,
            evidence: Some(RegressionEvidenceV1 {
                bundle_artifact: Some("target/fret-diag/bundle.schema2.json".to_string()),
                bundle_dir: None,
                triage_json: None,
                script_result_json: None,
                ai_packet_dir: None,
                pack_path: None,
                screenshots_manifest: None,
                perf_summary_json: Some("target/fret-diag/layout.perf.summary.json".to_string()),
                compare_json: Some("target/fret-diag/check.perf.thresholds.json".to_string()),
                extra: None,
            }),
            source: None,
            notes: None,
        })
        .expect("serialize perf item");

        assert_eq!(
            value
                .pointer("/evidence/perf_summary_json")
                .and_then(|value| value.as_str()),
            Some("target/fret-diag/layout.perf.summary.json")
        );
        assert_eq!(
            value
                .pointer("/evidence/compare_json")
                .and_then(|value| value.as_str()),
            Some("target/fret-diag/check.perf.thresholds.json")
        );
    }

    #[test]
    fn regression_totals_record_status_updates_expected_bucket() {
        let mut totals = RegressionTotalsV1::default();
        totals.record_status(RegressionStatusV1::Passed);
        totals.record_status(RegressionStatusV1::FailedDeterministic);
        totals.record_status(RegressionStatusV1::FailedTooling);

        assert_eq!(totals.items_total, 3);
        assert_eq!(totals.passed, 1);
        assert_eq!(totals.failed_deterministic, 1);
        assert_eq!(totals.failed_tooling, 1);
    }

    #[test]
    fn regression_highlights_from_items_collects_first_failure_and_reason_counts() {
        let highlights = RegressionHighlightsV1::from_items(&[
            RegressionItemSummaryV1 {
                item_id: "ok".to_string(),
                kind: RegressionItemKindV1::Script,
                name: "ok".to_string(),
                status: RegressionStatusV1::Passed,
                reason_code: None,
                source_reason_code: None,
                lane: RegressionLaneV1::Smoke,
                owner: None,
                feature_tags: Vec::new(),
                timing: None,
                attempts: None,
                evidence: None,
                source: None,
                notes: None,
            },
            RegressionItemSummaryV1 {
                item_id: "bad".to_string(),
                kind: RegressionItemKindV1::Script,
                name: "bad".to_string(),
                status: RegressionStatusV1::FailedDeterministic,
                reason_code: Some("assert.mismatch".to_string()),
                source_reason_code: None,
                lane: RegressionLaneV1::Smoke,
                owner: None,
                feature_tags: Vec::new(),
                timing: None,
                attempts: None,
                evidence: None,
                source: None,
                notes: None,
            },
            RegressionItemSummaryV1 {
                item_id: "bad-2".to_string(),
                kind: RegressionItemKindV1::Script,
                name: "bad-2".to_string(),
                status: RegressionStatusV1::FailedDeterministic,
                reason_code: Some("assert.mismatch".to_string()),
                source_reason_code: None,
                lane: RegressionLaneV1::Smoke,
                owner: None,
                feature_tags: Vec::new(),
                timing: None,
                attempts: None,
                evidence: None,
                source: None,
                notes: None,
            },
        ])
        .expect("expected highlights");

        assert_eq!(
            highlights
                .first_failure
                .as_ref()
                .map(|v| v.item_id.as_str()),
            Some("bad")
        );
        assert_eq!(
            highlights
                .top_reason_codes
                .first()
                .map(|v| v.reason_code.as_str()),
            Some("assert.mismatch")
        );
        assert_eq!(
            highlights.top_reason_codes.first().map(|v| v.count),
            Some(2)
        );
    }
}
