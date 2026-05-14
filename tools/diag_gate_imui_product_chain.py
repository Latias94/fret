#!/usr/bin/env python3
"""Validate the current IMUI editor-grade product chain as one maintainer gate.

The lightweight default mode checks discoverability, promoted script/suite inputs, and source
guards. Use `--launched` when a local machine should also execute the existing launched diagnostics
proofs across the cookbook, editor proof, and workspace shell apps.
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
WORKSPACE_SHELL = "workspace-shell"
SOURCE_GATES = "source-gates"

ALL_GATES = [
    DISCOVERY,
    GENERIC_ACTION,
    EDITOR_CONTROLS,
    EDITOR_PROOF,
    WORKSPACE_SHELL,
    SOURCE_GATES,
]


@dataclass(frozen=True)
class ProductSurface:
    name: str
    suite: str | None = None
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
        name=WORKSPACE_SHELL,
        suite="tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json",
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


def _validate_discovery(repo_root: Path, fretboard_exe: Path) -> None:
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
    _assert_contains(native.stdout, "workspace_shell_demo", "list native demos")
    _assert_contains(native.stdout, "docking_arbitration_demo", "list native demos")

    tool_apps = _run_capture_checked(
        "list tool apps",
        [str(fretboard_exe), "list", "tool-apps"],
        cwd=repo_root,
    )
    _assert_contains(tool_apps.stdout, "docs/diagnostics-first-open.md", "list tool apps")
    _assert_contains(tool_apps.stdout, "diag doctor campaigns", "list tool apps")
    _assert_contains(tool_apps.stdout, "fret-devtools", "list tool apps")

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


def _validate_product_surface(repo_root: Path, fretboard_exe: Path, surface: ProductSurface) -> None:
    scripts = list(surface.scripts)
    if surface.suite is not None:
        scripts.extend(_suite_scripts(repo_root, surface.suite))
    for script in scripts:
        _validate_script(repo_root, fretboard_exe, script)


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


def _run_launched_gates(
    repo_root: Path,
    *,
    out_root: Path,
    timeout_ms: int,
    poll_ms: int,
    release: bool,
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
            *_cargo_run_demo_command(
                "fret-demo",
                bin_name="imui_editor_proof_demo",
                release=release,
            ),
        ]
        _run_checked("launched editor proof suite", cmd, cwd=repo_root)

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
            *_cargo_run_demo_command(
                "fret-demo",
                bin_name="workspace_shell_demo",
                release=release,
            ),
        ]
        _run_checked("launched workspace shell suite", cmd, cwd=repo_root)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", default="target/imui-product-chain")
    parser.add_argument("--timeout-ms", type=int, default=240000)
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument("--release", action="store_true")
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
            selected=selected,
        )

    print("[diag-gate-imui-product-chain] done", flush=True)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
