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
DEVTOOLS_GUI_SOURCE = "apps/fret-devtools/src/native.rs"
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
}

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
    if needle not in haystack:
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
    print(f"[diag-gate-imui-product-chain] {name}", flush=True)
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as err:
        raise SystemExit(f"Step failed: {name} (failed to read {path}: {err})") from err

    for marker in (
        'const IMUI_PRODUCT_WORKFLOW_ID: &str = "imui-product-chain"',
        'const IMUI_PRODUCT_WORKFLOW_DOC: &str =',
        'const IMUI_PRODUCT_WORKFLOW_COMMAND: &str = "python tools/diag_gate_imui_product_chain.py"',
        'const IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND: &str =',
        'const IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND: &str =',
        'const IMUI_PRODUCT_WORKFLOW_SUITE: &str =',
        'const IMUI_PRODUCT_WORKFLOW_ARTIFACTS: &[&str] = &[',
        'const DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID: &str = "demo-metrics-debug"',
        'const DEVTOOLS_DEMO_EDITOR_PROOF_COMMAND: &str =',
        'const DEVTOOLS_METRICS_STATS_COMMAND: &str =',
        'const DEVTOOLS_METRICS_LAYOUT_PERF_COMMAND: &str =',
        'const DEVTOOLS_DEBUG_TRIAGE_COMMAND: &str =',
        "Demo / Metrics / Debug Routes",
        "devtools_demo_metrics_debug_lines(st.cfg.fs_out_dir.as_ref())",
        "fn devtools_demo_metrics_debug_lines(artifacts_root: &str) -> Vec<String>",
        "route: {DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID}",
        "metrics stats: {DEVTOOLS_METRICS_STATS_COMMAND}",
        "debug triage: {DEVTOOLS_DEBUG_TRIAGE_COMMAND}",
        "devtools_demo_metrics_debug_lines_surface_canonical_routes",
        "regression_selected_perf_evidence",
        "regression_summary_drilldown(&summary)",
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
        'const IMUI_PRODUCT_WORKFLOW_ID: &str = "imui-product-chain"',
        'const IMUI_PRODUCT_WORKFLOW_COMMAND: &str = "python tools/diag_gate_imui_product_chain.py"',
        'const IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND: &str =',
        'const IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND: &str =',
        'const IMUI_PRODUCT_WORKFLOW_SUITE: &str =',
        'const IMUI_PRODUCT_WORKFLOW_ARTIFACTS: &[&str] = &[',
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
