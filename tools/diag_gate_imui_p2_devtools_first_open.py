#!/usr/bin/env python3
"""Run the bounded IMUI P2 first-open diagnostics/devtools smoke package."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path


CAMPAIGN_ID = "devtools-first-open-smoke"
SCRIPT_PATH = "tools/diag-scripts/tooling/todo/todo-baseline.json"
LABEL_AFTER_ADD = "todo-after-add"
LABEL_AFTER_TOGGLE = "todo-after-toggle-done"
LABEL_AFTER_REMOVE = "todo-after-remove"
FIRST_OPEN_DOC = "docs/diagnostics-first-open.md"
DEVTOOLS_GUI_DOC = "docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md"
DEVTOOLS_WORKSTREAM_DOC = "docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md"
DEVTOOLS_MCP_DOC = "docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md"
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
DEVTOOLS_GUI_SOURCE = "apps/fret-devtools/src/native.rs"
DEVTOOLS_GUI_WS_SOURCE = "apps/fret-devtools/src/ws.rs"
DEVTOOLS_GUI_SEMANTICS_SOURCE = "apps/fret-devtools/src/semantics.rs"
DEVTOOLS_GUI_GATE_RUN_SOURCE = "apps/fret-devtools/src/gate_run.rs"
DEVTOOLS_GUI_WORKFLOW_RUN_SOURCE = "apps/fret-devtools/src/workflow_run.rs"
DEVTOOLS_GUI_FOLLOWUP_SOURCE = "apps/fret-devtools/src/followup.rs"
DEVTOOLS_MCP_SOURCE = "apps/fret-devtools-mcp/src/native.rs"
DEVTOOLS_GATE_PROFILE_SOURCE = "crates/fret-diag/src/devtools_gate_profiles.rs"
DEVTOOLS_PROTOCOL_SOURCE = "crates/fret-diag-protocol/src/lib.rs"
BOOTSTRAP_DEVTOOLS_WS_SOURCE = (
    "ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs"
)
DEVTOOLS_REPRO_CONTRACT_SOURCE = "crates/fret-diag/src/cli/contracts/commands/repro.rs"
DEVTOOLS_CUTOVER_SOURCE = "crates/fret-diag/src/cli/cutover.rs"
BUNDLE_VIEWER_README = "tools/fret-bundle-viewer/README.md"
BUNDLE_VIEWER_PARSER_SOURCE = "tools/fret-bundle-viewer/lib/parser.ts"
BUNDLE_VIEWER_ZIP_SOURCE = "tools/fret-bundle-viewer/lib/zip.ts"
FRET_UI_README = "crates/fret-ui/README.md"
FRET_UI_SOURCE_DIR = "crates/fret-ui/src"
MAINTAINER_CHECKLIST_DOC = "docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_CHECKLIST.md"
REPO_PREFLIGHT_COMMAND = "cargo run -p fretboard-dev -- diag doctor campaigns"
REPO_PREFLIGHT_JSON_COMMAND = "cargo run -p fretboard-dev -- diag doctor campaigns --json"


class ProgressRecorder:
    def __init__(self, path: Path | None):
        self.path = path
        if self.path is not None:
            self.path.parent.mkdir(parents=True, exist_ok=True)

    def record(self, event: str, **fields: object) -> None:
        if self.path is None:
            return
        payload = {
            "ts_unix_ms": int(time.time() * 1000),
            "event": event,
            **fields,
        }
        with self.path.open("a", encoding="utf-8", newline="\n") as file:
            file.write(json.dumps(payload, sort_keys=True) + "\n")


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _exe_name(stem: str) -> str:
    return f"{stem}.exe" if os.name == "nt" else stem


def _run_checked(
    name: str,
    argv: list[str],
    *,
    cwd: Path,
    progress: ProgressRecorder | None = None,
) -> None:
    print(f"[diag-gate-imui-p2-devtools] {name}")
    if progress is not None:
        progress.record("step.start", name=name, argv=argv)
    proc = subprocess.run(argv, cwd=str(cwd), check=False)
    if proc.returncode != 0:
        if progress is not None:
            progress.record("step.fail", name=name, exit_code=proc.returncode)
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")
    if progress is not None:
        progress.record("step.pass", name=name, exit_code=proc.returncode)


def _run_capture_checked(
    name: str,
    argv: list[str],
    *,
    cwd: Path,
    progress: ProgressRecorder | None = None,
) -> subprocess.CompletedProcess[str]:
    print(f"[diag-gate-imui-p2-devtools] {name}")
    if progress is not None:
        progress.record("step.start", name=name, argv=argv, captured=True)
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
        if progress is not None:
            progress.record(
                "step.fail",
                name=name,
                exit_code=proc.returncode,
                stdout_len=len(proc.stdout),
                stderr_len=len(proc.stderr),
            )
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")
    if progress is not None:
        progress.record(
            "step.pass",
            name=name,
            exit_code=proc.returncode,
            stdout_len=len(proc.stdout),
            stderr_len=len(proc.stderr),
        )
    return proc


def _run_compare_expect_diff(
    name: str,
    argv: list[str],
    *,
    cwd: Path,
    progress: ProgressRecorder | None = None,
) -> dict:
    print(f"[diag-gate-imui-p2-devtools] {name}")
    if progress is not None:
        progress.record("step.start", name=name, argv=argv, captured=True)
    proc = subprocess.run(
        argv,
        cwd=str(cwd),
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode not in (0, 1):
        sys.stdout.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        if progress is not None:
            progress.record(
                "step.fail",
                name=name,
                exit_code=proc.returncode,
                stdout_len=len(proc.stdout),
                stderr_len=len(proc.stderr),
            )
        raise SystemExit(f"Step failed: {name} (unexpected exit code: {proc.returncode})")
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError as err:
        raise SystemExit(f"Step failed: {name} (invalid JSON: {err})") from err
    if payload.get("ok") is not False:
        raise SystemExit(f"Step failed: {name} (expected a non-empty diff report)")
    diffs = payload.get("diffs")
    if not isinstance(diffs, list) or not diffs:
        raise SystemExit(f"Step failed: {name} (expected at least one diff entry)")
    if progress is not None:
        progress.record(
            "step.pass",
            name=name,
            exit_code=proc.returncode,
            stdout_len=len(proc.stdout),
            stderr_len=len(proc.stderr),
            diff_count=len(diffs),
        )
    return payload


def _json_stdout(name: str, proc: subprocess.CompletedProcess[str]) -> dict:
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as err:
        raise SystemExit(f"Step failed: {name} (invalid JSON: {err})") from err


def _read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as err:
        raise SystemExit(f"failed to read JSON file: {path} ({err})") from err
    except json.JSONDecodeError as err:
        raise SystemExit(f"failed to parse JSON file: {path} ({err})") from err


def _single_child_dir(path: Path) -> Path:
    children = [child for child in path.iterdir() if child.is_dir()]
    if len(children) != 1:
        raise SystemExit(
            f"expected exactly one child directory under {path}, found {len(children)}"
        )
    return children[0]


def _find_bundle_dir(session_root: Path, label: str) -> Path:
    matches = [path for path in session_root.iterdir() if path.is_dir() and path.name.endswith(label)]
    if len(matches) != 1:
        raise SystemExit(
            f"expected exactly one bundle dir for label={label} under {session_root}, found {len(matches)}"
        )
    return matches[0]


def _assert_text_contains(name: str, text: str, marker: str) -> None:
    if marker not in text:
        raise SystemExit(f"Step failed: {name} (missing marker: {marker})")


def _read_text_for_gate(name: str, path: Path, progress: ProgressRecorder | None = None) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as err:
        if progress is not None:
            progress.record("step.fail", name=name, path=str(path), error=str(err))
        raise SystemExit(f"Step failed: {name} (failed to read {path}: {err})") from err


def _validate_tool_app_discovery(
    fretboard_exe: Path,
    *,
    cwd: Path,
    progress: ProgressRecorder | None = None,
) -> None:
    human = _run_capture_checked(
        "list tool-apps first-open human index",
        [str(fretboard_exe), "list", "tool-apps"],
        cwd=cwd,
        progress=progress,
    )
    human_text = human.stdout + human.stderr
    for marker in (
        f"first-open: {FIRST_OPEN_DOC}",
        f"repo preflight: {REPO_PREFLIGHT_COMMAND}",
        f"repo preflight json: {REPO_PREFLIGHT_JSON_COMMAND}",
        f"gui branch: {DEVTOOLS_GUI_DOC}",
        "route: demo-metrics-debug",
        f"owner: {DEMO_METRICS_DEBUG_OWNER_DOC}",
        f"action metadata owner: {DEMO_METRICS_DEBUG_ACTION_METADATA_DOC}",
        f"docking owner: {DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC}",
        f"wayland acceptance: {DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC}",
        "demo editor workbench: cargo run -p fret-demo --bin imui_editor_workbench_demo",
        "demo editor proof supporting: cargo run -p fret-demo --bin imui_editor_proof_demo",
        "demo editor notes: cargo run -p fret-demo --bin editor_notes_demo",
        "demo device shell: cargo run -p fret-demo --bin editor_notes_device_shell_demo",
        "metrics stats: cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json",
        "metrics layout perf: cargo run -p fretboard-dev -- diag layout-perf-summary <bundle-or-dir> --json",
        "metrics memory: cargo run -p fretboard-dev -- diag memory-summary <bundle-or-dir> --json",
        "debug triage: cargo run -p fretboard-dev -- diag triage <bundle-or-dir> --json",
        "debug hotspots: cargo run -p fretboard-dev -- diag hotspots <bundle-or-dir> --json",
        "debug trace: cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json",
        "docking arbitration supporting: cargo run -p fret-demo --bin docking_arbitration_demo",
        "docking campaign validate: cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json",
        "docking policy-skip local: python tools/diag_gate_docking_wayland_policy_skip.py",
        "fret-devtools",
        "cargo run -p fret-devtools",
        DEVTOOLS_GUI_DOC,
        "fret-devtools-mcp",
        "cargo run -p fret-devtools-mcp",
        DEVTOOLS_MCP_DOC,
    ):
        _assert_text_contains("list tool-apps first-open human index", human_text, marker)

    json_proc = _run_capture_checked(
        "list tool-apps first-open json index",
        [str(fretboard_exe), "list", "tool-apps", "--json"],
        cwd=cwd,
        progress=progress,
    )
    payload = _json_stdout("list tool-apps first-open json index", json_proc)
    if payload.get("kind") != "fretboard_tool_apps":
        raise SystemExit("list tool-apps --json should emit kind=fretboard_tool_apps")
    if payload.get("first_open_doc") != FIRST_OPEN_DOC:
        raise SystemExit("list tool-apps --json should expose the diagnostics first-open doc")
    if payload.get("branch_doc") != DEVTOOLS_GUI_DOC:
        raise SystemExit("list tool-apps --json should expose the DevTools GUI branch doc")
    repo_preflight = payload.get("repo_preflight")
    if not isinstance(repo_preflight, dict):
        raise SystemExit("list tool-apps --json should expose repo_preflight")
    if repo_preflight.get("command") != REPO_PREFLIGHT_COMMAND:
        raise SystemExit("list tool-apps --json should expose the repo preflight command")
    if repo_preflight.get("json_command") != REPO_PREFLIGHT_JSON_COMMAND:
        raise SystemExit("list tool-apps --json should expose the repo preflight JSON command")
    first_open_routes = payload.get("first_open_routes")
    if not isinstance(first_open_routes, list):
        raise SystemExit("list tool-apps --json should expose a first_open_routes array")
    demo_metrics_route = next(
        (
            item
            for item in first_open_routes
            if isinstance(item, dict) and item.get("id") == "demo-metrics-debug"
        ),
        None,
    )
    if demo_metrics_route is None:
        raise SystemExit("list tool-apps --json should expose demo-metrics-debug")
    if demo_metrics_route.get("docs") != FIRST_OPEN_DOC:
        raise SystemExit("list tool-apps --json should expose demo-metrics-debug docs")
    if demo_metrics_route.get("owner_doc") != DEMO_METRICS_DEBUG_OWNER_DOC:
        raise SystemExit("list tool-apps --json should expose demo-metrics-debug owner_doc")
    if demo_metrics_route.get("action_metadata_doc") != DEMO_METRICS_DEBUG_ACTION_METADATA_DOC:
        raise SystemExit(
            "list tool-apps --json should expose demo-metrics-debug action_metadata_doc"
        )
    if demo_metrics_route.get("docking_owner_doc") != DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC:
        raise SystemExit("list tool-apps --json should expose demo-metrics-debug docking_owner_doc")
    if demo_metrics_route.get("wayland_acceptance_doc") != DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC:
        raise SystemExit(
            "list tool-apps --json should expose demo-metrics-debug wayland_acceptance_doc"
        )
    if not isinstance(demo_metrics_route.get("purpose"), str) or not demo_metrics_route["purpose"]:
        raise SystemExit("list tool-apps --json should expose demo-metrics-debug purpose")
    route_groups = {
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
    for group, expected_commands in route_groups.items():
        commands = demo_metrics_route.get(group)
        if not isinstance(commands, list):
            raise SystemExit(f"list tool-apps --json should expose demo-metrics-debug {group}")
        commands_by_label = {
            item.get("label"): item.get("command")
            for item in commands
            if isinstance(item, dict)
        }
        for label, command in expected_commands.items():
            if commands_by_label.get(label) != command:
                raise SystemExit(
                    f"list tool-apps --json should expose demo-metrics-debug {label}"
                )
    action_metadata = {
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
    action_items = demo_metrics_route.get("action_commands")
    action_items_by_label = {
        item.get("label"): item for item in action_items if isinstance(item, dict)
    }
    for label, expected_fields in action_metadata.items():
        item = action_items_by_label.get(label)
        if not isinstance(item, dict):
            raise SystemExit(
                f"list tool-apps --json should expose demo-metrics-debug action metadata for {label}"
            )
        for field, expected in expected_fields.items():
            if item.get(field) != expected:
                raise SystemExit(
                    f"list tool-apps --json should expose demo-metrics-debug action metadata {label} {field}"
                )
    tool_apps = payload.get("tool_apps")
    if not isinstance(tool_apps, list):
        raise SystemExit("list tool-apps --json should expose a tool_apps array")
    expected_tools = {
        "fret-devtools": ("cargo run -p fret-devtools", DEVTOOLS_GUI_DOC, "cargo build -p fret-devtools"),
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
            raise SystemExit(f"list tool-apps --json should expose {tool_id}")
        if tool.get("command") != command:
            raise SystemExit(f"list tool-apps --json should expose {tool_id} command")
        if tool.get("docs") != docs:
            raise SystemExit(f"list tool-apps --json should expose {tool_id} docs")
        if tool.get("gate") != gate:
            raise SystemExit(f"list tool-apps --json should expose {tool_id} gate")
        if not isinstance(tool.get("best_for"), str) or not tool["best_for"]:
            raise SystemExit(f"list tool-apps --json should expose {tool_id} best_for text")

    doctor = _run_capture_checked(
        "diag doctor campaigns first-open preflight",
        [str(fretboard_exe), "diag", "doctor", "campaigns", "--json"],
        cwd=cwd,
        progress=progress,
    )
    doctor_payload = _json_stdout("diag doctor campaigns first-open preflight", doctor)
    if doctor_payload.get("ok") is not True:
        raise SystemExit("diag doctor campaigns --json should report ok=true")


def _validate_devtools_gui_first_open_source(
    *,
    cwd: Path,
    progress: ProgressRecorder | None = None,
) -> None:
    name = "fret-devtools gui first-open source"
    print(f"[diag-gate-imui-p2-devtools] {name}")
    path = cwd / DEVTOOLS_GUI_SOURCE
    ws_path = cwd / DEVTOOLS_GUI_WS_SOURCE
    semantics_path = cwd / DEVTOOLS_GUI_SEMANTICS_SOURCE
    gate_run_path = cwd / DEVTOOLS_GUI_GATE_RUN_SOURCE
    workflow_run_path = cwd / DEVTOOLS_GUI_WORKFLOW_RUN_SOURCE
    followup_path = cwd / DEVTOOLS_GUI_FOLLOWUP_SOURCE
    gate_profile_path = cwd / DEVTOOLS_GATE_PROFILE_SOURCE
    protocol_path = cwd / DEVTOOLS_PROTOCOL_SOURCE
    bootstrap_ws_path = cwd / BOOTSTRAP_DEVTOOLS_WS_SOURCE
    repro_contract_path = cwd / DEVTOOLS_REPRO_CONTRACT_SOURCE
    cutover_path = cwd / DEVTOOLS_CUTOVER_SOURCE
    if progress is not None:
        progress.record(
            "step.start",
            name=name,
            path=str(path),
            ws_path=str(ws_path),
            semantics_path=str(semantics_path),
            gate_run_path=str(gate_run_path),
            workflow_run_path=str(workflow_run_path),
            followup_path=str(followup_path),
            gate_profile_path=str(gate_profile_path),
            protocol_path=str(protocol_path),
            bootstrap_ws_path=str(bootstrap_ws_path),
            repro_contract_path=str(repro_contract_path),
            cutover_path=str(cutover_path),
        )
    try:
        source = path.read_text(encoding="utf-8")
        ws_source = ws_path.read_text(encoding="utf-8")
        semantics_source = semantics_path.read_text(encoding="utf-8")
        gate_run_source = gate_run_path.read_text(encoding="utf-8")
        workflow_run_source = workflow_run_path.read_text(encoding="utf-8")
        followup_source = followup_path.read_text(encoding="utf-8")
        gate_profile_source = gate_profile_path.read_text(encoding="utf-8")
        protocol_source = protocol_path.read_text(encoding="utf-8")
        bootstrap_ws_source = bootstrap_ws_path.read_text(encoding="utf-8")
        repro_contract_source = repro_contract_path.read_text(encoding="utf-8")
        cutover_source = cutover_path.read_text(encoding="utf-8")
    except OSError as err:
        if progress is not None:
            progress.record("step.fail", name=name, error=str(err))
        raise SystemExit(f"Step failed: {name} (failed to read source: {err})") from err
    source = "\n".join(
        [
            source,
            ws_source,
            semantics_source,
            gate_run_source,
            workflow_run_source,
            followup_source,
            protocol_source,
            bootstrap_ws_source,
            repro_contract_source,
            cutover_source,
        ]
    )

    for marker in (
        f'const DEVTOOLS_FIRST_OPEN_DOC: &str = "{FIRST_OPEN_DOC}"',
        f'const DEVTOOLS_GUI_BRANCH_DOC: &str =\n    "{DEVTOOLS_GUI_DOC}"',
        "const DEVTOOLS_REPO_PREFLIGHT_COMMAND: &str =",
        "const DEVTOOLS_REPO_PREFLIGHT_JSON_COMMAND: &str =",
        "const DEVTOOLS_FIRST_OPEN_GATE_COMMAND: &str =",
        f'const DEVTOOLS_FIRST_OPEN_CAMPAIGN_ID: &str = "{CAMPAIGN_ID}"',
        'const DEVTOOLS_DOGFOOD_WORKFLOW_ID: &str = "ui-gallery-button-dogfood"',
        'const DEVTOOLS_DOGFOOD_TARGET_COMMAND: &str = "cargo run -p fret-ui-gallery --release"',
        'const DEVTOOLS_DOGFOOD_BASE_SCRIPT: &str = "tools/diag-scripts/ui-gallery-lite-smoke.json"',
        'const DEVTOOLS_DOGFOOD_BUTTON_SCRIPT: &str =',
        'const DEVTOOLS_DOGFOOD_PICK_SCRIPT_COMMAND: &str =',
        'const DEVTOOLS_DOGFOOD_PICK_APPLY_COMMAND: &str =',
        'const DEVTOOLS_DOGFOOD_RUN_PACK_COMMAND: &str =',
        'const DEVTOOLS_DOGFOOD_PACK_COMMAND: &str =',
        'const DEVTOOLS_DOGFOOD_VIEWER_COMMAND: &str = "pnpm -C tools/fret-bundle-viewer dev"',
        'const IMUI_PRODUCT_WORKFLOW_ID: &str = "imui-product-chain"',
        'const IMUI_PRODUCT_WORKFLOW_DOC: &str =',
        'const IMUI_PRODUCT_WORKFLOW_COMMAND: &str = "python tools/diag_gate_imui_product_chain.py"',
        'const IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND: &str =',
        'const IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND: &str =',
        'const IMUI_PRODUCT_WORKFLOW_SUITE: &str =',
        "const IMUI_PRODUCT_WORKFLOW_ARTIFACTS: &[&str] = &[",
        'const DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID: &str = "demo-metrics-debug"',
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
        'const DEVTOOLS_DEBUG_HOTSPOTS_COMMAND: &str =',
        'const DEVTOOLS_DEBUG_TRACE_COMMAND: &str =',
        'const CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS: &str =',
        "struct DemoMetricsDebugActionSpec",
        "id: &'static str",
        "category: &'static str",
        "requires_bundle: bool",
        "primary: bool",
        "const DEVTOOLS_DEMO_METRICS_DEBUG_ACTIONS: &[DemoMetricsDebugActionSpec] = &[",
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
        "mod gate_run;",
        "mod workflow_run;",
        'let details_tab = app.models_mut().insert(Some(Arc::<str>::from("guide")));',
        'const CMD_GATE_RUN_GENERATED: &str = "fret.devtools.gate.run_generated"',
        'const CMD_WORKFLOW_RUN_SELECTED: &str = "fret.devtools.workflow.run_selected"',
        "gate_run::poll_gate_run_jobs(cx.app, st)",
        "workflow_run::poll_workflow_run_jobs(cx.app, st)",
        "First-open Next Actions",
        "Stateful next-step summary stays in the header; full command references live in the Guide tab.",
        "First-open Evidence Path",
        "Canonical docs, repo preflight, artifact roots, product-chain evidence, and smoke gate stay visible in the GUI shell.",
        "Dogfood Workflow",
        "UI gallery selector capture, script patching, run/pack, and offline viewer handoff stay visible from the GUI shell.",
        "Demo / Metrics / Debug Routes",
        "Always-available editor demos, action commands, metrics commands, and debug drill-down entrypoints stay visible in the GUI shell.",
        "demo_metrics_debug_rows.push(devtools_demo_metrics_debug_action_row(cx))",
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
        "fn demo_metrics_debug_action_command_text() -> String",
        "fn demo_metrics_debug_action_metadata_lines() -> Vec<String>",
        "fn devtools_demo_metrics_debug_action_row(cx: &mut ElementContext<'_, App>) -> AnyElement",
        "Copy Demo/Metrics/Debug actions",
        "docking campaign validate: {DEVTOOLS_DOCKING_CAMPAIGN_VALIDATE_COMMAND}",
        "docking policy-skip local: {DEVTOOLS_DOCKING_POLICY_SKIP_COMMAND}",
        "Workflow Runs",
        "First-class campaign validation and selected-session suite runs reuse the shared diag command path from the GUI shell.",
        "Gate Commands",
        "First-class stale, pixels, perf-threshold, and resource-footprint gate entrypoints stay visible from the GUI shell.",
        "Live Inspect Hover Bounds",
        "Structured hovered-node bounds projected from inspect.hover.",
        "Live Inspect Overlay Hooks",
        "Viewport overlay hooks and overlay.summary root hints for live inspect overlays.",
        "Raw Inspect Payloads",
        'active_left_tab.as_ref() == "layout"',
        'active_left_tab.as_ref() == "elements"',
        'shadcn::TabsItem::new("layout", "Layout", [layout_tree])',
        'shadcn::TabsItem::new("elements", "Elements", [element_tree])',
        "enum InspectTreeMode",
        "fn layout_tree_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement",
        "fn element_tree_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement",
        "diagnostics_tree_panel(cx, st, InspectTreeMode::Layout)",
        "diagnostics_tree_panel(cx, st, InspectTreeMode::Elements)",
        "Layout tree search",
        "Element tree search",
        "layout-derived",
        "element-derived",
        "InspectTreeMode::Layout => semantics::layout_node_label(node)",
        "InspectTreeMode::Elements => semantics::element_node_label(node)",
        "pub(crate) fn layout_node_label(node: &UiSemanticsNodeV1) -> String",
        "pub(crate) fn element_node_label(node: &UiSemanticsNodeV1) -> String",
        'let parent_text = format!("parent={parent}")',
        "VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16)",
        "options.items_revision = rows_key",
        "let mut stack = Vec::with_capacity(index.roots.len().max(1));",
        "for child in children.iter().rev()",
        "fn selected_session_after_session_list_refresh",
        "fn message_session_matches_selected",
        "selected_session_refresh_keeps_valid_selection_or_falls_back_to_first_session",
        "message_session_matching_uses_selected_session_when_present",
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
        "devtools_first_open_lines(st.cfg.fs_out_dir.as_ref())",
        "devtools_dogfood_workflow_lines(st.cfg.fs_out_dir.as_ref())",
        "devtools_demo_metrics_debug_lines(st.cfg.fs_out_dir.as_ref())",
        "devtools_workflow_run_lines(st.cfg.fs_out_dir.as_ref())",
        "devtools_gate_command_lines(st.cfg.fs_out_dir.as_ref())",
        'shadcn::TabsItem::new("guide", "Guide", [guide])',
        "workflow_run_rows.push(devtools_workflow_run_panel(cx, st))",
        "gate_command_rows.push(devtools_gate_profile_command_builder(cx, st))",
        "devtools_gate_profile_lines(artifacts_root)",
        "generated_gate_command_from_state(cx.app, st)",
        "generated_gate_command_from_state(app, st)",
        "devtools_gate_perf_threshold_command(input)",
        "devtools_gate_profile_action_rows(cx)",
        "Copy generated command",
        "Run generated command",
        "Copy command",
        "missing inputs:",
        "diag args:",
        "workflow_run_selected_id",
        "workflow_run_in_flight",
        "workflow_run_last_result_path",
        "workflow_run_last_result_json",
        "workflow_run_result_history",
        "workflow_run_selected_result_path",
        "workflow_run_last_error",
        "last_workflow_result=",
        "workflow_run::start_workflow_run(app, st, command)",
        "workflow_run_history_list(",
        "workflow_run::workflow_run_result_summary_lines(",
        "workflow_run::workflow_run_result_history_summary_lines(",
        "workflow_run::workflow_run_result_history_selected_or_latest_entry(",
        "workflow_run::workflow_run_result_history_entry_detail_lines(",
        "workflow_run::load_recent_workflow_run_result_history(",
        "workflow_run::workflow_run_regression_summary_artifact_path_from_result_json(",
        "selected_workflow_run_regression_index_path_from_state",
        "workflow_regression_index_parent_dir",
        "refresh_regression_artifacts(app, st)",
        "fret_devtools_workflow_run_result",
        "output_artifacts: Vec<WorkflowRunOutputArtifactV1>",
        "workflow_run_output_artifacts_for_diag_args",
        "workflow_run_output_artifact_lines_from_result_json",
        "workflow_run_output_artifact_path_from_result_json",
        "suite.summary.json",
        "regression.summary.json",
        'join(".fret").join("diag").join("workflow-runs")',
        "new_workflow_run_channel",
        "Workflow Result Details",
        "Workflow Result Summary",
        "Workflow Result History",
        "Copy workflow result",
        "Open workflow JSON",
        "Copy workflow command",
        "Copy workflow JSON",
        "Copy workflow suite summary",
        "Open workflow suite summary",
        "Copy workflow regression summary",
        "Load workflow regression summary",
        "Load workflow regression index",
        "Copy workflow regression index",
        "Open workflow regression index",
        "Open workflow regression summary",
        "Run workflow",
        'const CMD_COPY_WORKFLOW_RESULT_PATH: &str = "fret.devtools.workflow.copy_result_path"',
        'const CMD_COPY_WORKFLOW_RESULT_JSON: &str = "fret.devtools.workflow.copy_result_json"',
        'const CMD_COPY_WORKFLOW_RESULT_COMMAND: &str = "fret.devtools.workflow.copy_result_command"',
        'const CMD_OPEN_WORKFLOW_RESULT_JSON: &str = "fret.devtools.workflow.open_result_json"',
        'const CMD_COPY_WORKFLOW_SUITE_SUMMARY_PATH: &str =',
        'const CMD_OPEN_WORKFLOW_SUITE_SUMMARY: &str =',
        'const CMD_COPY_WORKFLOW_REGRESSION_SUMMARY_PATH: &str =',
        'const CMD_OPEN_WORKFLOW_REGRESSION_SUMMARY: &str =',
        'const CMD_COPY_WORKFLOW_REGRESSION_INDEX_PATH: &str =',
        'const CMD_OPEN_WORKFLOW_REGRESSION_INDEX: &str =',
        'const CMD_LOAD_WORKFLOW_REGRESSION_SUMMARY: &str =',
        'const CMD_LOAD_WORKFLOW_REGRESSION_INDEX: &str =',
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
        "devtools.gate.perf_max_renderer_encode_scene_us",
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
        "fn devtools_first_open_next_action_lines(",
        "selected_session_id: Option<&str>",
        "fn devtools_first_open_lines(artifacts_root: &str) -> Vec<String>",
        "fn devtools_dogfood_workflow_lines(artifacts_root: &str) -> Vec<String>",
        "fn devtools_demo_metrics_debug_lines(artifacts_root: &str) -> Vec<String>",
        "fn devtools_workflow_run_lines(artifacts_root: &str) -> Vec<String>",
        "fn devtools_workflow_commands(",
        "fn devtools_workflow_commands_from_state(",
        "fn workflow_handoff_readiness_lines(",
        "fn devtools_workflow_run_panel(",
        "fn devtools_gate_command_lines(artifacts_root: &str) -> Vec<String>",
        "fn devtools_guide_panel(",
        "fn devtools_gate_profile_command_builder(",
        "fn devtools_gate_profile_action_rows(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement>",
        "selected_summary_loaded: bool",
        "selected_followup_result_loaded: bool",
        "recent_failed_evidence: Option<&RecentEvidenceTarget>",
        "selected_followup_result_loaded_from_state(cx.app, st)",
        "session scope: selected",
        "use the Session selector to retarget inspect, bundle, screenshot, and selected-session suite actions",
        "session scope: choose one available session before sending inspect, bundle, screenshot, or selected-session suite actions",
        "session scope: waiting for the first diagnostics session",
        "regression: selected summary loaded; follow-up actions can use selected bundle evidence",
        "regression: selected follow-up result loaded; inspect Follow-up Result Summary/History",
        "recent evidence: failed",
        "recent evidence: no failed restored GUI-launched evidence",
        "recent evidence command:",
        "recent evidence rerun:",
        "recent evidence next:",
        "first_open_recent_evidence_action_specs(",
        "first_open_recent_evidence_action_row(",
        "direct loop: diag run -> diag latest -> diag compare",
        "campaign loop: diag campaign run {DEVTOOLS_FIRST_OPEN_CAMPAIGN_ID} -> diag summarize -> diag dashboard",
        "dogfood workflow: {DEVTOOLS_DOGFOOD_WORKFLOW_ID}",
        "open ui gallery: {DEVTOOLS_DOGFOOD_TARGET_COMMAND}",
        'preferred selector: {\\"kind\\":\\"test_id\\",\\"id\\":\\"ui-gallery-nav-button\\"}',
        "apply pick to script: {DEVTOOLS_DOGFOOD_PICK_APPLY_COMMAND}",
        "run and pack: {DEVTOOLS_DOGFOOD_RUN_PACK_COMMAND}",
        "open viewer: {DEVTOOLS_DOGFOOD_VIEWER_COMMAND}",
        "route: {DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID}",
        "action metadata owner: {DEVTOOLS_DEMO_METRICS_DEBUG_ACTION_METADATA_DOC}",
        "metrics stats: {DEVTOOLS_METRICS_STATS_COMMAND}",
        "debug triage: {DEVTOOLS_DEBUG_TRIAGE_COMMAND}",
        "debug trace: {DEVTOOLS_DEBUG_TRACE_COMMAND}",
        "Recent Evidence",
        "Latest GUI-launched gate, workflow, and follow-up artifacts restored from the shared diagnostics histories.",
        "devtools_recent_evidence_lines(",
        "recent evidence: gates=",
        "latest gate:",
        "latest workflow:",
        "latest follow-up:",
        "recent failing evidence:",
        "failed_evidence_target:",
        "failed_evidence_path:",
        "failed_evidence_bundle_dir:",
        "failed_evidence_command:",
        "failed_evidence_rerunnable:",
        "failed_evidence_rerun_unavailable_reason:",
        "recent_evidence_next_action:",
        "recent_evidence_next_action(",
        "CMD_COPY_RECENT_EVIDENCE_REPORT",
        "CMD_SELECT_RECENT_FAILED_EVIDENCE",
        "CMD_RERUN_RECENT_FAILED_EVIDENCE",
        "CMD_COPY_RECENT_FAILED_EVIDENCE_PATH",
        "CMD_COPY_RECENT_FAILED_EVIDENCE_BUNDLE_DIR",
        "CMD_COPY_RECENT_FAILED_EVIDENCE_COMMAND",
        "CMD_COPY_RECENT_FAILED_EVIDENCE_JSON",
        "CMD_OPEN_RECENT_FAILED_EVIDENCE_JSON",
        "Copy recent evidence report",
        "Select failed evidence",
        "Rerun failed evidence",
        "Copy failed evidence path",
        "Copy failed bundle dir",
        "Copy failed evidence command",
        "Copy failed evidence JSON",
        "Open failed evidence JSON",
        "recent_failed_evidence_rerun_command(",
        "recent_failed_evidence_rerun_command_from_state(",
        "recent_failed_evidence_rerun_unavailable_reason_from_state(",
        "recent_failed_workflow_rerun_command_from_state(",
        "recent_failed_evidence_bundle_dir(",
        "devtools_recent_failed_evidence_target(",
        "devtools_recent_evidence_selection_effect(",
        "devtools_recent_evidence_lines_surface_restored_histories",
        "recent_evidence_status_failed_ignores_empty_placeholder_and_passed_case",
        "first_open_recent_evidence_action_specs_gate_disabled_states",
        "recent_evidence_next_action_projects_rerun_and_repair_steps",
        "devtools_recent_evidence_lines_use_current_workflow_state_for_rerunnable_status",
        "devtools_recent_evidence_lines_surface_failed_followup_bundle_dir",
        "recent_failed_evidence_bundle_dir_filters_empty_bundle_dir",
        "recent_failed_evidence_rerun_command_uses_structured_diag_args",
        "recent_failed_evidence_rerun_command_rejects_redacted_workflow_args",
        "recent_failed_evidence_rerun_reason_reports_diag_args_issues",
        "recent_failed_evidence_rerun_command_recovers_redacted_workflow_from_current_state",
        "recent_failed_evidence_rerun_command_uses_current_workflow_state_over_stored_args",
        "recent_failed_evidence_rerun_reason_reports_unregistered_workflow",
        "recent_failed_evidence_rerun_command_projects_followup_bundle",
        "devtools_recent_failed_evidence_target_prefers_visible_latest_then_history",
        "devtools_recent_failed_evidence_target_falls_back_to_lane_order_without_timestamps",
        "devtools_recent_failed_evidence_target_prefers_result_json_time_over_path_time",
        "devtools_recent_failed_evidence_target_carries_result_json_payload",
        "devtools_recent_evidence_selection_effect_routes_to_existing_history_state",
        "workflow route: {DEVTOOLS_WORKFLOW_ROUTE_ID}",
        "Workflow Handoff Readiness",
        "next_action: Load workflow regression summary",
        "next_action: Run workflow summarize",
        "next_action: use Regression Workspace follow-up actions",
        "aggregate_index_ready:",
        "aggregate_index_loaded:",
        "aggregate_workspace: index ready but not loaded",
        "aggregate_workspace: workflow index loaded",
        "aggregate_next_action: Load workflow regression index",
        "aggregate_next_action: aggregate index already loaded",
        "workflow_aggregate_index_loaded(",
        "workflow_aggregate_index_loaded_matches_loaded_artifact_root",
        "Workflow Summarize Handoff",
        "workflow_summarize_command_from_summary_path(",
        "workflow_run_regression_index_artifact_path_from_result_json",
        "workflow_run_result_summary_lines_project_summarize_output_artifacts",
        "workflow_regression_index_parent_dir_targets_artifact_root",
        "workflow_regression_index_action_ids_cover_copy_open_load",
        "Run workflow summarize",
        "Copy workflow summarize command",
        "result artifacts: .fret/diag/workflow-runs/*.json",
        "handoff: load suite regression.summary.json into Regression Workspace",
        "handoff: run workflow summarize to create regression.index.json when missing",
        "campaign validate: cargo run -p fretboard-dev -- diag campaign validate {DEVTOOLS_WORKFLOW_FIRST_OPEN_CAMPAIGN_MANIFEST} --json",
        "imui p3 validate: cargo run -p fretboard-dev -- diag campaign validate {DEVTOOLS_WORKFLOW_IMUI_P3_CAMPAIGN_MANIFEST} --json",
        "suite ws: cargo run -p fretboard-dev -- diag suite {DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE}",
        "regression_selected_perf_evidence",
        "regression_summary_drilldown(&summary)",
        "regression_bundle_followup_command_lines(selected_bundle_dirs.iter().map(|v| v.as_ref()))",
        "followup::load_recent_followup_result_history(",
        "Copy follow-up commands",
        "Follow-up Readiness",
        "selected_followup_readiness_lines",
        "runnable_followups:",
        "visual_compare_ready:",
        "Baseline Compare Actions",
        "materialize_baseline_compare_followup_command(",
        "Run visual compare",
        "Run footprint compare",
        "first_runnable:",
        "Follow-up Commands",
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
        "devtools_first_open_lines_surface_canonical_paths",
        "devtools_first_open_next_action_lines_prioritize_stateful_workflow",
        "first_open_recent_evidence_action_specs_gate_disabled_states",
        "devtools_recent_evidence_lines_surface_restored_histories",
        "recent_evidence_next_action_projects_rerun_and_repair_steps",
        "devtools_recent_evidence_lines_use_current_workflow_state_for_rerunnable_status",
        "devtools_recent_evidence_lines_surface_failed_followup_bundle_dir",
        "recent_failed_evidence_bundle_dir_filters_empty_bundle_dir",
        "recent_failed_evidence_rerun_command_uses_structured_diag_args",
        "recent_failed_evidence_rerun_command_rejects_redacted_workflow_args",
        "recent_failed_evidence_rerun_reason_reports_diag_args_issues",
        "recent_failed_evidence_rerun_command_recovers_redacted_workflow_from_current_state",
        "recent_failed_evidence_rerun_command_uses_current_workflow_state_over_stored_args",
        "recent_failed_evidence_rerun_reason_reports_unregistered_workflow",
        "recent_failed_evidence_rerun_command_projects_followup_bundle",
        "devtools_recent_failed_evidence_target_prefers_visible_latest_then_history",
        "devtools_recent_evidence_selection_effect_routes_to_existing_history_state",
        "devtools_dogfood_workflow_lines_surface_ui_gallery_loop",
        "devtools_demo_metrics_debug_lines_surface_canonical_routes",
        "devtools_workflow_run_lines_surface_campaign_and_suite_entrypoints",
        "devtools_workflow_commands_mark_suite_ws_missing_without_session",
        "devtools_workflow_commands_include_selected_session_for_suite_ws",
        "workflow_run_result_record_has_stable_shape_and_redacts_token",
        "workflow_run_result_summary_lines_project_output_artifacts",
        "workflow_run_regression_summary_artifact_path_extracts_output_artifact",
        "workflow_run_result_history_entry_detail_lines_surface_output_artifacts",
        "load_recent_workflow_run_result_history_reads_latest_valid_records",
        "load_recent_workflow_run_result_history_prefers_record_time_over_file_mtime",
        "load_recent_gate_run_result_history_reads_latest_valid_records",
        "load_recent_gate_run_result_history_prefers_record_time_over_file_mtime",
        "load_recent_followup_result_history_prefers_record_time_over_file_mtime",
        "file_url_from_path_projects_workflow_artifact_paths",
        "devtools_gate_command_lines_surface_first_class_gates",
        "compute_rows_handles_50k_flat_semantics_nodes",
        "compute_rows_handles_50k_deep_semantics_tree_without_recursion",
        "compute_rows_search_forces_visible_ancestor_path_on_large_tree",
        "compute_rows_search_matches_id_parent_and_bounds",
        "secondary_tree_labels_surface_layout_and_identity_fields",
        "live_semantics_request_decision_throttles_unchanged_selection_to_one_hz",
        "live_semantics_request_decision_allows_selection_change_and_manual_refresh",
    ):
        _assert_text_contains(name, source, marker)
    for marker in (
        "pub struct DevtoolsGateProfileV1",
        "pub struct DevtoolsGateCommandV1",
        "pub struct DevtoolsGateScriptTargetCommandInputV1",
        "pub struct DevtoolsGatePerfThresholdCommandInputV1",
        "pub struct DevtoolsGateResourceFootprintThresholdCommandInputV1",
        "pub type DevtoolsGateScriptTargetCommandV1 = DevtoolsGateCommandV1",
        'pub const DEVTOOLS_GATE_PERF_THRESHOLD_PROFILE_ID_V1: &str = "perf-thresholds"',
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
        "pub fn devtools_gate_script_target_profile_ids_v1() -> &'static [&'static str]",
        "pub fn devtools_gate_script_target_command(",
        "pub fn devtools_gate_script_target_command_line(",
        "pub fn is_runnable(&self) -> bool",
        "devtools_gate_script_target_profiles_are_parameterized",
        "devtools_gate_script_target_commands_include_runnable_diag_args",
        "devtools_gate_perf_threshold_command_includes_runnable_diag_args",
        "devtools_gate_resource_footprint_threshold_command_includes_runnable_diag_args",
    ):
        _assert_text_contains(name, gate_profile_source, marker)
    if progress is not None:
        progress.record(
            "step.pass",
            name=name,
            path=str(path),
            ws_path=str(ws_path),
            gate_run_path=str(gate_run_path),
            workflow_run_path=str(workflow_run_path),
            gate_profile_path=str(gate_profile_path),
            protocol_path=str(protocol_path),
            bootstrap_ws_path=str(bootstrap_ws_path),
        )


def _validate_first_open_docs(
    *,
    cwd: Path,
    progress: ProgressRecorder | None = None,
) -> None:
    name = "diagnostics first-open policy-skip docs"
    print(f"[diag-gate-imui-p2-devtools] {name}")
    first_open_path = cwd / FIRST_OPEN_DOC
    checklist_path = cwd / MAINTAINER_CHECKLIST_DOC
    if progress is not None:
        progress.record(
            "step.start",
            name=name,
            first_open_path=str(first_open_path),
            checklist_path=str(checklist_path),
        )
    try:
        first_open_source = first_open_path.read_text(encoding="utf-8")
        checklist_source = checklist_path.read_text(encoding="utf-8")
    except OSError as err:
        if progress is not None:
            progress.record("step.fail", name=name, error=str(err))
        raise SystemExit(f"Step failed: {name} (failed to read docs: {err})") from err

    for marker in (
        "If an aggregate or dashboard reports `skipped_policy`",
        "`capability_source`: provenance for the available/missing capability view.",
        "`capabilities_check_path`: the campaign-local check artifact that explains the skip.",
        MAINTAINER_CHECKLIST_DOC,
    ):
        _assert_text_contains(name, first_open_source, marker)
    for marker in (
        "Treat these fields as one contract slice:",
        "`status = skipped_policy`",
        "`reason_code = capability.missing`",
        "`capability_source`",
        "`capabilities_check_path`",
        "Do not collapse those two concepts into one field",
        "Consumer rule:",
        "GUI, MCP, CLI, and maintainer docs should all preserve this distinction.",
    ):
        _assert_text_contains(name, checklist_source, marker)
    if progress is not None:
        progress.record(
            "step.pass",
            name=name,
            first_open_path=str(first_open_path),
            checklist_path=str(checklist_path),
        )


def _validate_devtools_mcp_ai_scenario_doc(
    *,
    cwd: Path,
    progress: ProgressRecorder | None = None,
) -> None:
    name = "devtools mcp ai scenario doc"
    print(f"[diag-gate-imui-p2-devtools] {name}")
    doc_path = cwd / DEVTOOLS_MCP_DOC
    source_path = cwd / DEVTOOLS_MCP_SOURCE
    if progress is not None:
        progress.record(
            "step.start",
            name=name,
            doc_path=str(doc_path),
            source_path=str(source_path),
        )
    try:
        doc_source = doc_path.read_text(encoding="utf-8")
        mcp_source = source_path.read_text(encoding="utf-8")
    except OSError as err:
        if progress is not None:
            progress.record("step.fail", name=name, error=str(err))
        raise SystemExit(f"Step failed: {name} (failed to read source/doc: {err})") from err

    for marker in (
        "## End-to-end AI scenario",
        "Enable inspect and pick a stable selector",
        "Choose a script and fork it into the user script library",
        "Run one or more scripts",
        "Aggregate regression summaries when you need a campaign view",
        "Pack the latest bundle and open the offline viewer",
        "fret_diag_inspect_set",
        "fret_diag_pick",
        "fret_diag_scripts_list",
        "fret_diag_run_script_file",
        "fret_diag_run",
        "fret_diag_regression_summarize",
        "fret_diag_regression_dashboard",
        "fret_diag_recent_evidence",
        ".fret/diag/gate-runs/*.json",
        ".fret/diag/workflow-runs/*.json",
        ".fret/diag/followups/*.json",
        "fret-diag://recent-evidence.json",
        "Workflow reruns still require the GUI's current selected session/token state.",
        "fret_diag_pack_last_bundle",
        "tools/fret-bundle-viewer",
        "fret-diag://first-open.md",
        "fret-diag://selected/bundle.json",
        "fret-diag://selected/bundle.zip",
        "resources/subscribe",
        "notifications/resources/updated",
    ):
        _assert_text_contains(name, doc_source, marker)

    for marker in (
        'const DEVTOOLS_MCP_DOC: &str =',
        'const RESOURCE_URI_FIRST_OPEN_MD: &str = "fret-diag://first-open.md"',
        "const DEMO_METRICS_DEBUG_OWNER_DOC: &str =",
        "const DEMO_METRICS_DEBUG_ACTION_METADATA_DOC: &str =",
        "const DEMO_METRICS_DEBUG_DOCKING_OWNER_DOC: &str =",
        "const DEMO_METRICS_DEBUG_WAYLAND_ACCEPTANCE_DOC: &str =",
        "const DOCKING_CAMPAIGN_VALIDATE_COMMAND: &str =",
        "const DOCKING_POLICY_SKIP_COMMAND: &str =",
        "struct DemoMetricsDebugActionSpec",
        "id: &'static str",
        "category: &'static str",
        "requires_bundle: bool",
        "primary: bool",
        "action metadata owner: {DEMO_METRICS_DEBUG_ACTION_METADATA_DOC}",
        "action metadata: {} | id={} | category={} | primary={} | requires_bundle={}",
        'const RESOURCE_URI_RECENT_EVIDENCE_JSON: &str = "fret-diag://recent-evidence.json"',
        "async fn fret_diag_inspect_set(",
        "async fn fret_diag_pick(",
        "async fn fret_diag_scripts_list(",
        "async fn fret_diag_run_script_file(",
        "async fn fret_diag_run(",
        "async fn fret_diag_regression_summarize(",
        "async fn fret_diag_regression_dashboard(",
        "async fn fret_diag_recent_evidence(",
        "async fn fret_diag_pack_last_bundle(",
        "async fn fret_diag_pack_last_bundle_zip_bytes(",
        "async fn fret_diag_bundle_dump_latest(",
        "async fn fret_diag_compare(",
        "build_recent_evidence_report(",
        "recent_evidence_latest_failed_entry(",
        "recent_evidence_entry_sort_timestamp(",
        "recent_evidence_result_path_timestamp(",
        "recent_evidence_resource_text(",
        "sessionless_resource_specs()",
        "load_recent_evidence_entries(",
        "RESOURCE_KIND_RECENT_EVIDENCE_JSON",
        "fret-diag://recent-evidence.json",
        "fret_devtools_gate_run_result",
        "fret_devtools_workflow_run_result",
        "fret_devtools_regression_followup_result",
        "sessionless_resource_specs_include_first_open_and_recent_evidence",
        "build_recent_evidence_report_prefers_latest_failed_result_across_lanes",
        "load_recent_evidence_entries_prefers_record_time_over_file_mtime",
        "recent_evidence_status_is_failing_ignores_empty_placeholder_and_passed_case",
        "regression_summary_drilldown(",
        "regression_bundle_followup_command_lines(",
        "ResourceUpdatedNotification",
        "ResourceListChangedNotification",
    ):
        _assert_text_contains(name, mcp_source, marker)

    if progress is not None:
        progress.record(
            "step.pass",
            name=name,
            doc_path=str(doc_path),
            source_path=str(source_path),
        )


def _validate_devtools_cross_cutting_hygiene(
    *,
    cwd: Path,
    progress: ProgressRecorder | None = None,
) -> None:
    name = "devtools cross-cutting hygiene"
    print(f"[diag-gate-imui-p2-devtools] {name}")
    doc_path = cwd / DEVTOOLS_WORKSTREAM_DOC
    viewer_readme_path = cwd / BUNDLE_VIEWER_README
    viewer_parser_path = cwd / BUNDLE_VIEWER_PARSER_SOURCE
    viewer_zip_path = cwd / BUNDLE_VIEWER_ZIP_SOURCE
    fret_ui_readme_path = cwd / FRET_UI_README
    fret_ui_source_dir = cwd / FRET_UI_SOURCE_DIR
    devtools_source_path = cwd / DEVTOOLS_GUI_SOURCE
    if progress is not None:
        progress.record(
            "step.start",
            name=name,
            doc_path=str(doc_path),
            viewer_readme_path=str(viewer_readme_path),
            viewer_parser_path=str(viewer_parser_path),
            viewer_zip_path=str(viewer_zip_path),
            fret_ui_readme_path=str(fret_ui_readme_path),
            fret_ui_source_dir=str(fret_ui_source_dir),
            devtools_source_path=str(devtools_source_path),
        )

    doc_source = _read_text_for_gate(name, doc_path, progress)
    viewer_readme = _read_text_for_gate(name, viewer_readme_path, progress)
    viewer_parser = _read_text_for_gate(name, viewer_parser_path, progress)
    viewer_zip = _read_text_for_gate(name, viewer_zip_path, progress)
    fret_ui_readme = _read_text_for_gate(name, fret_ui_readme_path, progress)
    devtools_source = _read_text_for_gate(name, devtools_source_path, progress)

    for marker in (
        "unknown fields must be ignored by default (forward compatibility).",
        "The GUI should treat `test_id` as the primary",
        "stable handle",
        "at recipe/component authoring time (`ecosystem/*`) when selectors are unstable.",
        "without moving gate policy into `fret-ui`",
    ):
        _assert_text_contains(name, doc_source, marker)

    for marker in (
        "best-effort / forward-compatible (unknown fields are ignored)",
        "`bundle.json`",
        "`bundle.zip`",
    ):
        _assert_text_contains(name, viewer_readme, marker)

    for marker in (
        "// Zod schemas for best-effort parsing",
        "const root = parsed as Record<string, unknown>",
        "const schemaVersion = typeof root.schema_version === 'number'",
        "const windowsRaw = root.windows ?? root.window_list ?? root.windowList",
        "else if (root.snapshots || root.frames || root.history)",
        "else if (root.window_id || root.windowId || root.semantics)",
        "warnings.push({ key: 'warn.cannotFindWindowsOrSnapshots' })",
    ):
        _assert_text_contains(name, viewer_parser, marker)
    if ".strict()" in viewer_parser:
        raise SystemExit(
            "Step failed: devtools cross-cutting hygiene "
            "(bundle parser must not use strict zod schemas)"
        )

    for marker in (
        "lower.endsWith('bundle.schema2.json') || lower.endsWith('bundle.json')",
        "export async function extractBundleAndArtifactsFromZipFile",
        "const artifacts = pickArtifacts(entries, bundlePathInZip)",
        "const screenshots = pickScreenshots(entries, bundlePathInZip)",
    ):
        _assert_text_contains(name, viewer_zip, marker)

    for marker in (
        "`fret-ui` is the UI runtime contract layer",
        "policy-heavy component library",
        "belong in the ecosystem layer (`fret-ui-kit`, `fret-ui-shadcn`) rather than here",
    ):
        _assert_text_contains(name, fret_ui_readme, marker)
    if not fret_ui_source_dir.is_dir():
        raise SystemExit(f"Step failed: {name} (missing source dir: {fret_ui_source_dir})")
    devtools_markers = (
        "DevTools",
        "devtools",
        "fret_devtools",
        "bundle viewer",
        "diag gate",
        "pick-to-fill",
        "script authoring",
    )
    offenders: list[str] = []
    for path in fret_ui_source_dir.rglob("*.rs"):
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError as err:
            raise SystemExit(f"Step failed: {name} (failed to decode {path}: {err})") from err
        for marker in devtools_markers:
            if marker in text:
                offenders.append(f"{path.relative_to(cwd)}:{marker}")
                break
    if offenders:
        joined = ", ".join(offenders[:8])
        suffix = "" if len(offenders) <= 8 else f", ... +{len(offenders) - 8} more"
        raise SystemExit(
            "Step failed: devtools cross-cutting hygiene "
            f"(DevTools policy markers found in fret-ui source: {joined}{suffix})"
        )

    for marker in (
        "let script_selector_kind = app.models_mut().insert(Some(Arc::<str>::from(\"test_id\")))",
        "shadcn::SelectItem::new(\"test_id\", \"test_id\")",
        ".unwrap_or_else(|| Arc::<str>::from(\"test_id\"))",
        "preferred selector: {\\\"kind\\\":\\\"test_id\\\",\\\"id\\\":\\\"ui-gallery-nav-button\\\"}",
        ".test_id(\"devtools.gate.test_id\")",
    ):
        _assert_text_contains(name, devtools_source, marker)

    if progress is not None:
        progress.record(
            "step.pass",
            name=name,
            doc_path=str(doc_path),
            viewer_readme_path=str(viewer_readme_path),
            viewer_parser_path=str(viewer_parser_path),
            viewer_zip_path=str(viewer_zip_path),
            fret_ui_readme_path=str(fret_ui_readme_path),
            devtools_source_path=str(devtools_source_path),
        )


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", default="target/imui-p2-devtools-first-open-smoke")
    parser.add_argument("--timeout-ms", type=int, default=180000)
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument("--release", action="store_true")
    parser.add_argument(
        "--discovery-only",
        action="store_true",
        help="Validate only the first-open DevTools/tool-app discovery index and repo preflight.",
    )
    parser.add_argument(
        "--progress-log",
        help="Write JSONL step progress to this path. Launched mode defaults to <run-root>/gate.progress.jsonl.",
    )
    parser.add_argument(
        "--reuse-built",
        action="store_true",
        help="Reuse existing target/<profile> binaries instead of building them first.",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    profile_dir = "release" if args.release else "debug"
    explicit_progress_path = (
        (repo_root / args.progress_log).resolve() if args.progress_log is not None else None
    )
    root: Path | None = None
    direct_root: Path | None = None
    campaign_base: Path | None = None
    if args.discovery_only:
        progress = ProgressRecorder(explicit_progress_path)
        progress.record("gate.start", mode="discovery")
    else:
        run_id = str(int(time.time() * 1000))
        root = (repo_root / args.out_dir / run_id).resolve()
        direct_root = root / "direct"
        campaign_base = root / "campaign"
        progress = ProgressRecorder(explicit_progress_path or root / "gate.progress.jsonl")
        progress.record("gate.start", mode="launched", run_root=str(root))

    fretboard_exe = repo_root / "target" / profile_dir / _exe_name("fretboard-dev")
    if args.reuse_built:
        progress.record("step.skip", name="cargo build -p fretboard-dev", reason="reuse_built")
    else:
        fretboard_build = ["cargo", "build", "-j", "1", "-p", "fretboard-dev"]
        if args.release:
            fretboard_build.append("--release")
        _run_checked("cargo build -p fretboard-dev", fretboard_build, cwd=repo_root, progress=progress)
    if not fretboard_exe.exists():
        progress.record("gate.fail", reason="missing_fretboard_exe", path=str(fretboard_exe))
        raise SystemExit(f"fretboard-dev exe not found: {fretboard_exe}")

    _validate_tool_app_discovery(fretboard_exe, cwd=repo_root, progress=progress)
    _validate_devtools_gui_first_open_source(cwd=repo_root, progress=progress)
    _validate_first_open_docs(cwd=repo_root, progress=progress)
    _validate_devtools_mcp_ai_scenario_doc(cwd=repo_root, progress=progress)
    _validate_devtools_cross_cutting_hygiene(cwd=repo_root, progress=progress)
    if args.discovery_only:
        progress.record("gate.pass", mode="discovery")
        print("[diag-gate-imui-p2-devtools] discovery done")
        return 0

    assert root is not None
    assert direct_root is not None
    assert campaign_base is not None

    demo_exe = repo_root / "target" / profile_dir / _exe_name("todo_demo")
    if args.reuse_built:
        progress.record("step.skip", name="cargo build -p fret-demo --bin todo_demo", reason="reuse_built")
    else:
        demo_build = ["cargo", "build", "-j", "1", "-p", "fret-demo", "--bin", "todo_demo"]
        if args.release:
            demo_build.append("--release")
        _run_checked(
            "cargo build -p fret-demo --bin todo_demo",
            demo_build,
            cwd=repo_root,
            progress=progress,
        )
    if not demo_exe.exists():
        progress.record("gate.fail", reason="missing_demo_exe", path=str(demo_exe))
        raise SystemExit(f"todo demo exe not found: {demo_exe}")

    launch_env_flags = [
        "--env",
        "FRET_DIAG_REDACT_TEXT=0",
        "--env",
        "FRET_DIAG_FIXED_FRAME_DELTA_MS=16",
        "--env",
        "RUST_LOG=warn",
    ]

    _run_checked(
        "diag run todo-baseline",
        [
            str(fretboard_exe),
            "diag",
            "run",
            SCRIPT_PATH,
            "--dir",
            str(direct_root),
            "--session-auto",
            "--timeout-ms",
            str(args.timeout_ms),
            "--poll-ms",
            str(args.poll_ms),
            *launch_env_flags,
            "--launch",
            "--",
            str(demo_exe),
        ],
        cwd=repo_root,
        progress=progress,
    )

    sessions_root = direct_root / "sessions"
    session_root = _single_child_dir(sessions_root)
    progress.record("artifact.session_root", path=str(session_root))
    after_add = _find_bundle_dir(session_root, LABEL_AFTER_ADD)
    after_toggle = _find_bundle_dir(session_root, LABEL_AFTER_TOGGLE)
    after_remove = _find_bundle_dir(session_root, LABEL_AFTER_REMOVE)
    progress.record(
        "artifact.direct_bundles",
        after_add=str(after_add),
        after_toggle=str(after_toggle),
        after_remove=str(after_remove),
    )
    script_result = _read_json(session_root / "script.result.json")
    recorded_last_bundle_dir = script_result.get("last_bundle_dir")
    expected_bundle_names = {after_add.name, after_toggle.name, after_remove.name}
    if not isinstance(recorded_last_bundle_dir, str) or recorded_last_bundle_dir not in expected_bundle_names:
        raise SystemExit("script.result.json should record one of the named bundle directories")

    resolve_latest = _run_capture_checked(
        "diag resolve latest",
        [
            str(fretboard_exe),
            "diag",
            "resolve",
            "latest",
            "--dir",
            str(direct_root),
            "--json",
        ],
        cwd=repo_root,
        progress=progress,
    )
    resolve_payload = json.loads(resolve_latest.stdout)
    latest_source = resolve_payload.get("latest_bundle_dir_source")
    latest_bundle_dir = resolve_payload.get("latest_bundle_dir")
    if latest_source != "script.result.json:last_bundle_dir":
        raise SystemExit(
            "diag resolve latest did not resolve through script.result.json:last_bundle_dir"
        )
    if not isinstance(latest_bundle_dir, str) or not latest_bundle_dir.endswith(
        recorded_last_bundle_dir
    ):
        raise SystemExit(
            "diag resolve latest did not return the script.result.json last bundle dir"
        )

    latest_human = _run_capture_checked(
        "diag latest",
        [
            str(fretboard_exe),
            "diag",
            "latest",
            "--dir",
            str(direct_root),
        ],
        cwd=repo_root,
        progress=progress,
    )
    latest_human_text = latest_human.stdout + latest_human.stderr
    if "script.result.json:last_bundle_dir" not in latest_human_text:
        raise SystemExit("diag latest should report script.result.json:last_bundle_dir")

    compare_payload = _run_compare_expect_diff(
        "diag compare todo-after-add vs todo-after-toggle-done",
        [
            str(fretboard_exe),
            "diag",
            "compare",
            str(after_add),
            str(after_toggle),
            "--json",
        ],
        cwd=repo_root,
        progress=progress,
    )
    if compare_payload.get("bundle_a") is None or compare_payload.get("bundle_b") is None:
        raise SystemExit("diag compare should report both bundle paths in JSON output")

    _run_checked(
        "diag campaign run devtools-first-open-smoke",
        [
            str(fretboard_exe),
            "diag",
            "campaign",
            "run",
            CAMPAIGN_ID,
            "--dir",
            str(campaign_base),
            "--timeout-ms",
            str(args.timeout_ms),
            "--poll-ms",
            str(args.poll_ms),
            *launch_env_flags,
            "--launch",
            "--",
            str(demo_exe),
        ],
        cwd=repo_root,
        progress=progress,
    )

    campaign_root = _single_child_dir(campaign_base / "campaigns" / CAMPAIGN_ID)
    progress.record("artifact.campaign_root", path=str(campaign_root))
    summarize = _run_capture_checked(
        "diag summarize campaign root",
        [
            str(fretboard_exe),
            "diag",
            "summarize",
            str(campaign_root),
            "--dir",
            str(campaign_root),
            "--json",
        ],
        cwd=repo_root,
        progress=progress,
    )
    summarize_payload = json.loads(summarize.stdout)
    if summarize_payload.get("kind") != "diag_regression_summary":
        raise SystemExit("diag summarize should emit the shared regression summary contract")
    if summarize_payload.get("totals", {}).get("items_total", 0) < 1:
        raise SystemExit("diag summarize should report at least one aggregate item")

    required_paths = [
        campaign_root / "campaign.manifest.json",
        campaign_root / "regression.summary.json",
        campaign_root / "regression.index.json",
    ]
    for path in required_paths:
        if not path.is_file():
            raise SystemExit(f"expected campaign artifact is missing: {path}")

    dashboard = _run_capture_checked(
        "diag dashboard campaign root",
        [
            str(fretboard_exe),
            "diag",
            "dashboard",
            str(campaign_root),
            "--json",
        ],
        cwd=repo_root,
        progress=progress,
    )
    dashboard_payload = json.loads(dashboard.stdout)
    if dashboard_payload.get("kind") != "diag_regression_index":
        raise SystemExit("diag dashboard --json should return the shared regression index contract")
    summaries = dashboard_payload.get("summaries")
    if not isinstance(summaries, list) or not summaries:
        raise SystemExit("diag dashboard should report at least one summarized entry")
    items_total = 0
    for summary in summaries:
        if isinstance(summary, dict):
            items_total += int(summary.get("items_total", 0))
    if items_total < 1:
        raise SystemExit("diag dashboard should report at least one aggregate item")

    campaign_summary = _read_json(campaign_root / "regression.summary.json")
    if campaign_summary.get("kind") != "diag_regression_summary":
        raise SystemExit("regression.summary.json should remain the shared aggregate contract")

    progress.record("gate.pass", mode="launched", run_root=str(root))
    print(f"[diag-gate-imui-p2-devtools] done (out_dir={root})")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
