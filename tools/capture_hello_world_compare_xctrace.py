#!/usr/bin/env python3
"""Capture Apple xctrace recordings for hello_world_compare_demo."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import signal
import subprocess
import time
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any


TEMPLATE_ALIASES = {
    "metal": "Metal System Trace",
    "game-memory": "Game Memory",
    "allocations": "Allocations",
    "metal-system-trace": "Metal System Trace",
}


@dataclass(frozen=True)
class Case:
    label: str
    env: dict[str, str]


@dataclass(frozen=True)
class ProcessRow:
    pid: int
    ppid: int
    command: str


DEFAULT_CASES = ["baseline", "empty", "size1000"]
DEFAULT_TEMPLATES = ["metal"]
DEFAULT_RECORD_MODE = "attach"


CASE_PRESETS: dict[str, Case] = {
    "baseline": Case("baseline", {}),
    "empty": Case(
        "empty",
        {
            "FRET_HELLO_WORLD_COMPARE_NO_TEXT": "1",
            "FRET_HELLO_WORLD_COMPARE_NO_SWATCHES": "1",
        },
    ),
    "size1000": Case(
        "size1000",
        {
            "FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH": "1000",
            "FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": "1000",
        },
    ),
    "size1000-empty": Case(
        "size1000-empty",
        {
            "FRET_HELLO_WORLD_COMPARE_NO_TEXT": "1",
            "FRET_HELLO_WORLD_COMPARE_NO_SWATCHES": "1",
            "FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH": "1000",
            "FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": "1000",
        },
    ),
}


def timestamp_slug() -> str:
    return datetime.now().strftime("%Y%m%d-%H%M%S")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--binary",
        default="target/release/hello_world_compare_demo",
        help="Path to the compare demo binary.",
    )
    parser.add_argument(
        "--out-dir",
        help="Output directory for trace artifacts. Defaults under target/diag/.",
    )
    parser.add_argument(
        "--time-limit",
        default="15s",
        help="Trace duration passed to xctrace.",
    )
    parser.add_argument(
        "--attach-delay-secs",
        type=float,
        default=1.0,
        help="Delay between launching the demo and attaching xctrace in attach mode.",
    )
    parser.add_argument(
        "--shutdown-grace-secs",
        type=float,
        default=5.0,
        help="Grace period before force-killing the launched demo.",
    )
    parser.add_argument(
        "--finalize-timeout-secs",
        type=float,
        default=90.0,
        help="How long to wait for xctrace to finalize after the target is stopped.",
    )
    parser.add_argument(
        "--launch-target-discovery-timeout-secs",
        type=float,
        default=5.0,
        help="How long launch mode spends discovering the spawned target pid under xctrace.",
    )
    parser.add_argument(
        "--record-mode",
        choices=["attach", "launch"],
        default=DEFAULT_RECORD_MODE,
        help="Use attach mode (legacy) or xctrace launch mode.",
    )
    parser.add_argument(
        "--target-exit-after-secs",
        type=float,
        help="Inject FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS into the launched target.",
    )
    parser.add_argument(
        "--pre-init-sleep-secs",
        type=float,
        help="Inject FRET_HELLO_WORLD_COMPARE_PRE_INIT_SLEEP_SECS into the launched target.",
    )
    parser.add_argument(
        "--template",
        action="append",
        default=[],
        help="Template alias or exact xctrace template name. Repeatable.",
    )
    parser.add_argument(
        "--case",
        action="append",
        default=[],
        help="Case preset name or custom case label:KEY=VALUE,KEY=VALUE.",
    )
    parser.add_argument(
        "--export-toc",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Export xctrace TOC XML next to each trace.",
    )
    parser.add_argument(
        "--keep-going",
        action="store_true",
        help="Continue after a failed recording.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print commands without executing them.",
    )
    parser.add_argument(
        "--allow-prompt",
        action="store_true",
        help="Allow xctrace to show interactive prompts.",
    )
    return parser.parse_args()


def resolve_template(raw: str) -> tuple[str, str]:
    normalized = raw.strip()
    if not normalized:
        raise SystemExit("empty template name")
    alias = normalized.lower().replace("_", "-")
    template_name = TEMPLATE_ALIASES.get(alias, normalized)
    slug = alias.replace(" ", "-")
    return slug, template_name


def parse_case(raw: str) -> Case:
    normalized = raw.strip()
    if not normalized:
        raise SystemExit("empty case name")
    preset = CASE_PRESETS.get(normalized)
    if preset is not None:
        return preset
    label, sep, env_raw = normalized.partition(":")
    if not sep:
        raise SystemExit(f"unknown case preset `{normalized}`")
    label = label.strip()
    if not label:
        raise SystemExit(f"invalid case `{normalized}`")
    env: dict[str, str] = {}
    for piece in env_raw.split(","):
        piece = piece.strip()
        if not piece:
            continue
        key, eq, value = piece.partition("=")
        if not eq:
            raise SystemExit(f"invalid case env `{piece}` in `{normalized}`")
        env[key.strip()] = value.strip()
    return Case(label=label, env=env)


def default_out_dir() -> Path:
    return Path("target/diag") / f"hello-world-compare-xctrace-{timestamp_slug()}"


def ensure_binary(path: Path) -> None:
    if not path.is_file():
        raise SystemExit(f"binary not found: {path}")


def parse_duration_secs(raw: str) -> float:
    token = raw.strip().lower()
    if not token:
        raise ValueError("empty time limit")
    units = [("ms", 0.001), ("s", 1.0), ("m", 60.0), ("h", 3600.0)]
    for suffix, scale in units:
        if token.endswith(suffix):
            return float(token[: -len(suffix)]) * scale
    return float(token)


def stop_process(process: subprocess.Popen[bytes], grace_secs: float) -> dict[str, Any]:
    result = {"terminated": False, "killed": False, "returncode": process.poll()}
    if process.poll() is not None:
        result["returncode"] = process.returncode
        return result
    process.terminate()
    result["terminated"] = True
    deadline = time.time() + grace_secs
    while time.time() < deadline:
        if process.poll() is not None:
            result["returncode"] = process.returncode
            return result
        time.sleep(0.1)
    process.kill()
    result["killed"] = True
    process.wait(timeout=5)
    result["returncode"] = process.returncode
    return result


def pid_is_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def stop_pid(pid: int, grace_secs: float) -> dict[str, Any]:
    result: dict[str, Any] = {
        "pid": pid,
        "terminated": False,
        "killed": False,
        "returncode": None,
        "found": pid_is_alive(pid),
    }
    if not result["found"]:
        return result
    os.kill(pid, signal.SIGTERM)
    result["terminated"] = True
    deadline = time.time() + grace_secs
    while time.time() < deadline:
        if not pid_is_alive(pid):
            result["found"] = False
            return result
        time.sleep(0.1)
    os.kill(pid, signal.SIGKILL)
    result["killed"] = True
    for _ in range(50):
        if not pid_is_alive(pid):
            result["found"] = False
            return result
        time.sleep(0.1)
    return result


def list_process_rows() -> list[ProcessRow]:
    proc = subprocess.run(
        ["ps", "-Ao", "pid=,ppid=,command="],
        capture_output=True,
        text=True,
        check=True,
    )
    rows: list[ProcessRow] = []
    for line in proc.stdout.splitlines():
        parts = line.strip().split(None, 2)
        if len(parts) != 3:
            continue
        pid_raw, ppid_raw, command = parts
        try:
            rows.append(ProcessRow(pid=int(pid_raw), ppid=int(ppid_raw), command=command))
        except ValueError:
            continue
    return rows


def command_matches_binary(command: str, binary: Path) -> bool:
    stripped = command.strip()
    if not stripped:
        return False
    try:
        argv0 = shlex.split(stripped)[0]
    except ValueError:
        argv0 = stripped.split()[0]
    binary_str = str(binary)
    return argv0 == binary_str or Path(argv0).name == binary.name


def find_descendant_process(root_pid: int, binary: Path) -> ProcessRow | None:
    rows = list_process_rows()
    children: dict[int, list[ProcessRow]] = {}
    for row in rows:
        children.setdefault(row.ppid, []).append(row)
    stack = [root_pid]
    seen: set[int] = set()
    while stack:
        current = stack.pop()
        if current in seen:
            continue
        seen.add(current)
        for child in children.get(current, []):
            if command_matches_binary(child.command, binary):
                return child
            stack.append(child.pid)
    return None


def export_toc_if_requested(
    *,
    trace_path: Path,
    toc_path: Path,
    export_toc: bool,
    no_prompt: bool,
) -> tuple[str | None, str | None]:
    if not export_toc:
        return None, None
    export_command = [
        "xcrun",
        "xctrace",
        "export",
        "--input",
        str(trace_path),
        "--toc",
        "--output",
        str(toc_path),
    ]
    if no_prompt:
        export_command.append("--quiet")
    try:
        subprocess.run(export_command, check=True)
    except subprocess.CalledProcessError as exc:
        return None, repr(exc)
    return str(toc_path), None


def finish_record_run(
    *,
    case: Case,
    template_name: str,
    template_slug: str,
    trace_path: Path,
    stdout_path: Path,
    stderr_path: Path,
    toc_path: Path,
    launch_command: list[str],
    trace_command: list[str],
    record_mode: str,
    stop: dict[str, Any],
    trace_returncode: int | None,
    export_toc: bool,
    no_prompt: bool,
    target_pid: int | None = None,
    extra_issues: list[str] | None = None,
) -> dict[str, Any]:
    toc_status, toc_error = export_toc_if_requested(
        trace_path=trace_path,
        toc_path=toc_path,
        export_toc=export_toc,
        no_prompt=no_prompt,
    )
    issues: list[str] = list(extra_issues or [])
    status = "recorded"
    if trace_returncode not in {0, None}:
        issues.append(f"xctrace record exited with code {trace_returncode}")
        status = "recorded-with-issues" if trace_path.exists() else "failed"
    elif issues and trace_path.exists():
        status = "recorded-with-issues"
    if toc_error is not None:
        issues.append(f"toc export failed: {toc_error}")
        if status == "recorded":
            status = "recorded-with-issues"
    return {
        "case": case.label,
        "template": template_name,
        "template_slug": template_slug,
        "record_mode": record_mode,
        "trace": str(trace_path),
        "stdout": str(stdout_path),
        "stderr": str(stderr_path),
        "toc": toc_status,
        "launch_command": launch_command,
        "trace_command": trace_command,
        "stop": stop,
        "trace_returncode": trace_returncode,
        "target_pid": target_pid,
        "issues": issues,
        "status": status,
    }


def record_trace_attach(
    *,
    binary: Path,
    case: Case,
    template_slug: str,
    template_name: str,
    out_dir: Path,
    time_limit: str,
    attach_delay_secs: float,
    shutdown_grace_secs: float,
    finalize_timeout_secs: float,
    export_toc: bool,
    dry_run: bool,
    no_prompt: bool,
) -> dict[str, Any]:
    case_dir = out_dir / case.label
    case_dir.mkdir(parents=True, exist_ok=True)

    trace_path = case_dir / f"{case.label}.{template_slug}.trace"
    stdout_path = case_dir / f"{case.label}.stdout.log"
    stderr_path = case_dir / f"{case.label}.stderr.log"
    toc_path = case_dir / f"{case.label}.{template_slug}.toc.xml"
    launch_command = [str(binary)]

    trace_command_template = [
        "xcrun",
        "xctrace",
        "record",
        "--template",
        template_name,
        "--time-limit",
        time_limit,
        "--output",
        str(trace_path),
        "--run-name",
        case.label,
    ]
    if no_prompt:
        trace_command_template.append("--no-prompt")

    if dry_run:
        return {
            "case": case.label,
            "template": template_name,
            "template_slug": template_slug,
            "record_mode": "attach",
            "trace": str(trace_path),
            "stdout": str(stdout_path),
            "stderr": str(stderr_path),
            "toc": str(toc_path) if export_toc else None,
            "launch_command": launch_command,
            "trace_command": trace_command_template + ["--attach", "<pid>"],
            "status": "dry-run",
        }

    env = os.environ.copy()
    env.update(case.env)

    trace_command = trace_command_template + ["--attach", "<pid>"]
    stop = {"terminated": False, "killed": False, "returncode": None}
    trace_returncode: int | None = None
    extra_issues: list[str] = []

    with stdout_path.open("wb") as stdout_file, stderr_path.open("wb") as stderr_file:
        process = subprocess.Popen(launch_command, stdout=stdout_file, stderr=stderr_file, env=env)
        trace_process = None
        trace_duration_secs = parse_duration_secs(time_limit)
        try:
            if attach_delay_secs > 0:
                time.sleep(attach_delay_secs)
            trace_command = trace_command_template + ["--attach", str(process.pid)]
            trace_process = subprocess.Popen(trace_command)
            xctrace_wait_deadline = time.time() + trace_duration_secs + 1.0
            while time.time() < xctrace_wait_deadline:
                if trace_process.poll() is not None:
                    break
                time.sleep(0.1)
            stop = stop_process(process, shutdown_grace_secs)
            try:
                trace_returncode = trace_process.wait(timeout=finalize_timeout_secs)
            except subprocess.TimeoutExpired:
                extra_issues.append(
                    f"xctrace finalize timed out after {finalize_timeout_secs:.1f}s"
                )
        finally:
            if process.poll() is None:
                stop = stop_process(process, shutdown_grace_secs)
            if trace_process is not None and trace_process.poll() is None:
                trace_process.terminate()
                try:
                    trace_process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    trace_process.kill()
                    trace_process.wait(timeout=5)

    return finish_record_run(
        case=case,
        template_name=template_name,
        template_slug=template_slug,
        trace_path=trace_path,
        stdout_path=stdout_path,
        stderr_path=stderr_path,
        toc_path=toc_path,
        launch_command=launch_command,
        trace_command=trace_command,
        record_mode="attach",
        stop=stop,
        trace_returncode=trace_returncode,
        export_toc=export_toc,
        no_prompt=no_prompt,
        target_pid=process.pid,
        extra_issues=extra_issues,
    )


def record_trace_launch(
    *,
    binary: Path,
    case: Case,
    template_slug: str,
    template_name: str,
    out_dir: Path,
    time_limit: str,
    shutdown_grace_secs: float,
    finalize_timeout_secs: float,
    launch_target_discovery_timeout_secs: float,
    export_toc: bool,
    dry_run: bool,
    no_prompt: bool,
) -> dict[str, Any]:
    case_dir = out_dir / case.label
    case_dir.mkdir(parents=True, exist_ok=True)

    trace_path = case_dir / f"{case.label}.{template_slug}.trace"
    stdout_path = case_dir / f"{case.label}.stdout.log"
    stderr_path = case_dir / f"{case.label}.stderr.log"
    toc_path = case_dir / f"{case.label}.{template_slug}.toc.xml"
    launch_command = [str(binary)]

    stdout_path.touch(exist_ok=True)
    stderr_path.touch(exist_ok=True)

    trace_command = [
        "xcrun",
        "xctrace",
        "record",
        "--template",
        template_name,
        "--time-limit",
        time_limit,
        "--output",
        str(trace_path),
        "--run-name",
        case.label,
        "--target-stdout",
        str(stdout_path),
    ]
    if no_prompt:
        trace_command.append("--no-prompt")
    for key, value in case.env.items():
        trace_command.extend(["--env", f"{key}={value}"])
    trace_command.extend(["--launch", "--", str(binary)])

    if dry_run:
        return {
            "case": case.label,
            "template": template_name,
            "template_slug": template_slug,
            "record_mode": "launch",
            "trace": str(trace_path),
            "stdout": str(stdout_path),
            "stderr": str(stderr_path),
            "toc": str(toc_path) if export_toc else None,
            "launch_command": launch_command,
            "trace_command": trace_command,
            "status": "dry-run",
        }

    trace_duration_secs = parse_duration_secs(time_limit)
    trace_process = subprocess.Popen(trace_command)
    target_pid: int | None = None
    stop: dict[str, Any] = {
        "pid": None,
        "terminated": False,
        "killed": False,
        "returncode": None,
        "found": False,
    }
    trace_returncode: int | None = None
    extra_issues: list[str] = []
    try:
        target_discovery_deadline = time.time() + max(launch_target_discovery_timeout_secs, 0.5)
        while time.time() < target_discovery_deadline:
            if trace_process.poll() is not None:
                break
            descendant = find_descendant_process(trace_process.pid, binary)
            if descendant is not None:
                target_pid = descendant.pid
                break
            time.sleep(0.1)

        xctrace_wait_deadline = time.time() + trace_duration_secs + 1.0
        while time.time() < xctrace_wait_deadline:
            if trace_process.poll() is not None:
                break
            if target_pid is None:
                descendant = find_descendant_process(trace_process.pid, binary)
                if descendant is not None:
                    target_pid = descendant.pid
            time.sleep(0.1)

        if trace_process.poll() is None and target_pid is not None:
            stop = stop_pid(target_pid, shutdown_grace_secs)
        elif target_pid is not None:
            stop = {
                "pid": target_pid,
                "terminated": False,
                "killed": False,
                "returncode": None,
                "found": pid_is_alive(target_pid),
            }
        else:
            extra_issues.append("launch mode did not discover a target pid under xctrace")

        try:
            trace_returncode = trace_process.wait(timeout=finalize_timeout_secs)
        except subprocess.TimeoutExpired:
            extra_issues.append(
                f"xctrace finalize timed out after {finalize_timeout_secs:.1f}s"
            )
    finally:
        if trace_process.poll() is None:
            trace_process.terminate()
            try:
                trace_process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                trace_process.kill()
                trace_process.wait(timeout=5)
        if target_pid is not None and pid_is_alive(target_pid):
            stop = stop_pid(target_pid, shutdown_grace_secs)

    return finish_record_run(
        case=case,
        template_name=template_name,
        template_slug=template_slug,
        trace_path=trace_path,
        stdout_path=stdout_path,
        stderr_path=stderr_path,
        toc_path=toc_path,
        launch_command=launch_command,
        trace_command=trace_command,
        record_mode="launch",
        stop=stop,
        trace_returncode=trace_returncode,
        export_toc=export_toc,
        no_prompt=no_prompt,
        target_pid=target_pid,
        extra_issues=extra_issues,
    )


def record_trace(
    *,
    binary: Path,
    case: Case,
    template_slug: str,
    template_name: str,
    out_dir: Path,
    time_limit: str,
    attach_delay_secs: float,
    shutdown_grace_secs: float,
    finalize_timeout_secs: float,
    launch_target_discovery_timeout_secs: float,
    export_toc: bool,
    dry_run: bool,
    no_prompt: bool,
    record_mode: str,
) -> dict[str, Any]:
    if record_mode == "launch":
        return record_trace_launch(
            binary=binary,
            case=case,
            template_slug=template_slug,
            template_name=template_name,
            out_dir=out_dir,
            time_limit=time_limit,
            shutdown_grace_secs=shutdown_grace_secs,
            finalize_timeout_secs=finalize_timeout_secs,
            launch_target_discovery_timeout_secs=launch_target_discovery_timeout_secs,
            export_toc=export_toc,
            dry_run=dry_run,
            no_prompt=no_prompt,
        )
    return record_trace_attach(
        binary=binary,
        case=case,
        template_slug=template_slug,
        template_name=template_name,
        out_dir=out_dir,
        time_limit=time_limit,
        attach_delay_secs=attach_delay_secs,
        shutdown_grace_secs=shutdown_grace_secs,
        finalize_timeout_secs=finalize_timeout_secs,
        export_toc=export_toc,
        dry_run=dry_run,
        no_prompt=no_prompt,
    )


def write_summary(out_dir: Path, runs: list[dict[str, Any]], failures: list[dict[str, Any]]) -> None:
    summary = {
        "schema_version": 2,
        "generated_at": datetime.now().isoformat(),
        "runs": runs,
        "failures": failures,
    }
    (out_dir / "summary.json").write_text(json.dumps(summary, indent=2))

    lines = [
        "# hello_world_compare xctrace capture",
        "",
        "## Runs",
        "",
        "| Case | Template | Mode | Status | Trace | TOC | Stdout | Stderr |",
        "| --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for run in runs:
        lines.append(
            f"| {run['case']} | {run['template']} | {run.get('record_mode', '')} | {run['status']} | {run['trace']} | {run.get('toc') or ''} | {run['stdout']} | {run['stderr']} |"
        )
    if failures:
        lines.extend(["", "## Failures", ""])
        for failure in failures:
            lines.append(f"- `{failure['case']}` / `{failure['template']}`: `{failure['error']}`")
    lines.extend(["", "## Commands", ""])
    for run in runs:
        lines.append(f"- launch: `{shlex.join(run['launch_command'])}`")
        lines.append(f"- trace: `{shlex.join(run['trace_command'])}`")
        if run.get("issues"):
            lines.append(f"- issues: `{'; '.join(run['issues'])}`")
    (out_dir / "summary.md").write_text("\n".join(lines) + "\n")


def main() -> int:
    args = parse_args()
    binary = Path(args.binary)
    ensure_binary(binary)

    out_dir = Path(args.out_dir) if args.out_dir else default_out_dir()
    out_dir.mkdir(parents=True, exist_ok=True)

    templates_raw = args.template or list(DEFAULT_TEMPLATES)
    templates = [resolve_template(raw) for raw in templates_raw]
    cases_raw = args.case or list(DEFAULT_CASES)
    cases = [parse_case(raw) for raw in cases_raw]
    if args.target_exit_after_secs is not None:
        exit_after_secs = args.target_exit_after_secs
        if not (exit_after_secs > 0.0):
            raise SystemExit("--target-exit-after-secs must be > 0")
        cases = [
            Case(
                label=case.label,
                env={
                    **case.env,
                    "FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS": str(exit_after_secs),
                },
            )
            for case in cases
        ]
    if args.pre_init_sleep_secs is not None:
        pre_init_sleep_secs = args.pre_init_sleep_secs
        if not (pre_init_sleep_secs > 0.0):
            raise SystemExit("--pre-init-sleep-secs must be > 0")
        cases = [
            Case(
                label=case.label,
                env={
                    **case.env,
                    "FRET_HELLO_WORLD_COMPARE_PRE_INIT_SLEEP_SECS": str(pre_init_sleep_secs),
                },
            )
            for case in cases
        ]

    runs: list[dict[str, Any]] = []
    failures: list[dict[str, Any]] = []

    for case in cases:
        for template_slug, template_name in templates:
            print(
                f"==> Recording case `{case.label}` with template `{template_name}` ({args.record_mode})",
                flush=True,
            )
            try:
                run = record_trace(
                    binary=binary,
                    case=case,
                    template_slug=template_slug,
                    template_name=template_name,
                    out_dir=out_dir,
                    time_limit=args.time_limit,
                    attach_delay_secs=args.attach_delay_secs,
                    shutdown_grace_secs=args.shutdown_grace_secs,
                    finalize_timeout_secs=args.finalize_timeout_secs,
                    launch_target_discovery_timeout_secs=args.launch_target_discovery_timeout_secs,
                    export_toc=args.export_toc,
                    dry_run=args.dry_run,
                    no_prompt=not args.allow_prompt,
                    record_mode=args.record_mode,
                )
                runs.append(run)
            except Exception as exc:  # noqa: BLE001
                failures.append(
                    {
                        "case": case.label,
                        "template": template_name,
                        "error": repr(exc),
                    }
                )
                if not args.keep_going:
                    write_summary(out_dir, runs, failures)
                    return 1

    write_summary(out_dir, runs, failures)
    print(json.dumps({"runs": len(runs), "failures": failures}, indent=2))
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
