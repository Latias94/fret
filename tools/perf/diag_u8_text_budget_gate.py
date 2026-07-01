#!/usr/bin/env python3
"""
U8 text budget gate wrapper.

This script keeps the U8 text/cache/glyph/upload contract thin:

- Native memory probes reuse `fretboard-dev diag repeat --check-memory-p90-max`.
- Web/wasm evidence is validated from an exported raw `bundle.json`.

It intentionally does not extend the Rust perf baseline schema. Promote calibrated hard thresholds
there only after target-machine evidence proves which fields should be stable contracts.
"""

from __future__ import annotations

import argparse
import json
import shlex
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any


MIB = 1024 * 1024

TEXT_HEAVY_SCRIPT = "tools/diag-scripts/tooling/text/text-heavy-memory-steady.json"
CODE_EDITOR_SCRIPT = "tools/diag-scripts/ui-gallery/memory/ui-gallery-code-editor-torture-memory-steady.json"

TEXT_HEAVY_LAUNCH_CMD = "cargo run -p fret-demo --release --bin text_heavy_memory_demo"
CODE_EDITOR_LAUNCH_CMD = "cargo run -p fret-ui-gallery --release --features gallery-full"

COMMON_NATIVE_ENVS = [
    "FRET_DIAG_SCRIPT_AUTO_DUMP=0",
    "FRET_DIAG_SEMANTICS=0",
]

CODE_EDITOR_ENVS = [
    "FRET_A11Y_DISABLE=1",
    "FRET_UI_GALLERY_BOOTSTRAP_FONTS=1",
    "FRET_UI_GALLERY_VIEW_CACHE=1",
    "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1",
    "FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0",
]

WEB_REQUIRED_RENDERER_METRICS = [
    "renderer_prepare_text_us",
    "renderer_text_atlas_upload_bytes",
    "renderer_text_atlas_evicted_pages",
    "renderer_geometry_upload_text_glyph_instance_bytes",
    "renderer_geometry_upload_text_glyph_instance_write_count",
    "renderer_geometry_upload_text_vertex_bytes",
    "renderer_geometry_upload_text_vertex_write_count",
    "renderer_encode_scene_text_ops",
]


@dataclass(frozen=True)
class NativeProbe:
    name: str
    script: str
    launch_cmd: str
    thresholds: list[tuple[str, int]]
    envs: list[str]


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _resolve_workspace_path(workspace_root: Path, value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return workspace_root / path


def _read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def _split_cmd(value: str, *, field: str) -> list[str]:
    try:
        cmd = shlex.split(value)
    except ValueError as exc:
        raise ValueError(f"invalid {field}: {exc}") from exc
    if not cmd:
        raise ValueError(f"{field} must not be empty")
    return cmd


def _is_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def _u64_from_dict(values: dict[str, Any], key: str) -> int | None:
    value = values.get(key)
    if _is_int(value) and value >= 0:
        return int(value)
    return None


def _sat_mul(a: int, b: int) -> int:
    return min(a * b, (1 << 64) - 1)


def _atlas_bytes(atlas: dict[str, Any], bytes_per_pixel: int, pages_key: str) -> int | None:
    width = _u64_from_dict(atlas, "width")
    height = _u64_from_dict(atlas, "height")
    pages = _u64_from_dict(atlas, pages_key)
    if width is None or height is None or pages is None:
        return None
    return _sat_mul(_sat_mul(_sat_mul(width, height), pages), bytes_per_pixel)


def _last_snapshot(bundle: object) -> dict[str, Any] | None:
    if not isinstance(bundle, dict):
        return None
    windows = bundle.get("windows")
    if not isinstance(windows, list) or not windows:
        return None
    first_window = windows[0]
    if not isinstance(first_window, dict):
        return None
    snapshots = first_window.get("snapshots")
    if not isinstance(snapshots, list) or not snapshots:
        return None
    snapshot = snapshots[-1]
    return snapshot if isinstance(snapshot, dict) else None


def _last_render_text(snapshot: dict[str, Any] | None) -> dict[str, Any]:
    if not isinstance(snapshot, dict):
        return {}
    resource_caches = snapshot.get("resource_caches")
    if not isinstance(resource_caches, dict):
        return {}
    render_text = resource_caches.get("render_text")
    return render_text if isinstance(render_text, dict) else {}


def _last_debug_stats(snapshot: dict[str, Any] | None) -> dict[str, Any]:
    if not isinstance(snapshot, dict):
        return {}
    debug = snapshot.get("debug")
    if not isinstance(debug, dict):
        return {}
    stats = debug.get("stats")
    return stats if isinstance(stats, dict) else {}


def _render_text_budget_observed(render_text: dict[str, Any]) -> dict[str, Any]:
    observed: dict[str, Any] = {
        "render_text_present": bool(render_text),
        "render_text_shape_cache_entries": _u64_from_dict(render_text, "shape_cache_entries"),
        "render_text_shape_cache_entry_limit": _u64_from_dict(render_text, "shape_cache_entry_limit"),
        "render_text_shape_cache_bytes_estimate_total": _u64_from_dict(
            render_text,
            "shape_cache_bytes_estimate_total",
        ),
        "render_text_frame_shape_cache_evictions": _u64_from_dict(
            render_text,
            "frame_shape_cache_evictions",
        ),
    }

    atlas_specs = [
        ("mask", "mask_atlas", 1),
        ("color", "color_atlas", 4),
        ("subpixel", "subpixel_atlas", 4),
    ]
    live_total = 0
    budget_total = 0
    live_complete = True
    budget_complete = True
    for prefix, key, bytes_per_pixel in atlas_specs:
        atlas = render_text.get(key)
        atlas = atlas if isinstance(atlas, dict) else {}
        pages = _u64_from_dict(atlas, "pages")
        max_pages = _u64_from_dict(atlas, "max_pages")
        live_bytes = _atlas_bytes(atlas, bytes_per_pixel, "pages")
        budget_bytes = _atlas_bytes(atlas, bytes_per_pixel, "max_pages")
        observed[f"render_text_{prefix}_atlas_pages"] = pages
        observed[f"render_text_{prefix}_atlas_max_pages"] = max_pages
        observed[f"render_text_{prefix}_atlas_bytes_live_estimate"] = live_bytes
        observed[f"render_text_{prefix}_atlas_bytes_budget_estimate"] = budget_bytes
        if live_bytes is None:
            live_complete = False
        else:
            live_total += live_bytes
        if budget_bytes is None:
            budget_complete = False
        else:
            budget_total += budget_bytes

    observed["render_text_atlas_bytes_live_estimate_total"] = live_total if live_complete else None
    observed["render_text_atlas_bytes_budget_estimate_total"] = budget_total if budget_complete else None
    return observed


def native_probes(args: argparse.Namespace) -> list[NativeProbe]:
    native_atlas_pages = int(args.native_atlas_max_pages)
    probes: list[NativeProbe] = []

    if not bool(args.skip_text_heavy):
        probes.append(
            NativeProbe(
                name="text-heavy",
                script=str(args.text_heavy_script),
                launch_cmd=str(args.text_heavy_launch_cmd),
                thresholds=[
                    (
                        "render_text_atlas_bytes_live_estimate_total",
                        int(args.text_heavy_max_atlas_bytes),
                    ),
                    (
                        "render_text_shape_cache_entries",
                        int(args.text_heavy_max_shape_cache_entries),
                    ),
                    (
                        "render_text_shape_cache_bytes_estimate_total",
                        int(args.text_heavy_max_shape_cache_bytes),
                    ),
                    ("render_text_mask_atlas_max_pages", native_atlas_pages),
                    ("render_text_color_atlas_max_pages", native_atlas_pages),
                    ("render_text_subpixel_atlas_max_pages", native_atlas_pages),
                ],
                envs=[*COMMON_NATIVE_ENVS],
            )
        )

    if not bool(args.skip_code_editor):
        probes.append(
            NativeProbe(
                name="code-editor",
                script=str(args.code_editor_script),
                launch_cmd=str(args.code_editor_launch_cmd),
                thresholds=[
                    (
                        "render_text_atlas_bytes_live_estimate_total",
                        int(args.code_editor_max_atlas_bytes),
                    ),
                    (
                        "render_text_shape_cache_entries",
                        int(args.code_editor_max_shape_cache_entries),
                    ),
                    (
                        "render_text_shape_cache_bytes_estimate_total",
                        int(args.code_editor_max_shape_cache_bytes),
                    ),
                    ("render_text_mask_atlas_max_pages", native_atlas_pages),
                    ("render_text_color_atlas_max_pages", native_atlas_pages),
                    ("render_text_subpixel_atlas_max_pages", native_atlas_pages),
                ],
                envs=[*COMMON_NATIVE_ENVS, *CODE_EDITOR_ENVS],
            )
        )

    return probes


def build_repeat_cmd(
    *,
    fretboard_cmd: str,
    probe: NativeProbe,
    out_dir: Path,
    repeat: int,
    timeout_ms: int,
) -> list[str]:
    cmd = [
        *_split_cmd(fretboard_cmd, field="--fretboard-cmd"),
        "diag",
        "repeat",
        probe.script,
        "--dir",
        str(out_dir),
        "--repeat",
        str(int(repeat)),
        "--timeout-ms",
        str(int(timeout_ms)),
        "--no-compare",
    ]
    for key, threshold in probe.thresholds:
        cmd.extend(["--check-memory-p90-max", f"{key}:{int(threshold)}"])
    for env in probe.envs:
        cmd.extend(["--env", env])
    cmd.extend(["--launch", "--", *_split_cmd(probe.launch_cmd, field=f"{probe.name} launch command")])
    return cmd


def _memory_check_failures(check_path: Path) -> int | None:
    if not check_path.is_file():
        return None
    try:
        doc = _read_json(check_path)
    except Exception:
        return None
    if not isinstance(doc, dict):
        return None
    failures = doc.get("failures")
    return len(failures) if isinstance(failures, list) else None


def run_native_probe(workspace_root: Path, cmd: list[str], out_dir: Path, *, dry_run: bool) -> dict[str, Any]:
    stdout_path = out_dir / "stdout.log"
    stderr_path = out_dir / "stderr.log"
    check_path = out_dir / "check.repeat_memory_p90_max.json"
    report: dict[str, Any] = {
        "out_dir": str(out_dir),
        "cmd": cmd,
        "dry_run": dry_run,
        "stdout": str(stdout_path),
        "stderr": str(stderr_path),
        "check": str(check_path),
        "rc": None,
        "memory_check_failures": None,
        "ok": bool(dry_run),
    }
    if dry_run:
        return report

    out_dir.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        rc = int(subprocess.run(cmd, cwd=str(workspace_root), stdout=stdout, stderr=stderr).returncode)
    failures = _memory_check_failures(check_path)
    report["rc"] = rc
    report["memory_check_failures"] = failures
    report["ok"] = rc == 0 and failures == 0
    return report


def validate_web_bundle(path: Path, args: argparse.Namespace) -> dict[str, Any]:
    failures: list[dict[str, Any]] = []
    observed: dict[str, Any] = {}

    try:
        bundle = _read_json(path)
    except Exception as exc:
        return {
            "path": str(path),
            "ok": False,
            "observed": observed,
            "failures": [{"kind": "read_failed", "error": str(exc)}],
        }

    snapshot = _last_snapshot(bundle)
    render_text = _last_render_text(snapshot)
    debug_stats = _last_debug_stats(snapshot)
    observed.update(_render_text_budget_observed(render_text))
    for key in WEB_REQUIRED_RENDERER_METRICS:
        observed[key] = _u64_from_dict(debug_stats, key)

    if not observed.get("render_text_present"):
        failures.append(
            {
                "kind": "missing_render_text",
                "field": "windows[0].snapshots[-1].resource_caches.render_text",
            }
        )

    shape_limit = observed.get("render_text_shape_cache_entry_limit")
    if not _is_int(shape_limit):
        failures.append({"kind": "missing_metric", "metric": "render_text_shape_cache_entry_limit"})
    elif int(shape_limit) > int(args.web_max_shape_cache_entry_limit):
        failures.append(
            {
                "kind": "threshold_exceeded",
                "metric": "render_text_shape_cache_entry_limit",
                "threshold": int(args.web_max_shape_cache_entry_limit),
                "observed": int(shape_limit),
            }
        )

    for metric in [
        "render_text_mask_atlas_max_pages",
        "render_text_color_atlas_max_pages",
        "render_text_subpixel_atlas_max_pages",
    ]:
        value = observed.get(metric)
        if not _is_int(value):
            failures.append({"kind": "missing_metric", "metric": metric})
        elif int(value) > int(args.web_max_atlas_pages):
            failures.append(
                {
                    "kind": "threshold_exceeded",
                    "metric": metric,
                    "threshold": int(args.web_max_atlas_pages),
                    "observed": int(value),
                }
            )

    live_bytes = observed.get("render_text_atlas_bytes_live_estimate_total")
    budget_bytes = observed.get("render_text_atlas_bytes_budget_estimate_total")
    if not _is_int(live_bytes):
        failures.append({"kind": "missing_metric", "metric": "render_text_atlas_bytes_live_estimate_total"})
    if not _is_int(budget_bytes):
        failures.append({"kind": "missing_metric", "metric": "render_text_atlas_bytes_budget_estimate_total"})
    if _is_int(live_bytes) and _is_int(budget_bytes) and int(live_bytes) > int(budget_bytes):
        failures.append(
            {
                "kind": "threshold_exceeded",
                "metric": "render_text_atlas_bytes_live_estimate_total",
                "threshold": int(budget_bytes),
                "observed": int(live_bytes),
            }
        )

    evicted_pages = observed.get("renderer_text_atlas_evicted_pages")
    if _is_int(evicted_pages) and int(evicted_pages) > int(args.web_max_text_atlas_evicted_pages):
        failures.append(
            {
                "kind": "threshold_exceeded",
                "metric": "renderer_text_atlas_evicted_pages",
                "threshold": int(args.web_max_text_atlas_evicted_pages),
                "observed": int(evicted_pages),
            }
        )

    for metric in WEB_REQUIRED_RENDERER_METRICS:
        if not _is_int(observed.get(metric)):
            failures.append({"kind": "missing_metric", "metric": metric})

    return {
        "path": str(path),
        "ok": not failures,
        "observed": observed,
        "failures": failures,
    }


def build_summary(workspace_root: Path, args: argparse.Namespace) -> dict[str, Any]:
    out_dir_arg = str(args.out_dir).strip()
    out_dir = _resolve_workspace_path(
        workspace_root,
        out_dir_arg if out_dir_arg else f"target/fret-diag-u8-text-budget-gate-{int(time.time())}",
    )

    probes = [] if bool(args.skip_native) else native_probes(args)
    web_paths = [
        _resolve_workspace_path(workspace_root, value)
        for value in (args.web_export_bundle or [])
    ]
    if not probes and not web_paths:
        raise ValueError("nothing to check: enable at least one native probe or pass --web-export-bundle")

    native_reports: list[dict[str, Any]] = []
    for probe in probes:
        script_path = _resolve_workspace_path(workspace_root, probe.script)
        if not script_path.is_file():
            raise ValueError(f"{probe.name} script not found: {script_path}")
        probe_out = out_dir / probe.name
        cmd = build_repeat_cmd(
            fretboard_cmd=str(args.fretboard_cmd),
            probe=probe,
            out_dir=probe_out,
            repeat=int(args.repeat),
            timeout_ms=int(args.timeout_ms),
        )
        report = run_native_probe(workspace_root, cmd, probe_out, dry_run=bool(args.dry_run))
        report["name"] = probe.name
        report["thresholds"] = [{"key": key, "threshold": threshold} for key, threshold in probe.thresholds]
        native_reports.append(report)

    web_reports = [validate_web_bundle(path, args) for path in web_paths]

    ok = all(bool(report.get("ok")) for report in native_reports) and all(
        bool(report.get("ok")) for report in web_reports
    )

    return {
        "kind": "u8_text_budget_gate_summary",
        "schema_version": 1,
        "generated_unix_ms": int(time.time() * 1000),
        "dry_run": bool(args.dry_run),
        "out_dir": str(out_dir),
        "repeat": int(args.repeat),
        "timeout_ms": int(args.timeout_ms),
        "ok": ok,
        "native": {
            "enabled": not bool(args.skip_native),
            "probes": native_reports,
        },
        "web": {
            "bundles": web_reports,
        },
    }


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run or verify U8 text/cache/glyph/upload budget gates.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--out-dir", default="")
    ap.add_argument("--dry-run", action="store_true", default=False)
    ap.add_argument("--skip-native", action="store_true", default=False)
    ap.add_argument("--skip-text-heavy", action="store_true", default=False)
    ap.add_argument("--skip-code-editor", action="store_true", default=False)
    ap.add_argument("--repeat", type=int, default=3)
    ap.add_argument("--timeout-ms", type=int, default=900_000)
    ap.add_argument("--fretboard-cmd", default="cargo run -q -p fretboard-dev --")
    ap.add_argument("--text-heavy-script", default=TEXT_HEAVY_SCRIPT)
    ap.add_argument("--code-editor-script", default=CODE_EDITOR_SCRIPT)
    ap.add_argument("--text-heavy-launch-cmd", default=TEXT_HEAVY_LAUNCH_CMD)
    ap.add_argument("--code-editor-launch-cmd", default=CODE_EDITOR_LAUNCH_CMD)
    ap.add_argument("--native-atlas-max-pages", type=int, default=2)
    ap.add_argument("--text-heavy-max-atlas-bytes", type=int, default=48 * MIB)
    ap.add_argument("--text-heavy-max-shape-cache-entries", type=int, default=4096)
    ap.add_argument("--text-heavy-max-shape-cache-bytes", type=int, default=32 * MIB)
    ap.add_argument("--code-editor-max-atlas-bytes", type=int, default=16 * MIB)
    ap.add_argument("--code-editor-max-shape-cache-entries", type=int, default=4096)
    ap.add_argument("--code-editor-max-shape-cache-bytes", type=int, default=16 * MIB)
    ap.add_argument(
        "--web-export-bundle",
        action="append",
        default=[],
        help="Raw web/wasm `bundle.json` to validate. Repeat for multiple bundles.",
    )
    ap.add_argument("--web-max-shape-cache-entry-limit", type=int, default=1024)
    ap.add_argument("--web-max-atlas-pages", type=int, default=1)
    ap.add_argument("--web-max-text-atlas-evicted-pages", type=int, default=0)
    ap.add_argument("--out-report", default="")

    args = ap.parse_args()
    if int(args.repeat) < 1:
        print("error: --repeat must be >= 1", file=sys.stderr)
        return 2

    workspace_root = _workspace_root()
    try:
        summary = build_summary(workspace_root, args)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    out_report = str(args.out_report).strip()
    if out_report:
        _write_json(_resolve_workspace_path(workspace_root, out_report), summary)
    else:
        print(json.dumps(summary, indent=2, sort_keys=False))

    if summary["ok"]:
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
