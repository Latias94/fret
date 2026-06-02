use std::path::Path;

use fret_diag::regression_summary::RegressionBundleFollowupCommandV1;

use crate::followup;
use crate::gate_run;
use crate::short_artifact_result_path;
use crate::workflow_run;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RecentEvidenceTarget {
    pub(super) kind: &'static str,
    pub(super) id: String,
    pub(super) status: String,
    pub(super) result_path: String,
    pub(super) result_json: String,
    pub(super) command_line: String,
    pub(super) bundle_dir: Option<String>,
}

impl RecentEvidenceTarget {
    pub(super) fn from_gate(entry: &gate_run::GateRunResultHistoryEntry) -> Self {
        Self {
            kind: "gate",
            id: entry.id.clone(),
            status: entry.status.clone(),
            result_path: entry.result_path.clone(),
            result_json: entry.result_json.clone(),
            command_line: entry.command_line.clone(),
            bundle_dir: None,
        }
    }

    pub(super) fn from_workflow(entry: &workflow_run::WorkflowRunResultHistoryEntry) -> Self {
        Self {
            kind: "workflow",
            id: entry.id.clone(),
            status: entry.status.clone(),
            result_path: entry.result_path.clone(),
            result_json: entry.result_json.clone(),
            command_line: entry.command_line.clone(),
            bundle_dir: None,
        }
    }

    pub(super) fn from_followup(entry: &followup::FollowupResultHistoryEntry) -> Self {
        Self {
            kind: "follow-up",
            id: entry.id.clone(),
            status: entry.status.clone(),
            result_path: entry.result_path.clone(),
            result_json: entry.result_json.clone(),
            command_line: entry.command_line.clone(),
            bundle_dir: entry.bundle_dir.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RecentEvidenceKindOrder {
    Followup = 0,
    Workflow = 1,
    Gate = 2,
}

impl RecentEvidenceKindOrder {
    fn from_kind(kind: &str) -> Self {
        match kind {
            "workflow" => Self::Workflow,
            "follow-up" => Self::Followup,
            _ => Self::Gate,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RecentEvidenceSelectionEffect {
    pub(super) details_tab: &'static str,
    pub(super) selected_path: String,
    pub(super) selected_bundle_dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RecentEvidenceRerunCommand {
    Gate(fret_diag::DevtoolsGateCommandV1),
    Workflow(workflow_run::DevtoolsWorkflowRunCommandV1),
    Followup(RegressionBundleFollowupCommandV1),
}

impl RecentEvidenceRerunCommand {
    pub(super) fn kind(&self) -> &'static str {
        match self {
            Self::Gate(_) => "gate",
            Self::Workflow(_) => "workflow",
            Self::Followup(_) => "follow-up",
        }
    }
}

pub(super) enum RecentEvidenceRerunStatus {
    Available(RecentEvidenceRerunCommand),
    Unavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecentEvidenceDiagArgsIssue {
    InvalidJson,
    Missing,
    NotArray,
    NonString,
}

pub(super) fn devtools_recent_evidence_selection_effect(
    target: &RecentEvidenceTarget,
) -> RecentEvidenceSelectionEffect {
    let details_tab = if target.kind == "follow-up" {
        "regression"
    } else {
        "guide"
    };
    RecentEvidenceSelectionEffect {
        details_tab,
        selected_path: target.result_path.clone(),
        selected_bundle_dir: target.bundle_dir.clone(),
    }
}

#[cfg(test)]
pub(super) fn devtools_recent_evidence_lines(
    gate_entries: &[gate_run::GateRunResultHistoryEntry],
    workflow_entries: &[workflow_run::WorkflowRunResultHistoryEntry],
    followup_entries: &[followup::FollowupResultHistoryEntry],
) -> Vec<String> {
    devtools_recent_evidence_lines_with_workflow_commands(
        gate_entries,
        workflow_entries,
        followup_entries,
        &[],
    )
}

pub(super) fn devtools_recent_evidence_lines_with_workflow_commands(
    gate_entries: &[gate_run::GateRunResultHistoryEntry],
    workflow_entries: &[workflow_run::WorkflowRunResultHistoryEntry],
    followup_entries: &[followup::FollowupResultHistoryEntry],
    workflow_commands: &[workflow_run::DevtoolsWorkflowRunCommandV1],
) -> Vec<String> {
    let failed_target =
        devtools_recent_failed_evidence_target(gate_entries, workflow_entries, followup_entries);
    let mut lines = vec![format!(
        "recent evidence: gates={} workflows={} followups={}",
        gate_entries.len(),
        workflow_entries.len(),
        followup_entries.len()
    )];
    lines.push(recent_gate_evidence_line(gate_entries.first()));
    lines.push(recent_workflow_evidence_line(workflow_entries.first()));
    lines.push(recent_followup_evidence_line(followup_entries.first()));
    let failing_count =
        recent_evidence_failing_count(gate_entries, workflow_entries, followup_entries);
    lines.push(format!("recent failing evidence: {failing_count}"));
    lines.push(recent_failed_evidence_target_line(failed_target.as_ref()));
    lines.push(recent_failed_evidence_path_line(failed_target.as_ref()));
    lines.push(recent_failed_evidence_bundle_line(failed_target.as_ref()));
    lines.push(recent_failed_evidence_command_line(failed_target.as_ref()));
    lines.push(recent_failed_evidence_rerun_line_with_workflow_commands(
        failed_target.as_ref(),
        workflow_commands,
    ));
    lines.push(recent_failed_evidence_rerun_reason_line_with_workflow_commands(
        failed_target.as_ref(),
        workflow_commands,
    ));
    let next_action = recent_evidence_next_action(
        failing_count,
        gate_entries.is_empty() && workflow_entries.is_empty() && followup_entries.is_empty(),
        failed_target.as_ref(),
        workflow_commands,
    );
    lines.push(format!("recent_evidence_next_action: {next_action}"));
    lines
}

pub(super) fn recent_evidence_status_failed(status: &str) -> bool {
    let status = status.trim();
    !status.is_empty() && status != "-" && !status.eq_ignore_ascii_case("passed")
}

pub(super) fn recent_evidence_failing_count(
    gate_entries: &[gate_run::GateRunResultHistoryEntry],
    workflow_entries: &[workflow_run::WorkflowRunResultHistoryEntry],
    followup_entries: &[followup::FollowupResultHistoryEntry],
) -> usize {
    gate_entries
        .iter()
        .filter(|entry| recent_evidence_status_failed(&entry.status))
        .count()
        + workflow_entries
            .iter()
            .filter(|entry| recent_evidence_status_failed(&entry.status))
            .count()
        + followup_entries
            .iter()
            .filter(|entry| recent_evidence_status_failed(&entry.status))
            .count()
}

pub(super) fn recent_evidence_next_action(
    failing_count: usize,
    evidence_empty: bool,
    failed_target: Option<&RecentEvidenceTarget>,
    workflow_commands: &[workflow_run::DevtoolsWorkflowRunCommandV1],
) -> String {
    if failing_count == 0 {
        return if evidence_empty {
            "run a workflow or generated gate".to_string()
        } else {
            "continue from latest passing evidence".to_string()
        };
    }

    let Some(target) = failed_target else {
        return "inspect failed recent evidence history".to_string();
    };

    match recent_failed_evidence_rerun_status_from_state(target, workflow_commands) {
        RecentEvidenceRerunStatus::Available(command) => {
            format!("rerun failed {} evidence", command.kind())
        }
        RecentEvidenceRerunStatus::Unavailable(reason)
            if reason == "missing current selected-session" =>
        {
            "select a diagnostics session, then rerun failed workflow evidence".to_string()
        }
        RecentEvidenceRerunStatus::Unavailable(reason)
            if reason == "workflow commands unavailable" =>
        {
            "refresh current workflow commands, then rerun failed workflow evidence".to_string()
        }
        RecentEvidenceRerunStatus::Unavailable(reason)
            if reason.starts_with("workflow command ")
                && reason.ends_with(" is no longer registered") =>
        {
            "run a current workflow for fresh evidence".to_string()
        }
        RecentEvidenceRerunStatus::Unavailable(_) => {
            format!("select failed {} evidence and inspect result JSON", target.kind)
        }
    }
}

pub(super) fn devtools_recent_failed_evidence_target(
    gate_entries: &[gate_run::GateRunResultHistoryEntry],
    workflow_entries: &[workflow_run::WorkflowRunResultHistoryEntry],
    followup_entries: &[followup::FollowupResultHistoryEntry],
) -> Option<RecentEvidenceTarget> {
    let mut candidates = Vec::new();
    candidates.extend(
        gate_entries
            .iter()
            .filter(|entry| recent_evidence_status_failed(&entry.status))
            .enumerate()
            .map(|(index, entry)| (index, RecentEvidenceTarget::from_gate(entry))),
    );
    candidates.extend(
        workflow_entries
            .iter()
            .filter(|entry| recent_evidence_status_failed(&entry.status))
            .enumerate()
            .map(|(index, entry)| (index, RecentEvidenceTarget::from_workflow(entry))),
    );
    candidates.extend(
        followup_entries
            .iter()
            .filter(|entry| recent_evidence_status_failed(&entry.status))
            .enumerate()
            .map(|(index, entry)| (index, RecentEvidenceTarget::from_followup(entry))),
    );

    candidates
        .iter()
        .filter_map(|(index, target)| {
            recent_evidence_result_sort_timestamp(target)
                .map(|timestamp| (timestamp, *index, target))
        })
        .max_by_key(|(timestamp, index, target)| {
            (
                *timestamp,
                std::cmp::Reverse(*index),
                RecentEvidenceKindOrder::from_kind(target.kind),
            )
        })
        .map(|(_, _, target)| target.clone())
        .or_else(|| {
            recent_failed_evidence_target_lane_order_fallback(
                gate_entries,
                workflow_entries,
                followup_entries,
            )
        })
}

fn recent_failed_evidence_target_lane_order_fallback(
    gate_entries: &[gate_run::GateRunResultHistoryEntry],
    workflow_entries: &[workflow_run::WorkflowRunResultHistoryEntry],
    followup_entries: &[followup::FollowupResultHistoryEntry],
) -> Option<RecentEvidenceTarget> {
    if let Some(target) = gate_entries
        .first()
        .filter(|entry| recent_evidence_status_failed(&entry.status))
        .map(RecentEvidenceTarget::from_gate)
        .or_else(|| {
            workflow_entries
                .first()
                .filter(|entry| recent_evidence_status_failed(&entry.status))
                .map(RecentEvidenceTarget::from_workflow)
        })
        .or_else(|| {
            followup_entries
                .first()
                .filter(|entry| recent_evidence_status_failed(&entry.status))
                .map(RecentEvidenceTarget::from_followup)
        })
    {
        return Some(target);
    }

    gate_entries
        .iter()
        .skip(1)
        .find(|entry| recent_evidence_status_failed(&entry.status))
        .map(RecentEvidenceTarget::from_gate)
        .or_else(|| {
            workflow_entries
                .iter()
                .skip(1)
                .find(|entry| recent_evidence_status_failed(&entry.status))
                .map(RecentEvidenceTarget::from_workflow)
        })
        .or_else(|| {
            followup_entries
                .iter()
                .skip(1)
                .find(|entry| recent_evidence_status_failed(&entry.status))
                .map(RecentEvidenceTarget::from_followup)
        })
}

fn recent_evidence_result_path_timestamp(path: &str) -> Option<u64> {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|file_name| file_name.split_once('-'))
        .and_then(|(prefix, _)| prefix.parse::<u64>().ok())
}

fn recent_evidence_result_sort_timestamp(target: &RecentEvidenceTarget) -> Option<u64> {
    recent_evidence_result_json_timestamp(&target.result_json)
        .or_else(|| recent_evidence_result_path_timestamp(&target.result_path))
}

fn recent_evidence_result_json_timestamp(result_json: &str) -> Option<u64> {
    let value = serde_json::from_str::<serde_json::Value>(result_json).ok()?;
    value
        .get("finished_unix_ms")
        .and_then(|value| value.as_u64())
        .or_else(|| {
            value
                .get("started_unix_ms")
                .and_then(|value| value.as_u64())
        })
}

fn recent_failed_evidence_target_line(target: Option<&RecentEvidenceTarget>) -> String {
    match target {
        Some(target) => format!(
            "failed_evidence_target: {} | {} | {} | {}",
            target.kind,
            target.status,
            target.id,
            short_artifact_result_path(&target.result_path)
        ),
        None => "failed_evidence_target: <none>".to_string(),
    }
}

fn recent_failed_evidence_command_line(target: Option<&RecentEvidenceTarget>) -> String {
    match target {
        Some(target) => format!("failed_evidence_command: {}", target.command_line),
        None => "failed_evidence_command: <none>".to_string(),
    }
}

#[cfg(test)]
pub(super) fn recent_failed_evidence_rerun_line(target: Option<&RecentEvidenceTarget>) -> String {
    recent_failed_evidence_rerun_line_with_workflow_commands(target, &[])
}

fn recent_failed_evidence_rerun_line_with_workflow_commands(
    target: Option<&RecentEvidenceTarget>,
    workflow_commands: &[workflow_run::DevtoolsWorkflowRunCommandV1],
) -> String {
    match target {
        Some(target) => match recent_failed_evidence_rerun_status_from_state(target, workflow_commands)
        {
            RecentEvidenceRerunStatus::Available(command) => {
                format!("failed_evidence_rerunnable: {}", command.kind())
            }
            RecentEvidenceRerunStatus::Unavailable(reason) => {
                format!("failed_evidence_rerunnable: no ({reason})")
            }
        },
        None => "failed_evidence_rerunnable: <none>".to_string(),
    }
}

fn recent_failed_evidence_rerun_reason_line_with_workflow_commands(
    target: Option<&RecentEvidenceTarget>,
    workflow_commands: &[workflow_run::DevtoolsWorkflowRunCommandV1],
) -> String {
    match target.and_then(|target| {
        recent_failed_evidence_rerun_unavailable_reason_from_state(target, workflow_commands)
    }) {
        Some(reason) => format!("failed_evidence_rerun_unavailable_reason: {reason}"),
        None => "failed_evidence_rerun_unavailable_reason: <none>".to_string(),
    }
}

fn recent_failed_evidence_path_line(target: Option<&RecentEvidenceTarget>) -> String {
    match target {
        Some(target) => format!("failed_evidence_path: {}", target.result_path),
        None => "failed_evidence_path: <none>".to_string(),
    }
}

pub(super) fn recent_failed_evidence_bundle_dir(target: &RecentEvidenceTarget) -> Option<&str> {
    target
        .bundle_dir
        .as_deref()
        .filter(|bundle_dir| !bundle_dir.trim().is_empty())
}

fn recent_failed_evidence_bundle_line(target: Option<&RecentEvidenceTarget>) -> String {
    match target.and_then(recent_failed_evidence_bundle_dir) {
        Some(bundle_dir) if !bundle_dir.trim().is_empty() => {
            format!("failed_evidence_bundle_dir: {bundle_dir}")
        }
        _ => "failed_evidence_bundle_dir: <none>".to_string(),
    }
}

pub(super) fn recent_failed_evidence_rerun_command(
    target: &RecentEvidenceTarget,
) -> Option<RecentEvidenceRerunCommand> {
    let diag_args = recent_evidence_diag_args_from_result_json(&target.result_json).ok()?;
    if !recent_evidence_diag_args_are_rerunnable(&diag_args) {
        return None;
    }
    match target.kind {
        "gate" => Some(RecentEvidenceRerunCommand::Gate(
            fret_diag::DevtoolsGateCommandV1 {
                id: target.id.clone(),
                label: format!("Rerun failed evidence {}", target.id),
                command_line: target.command_line.clone(),
                diag_args,
                missing_inputs: Vec::new(),
            },
        )),
        "follow-up" => Some(RecentEvidenceRerunCommand::Followup(
            RegressionBundleFollowupCommandV1 {
                id: target.id.clone(),
                label: format!("Rerun failed evidence {}", target.id),
                command_line: target.command_line.clone(),
                diag_args,
                requires_baseline: false,
                target_bundle_dir: target.bundle_dir.clone(),
            },
        )),
        _ => None,
    }
}

pub(super) fn recent_failed_evidence_rerun_command_from_state(
    target: &RecentEvidenceTarget,
    workflow_commands: &[workflow_run::DevtoolsWorkflowRunCommandV1],
) -> Option<RecentEvidenceRerunCommand> {
    match recent_failed_evidence_rerun_status_from_state(target, workflow_commands) {
        RecentEvidenceRerunStatus::Available(command) => Some(command),
        RecentEvidenceRerunStatus::Unavailable(_) => None,
    }
}

pub(super) fn recent_failed_evidence_rerun_unavailable_reason_from_state(
    target: &RecentEvidenceTarget,
    workflow_commands: &[workflow_run::DevtoolsWorkflowRunCommandV1],
) -> Option<String> {
    match recent_failed_evidence_rerun_status_from_state(target, workflow_commands) {
        RecentEvidenceRerunStatus::Available(_) => None,
        RecentEvidenceRerunStatus::Unavailable(reason) => Some(reason),
    }
}

pub(super) fn recent_failed_evidence_rerun_status_from_state(
    target: &RecentEvidenceTarget,
    workflow_commands: &[workflow_run::DevtoolsWorkflowRunCommandV1],
) -> RecentEvidenceRerunStatus {
    if target.kind == "workflow" {
        return recent_failed_workflow_rerun_status_from_state(target, workflow_commands);
    }
    recent_failed_evidence_rerun_command(target)
        .map(RecentEvidenceRerunStatus::Available)
        .unwrap_or_else(|| {
            RecentEvidenceRerunStatus::Unavailable(recent_evidence_diag_args_unavailable_reason(
                &target.result_json,
            ))
        })
}

fn recent_failed_workflow_rerun_command_from_state(
    target: &RecentEvidenceTarget,
    workflow_commands: &[workflow_run::DevtoolsWorkflowRunCommandV1],
) -> Option<RecentEvidenceRerunCommand> {
    if target.kind != "workflow" {
        return None;
    }
    workflow_commands
        .iter()
        .find(|command| command.id == target.id)
        .filter(|command| command.is_runnable())
        .cloned()
        .map(RecentEvidenceRerunCommand::Workflow)
}

fn recent_failed_workflow_rerun_status_from_state(
    target: &RecentEvidenceTarget,
    workflow_commands: &[workflow_run::DevtoolsWorkflowRunCommandV1],
) -> RecentEvidenceRerunStatus {
    if let Some(command) = recent_failed_workflow_rerun_command_from_state(target, workflow_commands)
    {
        return RecentEvidenceRerunStatus::Available(command);
    }

    if let Some(command) = workflow_commands
        .iter()
        .find(|command| command.id == target.id)
    {
        if !command.missing_inputs.is_empty() {
            return RecentEvidenceRerunStatus::Unavailable(format!(
                "missing current {}",
                command.missing_inputs.join(", ")
            ));
        }
        return RecentEvidenceRerunStatus::Unavailable("current workflow has no diag_args".to_string());
    }

    if workflow_commands.is_empty() {
        return RecentEvidenceRerunStatus::Unavailable("workflow commands unavailable".to_string());
    }

    RecentEvidenceRerunStatus::Unavailable(format!(
        "workflow command {} is no longer registered",
        target.id
    ))
}

fn recent_evidence_diag_args_from_result_json(
    result_json: &str,
) -> Result<Vec<String>, RecentEvidenceDiagArgsIssue> {
    let value = serde_json::from_str::<serde_json::Value>(result_json)
        .map_err(|_| RecentEvidenceDiagArgsIssue::InvalidJson)?;
    let Some(args) = value.get("diag_args") else {
        return Err(RecentEvidenceDiagArgsIssue::Missing);
    };
    let Some(args) = args.as_array() else {
        return Err(RecentEvidenceDiagArgsIssue::NotArray);
    };
    let args = args
        .iter()
        .map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<Option<Vec<_>>>()
        .ok_or(RecentEvidenceDiagArgsIssue::NonString)?;
    Ok(args)
}

fn recent_evidence_diag_args_are_rerunnable(args: &[String]) -> bool {
    !args.is_empty()
        && args
            .iter()
            .all(|arg| !arg.trim().is_empty() && arg.trim() != "<redacted>")
}

fn recent_evidence_diag_args_unavailable_reason(result_json: &str) -> String {
    match recent_evidence_diag_args_from_result_json(result_json) {
        Ok(args) if args.is_empty() => "diag_args empty".to_string(),
        Ok(args)
            if args
                .iter()
                .any(|arg| arg.trim().is_empty() || arg.trim() == "<redacted>") =>
        {
            "diag_args missing or redacted".to_string()
        }
        Ok(_) => "unsupported evidence kind".to_string(),
        Err(RecentEvidenceDiagArgsIssue::InvalidJson) => "result JSON invalid".to_string(),
        Err(RecentEvidenceDiagArgsIssue::Missing) => "diag_args missing".to_string(),
        Err(RecentEvidenceDiagArgsIssue::NotArray) => "diag_args is not an array".to_string(),
        Err(RecentEvidenceDiagArgsIssue::NonString) => "diag_args contains non-string values".to_string(),
    }
}

fn recent_gate_evidence_line(entry: Option<&gate_run::GateRunResultHistoryEntry>) -> String {
    match entry {
        Some(entry) => format!(
            "latest gate: {} | {} | {}",
            entry.status,
            entry.id,
            short_artifact_result_path(&entry.result_path)
        ),
        None => "latest gate: <none>".to_string(),
    }
}

fn recent_workflow_evidence_line(
    entry: Option<&workflow_run::WorkflowRunResultHistoryEntry>,
) -> String {
    match entry {
        Some(entry) => format!(
            "latest workflow: {} | {} | {}",
            entry.status,
            entry.id,
            short_artifact_result_path(&entry.result_path)
        ),
        None => "latest workflow: <none>".to_string(),
    }
}

fn recent_followup_evidence_line(entry: Option<&followup::FollowupResultHistoryEntry>) -> String {
    match entry {
        Some(entry) => format!(
            "latest follow-up: {} | {} | {} | bundle={}",
            entry.status,
            entry.id,
            short_artifact_result_path(&entry.result_path),
            entry.bundle_dir.as_deref().unwrap_or("-")
        ),
        None => "latest follow-up: <none>".to_string(),
    }
}
