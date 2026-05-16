#!/usr/bin/env python3
"""
Perf gate for resize-focused suites (cross-platform, no jq/bash).

Python alternative to:
  - tools/perf/diag_resize_probes_gate.sh (requires bash + jq)
"""

from __future__ import annotations

import argparse
import json
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path

DEFAULT_PREWARM_SCRIPT = "tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json"
DEFAULT_PRELUDE_SCRIPT = "tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json"


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _resolve_workspace_path(workspace_root: Path, p: str) -> Path:
    path = Path(p)
    if path.is_absolute():
        return path
    return workspace_root / path


def _read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def _write_json(path: Path, v: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(v, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def _host_platform_key() -> str:
    if sys.platform.startswith("win"):
        return "windows"
    if sys.platform == "darwin":
        return "macos"
    return "unknown"


def _default_baseline_for_suite(suite: str) -> str:
    platform_key = _host_platform_key()
    if suite == "ui-resize-probes":
        if platform_key == "windows":
            return "docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v2.json"
        if platform_key == "macos":
            return "docs/workstreams/perf-baselines/ui-resize-probes.macos-m4.v3.json"
    if suite == "ui-code-editor-resize-probes":
        if platform_key == "windows":
            return "docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v2.json"
        if platform_key == "macos":
            return "docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json"
    raise KeyError(suite)


def _run(cmd: list[str], cwd: Path, stdout_path: Path, stderr_path: Path) -> int:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        p = subprocess.run(cmd, cwd=str(cwd), stdout=out, stderr=err)
        return int(p.returncode)


def _diag_perf_prefix(fretboard_bin_path: Path | None, suite: str) -> list[str]:
    if fretboard_bin_path is not None:
        return [
            str(fretboard_bin_path),
            "diag",
            "perf",
            suite,
        ]
    return [
        "cargo",
        "run",
        "-q",
        "-p",
        "fretboard-dev",
        "--",
        "diag",
        "perf",
        suite,
    ]


def _default_launch_cmd() -> list[str]:
    return [
        "cargo",
        "run",
        "-p",
        "fret-ui-gallery",
        "--release",
        "--features",
        "gallery-full",
    ]


def _launch_cmd_from_args(workspace_root: Path, args: argparse.Namespace) -> tuple[list[str], Path | None]:
    launch_cmd_arg = getattr(args, "launch_cmd", None)
    if isinstance(launch_cmd_arg, list):
        launch_cmd = [str(token) for token in launch_cmd_arg if str(token)]
        if not launch_cmd:
            raise ValueError("--launch-cmd requires at least one token")
        return launch_cmd, None
    if isinstance(launch_cmd_arg, str) and launch_cmd_arg.strip():
        try:
            launch_cmd = shlex.split(launch_cmd_arg)
        except ValueError as exc:
            raise ValueError(f"invalid --launch-cmd: {exc}") from exc
        if not launch_cmd:
            raise ValueError("--launch-cmd requires at least one token")
        return launch_cmd, None

    launch_bin_raw = str(getattr(args, "launch_bin", "")).strip()
    if launch_bin_raw:
        launch_bin_path = _resolve_workspace_path(workspace_root, launch_bin_raw)
        return [str(launch_bin_path)], launch_bin_path

    return _default_launch_cmd(), None


def _failures_count(check_path: Path) -> int | None:
    if not check_path.is_file():
        return None
    try:
        doc = _read_json(check_path)
    except Exception:
        return None
    failures = None
    if isinstance(doc, dict):
        failures = doc.get("failures")
    if not isinstance(failures, list):
        return None
    return len(failures)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Run a resize-focused `fretboard-dev diag perf` suite and enforce perf baseline thresholds. "
            "Intended as a lightweight 'P0 resize must not regress' gate."
        ),
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--suite", default="ui-resize-probes")
    ap.add_argument("--out-dir", default="")
    ap.add_argument(
        "--baseline",
        default="",
        help="Perf baseline JSON path. Defaults to the checked-in Windows RTX4090 or macOS baseline for the host platform.",
    )
    ap.add_argument(
        "--launch-bin",
        default="",
        help=(
            "Legacy direct binary launch. Prefer the default cargo launch, or pass --launch-cmd as "
            "the final option, when scripts declare required_launch_features."
        ),
    )
    ap.add_argument(
        "--launch-cmd",
        default="",
        help=(
            "Shell-like launch command forwarded after `diag perf --launch --`. Quote the whole "
            "value when passing spaces. Defaults to `cargo run -p fret-ui-gallery --release "
            "--features gallery-full`."
        ),
    )
    ap.add_argument(
        "--fretboard-bin",
        default="",
        help="Optional prebuilt fretboard-dev binary. Defaults to `cargo run -q -p fretboard-dev --`.",
    )
    ap.add_argument("--timeout-ms", type=int, default=300_000)
    ap.add_argument("--attempts", type=int, default=1)
    ap.add_argument("--repeat", type=int, default=7)
    ap.add_argument("--warmup-frames", type=int, default=5)
    ap.add_argument(
        "--prewarm-script",
        action="append",
        default=[],
        help="Forwarded to `diag perf --prewarm-script <script.json>` (repeatable).",
    )
    ap.add_argument(
        "--prelude-script",
        action="append",
        default=[],
        help="Forwarded to `diag perf --prelude-script <script.json>` (repeatable).",
    )
    ap.add_argument(
        "--no-default-suite-hooks",
        action="store_true",
        default=False,
        help="Do not add the default font prewarm and reset-diagnostics prelude scripts.",
    )

    args = ap.parse_args()

    if args.attempts < 1:
        print("error: --attempts must be >= 1", file=sys.stderr)
        return 2

    workspace_root = _workspace_root()

    suite = str(args.suite)

    out_dir = args.out_dir.strip()
    if not out_dir:
        out_dir = f"target/fret-diag-resize-probes-gate-{int(time.time())}"
    out_dir_path = _resolve_workspace_path(workspace_root, out_dir)
    out_dir_path.mkdir(parents=True, exist_ok=True)

    baseline_raw = args.baseline.strip()
    if not baseline_raw:
        try:
            baseline_raw = _default_baseline_for_suite(suite)
        except KeyError:
            print(
                f"error: no default baseline for --suite {suite!r} on platform {_host_platform_key()!r} "
                "(provide --baseline explicitly)",
                file=sys.stderr,
            )
            return 2
    baseline_path = _resolve_workspace_path(workspace_root, baseline_raw)

    try:
        launch_cmd, launch_bin_path = _launch_cmd_from_args(workspace_root, args)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    fretboard_bin_raw = str(args.fretboard_bin).strip()
    fretboard_bin_path = _resolve_workspace_path(workspace_root, fretboard_bin_raw) if fretboard_bin_raw else None

    prewarm_scripts = list(args.prewarm_script)
    prelude_scripts = list(args.prelude_script)
    if not bool(args.no_default_suite_hooks):
        prewarm_scripts.insert(0, DEFAULT_PREWARM_SCRIPT)
        prelude_scripts.insert(0, DEFAULT_PRELUDE_SCRIPT)
    prewarm_script_paths = [_resolve_workspace_path(workspace_root, p) for p in prewarm_scripts]
    prelude_script_paths = [_resolve_workspace_path(workspace_root, p) for p in prelude_scripts]

    if not baseline_path.is_file():
        print(f"error: baseline not found: {baseline_path}", file=sys.stderr)
        return 2
    if fretboard_bin_path is not None and not fretboard_bin_path.is_file():
        print(f"error: fretboard binary not found: {fretboard_bin_path}", file=sys.stderr)
        return 2
    if launch_bin_path is not None and not launch_bin_path.is_file():
        print(f"error: launch binary not found: {launch_bin_path}", file=sys.stderr)
        return 2
    for hook_path in [*prewarm_script_paths, *prelude_script_paths]:
        if not hook_path.is_file():
            print(f"error: suite hook script not found: {hook_path}", file=sys.stderr)
            return 2

    print(f"[gate] {suite} -> {out_dir_path} (attempts={int(args.attempts)})")
    print(f"[gate] baseline: {baseline_path}")
    print(f"[gate] fretboard-bin: {fretboard_bin_path if fretboard_bin_path is not None else 'cargo run -q -p fretboard-dev --'}")
    print(f"[gate] launch-cmd: {shlex.join(launch_cmd)}")
    print(f"[gate] prewarm: {[str(p) for p in prewarm_script_paths]}")
    print(f"[gate] prelude: {[str(p) for p in prelude_script_paths]}")

    passes = 0
    fails = 0
    selected_attempt_dir: Path | None = None
    attempt_summaries: list[dict[str, object]] = []

    for i in range(1, int(args.attempts) + 1):
        attempt_dir = out_dir_path / f"attempt-{i}"
        attempt_dir.mkdir(parents=True, exist_ok=True)

        cmd = [
            *_diag_perf_prefix(fretboard_bin_path, suite),
            "--dir",
            str(attempt_dir),
            "--timeout-ms",
            str(int(args.timeout_ms)),
        ]
        for script in prewarm_script_paths:
            cmd += ["--prewarm-script", str(script)]
        for script in prelude_script_paths:
            cmd += ["--prelude-script", str(script)]
        cmd += [
            "--reuse-launch",
            "--repeat",
            str(int(args.repeat)),
            "--warmup-frames",
            str(int(args.warmup_frames)),
            "--sort",
            "time",
            "--top",
            "15",
            "--json",
            "--perf-baseline",
            str(baseline_path),
            "--env",
            "FRET_UI_GALLERY_VIEW_CACHE=1",
            "--env",
            "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1",
            "--env",
            "FRET_DIAG_SCRIPT_AUTO_DUMP=0",
            "--env",
            "FRET_DIAG_SEMANTICS=0",
        ]
        if suite == "ui-code-editor-resize-probes":
            cmd += [
                "--env",
                "FRET_CODE_EDITOR_DIAG_PAINT_PERF=1",
            ]
        cmd += [
            "--launch",
            "--",
            *launch_cmd,
        ]

        print(f"[gate] attempt {i}/{int(args.attempts)} -> {attempt_dir}")
        print("[gate] cmd:", " ".join(cmd))

        stdout_path = attempt_dir / "stdout.json"
        stderr_path = attempt_dir / "stderr.log"
        rc = _run(cmd, workspace_root, stdout_path, stderr_path)

        check_path = attempt_dir / "check.perf_thresholds.json"
        failures_count = _failures_count(check_path)

        attempt_pass = True
        if rc != 0:
            attempt_pass = False
        if failures_count is None:
            attempt_pass = False
        elif failures_count != 0:
            attempt_pass = False

        if attempt_pass:
            passes += 1
            if selected_attempt_dir is None:
                selected_attempt_dir = attempt_dir
        else:
            fails += 1

        attempt_summaries.append(
            {
                "attempt_dir": str(attempt_dir),
                "pass": attempt_pass,
                "rc": int(rc),
                "check": {
                    "perf_thresholds": str(check_path),
                    "failures": failures_count,
                },
                "stdout": str(stdout_path),
                "stderr": str(stderr_path),
            }
        )

    majority_required = int(args.attempts) // 2 + 1
    pass_gate = passes >= majority_required

    if selected_attempt_dir is None:
        selected_attempt_dir = out_dir_path / f"attempt-{int(args.attempts)}"

    # Preserve compatibility with downstream tooling by copying one attempt to the top-level paths.
    for name in ["stdout.json", "stderr.log", "check.perf_thresholds.json"]:
        src = selected_attempt_dir / name
        dst = out_dir_path / name
        try:
            if src.is_file():
                shutil.copyfile(src, dst)
        except Exception:
            pass

    summary = {
        "kind": "resize_probes_gate_summary",
        "pass": pass_gate,
        "out_dir": str(out_dir_path),
        "suite": suite,
        "baseline": str(baseline_path),
        "fretboard_bin": str(fretboard_bin_path) if fretboard_bin_path is not None else None,
        "launch_bin": str(launch_bin_path) if launch_bin_path is not None else None,
        "launch_cmd": launch_cmd,
        "suite_hooks": {
            "prewarm": [str(p) for p in prewarm_script_paths],
            "prelude": [str(p) for p in prelude_script_paths],
            "default_suite_hooks": not bool(args.no_default_suite_hooks),
        },
        "attempts": int(args.attempts),
        "pass_attempts": passes,
        "fail_attempts": fails,
        "majority_required": majority_required,
        "selected_attempt_dir": str(selected_attempt_dir),
        "repeat": int(args.repeat),
        "warmup_frames": int(args.warmup_frames),
        "check": {
            "perf_thresholds": str(out_dir_path / "check.perf_thresholds.json"),
            "failures": None,
        },
        "stdout": str(out_dir_path / "stdout.json"),
        "stderr": str(out_dir_path / "stderr.log"),
        "attempt_summaries": attempt_summaries,
    }
    summary_path = out_dir_path / "summary.json"
    _write_json(summary_path, summary)

    if not pass_gate:
        print(
            f"[gate] FAIL (passes={passes}/{int(args.attempts)}; required={majority_required}). See: {summary_path}",
            file=sys.stderr,
        )
        return 1

    print(f"[gate] PASS (passes={passes}/{int(args.attempts)}; required={majority_required}). Summary: {summary_path}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
