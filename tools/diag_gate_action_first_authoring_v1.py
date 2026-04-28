#!/usr/bin/env python3
"""Run a small, CI-friendly diagnostics gate set for Action-first authoring v1."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _exe_name(stem: str) -> str:
    return f"{stem}.exe" if os.name == "nt" else stem


def _run_checked(name: str, argv: list[str], *, cwd: Path) -> None:
    print(f"[diag-gate-afa-v1] {name}")
    proc = subprocess.run(argv, cwd=str(cwd), check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def _stream_checked_with_timeout_retry(
    *,
    gate_name: str,
    argv_builder,
    cwd: Path,
    timeout_ms_primary: int,
    timeout_ms_retry: int,
    retry_count: int,
) -> None:
    attempt = 0
    tail_limit = 200

    while True:
        attempt += 1
        timeout_for_attempt = timeout_ms_primary if attempt == 1 else timeout_ms_retry
        if attempt == 1:
            print(f"[diag-gate-afa-v1] fretboard-dev diag run {gate_name}")
        else:
            print(
                "[diag-gate-afa-v1] fretboard-dev diag run "
                f"{gate_name} (retry {attempt}, timeout_ms={timeout_for_attempt})"
            )

        tail: list[str] = []
        argv = argv_builder(timeout_for_attempt)
        proc = subprocess.Popen(
            argv,
            cwd=str(cwd),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
        )

        assert proc.stdout is not None
        for line in proc.stdout:
            print(line, end="")
            tail.append(line.rstrip("\n"))
            if len(tail) > tail_limit:
                tail.pop(0)

        proc.wait()
        if proc.returncode == 0:
            return

        tail_text = "\n".join(tail)
        timed_out = "timeout waiting for script result" in tail_text
        can_retry = (
            retry_count > 0
            and attempt <= (1 + retry_count)
            and timeout_ms_retry > timeout_ms_primary
        )
        if timed_out and can_retry:
            continue

        raise SystemExit(
            f"Step failed: fretboard-dev diag run {gate_name} (exit code: {proc.returncode})"
        )


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", default="target/dfa-v1")
    parser.add_argument("--timeout-ms", type=int, default=180000)
    parser.add_argument("--timeout-ms-retry", type=int, default=600000)
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument("--timeout-retry-count", type=int, default=1)
    parser.add_argument("--release", action="store_true")
    parser.add_argument(
        "--only",
        action="append",
        default=[],
        help="Run only the named gate. Can be repeated or comma-separated.",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    run_id = str(int(time.time() * 1000))
    run_out_dir = repo_root / args.out_dir / run_id
    profile_dir = "release" if args.release else "debug"

    fretboard_build = ["cargo", "build", "-j", "1", "-p", "fretboard"]
    if args.release:
        fretboard_build.append("--release")
    _run_checked("cargo build -p fretboard-dev", fretboard_build, cwd=repo_root)

    fretboard_exe = repo_root / "target" / profile_dir / _exe_name("fretboard")
    if not fretboard_exe.exists():
        raise SystemExit(f"fretboard exe not found: {fretboard_exe}")

    gates: list[dict[str, object]] = [
        {
            "name": "cookbook-hello-click-count",
            "dir_name": "h",
            "script_path": "tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json",
            "example_name": "hello",
            "cookbook_features": ["cookbook-diag"],
        },
        {
            "name": "cookbook-commands-keymap-basics-shortcut-and-gating",
            "dir_name": "k",
            "script_path": "tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json",
            "example_name": "commands_keymap_basics",
            "cookbook_features": ["cookbook-diag"],
        },
        {
            "name": "cookbook-overlay-basics-modal-barrier-shortcut-gating",
            "dir_name": "o",
            "script_path": "tools/diag-scripts/cookbook/overlay-basics/cookbook-overlay-basics-modal-barrier-shortcut-gating.json",
            "example_name": "overlay_basics",
            "cookbook_features": ["cookbook-diag"],
        },
        {
            "name": "cookbook-imui-action-basics-cross-frontend",
            "dir_name": "i",
            "script_path": "tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json",
            "example_name": "imui_action_basics",
            "cookbook_features": ["cookbook-diag", "cookbook-imui"],
        },
        {
            "name": "cookbook-payload-actions-basics-remove",
            "dir_name": "p",
            "script_path": "tools/diag-scripts/cookbook/payload-actions-basics/cookbook-payload-actions-basics-remove.json",
            "example_name": "payload_actions_basics",
            "cookbook_features": ["cookbook-diag"],
        },
        {
            "name": "workspace-shell-demo-tab-close-button-command-dispatch-trace",
            "dir_name": "w",
            "script_path": "tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json",
            "package_name": "fret-demo",
            "bin_name": "workspace_shell_demo",
        },
    ]
    only: set[str] = {
        name.strip()
        for raw in args.only
        for name in raw.split(",")
        if name.strip()
    }
    if only:
        known = {str(gate["name"]) for gate in gates}
        unknown = sorted(only - known)
        if unknown:
            raise SystemExit(
                "Unknown --only gate(s): "
                + ", ".join(unknown)
                + "\nKnown gates: "
                + ", ".join(sorted(known))
            )
        gates = [gate for gate in gates if str(gate["name"]) in only]

    for gate in gates:
        gate_name = str(gate["name"])
        script_path = str(gate["script_path"])
        demo_exe: Path

        if "example_name" in gate:
            example_name = str(gate["example_name"])
            build_args = [
                "cargo",
                "build",
                "-j",
                "1",
                "-p",
                "fret-cookbook",
                "--example",
                example_name,
            ]
            features = gate.get("cookbook_features")
            if features:
                build_args += ["--features", ",".join(features)]  # type: ignore[arg-type]
            if args.release:
                build_args.append("--release")
            _run_checked(
                f"cargo build -p fret-cookbook --example {example_name}",
                build_args,
                cwd=repo_root,
            )
            demo_exe = repo_root / "target" / profile_dir / "examples" / _exe_name(example_name)
            if not demo_exe.exists():
                raise SystemExit(f"cookbook example exe not found: {demo_exe}")
        else:
            package_name = str(gate["package_name"])
            bin_name = str(gate["bin_name"])
            build_args = [
                "cargo",
                "build",
                "-j",
                "1",
                "-p",
                package_name,
                "--bin",
                bin_name,
            ]
            if args.release:
                build_args.append("--release")
            _run_checked(
                f"cargo build -p {package_name} --bin {bin_name}",
                build_args,
                cwd=repo_root,
            )
            demo_exe = repo_root / "target" / profile_dir / _exe_name(bin_name)
            if not demo_exe.exists():
                raise SystemExit(f"demo exe not found: {demo_exe}")

        script_out_dir = run_out_dir / str(gate["dir_name"])
        _stream_checked_with_timeout_retry(
            gate_name=gate_name,
            argv_builder=lambda timeout_ms: [
                str(fretboard_exe),
                "diag",
                "run",
                script_path,
                "--dir",
                str(script_out_dir),
                "--timeout-ms",
                str(timeout_ms),
                "--poll-ms",
                str(args.poll_ms),
                "--pack",
                "--env",
                "FRET_DIAG_SEMANTICS=1",
                "--env",
                "FRET_DIAG_REDACT_TEXT=0",
                "--env",
                "FRET_DIAG_FIXED_FRAME_DELTA_MS=16",
                "--env",
                "RUST_LOG=warn",
                "--launch",
                "--",
                str(demo_exe),
            ],
            cwd=repo_root,
            timeout_ms_primary=args.timeout_ms,
            timeout_ms_retry=args.timeout_ms_retry,
            retry_count=args.timeout_retry_count,
        )

    print(f"[diag-gate-afa-v1] done (out_dir={run_out_dir})")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
