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
DEVTOOLS_MCP_DOC = "docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md"
DEVTOOLS_GUI_SOURCE = "apps/fret-devtools/src/native.rs"
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
    if progress is not None:
        progress.record("step.start", name=name, path=str(path))
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as err:
        if progress is not None:
            progress.record("step.fail", name=name, error=str(err))
        raise SystemExit(f"Step failed: {name} (failed to read {path}: {err})") from err

    for marker in (
        f'const DEVTOOLS_FIRST_OPEN_DOC: &str = "{FIRST_OPEN_DOC}"',
        f'const DEVTOOLS_GUI_BRANCH_DOC: &str =\n    "{DEVTOOLS_GUI_DOC}"',
        "const DEVTOOLS_REPO_PREFLIGHT_COMMAND: &str =",
        "const DEVTOOLS_REPO_PREFLIGHT_JSON_COMMAND: &str =",
        "const DEVTOOLS_FIRST_OPEN_GATE_COMMAND: &str =",
        f'const DEVTOOLS_FIRST_OPEN_CAMPAIGN_ID: &str = "{CAMPAIGN_ID}"',
        'const IMUI_PRODUCT_WORKFLOW_ID: &str = "imui-product-chain"',
        'const IMUI_PRODUCT_WORKFLOW_DOC: &str =',
        'const IMUI_PRODUCT_WORKFLOW_COMMAND: &str = "python tools/diag_gate_imui_product_chain.py"',
        'const IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND: &str =',
        'const IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND: &str =',
        'const IMUI_PRODUCT_WORKFLOW_SUITE: &str =',
        "const IMUI_PRODUCT_WORKFLOW_ARTIFACTS: &[&str] = &[",
        'const DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID: &str = "demo-metrics-debug"',
        'const DEVTOOLS_DEMO_EDITOR_PROOF_COMMAND: &str =',
        'const DEVTOOLS_METRICS_STATS_COMMAND: &str =',
        'const DEVTOOLS_METRICS_LAYOUT_PERF_COMMAND: &str =',
        'const DEVTOOLS_DEBUG_TRIAGE_COMMAND: &str =',
        'const DEVTOOLS_DEBUG_HOTSPOTS_COMMAND: &str =',
        "First-open Evidence Path",
        "Canonical docs, repo preflight, artifact roots, product-chain evidence, and smoke gate stay visible in the GUI shell.",
        "Demo / Metrics / Debug Routes",
        "Always-available editor demos, metrics commands, and debug drill-down entrypoints stay visible in the GUI shell.",
        "devtools_first_open_lines(st.cfg.fs_out_dir.as_ref())",
        "devtools_demo_metrics_debug_lines(st.cfg.fs_out_dir.as_ref())",
        "fn devtools_first_open_lines(artifacts_root: &str) -> Vec<String>",
        "fn devtools_demo_metrics_debug_lines(artifacts_root: &str) -> Vec<String>",
        "direct loop: diag run -> diag latest -> diag compare",
        "campaign loop: diag campaign run {DEVTOOLS_FIRST_OPEN_CAMPAIGN_ID} -> diag summarize -> diag dashboard",
        "route: {DEVTOOLS_DEMO_METRICS_DEBUG_ROUTE_ID}",
        "metrics stats: {DEVTOOLS_METRICS_STATS_COMMAND}",
        "debug triage: {DEVTOOLS_DEBUG_TRIAGE_COMMAND}",
        "product workflow: {IMUI_PRODUCT_WORKFLOW_ID}",
        "product workflow command: {IMUI_PRODUCT_WORKFLOW_COMMAND}",
        "product workflow focused: {IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND}",
        "product workflow launched: {IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND}",
        "product workflow suite: {IMUI_PRODUCT_WORKFLOW_SUITE}",
        "product workflow docs: {IMUI_PRODUCT_WORKFLOW_DOC}",
        "product workflow artifacts: {}",
        "IMUI_PRODUCT_WORKFLOW_ARTIFACTS.join(\", \")",
        "devtools_first_open_lines_surface_canonical_paths",
        "devtools_demo_metrics_debug_lines_surface_canonical_routes",
    ):
        _assert_text_contains(name, source, marker)
    if progress is not None:
        progress.record("step.pass", name=name, path=str(path))


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
