use std::path::Path;

use crate::cli::workspace_root;

pub(crate) fn list_native_demos(args: Vec<String>) -> Result<(), String> {
    let list_all = parse_list_all_flag(args)?;
    let root = workspace_root()?;
    let bin_dir = root.join("apps").join("fret-demo").join("src").join("bin");
    let mut demos = read_rs_stems(&bin_dir)?;
    demos.sort();

    let (official, maintainer) = split_official_native_demos(&demos);
    for demo in official.iter() {
        println!("{demo}");
    }

    if list_all {
        if !maintainer.is_empty() {
            println!();
        }
        for demo in maintainer.iter() {
            println!("{demo}");
        }
    } else if !maintainer.is_empty() {
        eprintln!(
            "note: {} maintainer/stress demos hidden (use: fretboard-dev list native-demos --all)",
            maintainer.len()
        );
    }
    Ok(())
}

pub(crate) fn list_web_demos(args: Vec<String>) -> Result<(), String> {
    if !args.is_empty() {
        return Err("list web-demos does not accept extra args".to_string());
    }
    for demo in web_demos() {
        println!("{demo}");
    }
    Ok(())
}

pub(crate) fn list_cookbook_examples(args: Vec<String>) -> Result<(), String> {
    let list_all = parse_list_all_flag(args)?;
    let root = workspace_root()?;
    let examples = list_cookbook_examples_from(&root)?;
    let (official, lab) = split_official_cookbook_examples(&examples);
    for ex in official.iter() {
        println!("{ex}");
    }
    if list_all {
        if !lab.is_empty() {
            println!();
        }
        for ex in lab.iter() {
            if let Some(feature_hint) = cookbook_example_feature_hint(ex) {
                println!("{ex}    # requires: {feature_hint}");
            } else {
                println!("{ex}");
            }
        }
    } else if !lab.is_empty() {
        eprintln!(
            "note: {} lab examples hidden (use: fretboard-dev list cookbook-examples --all)",
            lab.len()
        );
    }
    Ok(())
}

const DIAG_FIRST_OPEN_DOC: &str = "docs/diagnostics-first-open.md";
const DIAG_GUI_BRANCH_DOC: &str =
    "docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md";
const DIAG_REPO_PREFLIGHT_COMMAND: &str = "cargo run -p fretboard-dev -- diag doctor campaigns";
const DIAG_REPO_PREFLIGHT_JSON_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag doctor campaigns --json";
const IMUI_PRODUCT_CHAIN_DOC: &str =
    "docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md";
const IMUI_PRODUCT_CHAIN_COMMAND: &str = "python tools/diag_gate_imui_product_chain.py";
const IMUI_PRODUCT_CHAIN_DISCOVERY_COMMAND: &str =
    "python tools/diag_gate_imui_product_chain.py --only discovery";
const IMUI_DOCKING_PERF_COMMAND: &str = "python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release";
const IMUI_DOCKING_PERF_SUITE: &str =
    "tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json";
const IMUI_DOCKING_PERF_ARTIFACTS: &[&str] = &[
    "perf-docking/regression.summary.json",
    "perf-docking/check.perf_thresholds.json",
    "perf-docking/*/trace.chrome.json",
];
const DEMO_METRICS_DEBUG_ROUTE_ID: &str = "demo-metrics-debug";
const DEMO_METRICS_DEBUG_DOC: &str = DIAG_FIRST_OPEN_DOC;
const DEMO_METRICS_DEBUG_OWNER_DOC: &str =
    "docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json";
const DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC: &str =
    "docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json";
const DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC: &str = "docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md";
const DEMO_METRICS_DEBUG_PURPOSE: &str =
    "Dear ImGui-style demo, metrics, and debug first-open route";
const DEMO_EDITOR_WORKBENCH_COMMAND: &str =
    "cargo run -p fret-demo --bin imui_editor_workbench_demo";
const DEMO_EDITOR_PROOF_COMMAND: &str = "cargo run -p fret-demo --bin imui_editor_proof_demo";
const DEMO_EDITOR_NOTES_COMMAND: &str = "cargo run -p fret-demo --bin editor_notes_demo";
const DEMO_DEVICE_SHELL_COMMAND: &str =
    "cargo run -p fret-demo --bin editor_notes_device_shell_demo";
const DEMO_DOCKING_ARBITRATION_COMMAND: &str =
    "cargo run -p fret-demo --bin docking_arbitration_demo";
const DOCKING_CAMPAIGN_VALIDATE_COMMAND: &str = "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json";
const DOCKING_POLICY_SKIP_COMMAND: &str = "python tools/diag_gate_docking_wayland_policy_skip.py";
const METRICS_STATS_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json";
const METRICS_LAYOUT_PERF_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag layout-perf-summary <bundle-or-dir> --json";
const METRICS_MEMORY_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag memory-summary <bundle-or-dir> --json";
const DEBUG_TRIAGE_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag triage <bundle-or-dir> --json";
const DEBUG_HOTSPOTS_COMMAND: &str =
    "cargo run -p fretboard-dev -- diag hotspots <bundle-or-dir> --json";
const DEBUG_TRACE_COMMAND: &str = "cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json";
const DEMO_METRICS_DEBUG_DEMO_COMMANDS: &[RouteCommand] = &[
    RouteCommand {
        label: "demo editor workbench",
        command: DEMO_EDITOR_WORKBENCH_COMMAND,
    },
    RouteCommand {
        label: "demo editor proof supporting",
        command: DEMO_EDITOR_PROOF_COMMAND,
    },
    RouteCommand {
        label: "demo editor notes",
        command: DEMO_EDITOR_NOTES_COMMAND,
    },
    RouteCommand {
        label: "demo device shell",
        command: DEMO_DEVICE_SHELL_COMMAND,
    },
];
const DEMO_METRICS_DEBUG_METRICS_COMMANDS: &[RouteCommand] = &[
    RouteCommand {
        label: "metrics stats",
        command: METRICS_STATS_COMMAND,
    },
    RouteCommand {
        label: "metrics layout perf",
        command: METRICS_LAYOUT_PERF_COMMAND,
    },
    RouteCommand {
        label: "metrics memory",
        command: METRICS_MEMORY_COMMAND,
    },
];
const DEMO_METRICS_DEBUG_DEBUG_COMMANDS: &[RouteCommand] = &[
    RouteCommand {
        label: "debug triage",
        command: DEBUG_TRIAGE_COMMAND,
    },
    RouteCommand {
        label: "debug hotspots",
        command: DEBUG_HOTSPOTS_COMMAND,
    },
    RouteCommand {
        label: "debug trace",
        command: DEBUG_TRACE_COMMAND,
    },
];
const DEMO_METRICS_DEBUG_HANDOFF_COMMANDS: &[RouteCommand] = &[
    RouteCommand {
        label: "docking arbitration supporting",
        command: DEMO_DOCKING_ARBITRATION_COMMAND,
    },
    RouteCommand {
        label: "docking campaign validate",
        command: DOCKING_CAMPAIGN_VALIDATE_COMMAND,
    },
    RouteCommand {
        label: "docking policy-skip local",
        command: DOCKING_POLICY_SKIP_COMMAND,
    },
];
const DEMO_METRICS_DEBUG_ACTION_COMMANDS: &[RouteCommand] = &[
    RouteCommand {
        label: "open workbench",
        command: DEMO_EDITOR_WORKBENCH_COMMAND,
    },
    RouteCommand {
        label: "run product discovery",
        command: IMUI_PRODUCT_CHAIN_DISCOVERY_COMMAND,
    },
    RouteCommand {
        label: "inspect metrics stats",
        command: METRICS_STATS_COMMAND,
    },
    RouteCommand {
        label: "inspect debug trace",
        command: DEBUG_TRACE_COMMAND,
    },
    RouteCommand {
        label: "validate docking campaign",
        command: DOCKING_CAMPAIGN_VALIDATE_COMMAND,
    },
];

pub(crate) fn list_tool_apps(args: Vec<String>) -> Result<(), String> {
    let output_json = parse_tool_apps_json_flag(args)?;
    if output_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&tool_apps_json_value()).map_err(|err| err.to_string())?
        );
    } else {
        println!("first-open: {DIAG_FIRST_OPEN_DOC}");
        println!("repo preflight: {DIAG_REPO_PREFLIGHT_COMMAND}");
        println!("repo preflight json: {DIAG_REPO_PREFLIGHT_JSON_COMMAND}");
        println!("gui branch: {DIAG_GUI_BRANCH_DOC}");
        for workflow in product_workflows() {
            println!(
                "workflow: {}    # {} | command: {} | focused: {} | launched: {} | suite: {} | docs: {} | artifacts: {}",
                workflow.id,
                workflow.purpose,
                workflow.command,
                workflow.focused_command,
                workflow.launched_command,
                workflow.suite,
                workflow.docs,
                workflow.expected_artifacts.join(", ")
            );
        }
        for route in first_open_routes() {
            println!(
                "route: {}    # {} | actions: {} | demos: {} | metrics: {} | debug: {} | handoff: {} | docs: {} | owner: {} | docking owner: {} | wayland acceptance: {}",
                route.id,
                route.purpose,
                route_commands_human(route.action_commands),
                route_commands_human(route.demo_commands),
                route_commands_human(route.metrics_commands),
                route_commands_human(route.debug_commands),
                route_commands_human(route.handoff_commands),
                route.docs,
                route.owner_doc,
                route.docking_owner_doc,
                route.wayland_acceptance_doc
            );
        }
        for tool in tool_apps() {
            println!(
                "{}    # {} | best for: {} | run: {} | docs: {} | gate: {}",
                tool.id, tool.purpose, tool.best_for, tool.command, tool.docs, tool.gate
            );
        }
    }
    Ok(())
}

fn tool_apps_json_value() -> serde_json::Value {
    serde_json::json!({
        "kind": "fretboard_tool_apps",
        "schema_version": 1,
        "first_open_doc": DIAG_FIRST_OPEN_DOC,
        "branch_doc": DIAG_GUI_BRANCH_DOC,
        "repo_preflight": {
            "purpose": "Read-only maintainer preflight for checked-in diagnostics campaign manifests.",
            "command": DIAG_REPO_PREFLIGHT_COMMAND,
            "json_command": DIAG_REPO_PREFLIGHT_JSON_COMMAND,
        },
        "product_workflows": product_workflows().iter().map(|workflow| {
            serde_json::json!({
                "id": workflow.id,
                "purpose": workflow.purpose,
                "command": workflow.command,
                "focused_command": workflow.focused_command,
                "launched_command": workflow.launched_command,
                "docs": workflow.docs,
                "suite": workflow.suite,
                "expected_artifacts": workflow.expected_artifacts,
            })
        }).collect::<Vec<_>>(),
        "first_open_routes": first_open_routes().iter().map(|route| {
            serde_json::json!({
                "id": route.id,
                "purpose": route.purpose,
                "docs": route.docs,
                "owner_doc": route.owner_doc,
                "docking_owner_doc": route.docking_owner_doc,
                "wayland_acceptance_doc": route.wayland_acceptance_doc,
                "demo_commands": route_commands_json(route.demo_commands),
                "metrics_commands": route_commands_json(route.metrics_commands),
                "debug_commands": route_commands_json(route.debug_commands),
                "handoff_commands": route_commands_json(route.handoff_commands),
                "action_commands": route_commands_json(route.action_commands),
            })
        }).collect::<Vec<_>>(),
        "tool_apps": tool_apps().iter().map(|tool| {
            serde_json::json!({
                "id": tool.id,
                "purpose": tool.purpose,
                "command": tool.command,
                "best_for": tool.best_for,
                "docs": tool.docs,
                "gate": tool.gate,
            })
        }).collect::<Vec<_>>(),
    })
}

struct ToolApp {
    id: &'static str,
    purpose: &'static str,
    command: &'static str,
    best_for: &'static str,
    docs: &'static str,
    gate: &'static str,
}

#[derive(Clone, Copy)]
struct RouteCommand {
    label: &'static str,
    command: &'static str,
}

struct FirstOpenRoute {
    id: &'static str,
    purpose: &'static str,
    docs: &'static str,
    owner_doc: &'static str,
    docking_owner_doc: &'static str,
    wayland_acceptance_doc: &'static str,
    demo_commands: &'static [RouteCommand],
    metrics_commands: &'static [RouteCommand],
    debug_commands: &'static [RouteCommand],
    handoff_commands: &'static [RouteCommand],
    action_commands: &'static [RouteCommand],
}

struct ProductWorkflow {
    id: &'static str,
    purpose: &'static str,
    command: &'static str,
    focused_command: &'static str,
    launched_command: &'static str,
    docs: &'static str,
    suite: &'static str,
    expected_artifacts: &'static [&'static str],
}

fn product_workflows() -> &'static [ProductWorkflow] {
    &[ProductWorkflow {
        id: "imui-product-chain",
        purpose: "IMUI editor-grade product-chain gate with docking perf evidence",
        command: IMUI_PRODUCT_CHAIN_COMMAND,
        focused_command: IMUI_PRODUCT_CHAIN_DISCOVERY_COMMAND,
        launched_command: IMUI_DOCKING_PERF_COMMAND,
        docs: IMUI_PRODUCT_CHAIN_DOC,
        suite: IMUI_DOCKING_PERF_SUITE,
        expected_artifacts: IMUI_DOCKING_PERF_ARTIFACTS,
    }]
}

fn first_open_routes() -> &'static [FirstOpenRoute] {
    &[FirstOpenRoute {
        id: DEMO_METRICS_DEBUG_ROUTE_ID,
        purpose: DEMO_METRICS_DEBUG_PURPOSE,
        docs: DEMO_METRICS_DEBUG_DOC,
        owner_doc: DEMO_METRICS_DEBUG_OWNER_DOC,
        docking_owner_doc: DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC,
        wayland_acceptance_doc: DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC,
        demo_commands: DEMO_METRICS_DEBUG_DEMO_COMMANDS,
        metrics_commands: DEMO_METRICS_DEBUG_METRICS_COMMANDS,
        debug_commands: DEMO_METRICS_DEBUG_DEBUG_COMMANDS,
        handoff_commands: DEMO_METRICS_DEBUG_HANDOFF_COMMANDS,
        action_commands: DEMO_METRICS_DEBUG_ACTION_COMMANDS,
    }]
}

fn route_commands_human(commands: &[RouteCommand]) -> String {
    commands
        .iter()
        .map(|command| format!("{}: {}", command.label, command.command))
        .collect::<Vec<_>>()
        .join(", ")
}

fn route_commands_json(commands: &[RouteCommand]) -> Vec<serde_json::Value> {
    commands
        .iter()
        .map(|command| {
            serde_json::json!({
                "label": command.label,
                "command": command.command,
            })
        })
        .collect()
}

fn tool_apps() -> &'static [ToolApp] {
    &[
        ToolApp {
            id: "fret-devtools",
            purpose: "DevTools GUI over shared diagnostics artifacts",
            command: "cargo run -p fret-devtools",
            best_for: "human inspect/script/artifact dogfood over one diagnostics root",
            docs: "docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md",
            gate: "cargo build -p fret-devtools",
        },
        ToolApp {
            id: "fret-devtools-mcp",
            purpose: "MCP adapter over the same diagnostics operations",
            command: "cargo run -p fret-devtools-mcp",
            best_for: "AI/client automation over the same diagnostics operations",
            docs: "docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md",
            gate: "cargo build -p fret-devtools-mcp",
        },
    ]
}

fn parse_tool_apps_json_flag(args: Vec<String>) -> Result<bool, String> {
    let mut output_json = false;
    for arg in args {
        match arg.as_str() {
            "--json" => output_json = true,
            other => return Err(format!("unknown list tool-apps argument: {other}")),
        }
    }
    Ok(output_json)
}

fn read_rs_stems(dir: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir)
        .map_err(|e| format!("read_dir failed for `{}`: {e}", display_path(dir)))?;
    for ent in rd {
        let ent = ent.map_err(|e| e.to_string())?;
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        out.push(stem.to_string());
    }
    Ok(out)
}

pub(crate) fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn parse_list_all_flag(args: Vec<String>) -> Result<bool, String> {
    let mut all = false;
    for a in args {
        match a.as_str() {
            "--all" => all = true,
            other => return Err(format!("unknown list argument: {other}")),
        }
    }
    Ok(all)
}

fn is_official_native_demo(id: &str) -> bool {
    // Keep this list small and intentional: it defines the user-facing story.
    // Maintainer/stress harnesses remain runnable via `--bin` and discoverable via `--all`.
    matches!(id, "simple_todo_demo" | "todo_demo")
}

fn split_official_native_demos(all: &[String]) -> (Vec<String>, Vec<String>) {
    let mut official = Vec::new();
    let mut maintainer = Vec::new();
    for id in all {
        if is_official_native_demo(id) {
            official.push(id.clone());
        } else {
            maintainer.push(id.clone());
        }
    }
    (official, maintainer)
}

pub(crate) fn official_native_demos(all: &[String]) -> Vec<String> {
    let (official, _) = split_official_native_demos(all);
    official
}

fn is_official_cookbook_example(id: &str) -> bool {
    // Keep this list small and intentional: it defines the onboarding story.
    matches!(
        id,
        "hello"
            | "simple_todo"
            | "overlay_basics"
            | "text_input_basics"
            | "commands_keymap_basics"
            | "theme_switching_basics"
            | "virtual_list_basics"
            | "effects_layer_basics"
            | "hello_counter"
    )
}

pub(crate) fn cookbook_example_feature_hint(id: &str) -> Option<&'static str> {
    let hint = match id {
        "icons_and_assets_basics" => "--features cookbook-assets",
        "assets_reload_epoch_basics" => "--features cookbook-assets",
        "data_table_basics" => "--features cookbook-table",
        "image_asset_cache_basics" => "--features cookbook-image-assets,cookbook-renderer",
        "compositing_alpha_basics" => "--features cookbook-renderer",
        "drop_shadow_basics" => "--features cookbook-renderer",
        "query_basics" => "--features cookbook-query",
        "mutation_toast_feedback_basics" => "--features cookbook-mutation",
        "router_basics" => "--features cookbook-router",
        "undo_basics" => "--features cookbook-undo",
        "async_inbox_basics" => "--features cookbook-async",
        "imui_action_basics" => "--features cookbook-imui",
        "imui_debug_draw_basics" => "--features cookbook-imui",
        "imui_editor_controls_basics" => "--features cookbook-imui",
        "docking_basics" => "--features cookbook-docking",
        "embedded_viewport_basics" => "--features cookbook-interop",
        "external_texture_import_basics" => "--features cookbook-interop",
        "customv1_basics" => "--features cookbook-customv1",
        "utility_window_materials_windows" => "--features cookbook-bootstrap",
        "markdown_and_code_basics" => "--features cookbook-markdown",
        "canvas_pan_zoom_basics" => "--features cookbook-canvas",
        "chart_interactions_basics" => "--features cookbook-chart",
        "gizmo_basics" => "--features cookbook-gizmo",
        _ => return None,
    };
    Some(hint)
}

fn split_official_cookbook_examples(all: &[String]) -> (Vec<String>, Vec<String>) {
    let mut official = Vec::new();
    let mut lab = Vec::new();
    for id in all {
        if is_official_cookbook_example(id) {
            official.push(id.clone());
        } else {
            lab.push(id.clone());
        }
    }
    (official, lab)
}

fn web_demos() -> &'static [&'static str] {
    &[
        // Full UI Gallery app (pages: `?page=...`).
        "ui_gallery",
        // Simple onboarding baseline (matches `fretboard-dev new simple-todo`).
        "simple-todo",
        // Lightweight examples gallery (separate app from `fret-ui-gallery`).
        "components_gallery",
        // Custom effect authoring templates (WebGPU/WGSL; see workstreams renderer-effects-semantics-and-extensibility-v1).
        "custom_effect_v2_web_demo",
        "custom_effect_v2_lut_web_demo",
        "custom_effect_v2_identity_web_demo",
        "custom_effect_v2_glass_chrome_web_demo",
        "custom_effect_v3_web_demo",
        // External texture imports (web copy path; ADR 0234).
        "external_texture_imports_web_demo",
        "chart_demo",
        "plot_demo",
        "bars_demo",
        "grouped_bars_demo",
        "stacked_bars_demo",
        "area_demo",
        "candlestick_demo",
        "error_bars_demo",
        "heatmap_demo",
        "histogram_demo",
        "shaded_demo",
        "stairs_demo",
        "stems_demo",
        "linked_cursor_demo",
        "inf_lines_demo",
        "tags_demo",
        "drag_demo",
    ]
}

pub(crate) fn web_demos_as_vec() -> Vec<String> {
    web_demos().iter().copied().map(String::from).collect()
}

pub(crate) fn validate_web_demo(name: &str) -> Result<(), String> {
    if web_demos().contains(&name) {
        return Ok(());
    }
    Err(format!(
        "unknown web demo `{name}`\n  try: fretboard-dev list web-demos"
    ))
}

pub(crate) fn list_cookbook_examples_from(workspace_root: &Path) -> Result<Vec<String>, String> {
    let examples_dir = workspace_root
        .join("apps")
        .join("fret-cookbook")
        .join("examples");
    let mut examples = read_rs_stems(&examples_dir)?;
    examples.sort();
    Ok(examples)
}

pub(crate) fn validate_cookbook_example(examples: &[String], name: &str) -> Result<(), String> {
    if examples.iter().any(|e| e == name) {
        return Ok(());
    }

    let mut hint = String::new();
    for e in examples {
        if e.contains(name) || name.contains(e) {
            hint = format!("\n  hint: did you mean `{e}`?");
            break;
        }
    }

    Err(format!(
        "unknown cookbook example `{name}`{hint}\n  try: fretboard-dev list cookbook-examples"
    ))
}

pub(crate) fn list_native_demos_from(workspace_root: &Path) -> Result<Vec<String>, String> {
    let bin_dir = workspace_root
        .join("apps")
        .join("fret-demo")
        .join("src")
        .join("bin");
    read_rs_stems(&bin_dir)
}

pub(crate) fn validate_native_demo(demos: &[String], name: &str) -> Result<(), String> {
    if demos.iter().any(|d| d == name) {
        return Ok(());
    }

    let mut hint = String::new();
    for d in demos {
        if d.contains(name) || name.contains(d) {
            hint = format!("\n  hint: did you mean `{d}`?");
            break;
        }
    }

    Err(format!(
        "unknown native demo `{name}`{hint}\n  try: fretboard-dev list native-demos"
    ))
}

pub(crate) fn prompt_choose_demo(
    label: &str,
    demos: &[String],
    default: Option<&str>,
    validate: impl Fn(&str) -> Result<(), String>,
) -> Result<String, String> {
    if demos.is_empty() {
        return Err(format!("no {label} found"));
    }

    eprintln!("{label}:");
    for (i, demo) in demos.iter().enumerate() {
        eprintln!("  {:>2}) {demo}", i + 1);
    }

    if let Some(default) = default {
        eprint!("Enter number or name (blank = {default}): ");
    } else {
        eprint!("Enter number or name: ");
    }

    use std::io::Write as _;
    std::io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    let input = input.trim();
    if input.is_empty() {
        return default
            .map(|d| d.to_string())
            .ok_or_else(|| "selection cannot be empty".to_string());
    }

    if let Ok(n) = input.parse::<usize>() {
        if n == 0 || n > demos.len() {
            return Err(format!("invalid selection: {n}"));
        }
        return Ok(demos[n - 1].clone());
    }

    validate(input)?;
    Ok(input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookbook_feature_hints_cover_imui_teaching_examples() {
        assert_eq!(
            cookbook_example_feature_hint("imui_action_basics"),
            Some("--features cookbook-imui")
        );
        assert_eq!(
            cookbook_example_feature_hint("imui_debug_draw_basics"),
            Some("--features cookbook-imui")
        );
        assert_eq!(
            cookbook_example_feature_hint("imui_editor_controls_basics"),
            Some("--features cookbook-imui")
        );
    }

    #[test]
    fn tool_apps_list_devtools_entrypoints() {
        let tools = tool_apps();
        assert!(tools.iter().any(|tool| tool.id == "fret-devtools"
            && tool.command == "cargo run -p fret-devtools"
            && tool.docs.contains("DEVTOOLS_GUI_DOGFOOD_WORKFLOW")));
        assert!(tools.iter().any(|tool| tool.id == "fret-devtools-mcp"
            && tool.command == "cargo run -p fret-devtools-mcp"
            && tool.docs.contains("diag-devtools-gui-v1-ai-mcp.md")));
    }

    #[test]
    fn tool_apps_list_names_repo_preflight_entrypoints() {
        assert_eq!(
            DIAG_REPO_PREFLIGHT_COMMAND,
            "cargo run -p fretboard-dev -- diag doctor campaigns"
        );
        assert_eq!(
            DIAG_REPO_PREFLIGHT_JSON_COMMAND,
            "cargo run -p fretboard-dev -- diag doctor campaigns --json"
        );
    }

    #[test]
    fn tool_apps_list_names_product_workflows() {
        let workflows = product_workflows();
        assert!(workflows.iter().any(|workflow| {
            workflow.id == "imui-product-chain"
                && workflow.command == "python tools/diag_gate_imui_product_chain.py"
                && workflow.focused_command
                    == "python tools/diag_gate_imui_product_chain.py --only discovery"
                && workflow.launched_command.contains("--only perf-docking")
                && workflow
                    .suite
                    .ends_with("perf-docking-arbitration-steady/suite.json")
                && workflow
                    .expected_artifacts
                    .contains(&"perf-docking/regression.summary.json")
                && workflow
                    .expected_artifacts
                    .contains(&"perf-docking/check.perf_thresholds.json")
                && workflow
                    .expected_artifacts
                    .contains(&"perf-docking/*/trace.chrome.json")
        }));
    }

    #[test]
    fn tool_apps_list_names_first_open_routes() {
        let routes = first_open_routes();
        assert!(routes.iter().any(|route| {
            route.id == "demo-metrics-debug"
                && route.docs == "docs/diagnostics-first-open.md"
                && route.owner_doc
                    == "docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json"
                && route.docking_owner_doc
                    == "docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json"
                && route.wayland_acceptance_doc
                    == "docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md"
                && route.demo_commands.iter().any(|command| {
                    command.label == "demo editor workbench"
                        && command.command
                            == "cargo run -p fret-demo --bin imui_editor_workbench_demo"
                })
                && route.demo_commands.iter().any(|command| {
                    command.label == "demo editor proof supporting"
                        && command.command == "cargo run -p fret-demo --bin imui_editor_proof_demo"
                })
                && route.metrics_commands.iter().any(|command| {
                    command.label == "metrics stats"
                        && command.command
                            == "cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json"
                })
                && route.debug_commands.iter().any(|command| {
                    command.label == "debug hotspots"
                        && command.command
                            == "cargo run -p fretboard-dev -- diag hotspots <bundle-or-dir> --json"
                })
                && route.debug_commands.iter().any(|command| {
                    command.label == "debug trace"
                        && command.command
                            == "cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json"
                })
                && route.handoff_commands.iter().any(|command| {
                    command.label == "docking campaign validate"
                        && command.command
                            == "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json"
                })
                && route.handoff_commands.iter().any(|command| {
                    command.label == "docking policy-skip local"
                        && command.command == "python tools/diag_gate_docking_wayland_policy_skip.py"
                })
                && route.action_commands.iter().any(|command| {
                    command.label == "open workbench"
                        && command.command
                            == "cargo run -p fret-demo --bin imui_editor_workbench_demo"
                })
                && route.action_commands.iter().any(|command| {
                    command.label == "run product discovery"
                        && command.command
                            == "python tools/diag_gate_imui_product_chain.py --only discovery"
                })
                && route.action_commands.iter().any(|command| {
                    command.label == "validate docking campaign"
                        && command.command
                            == "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json"
                })
        }));
    }

    #[test]
    fn tool_apps_json_flag_parser_is_explicit() {
        assert!(!parse_tool_apps_json_flag(Vec::new()).unwrap());
        assert!(parse_tool_apps_json_flag(vec!["--json".to_string()]).unwrap());
        assert!(parse_tool_apps_json_flag(vec!["--bad".to_string()]).is_err());
    }

    #[test]
    fn tool_apps_json_value_exposes_stable_machine_readable_shape() {
        let value = tool_apps_json_value();
        assert_eq!(value["kind"], "fretboard_tool_apps");
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["first_open_doc"], "docs/diagnostics-first-open.md");
        assert_eq!(
            value["branch_doc"],
            "docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md"
        );
        assert_eq!(
            value["repo_preflight"]["command"],
            "cargo run -p fretboard-dev -- diag doctor campaigns"
        );
        assert_eq!(
            value["repo_preflight"]["json_command"],
            "cargo run -p fretboard-dev -- diag doctor campaigns --json"
        );

        let workflows = value["product_workflows"]
            .as_array()
            .expect("product_workflows array");
        assert!(workflows.iter().any(|workflow| {
            let artifacts = workflow["expected_artifacts"]
                .as_array()
                .expect("expected_artifacts array");
            workflow["id"] == "imui-product-chain"
                && workflow["command"] == "python tools/diag_gate_imui_product_chain.py"
                && workflow["focused_command"]
                    == "python tools/diag_gate_imui_product_chain.py --only discovery"
                && workflow["launched_command"]
                    == "python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release"
                && workflow["docs"]
                    == "docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md"
                && workflow["suite"]
                    == "tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json"
                && artifacts
                    .iter()
                    .any(|artifact| artifact == "perf-docking/check.perf_thresholds.json")
                && artifacts
                    .iter()
                    .any(|artifact| artifact == "perf-docking/*/trace.chrome.json")
        }));

        let routes = value["first_open_routes"]
            .as_array()
            .expect("first_open_routes array");
        assert!(routes.iter().any(|route| {
            let demos = route["demo_commands"]
                .as_array()
                .expect("demo_commands array");
            let metrics = route["metrics_commands"]
                .as_array()
                .expect("metrics_commands array");
            let debug = route["debug_commands"]
                .as_array()
                .expect("debug_commands array");
            let handoff = route["handoff_commands"]
                .as_array()
                .expect("handoff_commands array");
            let actions = route["action_commands"]
                .as_array()
                .expect("action_commands array");
            route["id"] == "demo-metrics-debug"
                && route["docs"] == "docs/diagnostics-first-open.md"
                && route["owner_doc"]
                    == "docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json"
                && route["docking_owner_doc"]
                    == "docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json"
                && route["wayland_acceptance_doc"]
                    == "docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md"
                && demos.iter().any(|command| {
                    command["label"] == "demo editor notes"
                        && command["command"]
                            == "cargo run -p fret-demo --bin editor_notes_demo"
                })
                && metrics.iter().any(|command| {
                    command["label"] == "metrics layout perf"
                        && command["command"]
                            == "cargo run -p fretboard-dev -- diag layout-perf-summary <bundle-or-dir> --json"
                })
                && debug.iter().any(|command| {
                    command["label"] == "debug triage"
                        && command["command"]
                            == "cargo run -p fretboard-dev -- diag triage <bundle-or-dir> --json"
                })
                && debug.iter().any(|command| {
                    command["label"] == "debug trace"
                        && command["command"]
                            == "cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json"
                })
                && handoff.iter().any(|command| {
                    command["label"] == "docking arbitration supporting"
                        && command["command"]
                            == "cargo run -p fret-demo --bin docking_arbitration_demo"
                })
                && handoff.iter().any(|command| {
                    command["label"] == "docking campaign validate"
                        && command["command"]
                            == "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json"
                })
                && actions.iter().any(|command| {
                    command["label"] == "open workbench"
                        && command["command"]
                            == "cargo run -p fret-demo --bin imui_editor_workbench_demo"
                })
                && actions.iter().any(|command| {
                    command["label"] == "run product discovery"
                        && command["command"]
                            == "python tools/diag_gate_imui_product_chain.py --only discovery"
                })
                && actions.iter().any(|command| {
                    command["label"] == "inspect debug trace"
                        && command["command"]
                            == "cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json"
                })
        }));

        let tools = value["tool_apps"].as_array().expect("tool_apps array");
        assert!(tools.iter().any(|tool| tool["id"] == "fret-devtools"
            && tool["best_for"].as_str().is_some()
            && tool["gate"] == "cargo build -p fret-devtools"));
        assert!(tools.iter().any(|tool| tool["id"] == "fret-devtools-mcp"
            && tool["best_for"].as_str().is_some()
            && tool["gate"] == "cargo build -p fret-devtools-mcp"));
    }
}
