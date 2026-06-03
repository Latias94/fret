use fret_app::CommandId;

const DEVTOOLS_DEMO_METRICS_DEBUG_COPY_ACTION_PREFIX: &str =
    "fret.devtools.demo_metrics_debug.copy_action.";

// This file owns the Demo/Metrics/Debug action catalog plus copy/readiness line projection.

pub(super) type DemoMetricsDebugActionSpec = fret_first_open::demo_metrics_debug::RouteCommand;

pub(super) const DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS: &[DemoMetricsDebugActionSpec] =
    fret_first_open::demo_metrics_debug::ACTION_COMMANDS;

pub(crate) fn demo_metrics_debug_action_command_text() -> String {
    fret_first_open::demo_metrics_debug::action_command_text()
}

pub(crate) fn demo_metrics_debug_action_copy_command_id(action_id: &str) -> CommandId {
    CommandId::new(format!(
        "{DEVTOOLS_DEMO_METRICS_DEBUG_COPY_ACTION_PREFIX}{action_id}"
    ))
}

pub(crate) fn demo_metrics_debug_action_command_for_copy_command(
    command_id: &str,
) -> Option<String> {
    let action_id = command_id.strip_prefix(DEVTOOLS_DEMO_METRICS_DEBUG_COPY_ACTION_PREFIX)?;
    fret_first_open::demo_metrics_debug::action_by_id(action_id)
        .map(|action| action.command.to_string())
}

pub(crate) fn demo_metrics_debug_action_copy_command_lines() -> Vec<String> {
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS
        .iter()
        .map(|action| {
            format!(
                "action copy command: {} | id={} | copy_command={}",
                action.label,
                action.id,
                demo_metrics_debug_action_copy_command_id(action.id).as_str()
            )
        })
        .collect()
}

pub(crate) fn demo_metrics_debug_action_metadata_lines() -> Vec<String> {
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS
        .iter()
        .map(|action| {
            format!(
                "action metadata: {} | id={} | category={} | primary={} | requires_bundle={}",
                action.label, action.id, action.category, action.primary, action.requires_bundle
            )
        })
        .collect()
}

pub(crate) fn demo_metrics_debug_action_readiness_lines(
    selected_bundle_count: usize,
) -> Vec<String> {
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS
        .iter()
        .map(|action| {
            let runnable = !action.requires_bundle || selected_bundle_count > 0;
            let reason = if action.requires_bundle {
                if selected_bundle_count > 0 {
                    "selected bundle evidence available"
                } else {
                    "select a regression bundle"
                }
            } else {
                "no bundle required"
            };
            format!(
                "action readiness: {} | id={} | category={} | runnable={} | reason={}",
                action.label, action.id, action.category, runnable, reason
            )
        })
        .collect()
}
