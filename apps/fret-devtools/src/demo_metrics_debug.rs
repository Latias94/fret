use super::{
    DEVTOOLS_DEBUG_TRACE_COMMAND, DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND,
    DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND, DEVTOOLS_METRICS_STATS_COMMAND,
    IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DemoMetricsDebugActionSpec {
    id: &'static str,
    label: &'static str,
    command: &'static str,
    category: &'static str,
    requires_bundle: bool,
    primary: bool,
}

const DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS: &[DemoMetricsDebugActionSpec] = &[
    DemoMetricsDebugActionSpec {
        id: "open_workbench",
        label: "open workbench",
        command: DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND,
        category: "demo",
        requires_bundle: false,
        primary: true,
    },
    DemoMetricsDebugActionSpec {
        id: "product_discovery",
        label: "run product discovery",
        command: IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND,
        category: "product-gate",
        requires_bundle: false,
        primary: false,
    },
    DemoMetricsDebugActionSpec {
        id: "inspect_metrics_stats",
        label: "inspect metrics stats",
        command: DEVTOOLS_METRICS_STATS_COMMAND,
        category: "metrics",
        requires_bundle: true,
        primary: false,
    },
    DemoMetricsDebugActionSpec {
        id: "inspect_debug_trace",
        label: "inspect debug trace",
        command: DEVTOOLS_DEBUG_TRACE_COMMAND,
        category: "debug",
        requires_bundle: true,
        primary: false,
    },
    DemoMetricsDebugActionSpec {
        id: "validate_docking_campaign",
        label: "validate docking campaign",
        command: DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND,
        category: "handoff",
        requires_bundle: false,
        primary: false,
    },
];

pub(crate) fn demo_metrics_debug_action_command_text() -> String {
    DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS
        .iter()
        .map(|action| format!("{}: {}", action.label, action.command))
        .collect::<Vec<_>>()
        .join("\n")
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
