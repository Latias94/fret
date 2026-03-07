#!/usr/bin/env python3
"""Run a same-scene hello-world memory comparison between Fret and GPUI."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

GPUI_EXAMPLE_NAME = "fret_hello_world_compare"


def parse_env_kv(raw: str) -> tuple[str, str]:
    key, sep, value = raw.partition("=")
    if not sep:
        raise SystemExit(f"invalid env override `{raw}` (expected KEY=VALUE)")
    key = key.strip()
    value = value.strip()
    if not key:
        raise SystemExit(f"invalid env override `{raw}` (empty key)")
    return key, value


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", required=True)
    parser.add_argument("--repo-ref-zed", default="repo-ref/zed")
    parser.add_argument(
        "--gpui-template",
        default="tools/external-templates/gpui_hello_world_compare.rs",
    )
    parser.add_argument(
        "--external-sampler",
        default="tools/sample_external_process_memory.py",
    )
    parser.add_argument(
        "--summarizer",
        default="tools/summarize_fret_vs_gpui_hello_world_compare.py",
    )
    parser.add_argument(
        "--fret-binary",
        default="target/release/hello_world_compare_demo",
    )
    parser.add_argument(
        "--gpui-profile",
        choices=["debug", "release"],
        default="release",
    )
    parser.add_argument(
        "--sample-at-secs",
        default="1,2,6",
        help="Comma-separated offsets passed to both samplers.",
    )
    parser.add_argument("--steady-offset-secs", type=float, default=6.0)
    parser.add_argument("--post-sample-wait-secs", type=float, default=0.25)
    parser.add_argument(
        "--internal-report-wait-secs",
        type=float,
        default=2.0,
        help="Grace period after the external sampler exits before treating a missing internal report as fatal.",
    )
    parser.add_argument(
        "--exit-after-secs",
        type=float,
        help="Override the auto-computed demo self-exit time.",
    )
    parser.add_argument("--window-width", type=int, default=500)
    parser.add_argument("--window-height", type=int, default=500)
    parser.add_argument(
        "--active-mode",
        choices=["idle", "rerender-only", "paint-model", "layout-model"],
        default="rerender-only",
    )
    parser.add_argument(
        "--shared-env",
        action="append",
        default=[],
        help="Shared env override forwarded to all runs (KEY=VALUE).",
    )
    parser.add_argument(
        "--fret-env",
        action="append",
        default=[],
        help="Env override forwarded only to Fret runs (KEY=VALUE).",
    )
    parser.add_argument(
        "--gpui-env",
        action="append",
        default=[],
        help="Env override forwarded only to GPUI runs (KEY=VALUE).",
    )
    parser.add_argument(
        "--capture-vmmap-regions",
        action="store_true",
        help="Also capture `vmmap -sortBySize -wide -interleaved -noCoalesce` artifacts for each sample.",
    )
    parser.add_argument(
        "--capture-footprint-verbose",
        action="store_true",
        help="Also capture `footprint -v` artifacts for each sample.",
    )
    return parser.parse_args()


def parse_sample_offsets(raw: str) -> list[float]:
    out: list[float] = []
    for piece in raw.split(","):
        piece = piece.strip()
        if not piece:
            continue
        value = float(piece)
        if value < 0.0:
            raise SystemExit(f"sample offset must be >= 0, got {value}")
        out.append(value)
    if not out:
        raise SystemExit("no sample offsets configured")
    out.sort()
    return out


def capture_exit_grace_secs(
    capture_vmmap_regions: bool,
    capture_footprint_verbose: bool,
) -> float:
    if capture_vmmap_regions:
        return 3.0
    if capture_footprint_verbose:
        return 2.0
    return 1.0


def auto_exit_after_secs(
    sample_offsets: list[float],
    post_sample_wait_secs: float,
    *,
    capture_vmmap_regions: bool,
    capture_footprint_verbose: bool,
) -> float:
    return (
        max(sample_offsets)
        + max(post_sample_wait_secs, 0.0)
        + capture_exit_grace_secs(capture_vmmap_regions, capture_footprint_verbose)
    )


def wait_for_file(path: Path, timeout_secs: float) -> bool:
    deadline = time.monotonic() + max(timeout_secs, 0.0)
    while True:
        if path.is_file():
            return True
        if time.monotonic() >= deadline:
            return False
        time.sleep(0.05)


def materialize_gpui_example(repo_ref_zed: Path, template_path: Path) -> Path:
    if not repo_ref_zed.is_dir():
        raise FileNotFoundError(f"missing repo-ref zed checkout: {repo_ref_zed}")
    if not template_path.is_file():
        raise FileNotFoundError(f"missing GPUI template: {template_path}")
    example_path = repo_ref_zed / "crates" / "gpui" / "examples" / f"{GPUI_EXAMPLE_NAME}.rs"
    example_path.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(template_path, example_path)
    return example_path


def build_gpui_example(repo_ref_zed: Path, profile: str) -> Path:
    command = [
        "cargo",
        "build",
        "-p",
        "gpui",
        "--example",
        GPUI_EXAMPLE_NAME,
        "--features",
        "runtime_shaders",
    ]
    if profile == "release":
        command.append("--release")
    subprocess.run(command, check=True, cwd=repo_ref_zed)
    target_dir = repo_ref_zed / "target" / profile / "examples"
    binary_path = target_dir / GPUI_EXAMPLE_NAME
    if not binary_path.is_file():
        raise FileNotFoundError(f"missing built GPUI example: {binary_path}")
    return binary_path


def run_external_sample(
    *,
    out_dir: Path,
    label: str,
    binary: str,
    env_overrides: dict[str, str],
    external_sampler: str,
    sample_at_secs: str,
    post_sample_wait_secs: float,
    internal_report_wait_secs: float,
    capture_vmmap_regions: bool,
    capture_footprint_verbose: bool,
) -> tuple[Path, Path]:
    out_dir.mkdir(parents=True, exist_ok=True)
    external_summary_path = out_dir / "summary.json"
    internal_report_path = out_dir / "internal.gpu.json"
    env = os.environ.copy()
    env.update(env_overrides)
    command = [
        sys.executable,
        external_sampler,
        "--out-dir",
        str(out_dir),
        "--label",
        label,
        "--sample-at-secs",
        sample_at_secs,
        "--post-sample-wait-secs",
        str(post_sample_wait_secs),
    ]
    if capture_vmmap_regions:
        command.append("--capture-vmmap-regions")
    if capture_footprint_verbose:
        command.append("--capture-footprint-verbose")
    command.extend([
        "--",
        binary,
    ])
    subprocess.run(command, check=True, env=env)
    if not external_summary_path.is_file():
        raise FileNotFoundError(f"missing external summary: {external_summary_path}")
    if not wait_for_file(internal_report_path, internal_report_wait_secs):
        raise FileNotFoundError(f"missing internal report: {internal_report_path}")
    return external_summary_path, internal_report_path


def git_head(repo_dir: Path) -> str | None:
    try:
        return (
            subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo_dir, text=True)
            .strip()
        )
    except Exception:
        return None


def main() -> int:
    args = parse_args()
    sample_offsets = parse_sample_offsets(args.sample_at_secs)
    exit_after_secs = args.exit_after_secs or auto_exit_after_secs(
        sample_offsets,
        args.post_sample_wait_secs,
        capture_vmmap_regions=args.capture_vmmap_regions,
        capture_footprint_verbose=args.capture_footprint_verbose,
    )
    shared_env = dict(parse_env_kv(raw) for raw in args.shared_env)
    fret_env = dict(parse_env_kv(raw) for raw in args.fret_env)
    gpui_env = dict(parse_env_kv(raw) for raw in args.gpui_env)

    repo_ref_zed = Path(args.repo_ref_zed)
    gpui_template = Path(args.gpui_template)
    materialized_example = materialize_gpui_example(repo_ref_zed, gpui_template)
    gpui_binary = build_gpui_example(repo_ref_zed, args.gpui_profile)

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    runs_dir = out_dir / "runs"
    runs_dir.mkdir(parents=True, exist_ok=True)

    gpui_full_external, gpui_full_internal = run_external_sample(
        out_dir=runs_dir / "gpui-compare-full",
        label=f"gpui compare full ({args.active_mode})",
        binary=str(gpui_binary),
        external_sampler=args.external_sampler,
        sample_at_secs=args.sample_at_secs,
        post_sample_wait_secs=args.post_sample_wait_secs,
        internal_report_wait_secs=args.internal_report_wait_secs,
        capture_vmmap_regions=args.capture_vmmap_regions,
        capture_footprint_verbose=args.capture_footprint_verbose,
        env_overrides={
            **shared_env,
            **gpui_env,
            "GPUI_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH": str(
                runs_dir / "gpui-compare-full" / "internal.gpu.json"
            ),
            "GPUI_HELLO_WORLD_COMPARE_INTERNAL_SAMPLE_AT_SECS": args.sample_at_secs,
            "GPUI_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS": f"{exit_after_secs:.3f}",
            "GPUI_HELLO_WORLD_COMPARE_WINDOW_WIDTH": str(args.window_width),
            "GPUI_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": str(args.window_height),
            "GPUI_HELLO_WORLD_COMPARE_ACTIVE_MODE": args.active_mode,
        },
    )
    gpui_empty_external, gpui_empty_internal = run_external_sample(
        out_dir=runs_dir / "gpui-compare-empty",
        label=f"gpui compare empty ({args.active_mode})",
        binary=str(gpui_binary),
        external_sampler=args.external_sampler,
        sample_at_secs=args.sample_at_secs,
        post_sample_wait_secs=args.post_sample_wait_secs,
        internal_report_wait_secs=args.internal_report_wait_secs,
        capture_vmmap_regions=args.capture_vmmap_regions,
        capture_footprint_verbose=args.capture_footprint_verbose,
        env_overrides={
            **shared_env,
            **gpui_env,
            "GPUI_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH": str(
                runs_dir / "gpui-compare-empty" / "internal.gpu.json"
            ),
            "GPUI_HELLO_WORLD_COMPARE_INTERNAL_SAMPLE_AT_SECS": args.sample_at_secs,
            "GPUI_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS": f"{exit_after_secs:.3f}",
            "GPUI_HELLO_WORLD_COMPARE_WINDOW_WIDTH": str(args.window_width),
            "GPUI_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": str(args.window_height),
            "GPUI_HELLO_WORLD_COMPARE_ACTIVE_MODE": args.active_mode,
            "GPUI_HELLO_WORLD_COMPARE_NO_TEXT": "1",
            "GPUI_HELLO_WORLD_COMPARE_NO_SWATCHES": "1",
        },
    )
    fret_full_external, fret_full_internal = run_external_sample(
        out_dir=runs_dir / "fret-compare-full",
        label=f"fret compare full ({args.active_mode})",
        binary=args.fret_binary,
        external_sampler=args.external_sampler,
        sample_at_secs=args.sample_at_secs,
        post_sample_wait_secs=args.post_sample_wait_secs,
        internal_report_wait_secs=args.internal_report_wait_secs,
        capture_vmmap_regions=args.capture_vmmap_regions,
        capture_footprint_verbose=args.capture_footprint_verbose,
        env_overrides={
            **shared_env,
            **fret_env,
            "FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH": str(
                runs_dir / "fret-compare-full" / "internal.gpu.json"
            ),
            "FRET_HELLO_WORLD_COMPARE_INTERNAL_SAMPLE_AT_SECS": args.sample_at_secs,
            "FRET_DIAG_RENDERER_PERF": "1",
            "FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS": f"{exit_after_secs:.3f}",
            "FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH": str(args.window_width),
            "FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": str(args.window_height),
            "FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE": args.active_mode,
        },
    )
    fret_empty_external, fret_empty_internal = run_external_sample(
        out_dir=runs_dir / "fret-compare-empty",
        label=f"fret compare empty ({args.active_mode})",
        binary=args.fret_binary,
        external_sampler=args.external_sampler,
        sample_at_secs=args.sample_at_secs,
        post_sample_wait_secs=args.post_sample_wait_secs,
        internal_report_wait_secs=args.internal_report_wait_secs,
        capture_vmmap_regions=args.capture_vmmap_regions,
        capture_footprint_verbose=args.capture_footprint_verbose,
        env_overrides={
            **shared_env,
            **fret_env,
            "FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH": str(
                runs_dir / "fret-compare-empty" / "internal.gpu.json"
            ),
            "FRET_HELLO_WORLD_COMPARE_INTERNAL_SAMPLE_AT_SECS": args.sample_at_secs,
            "FRET_DIAG_RENDERER_PERF": "1",
            "FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS": f"{exit_after_secs:.3f}",
            "FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH": str(args.window_width),
            "FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": str(args.window_height),
            "FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE": args.active_mode,
            "FRET_HELLO_WORLD_COMPARE_NO_TEXT": "1",
            "FRET_HELLO_WORLD_COMPARE_NO_SWATCHES": "1",
        },
    )

    summary_dir = out_dir / "summary"
    summary_dir.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            sys.executable,
            args.summarizer,
            "--gpui-full-external",
            str(gpui_full_external),
            "--gpui-full-internal",
            str(gpui_full_internal),
            "--gpui-empty-external",
            str(gpui_empty_external),
            "--gpui-empty-internal",
            str(gpui_empty_internal),
            "--fret-full-external",
            str(fret_full_external),
            "--fret-full-internal",
            str(fret_full_internal),
            "--fret-empty-external",
            str(fret_empty_external),
            "--fret-empty-internal",
            str(fret_empty_internal),
            "--steady-offset-secs",
            str(args.steady_offset_secs),
            "--out-dir",
            str(summary_dir),
        ],
        check=True,
    )

    manifest = {
        "schema_version": 1,
        "kind": "run_fret_vs_gpui_hello_world_compare",
        "active_mode": args.active_mode,
        "sample_at_secs": args.sample_at_secs,
        "steady_offset_secs": args.steady_offset_secs,
        "post_sample_wait_secs": args.post_sample_wait_secs,
        "capture_vmmap_regions": args.capture_vmmap_regions,
        "capture_footprint_verbose": args.capture_footprint_verbose,
        "exit_after_secs": exit_after_secs,
        "window": {
            "width_px": args.window_width,
            "height_px": args.window_height,
        },
        "shared_env": shared_env,
        "fret_env": fret_env,
        "gpui_env": gpui_env,
        "repo_ref": {
            "zed_dir": str(repo_ref_zed),
            "zed_head": git_head(repo_ref_zed),
            "materialized_example": str(materialized_example),
            "gpui_profile": args.gpui_profile,
            "gpui_binary": str(gpui_binary),
        },
        "artifacts": {
            "gpui_full_external": str(gpui_full_external),
            "gpui_full_internal": str(gpui_full_internal),
            "gpui_empty_external": str(gpui_empty_external),
            "gpui_empty_internal": str(gpui_empty_internal),
            "fret_full_external": str(fret_full_external),
            "fret_full_internal": str(fret_full_internal),
            "fret_empty_external": str(fret_empty_external),
            "fret_empty_internal": str(fret_empty_internal),
            "summary_json": str(summary_dir / "summary.json"),
            "summary_md": str(summary_dir / "summary.md"),
        },
    }
    (out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2))
    print(json.dumps(manifest, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
