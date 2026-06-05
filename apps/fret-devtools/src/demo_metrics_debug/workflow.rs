use super::{
    CMD_COPY_WORKFLOW_RESULT_PATH, CMD_LOAD_WORKFLOW_REGRESSION_INDEX,
    CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY, CMD_OPEN_WORKFLOW_RESULT_JSON,
    DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID, DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID,
};

pub(crate) fn demo_metrics_debug_workflow_readiness_lines(
    workflow_run_in_flight: bool,
    perf_workflow_runnable: bool,
) -> Vec<String> {
    let docking_runnable = !workflow_run_in_flight;
    let docking_reason = if workflow_run_in_flight {
        "workflow run already in flight"
    } else {
        "no inputs required"
    };
    let perf_runnable = !workflow_run_in_flight && perf_workflow_runnable;
    let perf_reason = if workflow_run_in_flight {
        "workflow run already in flight"
    } else if perf_workflow_runnable {
        "selected session available"
    } else {
        "select a DevTools session"
    };
    vec![
        format!(
            "workflow readiness: validate docking campaign | workflow_id={DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID} | runnable={docking_runnable} | reason={docking_reason}"
        ),
        format!(
            "workflow readiness: run perf docking suite | workflow_id={DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID} | runnable={perf_runnable} | reason={perf_reason}"
        ),
    ]
}

pub(crate) fn demo_metrics_debug_workflow_status_lines(
    workflow_run_in_flight: bool,
    last_result_path: Option<&str>,
    last_error: Option<&str>,
) -> Vec<String> {
    let last_result = last_result_path
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("-");
    let last_error = last_error
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("-");
    vec![format!(
        "workflow status: in_flight={workflow_run_in_flight} | last_result={last_result} | last_error={last_error}"
    )]
}

pub(crate) fn demo_metrics_debug_workflow_result_action_lines(
    workflow_result_available: bool,
) -> Vec<String> {
    let reason = if workflow_result_available {
        "workflow result available"
    } else {
        "wait for workflow result artifact"
    };
    vec![
        format!(
            "workflow result action: copy workflow result | command={CMD_COPY_WORKFLOW_RESULT_PATH} | enabled={workflow_result_available} | reason={reason}"
        ),
        format!(
            "workflow result action: open workflow JSON | command={CMD_OPEN_WORKFLOW_RESULT_JSON} | enabled={workflow_result_available} | reason={reason}"
        ),
    ]
}

pub(crate) fn demo_metrics_debug_workflow_artifact_action_lines(
    regression_summary_available: bool,
    regression_index_available: bool,
) -> Vec<String> {
    let summary_reason = if regression_summary_available {
        "workflow regression summary available"
    } else {
        "wait for workflow regression summary artifact"
    };
    let index_reason = if regression_index_available {
        "workflow regression index available"
    } else {
        "wait for workflow regression index artifact"
    };
    vec![
        format!(
            "workflow artifact action: load regression summary | command={CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY} | enabled={regression_summary_available} | reason={summary_reason}"
        ),
        format!(
            "workflow artifact action: load regression index | command={CMD_LOAD_WORKFLOW_REGRESSION_INDEX} | enabled={regression_index_available} | reason={index_reason}"
        ),
    ]
}
