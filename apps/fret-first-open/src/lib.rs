pub mod demo_metrics_debug {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RouteCommand {
        pub id: &'static str,
        pub label: &'static str,
        pub command: &'static str,
        pub category: &'static str,
        pub requires_bundle: bool,
        pub primary: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FirstOpenRoute {
        pub id: &'static str,
        pub purpose: &'static str,
        pub docs: &'static str,
        pub owner_doc: &'static str,
        pub action_metadata_doc: &'static str,
        pub docking_owner_doc: &'static str,
        pub wayland_acceptance_doc: &'static str,
        pub demo_commands: &'static [RouteCommand],
        pub metrics_commands: &'static [RouteCommand],
        pub debug_commands: &'static [RouteCommand],
        pub handoff_commands: &'static [RouteCommand],
        pub action_commands: &'static [RouteCommand],
    }

    pub const ROUTE_ID: &str = "demo-metrics-debug";
    pub const ROUTE_DOC: &str = "docs/diagnostics-first-open.md";
    pub const OWNER_DOC: &str =
        "docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json";
    pub const ACTION_METADATA_DOC: &str =
        "docs/workstreams/imui-demo-metrics-debug-action-metadata-v1/WORKSTREAM.json";
    pub const DOCKING_OWNER_DOC: &str =
        "docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json";
    pub const WAYLAND_ACCEPTANCE_DOC: &str = "docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md";
    pub const PURPOSE: &str = "Dear ImGui-style demo, metrics, and debug first-open route";

    pub const DEMO_EDITOR_WORKBENCH_COMMAND: &str =
        "cargo run -p fret-demo --bin imui_editor_workbench_demo";
    pub const DEMO_EDITOR_PROOF_COMMAND: &str =
        "cargo run -p fret-demo --bin imui_editor_proof_demo";
    pub const DEMO_EDITOR_NOTES_COMMAND: &str = "cargo run -p fret-demo --bin editor_notes_demo";
    pub const DEMO_DEVICE_SHELL_COMMAND: &str =
        "cargo run -p fret-demo --bin editor_notes_device_shell_demo";
    pub const DOCKING_ARBITRATION_COMMAND: &str =
        "cargo run -p fret-demo --bin docking_arbitration_demo";
    pub const DOCKING_CAMPAIGN_VALIDATE_COMMAND: &str = "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json";
    pub const DOCKING_POLICY_SKIP_COMMAND: &str =
        "python tools/diag_gate_docking_wayland_policy_skip.py";
    pub const METRICS_STATS_COMMAND: &str =
        "cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json";
    pub const METRICS_LAYOUT_PERF_COMMAND: &str =
        "cargo run -p fretboard-dev -- diag layout-perf-summary <bundle-or-dir> --json";
    pub const METRICS_MEMORY_COMMAND: &str =
        "cargo run -p fretboard-dev -- diag memory-summary <bundle-or-dir> --json";
    pub const DEBUG_TRIAGE_COMMAND: &str =
        "cargo run -p fretboard-dev -- diag triage <bundle-or-dir> --json";
    pub const DEBUG_HOTSPOTS_COMMAND: &str =
        "cargo run -p fretboard-dev -- diag hotspots <bundle-or-dir> --json";
    pub const DEBUG_TRACE_COMMAND: &str =
        "cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json";
    pub const PRODUCT_DISCOVERY_COMMAND: &str =
        "python tools/diag_gate_imui_product_chain.py --only discovery";

    pub const DEMO_COMMANDS: &[RouteCommand] = &[
        RouteCommand {
            id: "imui_editor_workbench",
            label: "demo editor workbench",
            command: DEMO_EDITOR_WORKBENCH_COMMAND,
            category: "demo",
            requires_bundle: false,
            primary: true,
        },
        RouteCommand {
            id: "imui_editor_proof_supporting",
            label: "demo editor proof supporting",
            command: DEMO_EDITOR_PROOF_COMMAND,
            category: "demo",
            requires_bundle: false,
            primary: false,
        },
        RouteCommand {
            id: "editor_notes",
            label: "demo editor notes",
            command: DEMO_EDITOR_NOTES_COMMAND,
            category: "demo",
            requires_bundle: false,
            primary: false,
        },
        RouteCommand {
            id: "editor_notes_device_shell",
            label: "demo device shell",
            command: DEMO_DEVICE_SHELL_COMMAND,
            category: "demo",
            requires_bundle: false,
            primary: false,
        },
    ];

    pub const METRICS_COMMANDS: &[RouteCommand] = &[
        RouteCommand {
            id: "metrics_stats",
            label: "metrics stats",
            command: METRICS_STATS_COMMAND,
            category: "metrics",
            requires_bundle: true,
            primary: false,
        },
        RouteCommand {
            id: "metrics_layout_perf",
            label: "metrics layout perf",
            command: METRICS_LAYOUT_PERF_COMMAND,
            category: "metrics",
            requires_bundle: true,
            primary: false,
        },
        RouteCommand {
            id: "metrics_memory",
            label: "metrics memory",
            command: METRICS_MEMORY_COMMAND,
            category: "metrics",
            requires_bundle: true,
            primary: false,
        },
    ];

    pub const DEBUG_COMMANDS: &[RouteCommand] = &[
        RouteCommand {
            id: "debug_triage",
            label: "debug triage",
            command: DEBUG_TRIAGE_COMMAND,
            category: "debug",
            requires_bundle: true,
            primary: false,
        },
        RouteCommand {
            id: "debug_hotspots",
            label: "debug hotspots",
            command: DEBUG_HOTSPOTS_COMMAND,
            category: "debug",
            requires_bundle: true,
            primary: false,
        },
        RouteCommand {
            id: "debug_trace",
            label: "debug trace",
            command: DEBUG_TRACE_COMMAND,
            category: "debug",
            requires_bundle: true,
            primary: false,
        },
    ];

    pub const HANDOFF_COMMANDS: &[RouteCommand] = &[
        RouteCommand {
            id: "docking_arbitration_supporting",
            label: "docking arbitration supporting",
            command: DOCKING_ARBITRATION_COMMAND,
            category: "handoff",
            requires_bundle: false,
            primary: false,
        },
        RouteCommand {
            id: "docking_campaign_validate",
            label: "docking campaign validate",
            command: DOCKING_CAMPAIGN_VALIDATE_COMMAND,
            category: "handoff",
            requires_bundle: false,
            primary: false,
        },
        RouteCommand {
            id: "docking_policy_skip_local",
            label: "docking policy-skip local",
            command: DOCKING_POLICY_SKIP_COMMAND,
            category: "handoff",
            requires_bundle: false,
            primary: false,
        },
    ];

    pub const ACTION_COMMANDS: &[RouteCommand] = &[
        RouteCommand {
            id: "open_workbench",
            label: "open workbench",
            command: DEMO_EDITOR_WORKBENCH_COMMAND,
            category: "demo",
            requires_bundle: false,
            primary: true,
        },
        RouteCommand {
            id: "product_discovery",
            label: "run product discovery",
            command: PRODUCT_DISCOVERY_COMMAND,
            category: "product-gate",
            requires_bundle: false,
            primary: false,
        },
        RouteCommand {
            id: "inspect_metrics_stats",
            label: "inspect metrics stats",
            command: METRICS_STATS_COMMAND,
            category: "metrics",
            requires_bundle: true,
            primary: false,
        },
        RouteCommand {
            id: "inspect_debug_trace",
            label: "inspect debug trace",
            command: DEBUG_TRACE_COMMAND,
            category: "debug",
            requires_bundle: true,
            primary: false,
        },
        RouteCommand {
            id: "validate_docking_campaign",
            label: "validate docking campaign",
            command: DOCKING_CAMPAIGN_VALIDATE_COMMAND,
            category: "handoff",
            requires_bundle: false,
            primary: false,
        },
    ];

    pub const ROUTE: FirstOpenRoute = FirstOpenRoute {
        id: ROUTE_ID,
        purpose: PURPOSE,
        docs: ROUTE_DOC,
        owner_doc: OWNER_DOC,
        action_metadata_doc: ACTION_METADATA_DOC,
        docking_owner_doc: DOCKING_OWNER_DOC,
        wayland_acceptance_doc: WAYLAND_ACCEPTANCE_DOC,
        demo_commands: DEMO_COMMANDS,
        metrics_commands: METRICS_COMMANDS,
        debug_commands: DEBUG_COMMANDS,
        handoff_commands: HANDOFF_COMMANDS,
        action_commands: ACTION_COMMANDS,
    };

    pub fn action_command_text() -> String {
        ACTION_COMMANDS
            .iter()
            .map(|action| format!("{}: {}", action.label, action.command))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn action_by_id(action_id: &str) -> Option<&'static RouteCommand> {
        ACTION_COMMANDS.iter().find(|action| action.id == action_id)
    }

    pub fn route_commands_human(commands: &[RouteCommand]) -> String {
        commands
            .iter()
            .map(|command| format!("{}: {}", command.label, command.command))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
