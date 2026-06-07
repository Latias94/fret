#!/usr/bin/env python3
"""Validate the current IMUI editor-grade product chain as one maintainer gate.

The lightweight default mode checks discoverability, promoted script/suite inputs, and source
guards. Use `--launched` when a local machine should also execute the existing launched diagnostics
proofs across the cookbook, editor proof, editor notes, and workspace shell apps.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path


DISCOVERY = "discovery"
GENERIC_ACTION = "generic-action"
EDITOR_CONTROLS = "editor-controls"
EDITOR_PROOF = "editor-proof"
EDITOR_NOTES = "editor-notes"
EDITOR_NOTES_DEVICE_SHELL = "editor-notes-device-shell"
WORKSPACE_SHELL = "workspace-shell"
DOCKING = "docking"
PERF_DOCKING = "perf-docking"
SOURCE_GATES = "source-gates"

DOCKING_PERF_THRESHOLDS: tuple[tuple[str, str, int], ...] = (
    ("--max-top-total-us", "max_top_total_us", 20_000),
    ("--max-top-layout-us", "max_top_layout_us", 10_000),
    ("--max-top-solve-us", "max_top_solve_us", 10_000),
    ("--max-pointer-move-dispatch-us", "max_pointer_move_dispatch_us", 5_000),
    ("--max-pointer-move-hit-test-us", "max_pointer_move_hit_test_us", 5_000),
    ("--max-pointer-move-global-changes", "max_pointer_move_global_changes", 0),
    ("--max-renderer-encode-scene-us", "max_renderer_encode_scene_us", 5_000),
    ("--max-renderer-upload-us", "max_renderer_upload_us", 5_000),
    ("--max-renderer-record-passes-us", "max_renderer_record_passes_us", 2_000),
    ("--max-renderer-encoder-finish-us", "max_renderer_encoder_finish_us", 2_000),
    ("--max-renderer-prepare-text-us", "max_renderer_prepare_text_us", 5_000),
    ("--max-renderer-prepare-svg-us", "max_renderer_prepare_svg_us", 2_000),
    ("--max-renderer-instance-bytes", "max_renderer_instance_bytes", 500_000),
    ("--max-renderer-encode-scene-text-ops", "max_renderer_encode_scene_text_ops", 10_000),
)

FIRST_OPEN_DOC = "docs/diagnostics-first-open.md"
DEVTOOLS_GUI_DOC = "docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md"
DEVTOOLS_MCP_DOC = "docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md"
DEMO_METRICS_DEBUG_ROUTE_ID = "demo-metrics-debug"
DEMO_METRICS_DEBUG_OWNER_DOC = (
    "docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json"
)
DEMO_METRICS_DEBUG_ACTION_METADATA_DOC = (
    "docs/workstreams/imui-demo-metrics-debug-action-metadata-v1/WORKSTREAM.json"
)
DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC = (
    "docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json"
)
DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC = "docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md"
DEMO_METRICS_DEBUG_ROUTE_COMMANDS = {
    "demo_commands": {
        "demo editor workbench": "cargo run -p fret-demo --bin imui_editor_workbench_demo",
        "demo editor proof supporting": "cargo run -p fret-demo --bin imui_editor_proof_demo",
        "demo editor notes": "cargo run -p fret-demo --bin editor_notes_demo",
        "demo device shell": "cargo run -p fret-demo --bin editor_notes_device_shell_demo",
    },
    "metrics_commands": {
        "metrics stats": "cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json",
        "metrics layout perf": "cargo run -p fretboard-dev -- diag layout-perf-summary <bundle-or-dir> --json",
        "metrics memory": "cargo run -p fretboard-dev -- diag memory-summary <bundle-or-dir> --json",
    },
    "debug_commands": {
        "debug triage": "cargo run -p fretboard-dev -- diag triage <bundle-or-dir> --json",
        "debug hotspots": "cargo run -p fretboard-dev -- diag hotspots <bundle-or-dir> --json",
        "debug trace": "cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json",
    },
    "handoff_commands": {
        "docking arbitration supporting": "cargo run -p fret-demo --bin docking_arbitration_demo",
        "docking campaign validate": "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json",
        "docking policy-skip local": "python tools/diag_gate_docking_wayland_policy_skip.py",
    },
    "action_commands": {
        "open workbench": "cargo run -p fret-demo --bin imui_editor_workbench_demo",
        "run product discovery": "python tools/diag_gate_imui_product_chain.py --only discovery",
        "inspect metrics stats": "cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json",
        "inspect debug trace": "cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json",
        "validate docking campaign": "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json",
    },
}
DEMO_METRICS_DEBUG_ACTION_COMMANDS = DEMO_METRICS_DEBUG_ROUTE_COMMANDS["action_commands"]
DEMO_METRICS_DEBUG_ACTION_METADATA = {
    "open workbench": {
        "id": "open_workbench",
        "category": "demo",
        "requires_bundle": False,
        "primary": True,
    },
    "run product discovery": {
        "id": "product_discovery",
        "category": "product-gate",
        "requires_bundle": False,
        "primary": False,
    },
    "inspect metrics stats": {
        "id": "inspect_metrics_stats",
        "category": "metrics",
        "requires_bundle": True,
        "primary": False,
    },
    "inspect debug trace": {
        "id": "inspect_debug_trace",
        "category": "debug",
        "requires_bundle": True,
        "primary": False,
    },
    "validate docking campaign": {
        "id": "validate_docking_campaign",
        "category": "handoff",
        "requires_bundle": False,
        "primary": False,
    },
}
DEVTOOLS_GUI_SOURCE = "apps/fret-devtools/src/native.rs"
DEVTOOLS_GUI_TEST_SOURCE = "apps/fret-devtools/src/native/tests.rs"
DEVTOOLS_GUI_COMMAND_CATALOG_SOURCE = "apps/fret-devtools/src/native/command_catalog.rs"
DEVTOOLS_GUI_WS_SOURCE = "apps/fret-devtools/src/ws.rs"
DEVTOOLS_GUI_SEMANTICS_SOURCE = "apps/fret-devtools/src/semantics.rs"
DEVTOOLS_GUI_GATE_RUN_SOURCE = "apps/fret-devtools/src/gate_run.rs"
DEVTOOLS_GUI_DISCOVERY_LINES_SOURCE = "apps/fret-devtools/src/native/discovery_lines.rs"
DEVTOOLS_GUI_GUIDE_REFERENCE_PANELS_SOURCE = (
    "apps/fret-devtools/src/native/guide_reference_panels.rs"
)
DEVTOOLS_GUI_GUIDE_RECENT_EVIDENCE_PANEL_SOURCE = (
    "apps/fret-devtools/src/native/guide_recent_evidence_panel.rs"
)
DEVTOOLS_GUI_HEADER_STATE_SOURCE = "apps/fret-devtools/src/native/header_state.rs"
DEVTOOLS_GUI_DIAGNOSTICS_TREE_PANEL_SOURCE = (
    "apps/fret-devtools/src/native/diagnostics_tree_panel.rs"
)
DEVTOOLS_GUI_SEMANTICS_DETAIL_PANEL_SOURCE = (
    "apps/fret-devtools/src/native/semantics_detail_panel.rs"
)
DEVTOOLS_GUI_INSPECT_PANEL_SOURCE = "apps/fret-devtools/src/native/inspect_panel.rs"
DEVTOOLS_GUI_GATE_PROFILE_STATE_SOURCE = "apps/fret-devtools/src/native/gate_profile_state.rs"
DEVTOOLS_GUI_WORKFLOW_PANEL_STATE_SOURCE = "apps/fret-devtools/src/native/workflow_panel_state.rs"
DEVTOOLS_GUI_RECENT_EVIDENCE_SOURCE = "apps/fret-devtools/src/native/recent_evidence.rs"
DEVTOOLS_GUI_DEMO_METRICS_DEBUG_SOURCE = "apps/fret-devtools/src/demo_metrics_debug.rs"
DEVTOOLS_GUI_DEMO_METRICS_DEBUG_ACTIONS_SOURCE = (
    "apps/fret-devtools/src/demo_metrics_debug/actions.rs"
)
DEVTOOLS_GUI_DEMO_METRICS_DEBUG_WORKFLOW_SOURCE = (
    "apps/fret-devtools/src/demo_metrics_debug/workflow.rs"
)
DEVTOOLS_GATE_PROFILE_SOURCE = "crates/fret-diag/src/devtools_gate_profiles.rs"
DEVTOOLS_PROTOCOL_SOURCE = "crates/fret-diag-protocol/src/lib.rs"
BOOTSTRAP_DEVTOOLS_WS_SOURCE = (
    "ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs"
)
DEVTOOLS_REPRO_CONTRACT_SOURCE = "crates/fret-diag/src/cli/contracts/commands/repro.rs"
DEVTOOLS_CUTOVER_SOURCE = "crates/fret-diag/src/cli/cutover.rs"
DEVTOOLS_GUI_FOLLOWUP_SOURCE = "apps/fret-devtools/src/followup.rs"
DEVTOOLS_GUI_FOLLOWUP_PANEL_SOURCE = (
    "apps/fret-devtools/src/native/followup_panel.rs"
)
DEVTOOLS_GUI_RUN_HISTORY_PANEL_SOURCE = (
    "apps/fret-devtools/src/native/run_history_panel.rs"
)
DEVTOOLS_MCP_SOURCE = "apps/fret-devtools-mcp/src/native.rs"
REPO_PREFLIGHT_COMMAND = "cargo run -p fretboard-dev -- diag doctor campaigns"
REPO_PREFLIGHT_JSON_COMMAND = "cargo run -p fretboard-dev -- diag doctor campaigns --json"
IMUI_PRODUCT_CHAIN_DOC = "docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md"
IMUI_PRODUCT_CHAIN_COMMAND = "python tools/diag_gate_imui_product_chain.py"
IMUI_PRODUCT_CHAIN_DISCOVERY_COMMAND = "python tools/diag_gate_imui_product_chain.py --only discovery"
IMUI_DOCKING_PERF_COMMAND = (
    "python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release"
)
IMUI_DOCKING_PERF_SUITE = "tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json"
IMUI_DOCKING_PERF_ARTIFACTS = {
    "perf-docking/regression.summary.json",
    "perf-docking/check.perf_thresholds.json",
    "perf-docking/*/trace.chrome.json",
}
PERF_TRACE_CHROME_KIND = "perf_trace_chrome"
PERF_TRACE_SOURCE_WITH_REAL_SPANS = "bundle_synthetic_phases_with_extension_spans"
PERF_TRACE_REAL_SPAN_EXTENSION_KEY = "fret.perf.spans.v1"

ALL_GATES = [
    DISCOVERY,
    GENERIC_ACTION,
    EDITOR_CONTROLS,
    EDITOR_PROOF,
    EDITOR_NOTES,
    EDITOR_NOTES_DEVICE_SHELL,
    WORKSPACE_SHELL,
    DOCKING,
    PERF_DOCKING,
    SOURCE_GATES,
]


@dataclass(frozen=True)
class ProductSurface:
    name: str
    suite: str | None = None
    campaign: str | None = None
    scripts: tuple[str, ...] = ()


PRODUCT_SURFACES = [
    ProductSurface(
        name=GENERIC_ACTION,
        scripts=(
            "tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json",
        ),
    ),
    ProductSurface(
        name=EDITOR_CONTROLS,
        suite="tools/diag-scripts/suites/cookbook-imui-editor-controls-basics/suite.json",
    ),
    ProductSurface(
        name=EDITOR_PROOF,
        suite="tools/diag-scripts/suites/imui-editor-proof-edit-outcomes/suite.json",
    ),
    ProductSurface(
        name=EDITOR_NOTES,
        suite="tools/diag-scripts/suites/editor-notes-demo/suite.json",
    ),
    ProductSurface(
        name=EDITOR_NOTES_DEVICE_SHELL,
        suite="tools/diag-scripts/suites/editor-notes-device-shell-demo/suite.json",
    ),
    ProductSurface(
        name=WORKSPACE_SHELL,
        suite="tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json",
    ),
    ProductSurface(
        name=DOCKING,
        campaign="tools/diag-campaigns/imui-p3-multiwindow-parity.json",
    ),
    ProductSurface(
        name=PERF_DOCKING,
        suite="tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json",
    ),
]


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _exe_name(stem: str) -> str:
    return f"{stem}.exe" if os.name == "nt" else stem


def _run_checked(name: str, argv: list[str], *, cwd: Path) -> None:
    print(f"[diag-gate-imui-product-chain] {name}", flush=True)
    proc = subprocess.run(argv, cwd=str(cwd), check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def _run_capture_checked(
    name: str,
    argv: list[str],
    *,
    cwd: Path,
) -> subprocess.CompletedProcess[str]:
    print(f"[diag-gate-imui-product-chain] {name}", flush=True)
    proc = subprocess.run(
        argv,
        cwd=str(cwd),
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        sys.stdout.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")
    return proc


def _read_json_file(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as err:
        raise SystemExit(f"failed to read JSON file: {path} ({err})") from err
    except json.JSONDecodeError as err:
        raise SystemExit(f"failed to parse JSON file: {path} ({err})") from err


def _parse_json_stdout(name: str, proc: subprocess.CompletedProcess[str]) -> dict:
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as err:
        raise SystemExit(f"Step failed: {name} (invalid JSON: {err})") from err


def _suite_scripts(repo_root: Path, suite_path: str) -> list[str]:
    suite = _read_json_file(repo_root / suite_path)
    if suite.get("kind") != "diag_script_suite_manifest":
        raise SystemExit(f"unexpected suite kind in {suite_path}: {suite.get('kind')!r}")
    scripts = suite.get("scripts")
    if not isinstance(scripts, list) or not scripts:
        raise SystemExit(f"suite has no scripts: {suite_path}")
    for script in scripts:
        if not isinstance(script, str) or not script.endswith(".json"):
            raise SystemExit(f"suite contains invalid script entry: {suite_path}")
    return scripts


def _path_from_json(value: object) -> Path | None:
    if not isinstance(value, str) or not value:
        return None
    return Path(value)


def _docking_perf_threshold_args() -> list[str]:
    args: list[str] = []
    for flag, _json_key, value in DOCKING_PERF_THRESHOLDS:
        args.extend([flag, str(value)])
    return args


def _docking_perf_threshold_values() -> dict[str, int]:
    return {json_key: value for _flag, json_key, value in DOCKING_PERF_THRESHOLDS}


def _selected_gate_names(raw_only: list[str]) -> set[str]:
    selected = {name for raw in raw_only for name in raw.split(",") if name.strip()}
    selected = {name.strip() for name in selected}
    if not selected:
        return set(ALL_GATES)
    unknown = sorted(selected - set(ALL_GATES))
    if unknown:
        raise SystemExit(
            "Unknown --only gate(s): "
            + ", ".join(unknown)
            + "\nKnown gates: "
            + ", ".join(ALL_GATES)
        )
    return selected


def _build_fretboard_dev(repo_root: Path, release: bool) -> Path:
    build_args = ["cargo", "build", "-j", "1", "-p", "fretboard-dev"]
    if release:
        build_args.append("--release")
    _run_checked("cargo build -p fretboard-dev", build_args, cwd=repo_root)
    profile_dir = "release" if release else "debug"
    fretboard_exe = repo_root / "target" / profile_dir / _exe_name("fretboard-dev")
    if not fretboard_exe.exists():
        raise SystemExit(f"fretboard-dev exe not found: {fretboard_exe}")
    return fretboard_exe


def _assert_contains(haystack: str, needle: str, name: str) -> None:
    if needle in haystack:
        return
    if " ".join(needle.split()) in " ".join(haystack.split()):
        return
    else:
        raise SystemExit(f"Step failed: {name} (missing marker: {needle})")


def _validate_tool_apps_json(payload: dict) -> None:
    if payload.get("kind") != "fretboard_tool_apps":
        raise SystemExit("Step failed: list tool apps json (expected kind=fretboard_tool_apps)")
    if payload.get("schema_version") != 1:
        raise SystemExit("Step failed: list tool apps json (expected schema_version=1)")
    if payload.get("first_open_doc") != FIRST_OPEN_DOC:
        raise SystemExit("Step failed: list tool apps json (missing canonical first-open doc)")
    if payload.get("branch_doc") != DEVTOOLS_GUI_DOC:
        raise SystemExit("Step failed: list tool apps json (missing DevTools GUI branch doc)")

    repo_preflight = payload.get("repo_preflight")
    if not isinstance(repo_preflight, dict):
        raise SystemExit("Step failed: list tool apps json (missing repo_preflight object)")
    if repo_preflight.get("command") != REPO_PREFLIGHT_COMMAND:
        raise SystemExit("Step failed: list tool apps json (unexpected repo preflight command)")
    if repo_preflight.get("json_command") != REPO_PREFLIGHT_JSON_COMMAND:
        raise SystemExit("Step failed: list tool apps json (unexpected repo preflight JSON command)")
    if not isinstance(repo_preflight.get("purpose"), str) or not repo_preflight["purpose"]:
        raise SystemExit("Step failed: list tool apps json (missing repo preflight purpose)")

    workflows = payload.get("product_workflows")
    if not isinstance(workflows, list):
        raise SystemExit("Step failed: list tool apps json (missing product_workflows array)")
    imui_workflow = next(
        (
            item
            for item in workflows
            if isinstance(item, dict) and item.get("id") == "imui-product-chain"
        ),
        None,
    )
    if imui_workflow is None:
        raise SystemExit("Step failed: list tool apps json (missing imui-product-chain workflow)")
    expected_workflow_fields = {
        "command": IMUI_PRODUCT_CHAIN_COMMAND,
        "focused_command": IMUI_PRODUCT_CHAIN_DISCOVERY_COMMAND,
        "launched_command": IMUI_DOCKING_PERF_COMMAND,
        "docs": IMUI_PRODUCT_CHAIN_DOC,
        "suite": IMUI_DOCKING_PERF_SUITE,
    }
    for field, expected in expected_workflow_fields.items():
        if imui_workflow.get(field) != expected:
            raise SystemExit(
                f"Step failed: list tool apps json (unexpected imui-product-chain {field})"
            )
    if not isinstance(imui_workflow.get("purpose"), str) or not imui_workflow["purpose"]:
        raise SystemExit("Step failed: list tool apps json (missing imui-product-chain purpose)")
    artifacts = imui_workflow.get("expected_artifacts")
    artifact_paths = (
        {artifact for artifact in artifacts if isinstance(artifact, str)}
        if isinstance(artifacts, list)
        else set()
    )
    if not IMUI_DOCKING_PERF_ARTIFACTS.issubset(artifact_paths):
        raise SystemExit(
            "Step failed: list tool apps json (missing imui-product-chain perf artifacts)"
        )

    first_open_routes = payload.get("first_open_routes")
    if not isinstance(first_open_routes, list):
        raise SystemExit("Step failed: list tool apps json (missing first_open_routes array)")
    demo_metrics_route = next(
        (
            item
            for item in first_open_routes
            if isinstance(item, dict) and item.get("id") == DEMO_METRICS_DEBUG_ROUTE_ID
        ),
        None,
    )
    if demo_metrics_route is None:
        raise SystemExit("Step failed: list tool apps json (missing demo-metrics-debug route)")
    if demo_metrics_route.get("docs") != FIRST_OPEN_DOC:
        raise SystemExit("Step failed: list tool apps json (unexpected demo-metrics-debug docs)")
    if demo_metrics_route.get("owner_doc") != DEMO_METRICS_DEBUG_OWNER_DOC:
        raise SystemExit("Step failed: list tool apps json (unexpected demo-metrics-debug owner_doc)")
    if demo_metrics_route.get("action_metadata_doc") != DEMO_METRICS_DEBUG_ACTION_METADATA_DOC:
        raise SystemExit(
            "Step failed: list tool apps json (unexpected demo-metrics-debug action_metadata_doc)"
        )
    if demo_metrics_route.get("docking_owner_doc") != DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC:
        raise SystemExit(
            "Step failed: list tool apps json (unexpected demo-metrics-debug docking_owner_doc)"
        )
    if demo_metrics_route.get("wayland_acceptance_doc") != DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC:
        raise SystemExit(
            "Step failed: list tool apps json (unexpected demo-metrics-debug wayland_acceptance_doc)"
        )
    if not isinstance(demo_metrics_route.get("purpose"), str) or not demo_metrics_route["purpose"]:
        raise SystemExit("Step failed: list tool apps json (missing demo-metrics-debug purpose)")
    for group, expected_commands in DEMO_METRICS_DEBUG_ROUTE_COMMANDS.items():
        commands = demo_metrics_route.get(group)
        if not isinstance(commands, list):
            raise SystemExit(f"Step failed: list tool apps json (missing {group})")
        commands_by_label = {
            item.get("label"): item.get("command")
            for item in commands
            if isinstance(item, dict)
        }
        for label, command in expected_commands.items():
            if commands_by_label.get(label) != command:
                raise SystemExit(
                    f"Step failed: list tool apps json (unexpected demo-metrics-debug {label})"
                )

    action_items = demo_metrics_route.get("action_commands")
    action_items_by_label = {
        item.get("label"): item for item in action_items if isinstance(item, dict)
    }
    for label, expected_metadata in DEMO_METRICS_DEBUG_ACTION_METADATA.items():
        item = action_items_by_label.get(label)
        if not isinstance(item, dict):
            raise SystemExit(
                f"Step failed: list tool apps json (missing demo-metrics-debug action metadata for {label})"
            )
        for field, expected_value in expected_metadata.items():
            if item.get(field) != expected_value:
                raise SystemExit(
                    f"Step failed: list tool apps json (unexpected demo-metrics-debug action metadata {label} {field})"
                )

    tool_apps = payload.get("tool_apps")
    if not isinstance(tool_apps, list):
        raise SystemExit("Step failed: list tool apps json (missing tool_apps array)")

    expected_tools = {
        "fret-devtools": (
            "cargo run -p fret-devtools",
            DEVTOOLS_GUI_DOC,
            "cargo build -p fret-devtools",
        ),
        "fret-devtools-mcp": (
            "cargo run -p fret-devtools-mcp",
            DEVTOOLS_MCP_DOC,
            "cargo build -p fret-devtools-mcp",
        ),
    }
    for tool_id, (command, docs, gate) in expected_tools.items():
        tool = next(
            (item for item in tool_apps if isinstance(item, dict) and item.get("id") == tool_id),
            None,
        )
        if tool is None:
            raise SystemExit(f"Step failed: list tool apps json (missing {tool_id})")
        if tool.get("command") != command:
            raise SystemExit(f"Step failed: list tool apps json (unexpected {tool_id} command)")
        if tool.get("docs") != docs:
            raise SystemExit(f"Step failed: list tool apps json (unexpected {tool_id} docs)")
        if tool.get("gate") != gate:
            raise SystemExit(f"Step failed: list tool apps json (unexpected {tool_id} gate)")
        if not isinstance(tool.get("purpose"), str) or not tool["purpose"]:
            raise SystemExit(f"Step failed: list tool apps json (missing {tool_id} purpose)")
        if not isinstance(tool.get("best_for"), str) or not tool["best_for"]:
            raise SystemExit(f"Step failed: list tool apps json (missing {tool_id} best_for)")


def _validate_devtools_gui_product_workflow_source(repo_root: Path) -> None:
    name = "devtools gui product workflow source"
    path = repo_root / DEVTOOLS_GUI_SOURCE
    tests_path = repo_root / DEVTOOLS_GUI_TEST_SOURCE
    command_catalog_path = repo_root / DEVTOOLS_GUI_COMMAND_CATALOG_SOURCE
    ws_path = repo_root / DEVTOOLS_GUI_WS_SOURCE
    semantics_path = repo_root / DEVTOOLS_GUI_SEMANTICS_SOURCE
    gate_run_path = repo_root / DEVTOOLS_GUI_GATE_RUN_SOURCE
    followup_panel_path = repo_root / DEVTOOLS_GUI_FOLLOWUP_PANEL_SOURCE
    run_history_panel_path = repo_root / DEVTOOLS_GUI_RUN_HISTORY_PANEL_SOURCE
    discovery_lines_path = repo_root / DEVTOOLS_GUI_DISCOVERY_LINES_SOURCE
    guide_reference_panels_path = repo_root / DEVTOOLS_GUI_GUIDE_REFERENCE_PANELS_SOURCE
    guide_recent_evidence_panel_path = (
        repo_root / DEVTOOLS_GUI_GUIDE_RECENT_EVIDENCE_PANEL_SOURCE
    )
    header_state_path = repo_root / DEVTOOLS_GUI_HEADER_STATE_SOURCE
    diagnostics_tree_panel_path = repo_root / DEVTOOLS_GUI_DIAGNOSTICS_TREE_PANEL_SOURCE
    semantics_detail_panel_path = repo_root / DEVTOOLS_GUI_SEMANTICS_DETAIL_PANEL_SOURCE
    inspect_panel_path = repo_root / DEVTOOLS_GUI_INSPECT_PANEL_SOURCE
    gate_profile_state_path = repo_root / DEVTOOLS_GUI_GATE_PROFILE_STATE_SOURCE
    workflow_panel_state_path = repo_root / DEVTOOLS_GUI_WORKFLOW_PANEL_STATE_SOURCE
    recent_evidence_path = repo_root / DEVTOOLS_GUI_RECENT_EVIDENCE_SOURCE
    demo_metrics_debug_path = repo_root / DEVTOOLS_GUI_DEMO_METRICS_DEBUG_SOURCE
    demo_metrics_debug_actions_path = (
        repo_root / DEVTOOLS_GUI_DEMO_METRICS_DEBUG_ACTIONS_SOURCE
    )
    demo_metrics_debug_workflow_path = (
        repo_root / DEVTOOLS_GUI_DEMO_METRICS_DEBUG_WORKFLOW_SOURCE
    )
    gate_profile_path = repo_root / DEVTOOLS_GATE_PROFILE_SOURCE
    protocol_path = repo_root / DEVTOOLS_PROTOCOL_SOURCE
    bootstrap_ws_path = repo_root / BOOTSTRAP_DEVTOOLS_WS_SOURCE
    repro_contract_path = repo_root / DEVTOOLS_REPRO_CONTRACT_SOURCE
    cutover_path = repo_root / DEVTOOLS_CUTOVER_SOURCE
    print(f"[diag-gate-imui-product-chain] {name}", flush=True)
    try:
        source = path.read_text(encoding="utf-8")
        test_source = tests_path.read_text(encoding="utf-8")
        command_catalog_source = command_catalog_path.read_text(encoding="utf-8")
        ws_source = ws_path.read_text(encoding="utf-8")
        semantics_source = semantics_path.read_text(encoding="utf-8")
        gate_run_source = gate_run_path.read_text(encoding="utf-8")
        followup_panel_source = followup_panel_path.read_text(encoding="utf-8")
        run_history_panel_source = run_history_panel_path.read_text(encoding="utf-8")
        discovery_lines_source = discovery_lines_path.read_text(encoding="utf-8")
        guide_reference_panels_source = guide_reference_panels_path.read_text(
            encoding="utf-8"
        )
        guide_recent_evidence_panel_source = (
            guide_recent_evidence_panel_path.read_text(encoding="utf-8")
        )
        header_state_source = header_state_path.read_text(encoding="utf-8")
        diagnostics_tree_panel_source = diagnostics_tree_panel_path.read_text(
            encoding="utf-8"
        )
        semantics_detail_panel_source = semantics_detail_panel_path.read_text(
            encoding="utf-8"
        )
        inspect_panel_source = inspect_panel_path.read_text(encoding="utf-8")
        gate_profile_state_source = gate_profile_state_path.read_text(encoding="utf-8")
        workflow_panel_state_source = workflow_panel_state_path.read_text(
            encoding="utf-8"
        )
        recent_evidence_source = recent_evidence_path.read_text(encoding="utf-8")
        demo_metrics_debug_source = demo_metrics_debug_path.read_text(encoding="utf-8")
        demo_metrics_debug_actions_source = demo_metrics_debug_actions_path.read_text(
            encoding="utf-8"
        )
        demo_metrics_debug_workflow_source = (
            demo_metrics_debug_workflow_path.read_text(encoding="utf-8")
        )
        gate_profile_source = gate_profile_path.read_text(encoding="utf-8")
        protocol_source = protocol_path.read_text(encoding="utf-8")
        bootstrap_ws_source = bootstrap_ws_path.read_text(encoding="utf-8")
        repro_contract_source = repro_contract_path.read_text(encoding="utf-8")
        cutover_source = cutover_path.read_text(encoding="utf-8")
    except OSError as err:
        raise SystemExit(f"Step failed: {name} (failed to read source: {err})") from err
    source = "\n".join(
        [
            source,
            test_source,
            command_catalog_source,
            ws_source,
            semantics_source,
            gate_run_source,
            followup_panel_source,
            run_history_panel_source,
            discovery_lines_source,
            guide_reference_panels_source,
            guide_recent_evidence_panel_source,
            header_state_source,
            diagnostics_tree_panel_source,
            semantics_detail_panel_source,
            inspect_panel_source,
            gate_profile_state_source,
            workflow_panel_state_source,
            recent_evidence_source,
            demo_metrics_debug_source,
            demo_metrics_debug_actions_source,
            demo_metrics_debug_workflow_source,
            gate_profile_source,
            protocol_source,
            bootstrap_ws_source,
            repro_contract_source,
            cutover_source,
        ]
    )

    for marker in (
        "const IMUI_PRODUCT_WORKFLOW_ID: &str = fret_first_open::product_workflow::ID;",
        "const IMUI_PRODUCT_WORKFLOW_DOC: &str = fret_first_open::product_workflow::DOC;",
        "const IMUI_PRODUCT_WORKFLOW_COMMAND: &str = fret_first_open::product_workflow::COMMAND;",
        'const IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND: &str =',
        "fret_first_open::product_workflow::FOCUSED_COMMAND;",
        'const IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND: &str =',
        "fret_first_open::product_workflow::LAUNCHED_COMMAND;",
        "const IMUI_PRODUCT_WORKFLOW_SUITE: &str = fret_first_open::product_workflow::SUITE;",
        'const IMUI_PRODUCT_WORKFLOW_ARTIFACTS: &[&str] =',
        "fret_first_open::product_workflow::EXPECTED_ARTIFACTS;",
        'const DEVTOOLS_DOGFOOD_WORKFLOW_ID: &str = "ui-gallery-button-dogfood"',
        'const DEVTOOLS_DOGFOOD_TARGET_COMMAND: &str = "cargo run -p fret-ui-gallery --release"',
        "const DEVTOOLS_DOGFOOD_BASE_SCRIPT: &str =",
        '"tools/diag-scripts/ui-gallery-lite-smoke.json"',
        'const DEVTOOLS_DOGFOOD_BUTTON_SCRIPT: &str =',
        'const DEVTOOLS_DOGFOOD_PICK_SCRIPT_COMMAND: &str =',
        'const DEVTOOLS_DOGFOOD_PICK_APPLY_COMMAND: &str =',
        'const DEVTOOLS_DOGFOOD_RUN_PACK_COMMAND: &str =',
        'const DEVTOOLS_DOGFOOD_PACK_COMMAND: &str =',
        'const DEVTOOLS_DOGFOOD_VIEWER_COMMAND: &str = "pnpm -C tools/fret-bundle-viewer dev"',
        "const DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID: &str = fret_first_open::demo_metrics_debug::ROUTE_ID;",
        "const DEVTOOLS_DEMO_METRICS_DEBUG_OWNER_DOC: &str =",
        "const DEVTOOLS_DEMO_METRICS_DEBUG_ACTION_METADATA_DOC: &str =",
        "const DEVTOOLS_DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC: &str =",
        "const DEVTOOLS_DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC: &str =",
        "const DEVTOOLS_DOCKING_ARBITRATION_COMMAND: &str =",
        "const DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND: &str =",
        "const DEVTOOLS_DOCKING_POLICY_SKIP_COMMAND: &str =",
        'const DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND: &str =',
        'const DEVTOOLS_DEMO_EDITOR_PROOF_COMMAND: &str =',
        'const DEVTOOLS_METRICS_STATS_COMMAND: &str =',
        'const DEVTOOLS_METRICS_LAYOUT_PERF_COMMAND: &str =',
        'const DEVTOOLS_DEBUG_TRIAGE_COMMAND: &str =',
        'const DEVTOOLS_DEBUG_TRACE_COMMAND: &str =',
        'const CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS: &str =',
        "type DemoMetricsDebugActionSpec = fret_first_open::demo_metrics_debug::RouteCommand;",
        "fret_first_open::demo_metrics_debug::ACTION_COMMANDS",
        "DevtoolsGateScriptTargetCommandInputV1",
        "DevtoolsGatePerfThresholdCommandInputV1",
        "DevtoolsGateResourceFootprintThresholdCommandInputV1",
        "UiInspectHoverV1",
        "UiInspectFocusV1",
        "UiInspectOverlayHookV1",
        "UiInspectNodeSummaryV1",
        "UiOverlayRootHintV1",
        "UiOverlaySummaryV1",
        "devtools_gate_profile_lines",
        "devtools_gate_profiles_v1",
        "devtools_gate_perf_threshold_command",
        "devtools_gate_resource_footprint_threshold_command",
        "devtools_gate_script_target_command",
        "devtools_gate_script_target_profile_ids_v1",
        "mod demo_metrics_debug;",
        "mod followup;",
        "#[path = \"native/followup_panel.rs\"]",
        "mod followup_panel;",
        "mod gate_run;",
        "#[path = \"native/diagnostics_tree_panel.rs\"]",
        "mod diagnostics_tree_panel;",
        "use diagnostics_tree_panel::{element_tree_panel, layout_tree_panel, semantics_panel};",
        "#[path = \"native/semantics_detail_panel.rs\"]",
        "mod semantics_detail_panel;",
        "use semantics_detail_panel::sem_node_panel;",
        "#[path = \"native/inspect_panel.rs\"]",
        "mod inspect_panel;",
        'const CMD_GATE_RUN_GENERATED: &str = "fret.devtools.gate.run_generated"',
        'const CMD_REGRESSION_RUN_FOLLOWUP_STATS: &str =',
        'const CMD_REGRESSION_RUN_FOLLOWUP_LAYOUT_PERF: &str =',
        'const CMD_REGRESSION_RUN_FOLLOWUP_MEMORY: &str =',
        'const CMD_REGRESSION_RUN_FOLLOWUP_TRIAGE: &str =',
        'const CMD_REGRESSION_RUN_FOLLOWUP_HOTSPOTS: &str =',
        'const CMD_COPY_FOLLOWUP_RESULT_JSON: &str =',
        'const CMD_COPY_FOLLOWUP_RESULT_COMMAND: &str =',
        'const CMD_OPEN_FOLLOWUP_RESULT_JSON: &str =',
        "followup::poll_followup_jobs(cx.app, st)",
        "gate_run::poll_gate_run_jobs(cx.app, st)",
        "followup_in_flight",
        "followup_last_result_path",
        "followup_last_result_json",
        "followup_result_history",
        "followup_selected_result_path",
        "followup::followup_result_summary_lines(&selected_followup_result_json)",
        "followup::followup_result_history_summary_lines(",
        "followup::followup_result_history_entries_for_selected_bundle(",
        "followup::followup_result_history_selected_or_latest_entry(",
        "followup::followup_result_history_entry_detail_lines(",
        "fn followup_history_list(",
        "fn selected_followup_history_filter_dirs_from_bundle_dirs(",
        "fn selected_followup_result_entry_from_state(",
        "fn selected_followup_result_path_from_state(",
        "fn selected_followup_result_command_from_state(",
        "fn selected_followup_result_json_from_state(",
        "fn file_url_from_path(",
        "fn percent_encode_file_url_path(",
        "10%20stats%23failed.json",
        "Dogfood Workflow",
        "UI gallery selector capture, script patching, run/pack, and offline viewer handoff stay visible from the GUI shell.",
        "devtools_dogfood_workflow_lines(st.cfg.fs_out_dir.as_ref())",
        "Demo / Metrics / Debug Routes",
        "devtools_demo_metrics_debug_lines_with_state(",
        "demo_metrics_debug_rows.push(devtools_demo_metrics_debug_action_row(",
        "Always-available editor demos, action commands, metrics commands, and debug drill-down entrypoints stay visible in the GUI shell.",
        "Copy Demo/Metrics/Debug actions",
        "route owner: {DEVTOOLS_DEMO_METRICS_DEBUG_OWNER_DOC}",
        "action metadata owner: {DEVTOOLS_DEMO_METRICS_DEBUG_ACTION_METADATA_DOC}",
        "docking owner: {DEVTOOLS_DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC}",
        "wayland acceptance: {DEVTOOLS_DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC}",
        "action surface: dedicated DevTools guide panel + copyable action command bundle",
        "command palette: deferred until DevTools has a shared command palette contract",
        "action: open workbench -> {DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND}",
        "action: run product discovery -> {IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND}",
        "action: inspect metrics stats -> {DEVTOOLS_METRICS_STATS_COMMAND}",
        "action: inspect debug trace -> {DEVTOOLS_DEBUG_TRACE_COMMAND}",
        "action: validate docking campaign -> {DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND}",
        "action metadata: {} | id={} | category={} | primary={} | requires_bundle={}",
        "action readiness: {} | id={} | category={} | runnable={} | reason={}",
        "workflow readiness: validate docking campaign",
        "workflow readiness: run perf docking suite",
        "workflow status: in_flight={workflow_run_in_flight} | last_result={last_result} | last_error={last_error}",
        "fn demo_metrics_debug_action_command_text() -> String",
        "fn demo_metrics_debug_action_metadata_lines() -> Vec<String>",
        "fn demo_metrics_debug_action_readiness_lines(",
        "fn demo_metrics_debug_workflow_readiness_lines(",
        "fn demo_metrics_debug_workflow_status_lines(",
        "fn demo_metrics_debug_workflow_result_action_lines(",
        "fn demo_metrics_debug_workflow_artifact_action_lines(",
        "selected_bundle_count: usize",
        "workflow_result_available: bool",
        "regression_summary_available: bool",
        "regression_index_available: bool",
        "fn devtools_demo_metrics_debug_lines_with_state(",
        "demo_metrics_debug_selected_bundle_count",
        "select a regression bundle",
        "selected bundle evidence available",
        "select a DevTools session",
        "selected session available",
        "workflow run already in flight",
        "workflow result action: copy workflow result",
        "workflow result action: open workflow JSON",
        "workflow artifact action: load regression summary",
        "workflow artifact action: load regression index",
        "wait for workflow result artifact",
        "workflow result available",
        "wait for workflow regression summary artifact",
        "wait for workflow regression index artifact",
        "workflow regression summary available",
        "workflow regression index available",
        "fn devtools_demo_metrics_debug_action_row(",
        "workflow_run_in_flight: bool",
        "perf_workflow_runnable: bool",
        "Copy Demo/Metrics/Debug actions",
        "Run docking workflow",
        "Run perf workflow",
        "Copy workflow result",
        "Open workflow JSON",
        "Load workflow regression summary",
        "Load workflow regression index",
        "CMD_COPY_WORKFLOW_RESULT_PATH",
        "CMD_OPEN_WORKFLOW_RESULT_JSON",
        "CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY",
        "CMD_LOAD_WORKFLOW_REGRESSION_INDEX",
        "requires=selected-session",
        "docking campaign validate: {DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND}",
        "docking policy-skip local: {DEVTOOLS_DOCKING_POLICY_SKIP_COMMAND}",
        "Gate Commands",
        "Live Inspect Hover Bounds",
        "Structured hovered-node bounds projected from inspect.hover.",
        "Live Inspect Overlay Hooks",
        "Viewport overlay hooks and overlay.summary root hints for live inspect overlays.",
        "Raw Inspect Payloads",
        "Live semantics JSON",
        "hit_test.explain status={}",
        "children: <none>",
        "VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16)",
        "options.items_revision = rows_key",
        "let mut stack = Vec::with_capacity(index.roots.len().max(1));",
        "for child in children.iter().rev()",
        "live_semantics_request_decision(",
        "now_unix_ms.saturating_sub(prev) >= 1000",
        "devtools.inspect.hover_bounds",
        "devtools.inspect.overlay_hooks",
        "devtools.inspect.raw_payloads",
        "last_overlay_summary_json",
        '"overlay.summary"',
        "inspect_hover_bounds_lines(",
        "inspect_overlay_hook_lines(",
        "ws_publish_live_inspect_payloads(",
        "ws_send_live_payload_if_changed(",
        "inspect_node_summary_v1(",
        "overlay_summary_v1(",
        "hovered-node-bounds",
        "focused-node-bounds",
        "devtools_gate_command_lines(st.cfg.fs_out_dir.as_ref())",
        "gate_command_rows.push(devtools_gate_profile_command_builder(cx, st))",
        "devtools_gate_profile_lines(artifacts_root)",
        "generated_gate_command_from_state(app, st)",
        "devtools_gate_perf_threshold_command(input)",
        "devtools_gate_profile_action_rows(cx)",
        "Copy generated command",
        "Run generated command",
        "Copy command",
        "missing inputs:",
        "diag args:",
        "gate_run_in_flight",
        "gate_run_last_result_path",
        "gate_run_last_result_json",
        "gate_run_result_history",
        "gate_run_selected_result_path",
        "gate_run_last_error",
        "last_gate_result=",
        "gate_run::start_gate_run(app, st, command)",
        "gate_run_history_list(",
        "gate_run::gate_run_result_summary_lines(",
        "gate_run::gate_run_result_history_summary_lines(",
        "gate_run::gate_run_result_history_selected_or_latest_entry(",
        "gate_run::gate_run_result_history_entry_detail_lines(",
        "gate_run::load_recent_gate_run_result_history(",
        "fret_devtools_gate_run_result",
        'join(".fret").join("diag").join("gate-runs")',
        "new_gate_run_channel",
        "Generated Gate Result Details",
        "Generated Gate Result Summary",
        "Generated Gate Result History",
        "Copy gate result",
        "Open gate JSON",
        "Copy gate command",
        "Copy gate JSON",
        'const CMD_COPY_GATE_RESULT_PATH: &str = "fret.devtools.gate.copy_result_path"',
        'const CMD_COPY_GATE_RESULT_JSON: &str = "fret.devtools.gate.copy_result_json"',
        'const CMD_COPY_GATE_RESULT_COMMAND: &str = "fret.devtools.gate.copy_result_command"',
        'const CMD_OPEN_GATE_RESULT_JSON: &str = "fret.devtools.gate.open_result_json"',
        "devtools.gate.script_json",
        "devtools.gate.test_id",
        "devtools.gate.perf_target",
        "devtools.gate.perf_repeat",
        "devtools.gate.perf_warmup_frames",
        "devtools.gate.perf_threshold_agg",
        "devtools.gate.perf_max_top_total_us",
        "devtools.gate.perf_max_top_layout_us",
        "devtools.gate.perf_max_top_solve_us",
        "devtools.gate.perf_max_pointer_move_dispatch_us",
        "devtools.gate.perf_max_pointer_move_hit_test_us",
        "devtools.gate.perf_max_pointer_move_global_changes",
        "devtools.gate.perf_max_renderer_encode_scene_us",
        "devtools.gate.perf_max_renderer_upload_us",
        "devtools.gate.perf_max_renderer_record_passes_us",
        "devtools.gate.perf_max_renderer_encoder_finish_us",
        "devtools.gate.perf_max_renderer_prepare_text_us",
        "devtools.gate.perf_max_renderer_prepare_svg_us",
        "devtools.gate.perf_max_renderer_instance_bytes",
        "devtools.gate.perf_max_renderer_encode_scene_text_ops",
        "devtools.gate.resource_target",
        "devtools.gate.resource_launch_command",
        "devtools.gate.resource_max_working_set_bytes",
        "devtools.gate.resource_max_peak_working_set_bytes",
        "devtools.gate.resource_max_cpu_avg_percent_total_cores",
        "max_working_set_bytes: args.max_working_set_bytes",
        "max_peak_working_set_bytes: args.max_peak_working_set_bytes",
        "max_cpu_avg_percent_total_cores: args.max_cpu_avg_percent_total_cores",
        "pub max_working_set_bytes: Option<u64>",
        "pub max_peak_working_set_bytes: Option<u64>",
        "pub max_cpu_avg_percent_total_cores: Option<f64>",
        "fn generated_gate_command_from_state(",
        "fn script_target_gate_inputs(",
        "fn perf_threshold_gate_inputs(",
        "fn resource_footprint_threshold_gate_inputs(",
        "fn devtools_dogfood_workflow_lines(artifacts_root: &str) -> Vec<String>",
        "fn devtools_demo_metrics_debug_lines(artifacts_root: &str) -> Vec<String>",
        "fn devtools_gate_command_lines(artifacts_root: &str) -> Vec<String>",
        "fn devtools_gate_profile_command_builder(",
        "fn devtools_gate_profile_action_rows(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement>",
        "dogfood workflow: {DEVTOOLS_DOGFOOD_WORKFLOW_ID}",
        "open ui gallery: {DEVTOOLS_DOGFOOD_TARGET_COMMAND}",
        'preferred selector: {\\"kind\\":\\"test_id\\",\\"id\\":\\"ui-gallery-nav-button\\"}',
        "apply pick to script: {DEVTOOLS_DOGFOOD_PICK_APPLY_COMMAND}",
        "run and pack: {DEVTOOLS_DOGFOOD_RUN_PACK_COMMAND}",
        "open viewer: {DEVTOOLS_DOGFOOD_VIEWER_COMMAND}",
        "route: {DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID}",
        "metrics stats: {DEVTOOLS_METRICS_STATS_COMMAND}",
        "debug triage: {DEVTOOLS_DEBUG_TRIAGE_COMMAND}",
        "debug trace: {DEVTOOLS_DEBUG_TRACE_COMMAND}",
        "devtools_demo_metrics_debug_lines_surface_canonical_routes",
        "devtools_dogfood_workflow_lines_surface_ui_gallery_loop",
        "devtools_gate_command_lines_surface_first_class_gates",
        "compute_rows_handles_50k_flat_semantics_nodes",
        "compute_rows_handles_50k_deep_semantics_tree_without_recursion",
        "compute_rows_search_forces_visible_ancestor_path_on_large_tree",
        "live_semantics_request_decision_throttles_unchanged_selection_to_one_hz",
        "live_semantics_request_decision_allows_selection_change_and_manual_refresh",
        "file_url_from_path_projects_native_artifact_paths",
        "regression_selected_perf_evidence",
        "regression_summary_drilldown(&summary)",
        "regression_bundle_followup_commands(selected_bundle_dirs.iter().map(|v| v.as_ref()))",
        "regression_bundle_followup_command_lines(selected_bundle_dirs.iter().map(|v| v.as_ref()))",
        "Copy follow-up commands",
        "Follow-up Commands",
        "Runnable Follow-ups",
        "Manual Compare Follow-ups",
        "Follow-up Run Status",
        "Run stats",
        "Run layout perf",
        "Run memory",
        "Run triage",
        "Run hotspots",
        "Copy selected follow-up result",
        "copy selected follow-up result refused (no selected-bundle result artifact yet)",
        "Copy selected follow-up command",
        "copy selected follow-up command refused (no selected-bundle result command yet)",
        "Copy selected follow-up JSON",
        "copy selected follow-up JSON refused (no selected-bundle result JSON yet)",
        "Open selected follow-up JSON",
        "open selected follow-up JSON refused (no selected-bundle result artifact yet)",
        "Copy selected trace artifact",
        "copy selected trace artifact refused (no selected-bundle trace artifact yet)",
        "Open selected trace artifact",
        "open selected trace artifact refused (no selected-bundle trace artifact yet)",
        "selected_followup_trace_artifact_path_from_state",
        "Follow-up Result Details",
        "Selected result status, path, command, bundle, and error preview for reproduction.",
        "Follow-up Result Summary",
        "Status, command, duration, and error preview from the latest selected-bundle follow-up result.",
        "Follow-up Result History",
        "Select a GUI-launched follow-up result for the selected bundle, newest first.",
        "follow-up history entries: <none for selected bundle>",
        "Follow-up Result JSON",
        "The latest selected-bundle follow-up result artifact is mirrored here for quick triage.",
        "<no selected-bundle follow-up result yet>",
        "last_followup_result={result}",
        "run_selected_regression_followup(app, st, \"stats\")",
        "run_selected_regression_followup(app, st, \"layout-perf-summary\")",
        "run_selected_regression_followup(app, st, \"memory-summary\")",
        "run_selected_regression_followup(app, st, \"triage\")",
        "run_selected_regression_followup(app, st, \"hotspots\")",
        "Runnable follow-up commands:",
        "Manual compare follow-up commands:",
        "Perf Evidence",
        "perf_summary_json",
        "compare_json",
        "threshold_failures",
        "load_regression_summary_drilldown_collects_perf_evidence",
        "product workflow: {IMUI_PRODUCT_WORKFLOW_ID}",
        "product workflow command: {IMUI_PRODUCT_WORKFLOW_COMMAND}",
        "product workflow focused: {IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND}",
        "product workflow launched: {IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND}",
        "product workflow suite: {IMUI_PRODUCT_WORKFLOW_SUITE}",
        "product workflow docs: {IMUI_PRODUCT_WORKFLOW_DOC}",
        "product workflow artifacts: {}",
        "IMUI_PRODUCT_WORKFLOW_ARTIFACTS.join(\", \")",
    ):
        _assert_contains(source, marker, name)
    for marker in (
        "pub struct DevtoolsGateProfileV1",
        "pub struct DevtoolsGateCommandV1",
        "pub struct DevtoolsGateScriptTargetCommandInputV1",
        "pub struct DevtoolsGatePerfThresholdCommandInputV1",
        "pub struct DevtoolsGateResourceFootprintThresholdCommandInputV1",
        "pub type DevtoolsGateScriptTargetCommandV1 = DevtoolsGateCommandV1",
        'pub const DEVTOOLS_GATE_PERF_THRESHOLD_PROFILE_ID_V1: &str = "perf-thresholds"',
        'pub const DEVTOOLS_GATE_PERF_DOCKING_TARGET_V1: &str = "perf-docking-arbitration-steady"',
        'pub const DEVTOOLS_GATE_PERF_DOCKING_REPEAT_V1: &str = "1"',
        'pub const DEVTOOLS_GATE_PERF_DOCKING_AGG_V1: &str = "max"',
        "pub const DEVTOOLS_GATE_PERF_DOCKING_MAX_TOP_LAYOUT_US_V1",
        "pub const DEVTOOLS_GATE_PERF_DOCKING_MAX_POINTER_MOVE_GLOBAL_CHANGES_V1",
        "pub const DEVTOOLS_GATE_PERF_DOCKING_MAX_RENDERER_ENCODE_SCENE_TEXT_OPS_V1",
        "pub fn product_chain_docking_defaults()",
        'pub const DEVTOOLS_GATE_RESOURCE_FOOTPRINT_THRESHOLD_PROFILE_ID_V1: &str =',
        "pub const DEVTOOLS_GATE_SCRIPT_TARGET_PROFILE_IDS_V1",
        'pub const DEVTOOLS_GATE_STALE_COMMAND: &str =',
        'pub const DEVTOOLS_GATE_PIXELS_CHANGED_COMMAND: &str =',
        'pub const DEVTOOLS_GATE_PERF_THRESHOLDS_COMMAND: &str =',
        'pub const DEVTOOLS_GATE_RESOURCE_FOOTPRINT_THRESHOLDS_COMMAND: &str =',
        'pub const DEVTOOLS_GATE_RESOURCE_FOOTPRINT_COMPARE_COMMAND: &str =',
        'id: "stale-paint-scene"',
        'id: "pixels-changed"',
        'id: "perf-thresholds"',
        'id: "resource-footprint-thresholds"',
        'id: "resource-footprint-compare"',
        "gate route: first-class-gates",
        "stale paint/scene",
        "pixels changed",
        "perf thresholds",
        "resource footprint thresholds",
        "resource footprint compare",
        "check.pixels_changed.json",
        "check.perf_thresholds.json",
        "check.resource_footprint.json",
        "resource.footprint.json",
        "pub fn devtools_gate_profile_lines(artifacts_root: &str) -> Vec<String>",
        "pub fn devtools_gate_profiles_v1() -> &'static [DevtoolsGateProfileV1]",
        "pub fn devtools_gate_perf_threshold_command(",
        "pub fn devtools_gate_perf_threshold_command_line(",
        "pub fn devtools_gate_resource_footprint_threshold_command(",
        "pub fn devtools_gate_resource_footprint_threshold_command_line(",
        "--max-top-layout-us",
        "--max-top-solve-us",
        "--max-pointer-move-global-changes",
        "--max-renderer-prepare-svg-us",
        "--max-renderer-instance-bytes",
        "--max-renderer-encode-scene-text-ops",
        "pub fn devtools_gate_script_target_profile_ids_v1() -> &'static [&'static str]",
        "pub fn devtools_gate_script_target_command(",
        "pub fn devtools_gate_script_target_command_line(",
        "pub fn is_runnable(&self) -> bool",
        "devtools_gate_script_target_profiles_are_parameterized",
        "devtools_gate_script_target_commands_include_runnable_diag_args",
        "devtools_gate_perf_threshold_command_includes_runnable_diag_args",
        "devtools_gate_perf_threshold_product_chain_defaults_are_runnable",
        "devtools_gate_resource_footprint_threshold_command_includes_runnable_diag_args",
    ):
        _assert_contains(gate_profile_source, marker, name)


def _validate_devtools_gui_followup_source(repo_root: Path) -> None:
    name = "devtools gui followup source"
    path = repo_root / DEVTOOLS_GUI_FOLLOWUP_SOURCE
    print(f"[diag-gate-imui-product-chain] {name}", flush=True)
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as err:
        raise SystemExit(f"Step failed: {name} (failed to read {path}: {err})") from err

    for marker in (
        "pub(crate) fn runnable_diag_args_for_followup_command",
        "pub(crate) struct FollowupResultHistoryEntry",
        "pub(crate) fn followup_result_summary_lines",
        "pub(crate) fn followup_result_history_summary_lines",
        "pub(crate) fn followup_result_history_entries_for_selected_bundle",
        "pub(crate) fn followup_result_history_selected_or_latest_entry",
        "pub(crate) fn followup_result_history_entry_detail_lines",
        "pub(crate) fn load_recent_followup_result_history",
        "load_recent_followup_result_history_from_dir",
        "fret_devtools_regression_followup_result",
        "follow-up result: <invalid json>",
        "follow-up history: <none for selected bundle>",
        "followup_result_record_json",
        "write_followup_result_record",
        "command.requires_baseline",
        "follow-up command already in progress",
        "fret_diag::diag_cmd(args)",
        "follow-up started: {label} ({id})",
        "regression_followup_command_rejects_baseline_required_commands",
        "regression_followup_command_returns_direct_diag_args",
        "regression_followup_result_record_has_stable_shape",
        "trace_report: Option<FollowupTraceReportV1>",
        "struct FollowupTraceReportV1",
        "followup_trace_report_for_artifacts(&output_artifacts, repo_root)",
        "followup_trace_artifact_path_from_result_json",
        "regression_followup_trace_artifact_path_prefers_trace_report",
        "regression_followup_trace_artifact_path_falls_back_to_output_artifacts",
        "trace_source: {source}",
        "real_spans_included: {included}",
        "regression_followup_result_summary_lines_project_status_and_duration",
        "regression_followup_result_history_summary_filters_to_selected_bundle",
        "load_recent_followup_result_history_reads_latest_valid_records",
        "regression_followup_result_history_latest_path_prefers_selected_bundle",
        "regression_followup_result_history_selected_entry_overrides_latest_when_matching",
        "regression_followup_result_history_entry_detail_lines_surface_repro_fields",
    ):
        _assert_contains(source, marker, name)


def _validate_devtools_mcp_product_workflow_source(repo_root: Path) -> None:
    name = "devtools mcp product workflow source"
    path = repo_root / DEVTOOLS_MCP_SOURCE
    print(f"[diag-gate-imui-product-chain] {name}", flush=True)
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as err:
        raise SystemExit(f"Step failed: {name} (failed to read {path}: {err})") from err

    for marker in (
        'const RESOURCE_URI_FIRST_OPEN_MD: &str = "fret-diag://first-open.md"',
        'const RESOURCE_KIND_FIRST_OPEN_MD: &str = "first-open.md"',
        'const DEVTOOLS_FIRST_OPEN_DOC: &str = "docs/diagnostics-first-open.md"',
        'const DEVTOOLS_MCP_DOC: &str =',
        "docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md",
        "const IMUI_PRODUCT_WORKFLOW_ID: &str = fret_first_open::product_workflow::ID;",
        "const IMUI_PRODUCT_WORKFLOW_COMMAND: &str = fret_first_open::product_workflow::COMMAND;",
        'const IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND: &str =',
        "fret_first_open::product_workflow::FOCUSED_COMMAND;",
        'const IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND: &str =',
        "fret_first_open::product_workflow::LAUNCHED_COMMAND;",
        "const IMUI_PRODUCT_WORKFLOW_SUITE: &str = fret_first_open::product_workflow::SUITE;",
        'const IMUI_PRODUCT_WORKFLOW_ARTIFACTS: &[&str] =',
        "fret_first_open::product_workflow::EXPECTED_ARTIFACTS;",
        "const DEMO_METRICS_DEBUG_ROUTE_ID: &str = fret_first_open::demo_metrics_debug::ROUTE_ID;",
        "const DEMO_METRICS_DEBUG_OWNER_DOC: &str =",
        "const DEMO_METRICS_DEBUG_ACTION_METADATA_DOC: &str =",
        "const DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC: &str =",
        "const DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC: &str =",
        "const DOCKING_ARBITRATION_COMMAND: &str =",
        "const DOCKING_CAMPAIGN_VALIDATE_COMMAND: &str =",
        "const DOCKING_POLICY_SKIP_COMMAND: &str =",
        "type DemoMetricsDebugActionSpec = fret_first_open::demo_metrics_debug::RouteCommand;",
        "fret_first_open::demo_metrics_debug::ACTION_COMMANDS",
        'const DEMO_EDITOR_WORKBENCH_COMMAND: &str =',
        'const DEMO_EDITOR_PROOF_COMMAND: &str =',
        'const DEBUG_TRACE_COMMAND: &str =',
        "mcp_first_open_resource_text",
        "mcp first-open: {DEVTOOLS_FIRST_OPEN_DOC}",
        "mcp workflow: {DEVTOOLS_MCP_DOC}",
        "tool-app index: {DEVTOOLS_TOOL_APP_INDEX_COMMAND}",
        "tool-app index json: {DEVTOOLS_TOOL_APP_INDEX_JSON_COMMAND}",
        "product workflow: {IMUI_PRODUCT_WORKFLOW_ID}",
        "product workflow command: {IMUI_PRODUCT_WORKFLOW_COMMAND}",
        "product workflow focused: {IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND}",
        "product workflow launched: {IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND}",
        "product workflow suite: {IMUI_PRODUCT_WORKFLOW_SUITE}",
        "product workflow docs: {IMUI_PRODUCT_WORKFLOW_DOC}",
        "product workflow artifacts: {}",
        "IMUI_PRODUCT_WORKFLOW_ARTIFACTS.join(\", \")",
        "route: {DEMO_METRICS_DEBUG_ROUTE_ID}",
        "route owner: {DEMO_METRICS_DEBUG_OWNER_DOC}",
        "action metadata owner: {DEMO_METRICS_DEBUG_ACTION_METADATA_DOC}",
        "docking owner: {DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC}",
        "wayland acceptance: {DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC}",
        "action surface: dedicated DevTools guide panel + MCP first-open action list",
        "command palette: deferred until DevTools has a shared command palette contract",
        "action: {} -> {}",
        "action metadata: {} | id={} | category={} | primary={} | requires_bundle={}",
        "docking campaign validate: {DOCKING_CAMPAIGN_VALIDATE_COMMAND}",
        "docking policy-skip local: {DOCKING_POLICY_SKIP_COMMAND}",
        "demo editor workbench: {DEMO_EDITOR_WORKBENCH_COMMAND}",
        "demo editor proof supporting: {DEMO_EDITOR_PROOF_COMMAND}",
        "debug trace: {DEBUG_TRACE_COMMAND}",
        "regression_summary_drilldown(&summary)",
        "regression_bundle_followup_commands",
        "regression_bundle_followup_command_lines(drilldown.bundle_dirs.iter().map(String::as_str))",
        "bundle_dirs: Vec<String>",
        "perf_evidence_lines: Vec<String>",
        "followup_command_lines: Vec<String>",
        "runnable_followup_command_lines: Vec<String>",
        "manual_followup_command_lines: Vec<String>",
        "bundle dirs:",
        "perf evidence:",
        "follow-up commands:",
        "runnable follow-up commands:",
        "manual compare follow-up commands:",
        "build_regression_dashboard_result_limits_top_rows_and_builds_human_summary",
        "mcp_first_open_resource_text_surfaces_imui_product_chain",
    ):
        _assert_contains(source, marker, name)


def _validate_discovery(repo_root: Path, fretboard_exe: Path) -> None:
    root_help = _run_capture_checked(
        "fretboard help",
        [str(fretboard_exe), "--help"],
        cwd=repo_root,
    )
    _assert_contains(root_help.stdout, "fretboard-dev list tool-apps", "fretboard help")
    _assert_contains(root_help.stdout, "fretboard-dev list tool-apps --json", "fretboard help")
    _assert_contains(root_help.stdout, IMUI_PRODUCT_CHAIN_COMMAND, "fretboard help")
    _assert_contains(root_help.stdout, IMUI_PRODUCT_CHAIN_DISCOVERY_COMMAND, "fretboard help")
    _assert_contains(root_help.stdout, IMUI_DOCKING_PERF_COMMAND, "fretboard help")
    _assert_contains(root_help.stdout, "cargo run -p fret-devtools", "fretboard help")
    _assert_contains(root_help.stdout, "cargo run -p fret-devtools-mcp", "fretboard help")

    list_help = _run_capture_checked(
        "list help",
        [str(fretboard_exe), "list", "--help"],
        cwd=repo_root,
    )
    _assert_contains(list_help.stdout, "tool-apps", "list help")
    _assert_contains(list_help.stdout, "List repo-maintainer tool apps", "list help")

    cookbook = _run_capture_checked(
        "list cookbook examples",
        [str(fretboard_exe), "list", "cookbook-examples", "--all"],
        cwd=repo_root,
    )
    _assert_contains(cookbook.stdout, "imui_action_basics", "list cookbook examples")
    _assert_contains(cookbook.stdout, "imui_editor_controls_basics", "list cookbook examples")

    native = _run_capture_checked(
        "list native demos",
        [str(fretboard_exe), "list", "native-demos", "--all"],
        cwd=repo_root,
    )
    _assert_contains(native.stdout, "imui_editor_workbench_demo", "list native demos")
    _assert_contains(native.stdout, "imui_editor_proof_demo", "list native demos")
    _assert_contains(native.stdout, "editor_notes_demo", "list native demos")
    _assert_contains(native.stdout, "editor_notes_device_shell_demo", "list native demos")
    _assert_contains(native.stdout, "workspace_shell_demo", "list native demos")
    _assert_contains(native.stdout, "docking_arbitration_demo", "list native demos")

    tool_apps = _run_capture_checked(
        "list tool apps",
        [str(fretboard_exe), "list", "tool-apps"],
        cwd=repo_root,
    )
    _assert_contains(tool_apps.stdout, f"first-open: {FIRST_OPEN_DOC}", "list tool apps")
    _assert_contains(tool_apps.stdout, f"repo preflight: {REPO_PREFLIGHT_COMMAND}", "list tool apps")
    _assert_contains(
        tool_apps.stdout,
        f"repo preflight json: {REPO_PREFLIGHT_JSON_COMMAND}",
        "list tool apps",
    )
    _assert_contains(tool_apps.stdout, f"gui branch: {DEVTOOLS_GUI_DOC}", "list tool apps")
    _assert_contains(tool_apps.stdout, "workflow: imui-product-chain", "list tool apps")
    _assert_contains(tool_apps.stdout, IMUI_PRODUCT_CHAIN_COMMAND, "list tool apps")
    _assert_contains(tool_apps.stdout, IMUI_PRODUCT_CHAIN_DISCOVERY_COMMAND, "list tool apps")
    _assert_contains(tool_apps.stdout, IMUI_DOCKING_PERF_COMMAND, "list tool apps")
    _assert_contains(tool_apps.stdout, IMUI_DOCKING_PERF_SUITE, "list tool apps")
    _assert_contains(tool_apps.stdout, "perf-docking/regression.summary.json", "list tool apps")
    _assert_contains(tool_apps.stdout, "perf-docking/check.perf_thresholds.json", "list tool apps")
    _assert_contains(tool_apps.stdout, "perf-docking/*/trace.chrome.json", "list tool apps")
    _assert_contains(tool_apps.stdout, "route: demo-metrics-debug", "list tool apps")
    _assert_contains(
        tool_apps.stdout,
        f"owner: {DEMO_METRICS_DEBUG_OWNER_DOC}",
        "list tool apps",
    )
    _assert_contains(
        tool_apps.stdout,
        f"action metadata owner: {DEMO_METRICS_DEBUG_ACTION_METADATA_DOC}",
        "list tool apps",
    )
    _assert_contains(
        tool_apps.stdout,
        f"docking owner: {DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC}",
        "list tool apps",
    )
    _assert_contains(
        tool_apps.stdout,
        f"wayland acceptance: {DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC}",
        "list tool apps",
    )
    for command_groups in DEMO_METRICS_DEBUG_ROUTE_COMMANDS.values():
        for label, command in command_groups.items():
            _assert_contains(tool_apps.stdout, f"{label}: {command}", "list tool apps")
    _assert_contains(tool_apps.stdout, "fret-devtools", "list tool apps")
    _assert_contains(tool_apps.stdout, "cargo run -p fret-devtools", "list tool apps")
    _assert_contains(tool_apps.stdout, DEVTOOLS_GUI_DOC, "list tool apps")
    _assert_contains(tool_apps.stdout, "cargo build -p fret-devtools", "list tool apps")
    _assert_contains(tool_apps.stdout, "fret-devtools-mcp", "list tool apps")
    _assert_contains(tool_apps.stdout, "cargo run -p fret-devtools-mcp", "list tool apps")
    _assert_contains(tool_apps.stdout, DEVTOOLS_MCP_DOC, "list tool apps")
    _assert_contains(tool_apps.stdout, "cargo build -p fret-devtools-mcp", "list tool apps")

    tool_apps_json = _run_capture_checked(
        "list tool apps json",
        [str(fretboard_exe), "list", "tool-apps", "--json"],
        cwd=repo_root,
    )
    _validate_tool_apps_json(_parse_json_stdout("list tool apps json", tool_apps_json))
    _validate_devtools_gui_product_workflow_source(repo_root)
    _validate_devtools_gui_followup_source(repo_root)
    _validate_devtools_mcp_product_workflow_source(repo_root)

    doctor = _run_capture_checked(
        "diag doctor campaigns",
        [str(fretboard_exe), "diag", "doctor", "campaigns", "--json"],
        cwd=repo_root,
    )
    payload = _parse_json_stdout("diag doctor campaigns", doctor)
    if payload.get("ok") is not True:
        raise SystemExit("Step failed: diag doctor campaigns (expected ok=true)")


def _validate_script(repo_root: Path, fretboard_exe: Path, script_path: str) -> None:
    proc = _run_capture_checked(
        f"diag script validate {script_path}",
        [str(fretboard_exe), "diag", "script", "validate", script_path, "--json"],
        cwd=repo_root,
    )
    payload = _parse_json_stdout(f"diag script validate {script_path}", proc)
    if payload.get("status") != "passed" or payload.get("error_scripts") != 0:
        raise SystemExit(f"script validation failed: {script_path}")


def _validate_campaign(repo_root: Path, fretboard_exe: Path, campaign_path: str) -> None:
    proc = _run_capture_checked(
        f"diag campaign validate {campaign_path}",
        [str(fretboard_exe), "diag", "campaign", "validate", campaign_path, "--json"],
        cwd=repo_root,
    )
    payload = _parse_json_stdout(f"diag campaign validate {campaign_path}", proc)
    if payload.get("kind") != "diag_campaign_validate_result" or payload.get("count") != 1:
        raise SystemExit(f"campaign validation failed: {campaign_path}")
    campaigns = payload.get("campaigns")
    if not isinstance(campaigns, list) or len(campaigns) != 1:
        raise SystemExit(f"campaign validation returned unexpected campaign list: {campaign_path}")
    campaign = campaigns[0]
    expected_id = Path(campaign_path).stem
    if campaign.get("id") != expected_id:
        raise SystemExit(f"campaign validation returned unexpected campaign id: {campaign_path}")
    scripts = campaign.get("scripts")
    if not isinstance(scripts, list) or len(scripts) < 4:
        raise SystemExit(f"campaign validation returned too few scripts: {campaign_path}")


def _validate_product_surface(repo_root: Path, fretboard_exe: Path, surface: ProductSurface) -> None:
    scripts = list(surface.scripts)
    if surface.suite is not None:
        scripts.extend(_suite_scripts(repo_root, surface.suite))
    for script in scripts:
        _validate_script(repo_root, fretboard_exe, script)
    if surface.campaign is not None:
        _validate_campaign(repo_root, fretboard_exe, surface.campaign)


def _validate_docking_perf_thresholds(
    threshold_path: Path,
    expected_scripts: set[str],
) -> None:
    thresholds = _read_json_file(threshold_path)
    if thresholds.get("kind") != "perf_thresholds" or thresholds.get("schema_version") != 1:
        raise SystemExit(f"docking perf thresholds JSON has unexpected shape: {threshold_path}")
    if thresholds.get("observed_aggregate") != "max":
        raise SystemExit(f"docking perf thresholds JSON has unexpected aggregate: {threshold_path}")
    if thresholds.get("failures") != []:
        raise SystemExit(f"docking perf thresholds JSON recorded failures: {threshold_path}")

    expected_thresholds = _docking_perf_threshold_values()
    threshold_values = thresholds.get("thresholds")
    if not isinstance(threshold_values, dict):
        raise SystemExit(f"docking perf thresholds JSON is missing thresholds: {threshold_path}")
    for key, expected in expected_thresholds.items():
        if threshold_values.get(key) != expected:
            raise SystemExit(
                f"docking perf thresholds JSON has unexpected {key}: {threshold_path}"
            )

    rows = thresholds.get("rows")
    if not isinstance(rows, list) or len(rows) != len(expected_scripts):
        raise SystemExit(f"docking perf thresholds JSON did not record both rows: {threshold_path}")
    seen_scripts: set[str] = set()
    for row in rows:
        if not isinstance(row, dict):
            raise SystemExit(f"docking perf thresholds JSON contains a non-object row: {threshold_path}")
        script = row.get("script")
        if not isinstance(script, str):
            raise SystemExit(f"docking perf thresholds row is missing script: {threshold_path}")
        seen_scripts.add(script)
        row_thresholds = row.get("thresholds")
        row_sources = row.get("threshold_sources")
        if not isinstance(row_thresholds, dict) or not isinstance(row_sources, dict):
            raise SystemExit(f"docking perf thresholds row is missing threshold metadata: {script}")
        for key, expected in expected_thresholds.items():
            if row_thresholds.get(key) != expected:
                raise SystemExit(f"docking perf thresholds row has unexpected {key}: {script}")
            if row_sources.get(key) != "cli":
                raise SystemExit(f"docking perf thresholds row did not source {key} from CLI: {script}")
    if seen_scripts != expected_scripts:
        raise SystemExit("docking perf thresholds rows do not match the promoted suite")


def _validate_docking_perf_trace(bundle_path: Path, script: str) -> Path:
    trace_path = bundle_path.parent / "trace.chrome.json"
    if not trace_path.is_file():
        raise SystemExit(f"docking perf item has no readable Chrome trace artifact: {script}")
    trace = _read_json_file(trace_path)
    if trace.get("kind") != PERF_TRACE_CHROME_KIND:
        raise SystemExit(f"docking perf trace has unexpected kind: {trace_path}")
    if trace.get("trace_source") != PERF_TRACE_SOURCE_WITH_REAL_SPANS:
        raise SystemExit(f"docking perf trace did not include real span source: {trace_path}")
    if trace.get("real_spans_included") is not True:
        raise SystemExit(f"docking perf trace did not include real spans: {trace_path}")
    real_span_event_count = trace.get("real_span_event_count")
    if not isinstance(real_span_event_count, int) or real_span_event_count <= 0:
        raise SystemExit(f"docking perf trace has no real span events: {trace_path}")
    real_span_extension_keys = trace.get("real_span_extension_keys")
    if (
        not isinstance(real_span_extension_keys, list)
        or PERF_TRACE_REAL_SPAN_EXTENSION_KEY not in real_span_extension_keys
    ):
        raise SystemExit(f"docking perf trace is missing real span extension key: {trace_path}")
    trace_events = trace.get("traceEvents")
    if not isinstance(trace_events, list) or not trace_events:
        raise SystemExit(f"docking perf trace has no Chrome trace events: {trace_path}")
    return trace_path


def _validate_docking_perf_summary(repo_root: Path, out_dir: Path) -> None:
    summary_path = out_dir / "regression.summary.json"
    summary = _read_json_file(summary_path)
    if summary.get("kind") != "diag_regression_summary":
        raise SystemExit("docking perf regression summary has unexpected kind")
    run = summary.get("run")
    if not isinstance(run, dict) or run.get("tool") != "fretboard-dev diag perf":
        raise SystemExit("docking perf regression summary has unexpected run metadata")
    totals = summary.get("totals")
    if not isinstance(totals, dict):
        raise SystemExit("docking perf regression summary is missing totals")
    if totals.get("items_total") != 2 or totals.get("passed") != 2 or totals.get("failed_tooling") != 0:
        raise SystemExit("docking perf regression summary did not pass both perf cases")
    filters = summary.get("campaign", {}).get("filters")
    if not isinstance(filters, dict) or filters.get("wants_perf_thresholds") is not True:
        raise SystemExit("docking perf regression summary did not record threshold gating")

    expected_scripts = set(
        _suite_scripts(
            repo_root,
            "tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json",
        )
    )
    seen_scripts: set[str] = set()
    seen_bundles: set[Path] = set()
    seen_perf_summaries: set[Path] = set()
    seen_traces: set[Path] = set()
    seen_thresholds: set[Path] = set()
    items = summary.get("items")
    if not isinstance(items, list) or len(items) != len(expected_scripts):
        raise SystemExit("docking perf regression summary did not record both perf scripts")
    for item in items:
        if item.get("kind") != "perf_case" or item.get("status") != "passed":
            raise SystemExit("docking perf regression summary contains a non-passing item")
        script = item.get("source", {}).get("script")
        if not isinstance(script, str):
            raise SystemExit("docking perf regression summary item is missing source.script")
        seen_scripts.add(script)
        evidence = item.get("evidence")
        if not isinstance(evidence, dict):
            raise SystemExit(f"docking perf item is missing evidence: {script}")
        extra = evidence.get("extra")
        if not isinstance(extra, dict):
            raise SystemExit(f"docking perf item is missing evidence.extra: {script}")
        metrics = extra.get("metrics")
        if not isinstance(metrics, dict):
            raise SystemExit(f"docking perf item is missing summary metrics: {script}")
        if extra.get("threshold_failures") != []:
            raise SystemExit(f"docking perf item recorded threshold failures: {script}")
        for metric_key in (
            "top_total_time_us",
            "top_layout_time_us",
            "top_layout_engine_solve_time_us",
            "pointer_move_frames_present",
            "pointer_move_max_dispatch_time_us",
            "pointer_move_max_hit_test_time_us",
            "top_renderer_encode_scene_us",
            "top_renderer_instance_bytes",
        ):
            if metric_key not in metrics:
                raise SystemExit(
                    f"docking perf item summary metrics are missing {metric_key}: {script}"
                )
        bundle_path = _path_from_json(evidence.get("bundle_artifact"))
        if bundle_path is None or not bundle_path.is_file():
            raise SystemExit(f"docking perf item has no readable bundle artifact: {script}")
        seen_bundles.add(bundle_path)
        seen_traces.add(_validate_docking_perf_trace(bundle_path, script))
        perf_summary_path = _path_from_json(evidence.get("perf_summary_json"))
        if perf_summary_path is None or not perf_summary_path.is_file():
            raise SystemExit(f"docking perf item has no readable perf summary artifact: {script}")
        seen_perf_summaries.add(perf_summary_path)
        threshold_path = _path_from_json(evidence.get("compare_json"))
        if threshold_path is None or not threshold_path.is_file():
            raise SystemExit(f"docking perf item has no readable threshold artifact: {script}")
        seen_thresholds.add(threshold_path)

    if seen_scripts != expected_scripts:
        raise SystemExit("docking perf regression summary scripts do not match the promoted suite")
    if len(seen_thresholds) != 1:
        raise SystemExit("docking perf regression summary should point at one shared threshold artifact")
    if len(seen_traces) != len(expected_scripts):
        raise SystemExit("docking perf regression summary should point at one trace per perf script")
    for threshold_path in seen_thresholds:
        _validate_docking_perf_thresholds(threshold_path, expected_scripts)
    for perf_summary_path in seen_perf_summaries:
        perf_summary = _read_json_file(perf_summary_path)
        if perf_summary.get("kind") != "layout_perf_summary":
            raise SystemExit(f"docking perf layout summary has unexpected kind: {perf_summary_path}")
        summary_bundle_path = _path_from_json(perf_summary.get("bundle_artifact"))
        if summary_bundle_path not in seen_bundles:
            raise SystemExit(
                f"docking perf layout summary does not point at a recorded item bundle: {perf_summary_path}"
            )
        stats = perf_summary.get("stats")
        if not isinstance(stats, dict) or not isinstance(stats.get("total_time_us"), int):
            raise SystemExit(f"docking perf layout summary has invalid stats: {perf_summary_path}")


def _run_source_gates(repo_root: Path) -> None:
    _run_checked(
        "imui facade teaching source gate",
        [sys.executable, "tools/gate_imui_facade_teaching_source.py"],
        cwd=repo_root,
    )
    _run_checked(
        "imui workstream source gate",
        [sys.executable, "tools/gate_imui_workstream_source.py"],
        cwd=repo_root,
    )


def _cargo_run_demo_command(
    package: str,
    *,
    example: str | None = None,
    bin_name: str | None = None,
    features: str | None = None,
    release: bool,
) -> list[str]:
    cmd = ["cargo", "run", "-p", package]
    if release:
        cmd.append("--release")
    if features is not None:
        cmd.extend(["--features", features])
    if example is not None:
        cmd.extend(["--example", example])
    if bin_name is not None:
        cmd.extend(["--bin", bin_name])
    return cmd


def _built_bin_command(repo_root: Path, stem: str, *, release: bool) -> list[str]:
    profile_dir = "release" if release else "debug"
    exe = repo_root / "target" / profile_dir / _exe_name(stem)
    if not exe.exists():
        raise SystemExit(
            f"built binary not found: {exe}\n"
            f"hint: build it first or rerun without --reuse-built"
        )
    return [str(exe)]


def _fret_demo_launch_command(
    repo_root: Path,
    bin_name: str,
    *,
    release: bool,
    reuse_built: bool,
) -> list[str]:
    if reuse_built:
        return _built_bin_command(repo_root, bin_name, release=release)
    return _cargo_run_demo_command(
        "fret-demo",
        bin_name=bin_name,
        release=release,
    )


def _run_launched_gates(
    repo_root: Path,
    *,
    out_root: Path,
    timeout_ms: int,
    poll_ms: int,
    release: bool,
    reuse_built: bool,
    selected: set[str],
) -> None:
    out_root.mkdir(parents=True, exist_ok=True)

    if GENERIC_ACTION in selected:
        cmd = [
            sys.executable,
            "tools/diag_gate_action_first_authoring_v1.py",
            "--only",
            "cookbook-imui-action-basics-cross-frontend",
            "--out-dir",
            str(out_root / "generic-action"),
            "--timeout-ms",
            str(timeout_ms),
            "--poll-ms",
            str(poll_ms),
        ]
        if release:
            cmd.append("--release")
        _run_checked("launched generic IMUI action gate", cmd, cwd=repo_root)

    if EDITOR_CONTROLS in selected:
        cmd = [
            "cargo",
            "run",
            "-p",
            "fretboard-dev",
            "--",
            "diag",
            "suite",
            "cookbook-imui-editor-controls-basics",
            "--dir",
            str(out_root / "editor-controls"),
            "--timeout-ms",
            str(timeout_ms),
            "--poll-ms",
            str(poll_ms),
            "--launch",
            "--",
            *_cargo_run_demo_command(
                "fret-cookbook",
                example="imui_editor_controls_basics",
                features="cookbook-imui,cookbook-diag",
                release=release,
            ),
        ]
        _run_checked("launched editor controls suite", cmd, cwd=repo_root)

    if EDITOR_PROOF in selected:
        cmd = [
            "cargo",
            "run",
            "-p",
            "fretboard-dev",
            "--",
            "diag",
            "suite",
            "imui-editor-proof-edit-outcomes",
            "--dir",
            str(out_root / "editor-proof"),
            "--timeout-ms",
            str(timeout_ms),
            "--poll-ms",
            str(poll_ms),
            "--launch",
            "--",
            *_fret_demo_launch_command(
                repo_root,
                "imui_editor_proof_demo",
                release=release,
                reuse_built=reuse_built,
            ),
        ]
        _run_checked("launched editor proof suite", cmd, cwd=repo_root)

    if EDITOR_NOTES in selected:
        cmd = [
            "cargo",
            "run",
            "-p",
            "fretboard-dev",
            "--",
            "diag",
            "suite",
            "editor-notes-demo",
            "--dir",
            str(out_root / "editor-notes"),
            "--timeout-ms",
            str(timeout_ms),
            "--poll-ms",
            str(poll_ms),
            "--launch",
            "--",
            *_fret_demo_launch_command(
                repo_root,
                "editor_notes_demo",
                release=release,
                reuse_built=reuse_built,
            ),
        ]
        _run_checked("launched editor notes suite", cmd, cwd=repo_root)

    if EDITOR_NOTES_DEVICE_SHELL in selected:
        cmd = [
            "cargo",
            "run",
            "-p",
            "fretboard-dev",
            "--",
            "diag",
            "suite",
            "editor-notes-device-shell-demo",
            "--dir",
            str(out_root / "editor-notes-device-shell"),
            "--timeout-ms",
            str(timeout_ms),
            "--poll-ms",
            str(poll_ms),
            "--launch",
            "--",
            *_fret_demo_launch_command(
                repo_root,
                "editor_notes_device_shell_demo",
                release=release,
                reuse_built=reuse_built,
            ),
        ]
        _run_checked("launched editor notes device shell suite", cmd, cwd=repo_root)

    if WORKSPACE_SHELL in selected:
        cmd = [
            "cargo",
            "run",
            "-p",
            "fretboard-dev",
            "--",
            "diag",
            "suite",
            "diag-hardening-smoke-workspace",
            "--dir",
            str(out_root / "workspace-shell"),
            "--timeout-ms",
            str(timeout_ms),
            "--poll-ms",
            str(poll_ms),
            "--launch",
            "--",
            *_fret_demo_launch_command(
                repo_root,
                "workspace_shell_demo",
                release=release,
                reuse_built=reuse_built,
            ),
        ]
        _run_checked("launched workspace shell suite", cmd, cwd=repo_root)

    if PERF_DOCKING in selected:
        out_dir = out_root / "perf-docking"
        cmd = [
            "cargo",
            "run",
            "-p",
            "fretboard-dev",
            "--",
            "diag",
            "perf",
            "perf-docking-arbitration-steady",
            "--dir",
            str(out_dir),
            "--repeat",
            "1",
            "--warmup-frames",
            "5",
            "--trace-real-spans",
            *_docking_perf_threshold_args(),
            "--reuse-launch",
            "--env",
            "FRET_DOCK_ARB_PRESET=large",
            "--env",
            "FRET_DOCK_ARB_NO_PERSIST=1",
            "--env",
            "FRET_DOCK_ARB_DISALLOW_DROP_TARGETS=1",
            "--launch",
            "--",
            *_fret_demo_launch_command(
                repo_root,
                "docking_arbitration_demo",
                release=release,
                reuse_built=reuse_built,
            ),
        ]
        _run_checked("launched docking perf suite", cmd, cwd=repo_root)
        _validate_docking_perf_summary(repo_root, out_dir)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", default="target/imui-product-chain")
    parser.add_argument("--timeout-ms", type=int, default=240000)
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument("--release", action="store_true")
    parser.add_argument(
        "--reuse-built",
        action="store_true",
        help="For launched fret-demo surfaces, run existing target/{debug,release} binaries instead of cargo run.",
    )
    parser.add_argument(
        "--launched",
        action="store_true",
        help="Also run launched diagnostics gates for selected product-chain surfaces.",
    )
    parser.add_argument(
        "--only",
        action="append",
        default=[],
        help="Run only named gates. Can be repeated or comma-separated.",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    selected = _selected_gate_names(args.only)
    fretboard_exe = _build_fretboard_dev(repo_root, args.release)

    if DISCOVERY in selected:
        _validate_discovery(repo_root, fretboard_exe)

    selected_surfaces = [surface for surface in PRODUCT_SURFACES if surface.name in selected]
    for surface in selected_surfaces:
        _validate_product_surface(repo_root, fretboard_exe, surface)

    if SOURCE_GATES in selected:
        _run_source_gates(repo_root)

    if args.launched:
        run_id = str(int(time.time() * 1000))
        _run_launched_gates(
            repo_root,
            out_root=(repo_root / args.out_dir / run_id).resolve(),
            timeout_ms=args.timeout_ms,
            poll_ms=args.poll_ms,
            release=args.release,
            reuse_built=args.reuse_built,
            selected=selected,
        )

    print("[diag-gate-imui-product-chain] done", flush=True)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
