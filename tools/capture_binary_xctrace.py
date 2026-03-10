#!/usr/bin/env python3
"""Capture an arbitrary binary under Apple xctrace and persist bounded artifacts."""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import signal
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

TEMPLATE_ALIASES = {
    "game-memory": "Game Memory",
    "time-profiler": "Time Profiler",
    "metal-system-trace": "Metal System Trace",
}

INSTRUMENT_ALIASES = {
    "vm-tracker": "VM Tracker",
    "virtual-memory-trace": "Virtual Memory Trace",
    "metal-application": "Metal Application",
    "metal-resource-events": "Metal Resource Events",
    "gpu": "GPU",
    "time-profiler": "Time Profiler",
}


@dataclass(frozen=True)
class ProcessRow:
    pid: int
    ppid: int
    command: str

    @property
    def name(self) -> str:
        stripped = self.command.strip()
        if not stripped:
            return ""
        try:
            argv0 = shlex.split(stripped)[0]
        except ValueError:
            argv0 = stripped.split()[0]
        return Path(argv0).name

    def to_json(self) -> dict[str, Any]:
        return {
            "pid": self.pid,
            "ppid": self.ppid,
            "command": self.command,
            "name": self.name,
        }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("binary", help="Path to the target binary")
    parser.add_argument("--out-dir", required=True, help="Directory that receives trace/log/summary artifacts")
    parser.add_argument(
        "--template",
        default="game-memory",
        help="Template alias or exact xctrace template name. Pass an empty string to rely on --instrument only.",
    )
    parser.add_argument(
        "--instrument",
        action="append",
        default=[],
        help="Instrument alias or exact xctrace instrument name. Repeatable.",
    )
    parser.add_argument(
        "--record-mode",
        choices=["attach", "launch"],
        default="attach",
        help="Whether xctrace attaches to a separately launched target or launches it directly.",
    )
    parser.add_argument("--time-limit", default="10s", help="xctrace record duration")
    parser.add_argument("--attach-delay-secs", type=float, default=2.0, help="Delay before attaching xctrace")
    parser.add_argument(
        "--launch-target-discovery-timeout-secs",
        type=float,
        default=20.0,
        help="How long launch mode spends discovering the spawned target pid under xctrace.",
    )
    parser.add_argument(
        "--shutdown-grace-secs",
        type=float,
        default=5.0,
        help="How long to wait after terminating the target before force kill",
    )
    parser.add_argument(
        "--finalize-timeout-secs",
        type=float,
        default=30.0,
        help="How long to wait for xctrace finalization after the target stops",
    )
    parser.add_argument(
        "--leave-target-running",
        action="store_true",
        help="Do not terminate the target when the trace window ends; only wait for xctrace finalize.",
    )
    parser.add_argument("--run-name", default="baseline", help="Run name used inside the trace")
    parser.add_argument("--trace-name", default="capture", help="Trace basename without extension")
    parser.add_argument(
        "--env",
        action="append",
        default=[],
        help="Environment override in KEY=VALUE form. Repeatable.",
    )
    parser.add_argument(
        "--export-toc",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Export `xctrace --toc` next to the trace when possible.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print/write the planned commands without launching xctrace.",
    )
    parser.add_argument(
        "--prompt",
        action="store_false",
        dest="no_prompt",
        help="Allow xctrace to show interactive prompts instead of forcing `--no-prompt`.",
    )
    parser.set_defaults(no_prompt=True)
    return parser.parse_args()


def parse_env_overrides(items: list[str]) -> dict[str, str]:
    out: dict[str, str] = {}
    for item in items:
        if "=" not in item:
            raise SystemExit(f"invalid --env override `{item}` (expected KEY=VALUE)")
        key, value = item.split("=", 1)
        key = key.strip()
        if not key:
            raise SystemExit(f"invalid --env override `{item}` (empty key)")
        out[key] = value
    return out


def resolve_template(raw: str | None) -> str | None:
    if raw is None:
        return None
    token = raw.strip()
    if not token:
        return None
    return TEMPLATE_ALIASES.get(token, token)


def resolve_instruments(items: list[str]) -> list[str]:
    resolved: list[str] = []
    for item in items:
        token = item.strip()
        if not token:
            continue
        resolved.append(INSTRUMENT_ALIASES.get(token, token))
    return resolved


def ensure_capture_profile(template_name: str | None, instruments: list[str]) -> None:
    if template_name is None and not instruments:
        raise SystemExit("capture requires either --template or --instrument")


def parse_duration_secs(raw: str) -> float:
    value = raw.strip().lower()
    if value.endswith("ms"):
        return float(value[:-2]) / 1000.0
    if value.endswith("s"):
        return float(value[:-1])
    if value.endswith("m"):
        return float(value[:-1]) * 60.0
    if value.endswith("h"):
        return float(value[:-1]) * 3600.0
    return float(value)


def ensure_binary(path: Path) -> None:
    if not path.is_file():
        raise SystemExit(f"binary not found: {path}")


def stop_process(process: subprocess.Popen[Any], grace_secs: float) -> dict[str, Any]:
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


def export_toc(trace_path: Path, toc_path: Path, no_prompt: bool) -> tuple[str | None, str | None]:
    command = [
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
        command.append("--quiet")
    try:
        subprocess.run(command, capture_output=True, text=True, check=True)
    except subprocess.CalledProcessError as exc:
        detail = exc.stderr.strip() or exc.stdout.strip() or repr(exc)
        return None, detail
    return str(toc_path), None


def parse_xctrace_attached_pid(stdout_path: Path) -> int | None:
    if not stdout_path.exists():
        return None
    match = re.search(r"Attaching to: .*? \((\d+)\)", stdout_path.read_text(errors="replace"))
    if match is None:
        return None
    return int(match.group(1))


def list_process_rows() -> list[ProcessRow]:
    proc = subprocess.run(
        ["ps", "-Ao", "pid=,ppid=,command="],
        capture_output=True,
        text=True,
        check=True,
    )
    rows: list[ProcessRow] = []
    for raw_line in proc.stdout.splitlines():
        parts = raw_line.strip().split(None, 2)
        if len(parts) != 3:
            continue
        pid_raw, ppid_raw, command = parts
        try:
            rows.append(ProcessRow(pid=int(pid_raw), ppid=int(ppid_raw), command=command))
        except ValueError:
            continue
    return rows


def write_process_snapshot(path: Path, label: str) -> dict[str, Any]:
    rows = [row.to_json() for row in list_process_rows()]
    payload = {
        "label": label,
        "captured_unix_ms": int(time.time() * 1000),
        "rows": rows,
    }
    path.write_text(json.dumps(payload, indent=2) + "\n")
    return payload


def build_pid_index(rows: list[dict[str, Any]]) -> dict[int, dict[str, Any]]:
    return {int(row["pid"]): row for row in rows}


def collect_descendant_pids(rows: list[dict[str, Any]], root_pid: int | None) -> list[int]:
    if root_pid is None:
        return []
    children: dict[int, list[int]] = {}
    for row in rows:
        children.setdefault(int(row["ppid"]), []).append(int(row["pid"]))
    ordered: list[int] = []
    stack = [root_pid]
    seen: set[int] = set()
    while stack:
        current = stack.pop()
        if current in seen:
            continue
        seen.add(current)
        ordered.append(current)
        for child_pid in reversed(children.get(current, [])):
            stack.append(child_pid)
    return ordered


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


def matching_binary_rows(rows: list[dict[str, Any]], binary: Path) -> list[dict[str, Any]]:
    matches = []
    for row in rows:
        command = str(row.get("command") or "")
        if command_matches_binary(command, binary):
            matches.append(row)
    return matches


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


def summarize_process_snapshot(
    snapshot: dict[str, Any],
    binary: Path,
    spawned_target_pid: int | None,
    xctrace_attached_pid: int | None,
) -> dict[str, Any]:
    rows = snapshot.get("rows") or []
    pid_index = build_pid_index(rows)
    descendant_pids = collect_descendant_pids(rows, spawned_target_pid)
    descendants = [pid_index[pid] for pid in descendant_pids if pid in pid_index and pid != spawned_target_pid]
    binary_matches = matching_binary_rows(rows, binary)
    return {
        "label": snapshot.get("label"),
        "captured_unix_ms": snapshot.get("captured_unix_ms"),
        "row_count": len(rows),
        "spawned_target_process": pid_index.get(spawned_target_pid) if spawned_target_pid is not None else None,
        "xctrace_attached_process": pid_index.get(xctrace_attached_pid) if xctrace_attached_pid is not None else None,
        "spawned_target_descendant_count": len(descendants),
        "spawned_target_descendants": descendants[:32],
        "binary_name_matches": binary_matches[:32],
    }


def dir_size_bytes(path: Path) -> int:
    if not path.exists():
        return 0
    if path.is_file():
        return path.stat().st_size
    total = 0
    for child in path.rglob("*"):
        if child.is_file():
            total += child.stat().st_size
    return total


def trace_top_level_entries(trace_path: Path) -> list[str]:
    if not trace_path.exists() or not trace_path.is_dir():
        return []
    return sorted(path.name for path in trace_path.iterdir())


def trace_complete_guess(trace_path: Path) -> bool:
    entries = trace_top_level_entries(trace_path)
    return any(entry != "Trace1.run" for entry in entries)


def profile_label(template_name: str | None, instruments: list[str]) -> str:
    if template_name and instruments:
        return f"{template_name} + {', '.join(instruments)}"
    if template_name:
        return template_name
    return "Blank + " + ", ".join(instruments)


def build_trace_command(
    *,
    template_name: str | None,
    instruments: list[str],
    time_limit: str,
    trace_path: Path,
    run_name: str,
    no_prompt: bool,
) -> list[str]:
    command = [
        "xcrun",
        "xctrace",
        "record",
    ]
    if template_name is not None:
        command.extend(["--template", template_name])
    for instrument in instruments:
        command.extend(["--instrument", instrument])
    command.extend(
        [
            "--time-limit",
            time_limit,
            "--output",
            str(trace_path),
            "--run-name",
            run_name,
        ]
    )
    if no_prompt:
        command.append("--no-prompt")
    return command


def render_summary_markdown(summary: dict[str, Any]) -> str:
    issues = summary.get("issues") or []
    issue_text = "<br>".join(issues) if issues else ""
    spawned_target_pid = summary.get("spawned_target_pid") or ""
    xctrace_attached_pid = summary.get("xctrace_attached_pid") or ""
    instruments = ", ".join(summary.get("instruments") or [])
    return "\n".join(
        [
            "| Profile | Mode | Status | Trace | Spawned PID | Attached PID | TOC | Instruments | Issues |",
            "| --- | --- | --- | --- | --- | --- | --- | --- | --- |",
            f"| {summary['profile']} | {summary['record_mode']} | {summary['status']} | {summary['trace']} | {spawned_target_pid} | {xctrace_attached_pid} | {summary.get('toc') or ''} | {instruments} | {issue_text} |",
            "",
        ]
    )


def record_attach(
    *,
    binary: Path,
    out_dir: Path,
    trace_path: Path,
    stdout_path: Path,
    stderr_path: Path,
    xctrace_stdout_path: Path,
    xctrace_stderr_path: Path,
    trace_command_base: list[str],
    env: dict[str, str],
    attach_delay_secs: float,
    shutdown_grace_secs: float,
    finalize_timeout_secs: float,
    leave_target_running: bool,
    dry_run: bool,
) -> tuple[dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    process_snapshots: list[dict[str, Any]] = []
    process_snapshot_artifacts: list[dict[str, Any]] = []
    issues: list[str] = []

    def capture_snapshot(label: str) -> None:
        path = out_dir / f"processes.{label}.json"
        try:
            snapshot = write_process_snapshot(path, label)
        except subprocess.CalledProcessError as exc:
            issues.append(f"process snapshot `{label}` failed: {exc}")
            return
        process_snapshots.append(snapshot)
        process_snapshot_artifacts.append(
            {
                "label": label,
                "path": str(path),
                "captured_unix_ms": snapshot["captured_unix_ms"],
                "row_count": len(snapshot["rows"]),
            }
        )

    launch_command = [str(binary)]
    trace_command = trace_command_base + ["--attach", "<pid>"]
    stop: dict[str, Any] = {"terminated": False, "killed": False, "returncode": None}
    trace_returncode: int | None = None
    target_pid: int | None = None

    if dry_run:
        capture_snapshot("before-launch")
        return (
            {
                "launch_command": launch_command,
                "trace_command": trace_command,
                "trace_returncode": None,
                "stop": stop,
                "issues": issues,
                "target_pid": None,
                "xctrace_attached_pid": None,
                "status": "dry-run",
            },
            process_snapshots,
            process_snapshot_artifacts,
        )

    with stdout_path.open("wb") as stdout_file, stderr_path.open("wb") as stderr_file, xctrace_stdout_path.open("wb") as xctrace_stdout_file, xctrace_stderr_path.open("wb") as xctrace_stderr_file:
        capture_snapshot("before-launch")
        process = subprocess.Popen(launch_command, stdout=stdout_file, stderr=stderr_file, env=env)
        target_pid = process.pid
        capture_snapshot("after-launch")
        trace_process = None
        try:
            if attach_delay_secs > 0:
                time.sleep(attach_delay_secs)
            capture_snapshot("before-attach")
            trace_command = trace_command_base + ["--attach", str(process.pid)]
            trace_process = subprocess.Popen(
                trace_command,
                stdout=xctrace_stdout_file,
                stderr=xctrace_stderr_file,
            )
            time.sleep(0.25)
            capture_snapshot("after-attach")
            wait_deadline = time.time() + parse_duration_secs(trace_command_base[trace_command_base.index("--time-limit") + 1]) + 1.0
            while time.time() < wait_deadline:
                if trace_process.poll() is not None:
                    break
                time.sleep(0.1)
            if not leave_target_running:
                stop = stop_process(process, shutdown_grace_secs)
            else:
                stop = {"terminated": False, "killed": False, "returncode": process.poll()}
            capture_snapshot("after-target-stop")
            try:
                trace_returncode = trace_process.wait(timeout=finalize_timeout_secs)
            except subprocess.TimeoutExpired:
                issues.append(f"xctrace finalize timed out after {finalize_timeout_secs:.1f}s")
        finally:
            if process.poll() is None and not leave_target_running:
                stop = stop_process(process, shutdown_grace_secs)
            if trace_process is not None and trace_process.poll() is None:
                trace_process.terminate()
                try:
                    trace_process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    trace_process.kill()
                    trace_process.wait(timeout=5)
            capture_snapshot("after-finalize")

    xctrace_attached_pid = parse_xctrace_attached_pid(xctrace_stdout_path)
    return (
        {
            "launch_command": launch_command,
            "trace_command": trace_command,
            "trace_returncode": trace_returncode,
            "stop": stop,
            "issues": issues,
            "target_pid": target_pid,
            "xctrace_attached_pid": xctrace_attached_pid,
            "status": "recorded",
        },
        process_snapshots,
        process_snapshot_artifacts,
    )


def record_launch(
    *,
    binary: Path,
    out_dir: Path,
    trace_path: Path,
    stdout_path: Path,
    stderr_path: Path,
    xctrace_stdout_path: Path,
    xctrace_stderr_path: Path,
    trace_command_base: list[str],
    env_overrides: dict[str, str],
    shutdown_grace_secs: float,
    finalize_timeout_secs: float,
    launch_target_discovery_timeout_secs: float,
    leave_target_running: bool,
    dry_run: bool,
) -> tuple[dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    process_snapshots: list[dict[str, Any]] = []
    process_snapshot_artifacts: list[dict[str, Any]] = []
    issues: list[str] = []

    def capture_snapshot(label: str) -> None:
        path = out_dir / f"processes.{label}.json"
        try:
            snapshot = write_process_snapshot(path, label)
        except subprocess.CalledProcessError as exc:
            issues.append(f"process snapshot `{label}` failed: {exc}")
            return
        process_snapshots.append(snapshot)
        process_snapshot_artifacts.append(
            {
                "label": label,
                "path": str(path),
                "captured_unix_ms": snapshot["captured_unix_ms"],
                "row_count": len(snapshot["rows"]),
            }
        )

    stdout_path.touch(exist_ok=True)
    stderr_path.touch(exist_ok=True)

    trace_command = list(trace_command_base)
    trace_command.extend(["--target-stdout", str(stdout_path)])
    for key, value in env_overrides.items():
        trace_command.extend(["--env", f"{key}={value}"])
    trace_command.extend(["--launch", "--", str(binary)])

    if dry_run:
        capture_snapshot("before-launch")
        return (
            {
                "launch_command": [str(binary)],
                "trace_command": trace_command,
                "trace_returncode": None,
                "stop": {
                    "pid": None,
                    "terminated": False,
                    "killed": False,
                    "returncode": None,
                    "found": False,
                },
                "issues": issues,
                "target_pid": None,
                "xctrace_attached_pid": None,
                "status": "dry-run",
            },
            process_snapshots,
            process_snapshot_artifacts,
        )

    trace_process = None
    target_pid: int | None = None
    stop: dict[str, Any] = {
        "pid": None,
        "terminated": False,
        "killed": False,
        "returncode": None,
        "found": False,
    }
    trace_returncode: int | None = None
    target_discovery_captured = False
    trace_duration_secs = parse_duration_secs(trace_command_base[trace_command_base.index("--time-limit") + 1])

    capture_snapshot("before-launch")
    with xctrace_stdout_path.open("wb") as xctrace_stdout_file, xctrace_stderr_path.open("wb") as xctrace_stderr_file:
        try:
            trace_process = subprocess.Popen(
                trace_command,
                stdout=xctrace_stdout_file,
                stderr=xctrace_stderr_file,
            )
            capture_snapshot("after-launch")
            target_discovery_deadline = time.time() + max(launch_target_discovery_timeout_secs, 0.5)
            while time.time() < target_discovery_deadline:
                if trace_process.poll() is not None:
                    break
                descendant = find_descendant_process(trace_process.pid, binary)
                if descendant is not None:
                    target_pid = descendant.pid
                    if not target_discovery_captured:
                        capture_snapshot("after-target-discovery")
                        target_discovery_captured = True
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
                        if not target_discovery_captured:
                            capture_snapshot("after-target-discovery")
                            target_discovery_captured = True
                time.sleep(0.1)

            if trace_process.poll() is None and target_pid is not None and not leave_target_running:
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
                issues.append("launch mode did not discover a target pid under xctrace")
            capture_snapshot("after-target-stop")

            try:
                trace_returncode = trace_process.wait(timeout=finalize_timeout_secs)
            except subprocess.TimeoutExpired:
                issues.append(f"xctrace finalize timed out after {finalize_timeout_secs:.1f}s")
        finally:
            if trace_process is not None and trace_process.poll() is None:
                trace_process.terminate()
                try:
                    trace_process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    trace_process.kill()
                    trace_process.wait(timeout=5)
            if target_pid is not None and pid_is_alive(target_pid) and not leave_target_running:
                stop = stop_pid(target_pid, shutdown_grace_secs)
            capture_snapshot("after-finalize")

    return (
        {
            "launch_command": [str(binary)],
            "trace_command": trace_command,
            "trace_returncode": trace_returncode,
            "stop": stop,
            "issues": issues,
            "target_pid": target_pid,
            "xctrace_attached_pid": None,
            "status": "recorded",
        },
        process_snapshots,
        process_snapshot_artifacts,
    )


def main() -> int:
    args = parse_args()
    binary = Path(args.binary)
    ensure_binary(binary)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    template_name = resolve_template(args.template)
    instruments = resolve_instruments(args.instrument)
    ensure_capture_profile(template_name, instruments)

    trace_path = out_dir / f"{args.trace_name}.trace"
    stdout_path = out_dir / f"{args.trace_name}.stdout.log"
    stderr_path = out_dir / f"{args.trace_name}.stderr.log"
    xctrace_stdout_path = out_dir / f"{args.trace_name}.xctrace.stdout.log"
    xctrace_stderr_path = out_dir / f"{args.trace_name}.xctrace.stderr.log"
    toc_path = out_dir / f"{args.trace_name}.toc.xml"
    summary_path = out_dir / "summary.json"
    summary_md_path = out_dir / "summary.md"

    env_overrides = parse_env_overrides(args.env)
    env = os.environ.copy()
    env.update(env_overrides)

    trace_command_base = build_trace_command(
        template_name=template_name,
        instruments=instruments,
        time_limit=args.time_limit,
        trace_path=trace_path,
        run_name=args.run_name,
        no_prompt=args.no_prompt,
    )

    if args.record_mode == "launch":
        run_result, process_snapshots, process_snapshot_artifacts = record_launch(
            binary=binary,
            out_dir=out_dir,
            trace_path=trace_path,
            stdout_path=stdout_path,
            stderr_path=stderr_path,
            xctrace_stdout_path=xctrace_stdout_path,
            xctrace_stderr_path=xctrace_stderr_path,
            trace_command_base=trace_command_base,
            env_overrides=env_overrides,
            shutdown_grace_secs=args.shutdown_grace_secs,
            finalize_timeout_secs=args.finalize_timeout_secs,
            launch_target_discovery_timeout_secs=args.launch_target_discovery_timeout_secs,
            leave_target_running=args.leave_target_running,
            dry_run=args.dry_run,
        )
    else:
        run_result, process_snapshots, process_snapshot_artifacts = record_attach(
            binary=binary,
            out_dir=out_dir,
            trace_path=trace_path,
            stdout_path=stdout_path,
            stderr_path=stderr_path,
            xctrace_stdout_path=xctrace_stdout_path,
            xctrace_stderr_path=xctrace_stderr_path,
            trace_command_base=trace_command_base,
            env=env,
            attach_delay_secs=args.attach_delay_secs,
            shutdown_grace_secs=args.shutdown_grace_secs,
            finalize_timeout_secs=args.finalize_timeout_secs,
            leave_target_running=args.leave_target_running,
            dry_run=args.dry_run,
        )

    issues = list(run_result["issues"])
    status = run_result["status"]
    trace_returncode = run_result["trace_returncode"]
    if trace_returncode not in {0, None}:
        issues.append(f"xctrace record exited with code {trace_returncode}")
        if status == "recorded":
            status = "recorded-with-issues" if trace_path.exists() else "failed"
    elif issues and trace_path.exists() and status == "recorded":
        status = "recorded-with-issues"

    toc_status = None
    toc_error = None
    if args.export_toc and trace_path.exists() and not args.dry_run:
        toc_status, toc_error = export_toc(trace_path, toc_path, args.no_prompt)
        if toc_error is not None:
            issues.append(f"toc export failed: {toc_error}")
            if status == "recorded":
                status = "recorded-with-issues"

    process_snapshot_summaries = [
        summarize_process_snapshot(
            snapshot,
            binary,
            run_result["target_pid"],
            run_result["xctrace_attached_pid"],
        )
        for snapshot in process_snapshots
    ]

    summary = {
        "binary": str(binary),
        "profile": profile_label(template_name, instruments),
        "record_mode": args.record_mode,
        "template": template_name,
        "instruments": instruments,
        "spawned_target_pid": run_result["target_pid"],
        "xctrace_attached_pid": run_result["xctrace_attached_pid"],
        "trace": str(trace_path),
        "stdout": str(stdout_path),
        "stderr": str(stderr_path),
        "xctrace_stdout": str(xctrace_stdout_path),
        "xctrace_stderr": str(xctrace_stderr_path),
        "toc": toc_status,
        "status": status,
        "issues": issues,
        "trace_returncode": trace_returncode,
        "stop": run_result["stop"],
        "attach_delay_secs": args.attach_delay_secs,
        "launch_target_discovery_timeout_secs": args.launch_target_discovery_timeout_secs,
        "time_limit": args.time_limit,
        "finalize_timeout_secs": args.finalize_timeout_secs,
        "leave_target_running": args.leave_target_running,
        "launch_env_overrides": env_overrides,
        "dry_run": args.dry_run,
        "launch_command": run_result["launch_command"],
        "trace_command": run_result["trace_command"],
        "trace_exists": trace_path.exists(),
        "trace_top_level_entries": trace_top_level_entries(trace_path),
        "trace_size_bytes": dir_size_bytes(trace_path),
        "trace_complete_guess": trace_complete_guess(trace_path),
        "trace_only_run_dir_guess": trace_top_level_entries(trace_path) == ["Trace1.run"],
        "process_snapshots": process_snapshot_artifacts,
        "process_snapshot_summaries": process_snapshot_summaries,
    }
    summary_path.write_text(json.dumps(summary, indent=2) + "\n")
    summary_md_path.write_text(render_summary_markdown(summary))
    print(json.dumps(summary, indent=2))
    return 0 if status in {"recorded", "recorded-with-issues", "dry-run"} else 1


if __name__ == "__main__":
    raise SystemExit(main())
