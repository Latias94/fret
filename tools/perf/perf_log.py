#!/usr/bin/env python3

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import subprocess
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple


def _repo_root() -> Path:
    # tools/perf/perf_log.py -> tools/perf -> tools -> repo root
    return Path(__file__).resolve().parents[2]


def _run_git(args: List[str]) -> Optional[str]:
    try:
        out = subprocess.check_output(["git", *args], cwd=_repo_root(), stderr=subprocess.DEVNULL)
        return out.decode("utf-8").strip()
    except Exception:
        return None


def _extract_perf_json_from_stdout(text: str) -> Dict[str, Any]:
    # `fretboard diag perf ... --json` prints a single JSON object at the end of stdout, but the
    # key order is not stable (serde_json may emit maps with sorted keys). We therefore look for
    # the last JSON object that starts at the beginning of a line and validates as a perf payload.
    starts = [m.start() for m in re.finditer(r"(?m)^\{", text)]
    if not starts:
        raise RuntimeError(
            "Failed to locate perf JSON in stdout (no '{' at beginning-of-line found). "
            "Ensure `fretboard diag perf ... --json` output was captured."
        )

    for idx in reversed(starts):
        candidate = text[idx:].strip()
        try:
            parsed = json.loads(candidate)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, dict) and parsed.get("schema_version") == 1 and "rows" in parsed:
            return parsed

    raise RuntimeError("Failed to parse perf JSON from stdout (no valid payload found).")


def _rel_script_path(script: str, repo_root: Path) -> str:
    try:
        p = Path(script)
        if p.is_absolute():
            try:
                return str(p.relative_to(repo_root))
            except ValueError:
                return script
        return script
    except Exception:
        return script


def _summarize_rows(perf: Dict[str, Any], repo_root: Path) -> List[Dict[str, Any]]:
    rows = perf.get("rows", [])
    out: List[Dict[str, Any]] = []
    for row in rows:
        script = _rel_script_path(str(row.get("script", "")), repo_root)
        stats = row.get("stats", {}) or {}
        runs = row.get("runs", []) or []
        bundles: List[str] = []
        for run in runs:
            if isinstance(run, dict):
                bundle = run.get("bundle")
                if bundle:
                    bundles.append(str(bundle))

        def get_stats(name: str) -> Dict[str, int]:
            raw = stats.get(name, {}) or {}
            return {
                "p50": int(raw.get("p50", 0) or 0),
                "p95": int(raw.get("p95", 0) or 0),
                "max": int(raw.get("max", 0) or 0),
            }

        total = get_stats("total_time_us")
        layout = get_stats("layout_time_us")
        solve = get_stats("layout_engine_solve_time_us")
        paint = get_stats("paint_time_us")
        prepaint = get_stats("prepaint_time_us")
        dispatch = get_stats("dispatch_time_us")
        hit_test = get_stats("hit_test_time_us")

        churn = {
            "text_atlas_upload_bytes": get_stats("top_renderer_text_atlas_upload_bytes"),
            "text_atlas_evicted_pages": get_stats("top_renderer_text_atlas_evicted_pages"),
            "svg_upload_bytes": get_stats("top_renderer_svg_upload_bytes"),
            "image_upload_bytes": get_stats("top_renderer_image_upload_bytes"),
            "svg_raster_cache_misses": get_stats("top_renderer_svg_raster_cache_misses"),
            "svg_raster_budget_evictions": get_stats("top_renderer_svg_raster_budget_evictions"),
            "intermediate_budget_bytes": get_stats("top_renderer_intermediate_budget_bytes"),
            "intermediate_in_use_bytes": get_stats("top_renderer_intermediate_in_use_bytes"),
            "intermediate_peak_in_use_bytes": get_stats(
                "top_renderer_intermediate_peak_in_use_bytes"
            ),
            "intermediate_release_targets": get_stats(
                "top_renderer_intermediate_release_targets"
            ),
            "intermediate_pool_allocations": get_stats(
                "top_renderer_intermediate_pool_allocations"
            ),
            "intermediate_pool_reuses": get_stats("top_renderer_intermediate_pool_reuses"),
            "intermediate_pool_releases": get_stats("top_renderer_intermediate_pool_releases"),
            "intermediate_pool_evictions": get_stats(
                "top_renderer_intermediate_pool_evictions"
            ),
            "intermediate_pool_free_bytes": get_stats(
                "top_renderer_intermediate_pool_free_bytes"
            ),
            "intermediate_pool_free_textures": get_stats(
                "top_renderer_intermediate_pool_free_textures"
            ),
        }

        worst_run = row.get("worst_run") or None
        worst_bundle = None
        worst_us = None
        if isinstance(worst_run, dict):
            worst_us = worst_run.get("top_total_time_us")
            worst_bundle = worst_run.get("bundle")

        out.append(
            {
                "script": script,
                "total": total,
                "layout": layout,
                "solve": solve,
                "prepaint": prepaint,
                "paint": paint,
                "dispatch": dispatch,
                "hit_test": hit_test,
                "churn": churn,
                "worst_us": int(worst_us or 0),
                "worst_bundle": str(worst_bundle or ""),
                "bundles": bundles,
            }
        )

    out.sort(key=lambda r: r["script"])
    return out


def _p50_p95_max(values: List[int]) -> Tuple[int, int, int]:
    if not values:
        return (0, 0, 0)
    s = sorted(values)
    n = len(s)

    def pick(q: float) -> int:
        idx = int(round((n - 1) * q))
        idx = max(0, min(n - 1, idx))
        return int(s[idx])

    return (pick(0.5), pick(0.95), int(s[-1]))


def _extract_pointer_move_frame_ids(bundle: Dict[str, Any]) -> List[int]:
    frame_ids: List[int] = []
    for w in bundle.get("windows", []) or []:
        if not isinstance(w, dict):
            continue
        for e in w.get("events", []) or []:
            if not isinstance(e, dict):
                continue
            if e.get("kind") == "pointer.move" and isinstance(e.get("frame_id"), int):
                frame_ids.append(int(e["frame_id"]))
    return frame_ids


def _derive_pointer_move_metrics_from_bundle(
    *,
    bundle_path: Path,
    repo_root: Path,
) -> Optional[Dict[str, Any]]:
    try:
        bundle = json.loads(bundle_path.read_text(encoding="utf-8", errors="replace"))
    except Exception:
        return None

    pointer_move_frame_ids = set(_extract_pointer_move_frame_ids(bundle))

    max_dispatch_us = 0
    max_hit_test_us = 0
    snapshots_with_global_changes = 0

    for w in bundle.get("windows", []) or []:
        if not isinstance(w, dict):
            continue

        for s in w.get("snapshots", []) or []:
            if not isinstance(s, dict):
                continue
            frame_id = s.get("frame_id")
            if not isinstance(frame_id, int):
                continue

            debug = s.get("debug")
            if not isinstance(debug, dict):
                continue
            stats = debug.get("stats")
            if not isinstance(stats, dict):
                continue

            dispatch_events = stats.get("dispatch_events")
            if not isinstance(dispatch_events, int):
                dispatch_events = 0

            if pointer_move_frame_ids:
                # Real pointer-move frames. Keep the filter consistent with existing manual analysis:
                # only consider frames that actually dispatched events.
                if frame_id not in pointer_move_frame_ids or dispatch_events <= 0:
                    continue
            else:
                # Fallback when the event log is missing: treat “dispatch frames” as the target set.
                if dispatch_events <= 0:
                    continue

            dispatch_us = stats.get("dispatch_time_us")
            if isinstance(dispatch_us, int):
                max_dispatch_us = max(max_dispatch_us, dispatch_us)

            hit_test_us = stats.get("hit_test_time_us")
            if isinstance(hit_test_us, int):
                max_hit_test_us = max(max_hit_test_us, hit_test_us)

            global_change_globals = stats.get("global_change_globals")
            if isinstance(global_change_globals, int) and global_change_globals > 0:
                snapshots_with_global_changes += 1

    return {
        "pointer_move_frames_present": bool(pointer_move_frame_ids),
        "max_dispatch_time_us": int(max_dispatch_us),
        "max_hit_test_time_us": int(max_hit_test_us),
        "snapshots_with_global_changes": int(snapshots_with_global_changes),
        "bundle": _rel_script_path(str(bundle_path), repo_root),
    }


def _derive_worst_frame_text_prepare_metrics_from_bundle(
    *,
    bundle_path: Path,
    repo_root: Path,
) -> Optional[Dict[str, Any]]:
    try:
        bundle = json.loads(bundle_path.read_text(encoding="utf-8", errors="replace"))
    except Exception:
        return None

    worst_total_us = 0
    worst_frame_id: Optional[int] = None
    worst_snapshot: Optional[Dict[str, Any]] = None

    for w in bundle.get("windows", []) or []:
        if not isinstance(w, dict):
            continue
        for s in w.get("snapshots", []) or []:
            if not isinstance(s, dict):
                continue
            debug = s.get("debug")
            if not isinstance(debug, dict):
                continue
            stats = debug.get("stats")
            if not isinstance(stats, dict):
                continue

            total_us = stats.get("total_time_us")
            if not isinstance(total_us, int):
                # Some bundles do not include a monolithic `total_time_us` inside debug stats.
                # Approximate a “total” by summing the high-level phase timers.
                layout_us = stats.get("layout_time_us")
                prepaint_us = stats.get("prepaint_time_us")
                paint_us = stats.get("paint_time_us")
                dispatch_us = stats.get("dispatch_time_us")
                hit_test_us = stats.get("hit_test_time_us")
                if not all(
                    isinstance(v, int)
                    for v in [layout_us, prepaint_us, paint_us, dispatch_us, hit_test_us]
                ):
                    continue
                total_us = (
                    int(layout_us)
                    + int(prepaint_us)
                    + int(paint_us)
                    + int(dispatch_us)
                    + int(hit_test_us)
                )

            if total_us >= worst_total_us:
                worst_total_us = int(total_us)
                frame_id = s.get("frame_id")
                worst_frame_id = int(frame_id) if isinstance(frame_id, int) else None
                worst_snapshot = s

    if not worst_snapshot:
        return None

    stats = (worst_snapshot.get("debug") or {}).get("stats") or {}
    if not isinstance(stats, dict):
        return None

    def get_int(name: str) -> int:
        v = stats.get(name)
        return int(v) if isinstance(v, int) else 0

    return {
        "worst_total_time_us": int(worst_total_us),
        "worst_frame_id": worst_frame_id,
        "paint_text_prepare_time_us": get_int("paint_text_prepare_time_us"),
        "paint_text_prepare_calls": get_int("paint_text_prepare_calls"),
        "paint_text_prepare_reason_width_changed": get_int(
            "paint_text_prepare_reason_width_changed"
        ),
        "paint_text_prepare_reason_wrap_changed": get_int("paint_text_prepare_reason_wrap_changed"),
        "paint_text_prepare_reason_scale_changed": get_int(
            "paint_text_prepare_reason_scale_changed"
        ),
        "paint_text_prepare_reason_style_changed": get_int(
            "paint_text_prepare_reason_style_changed"
        ),
        "paint_text_prepare_reason_font_stack_changed": get_int(
            "paint_text_prepare_reason_font_stack_changed"
        ),
        "paint_text_prepare_reason_text_changed": get_int("paint_text_prepare_reason_text_changed"),
        "bundle": _rel_script_path(str(bundle_path), repo_root),
    }


def _format_entry_markdown(
    *,
    timestamp: str,
    commit: str,
    subject: str,
    change: str,
    suite: str,
    command: str,
    stdout: str,
    worst_overall: Optional[Dict[str, Any]],
    rows: List[Dict[str, Any]],
    repo_root: Path,
) -> str:
    lines: List[str] = []
    lines.append(f"## {timestamp} (commit `{commit}`)")
    lines.append("")
    if change:
        lines.append("Change:")
        lines.append(f"- {change}")
        lines.append("")
    elif subject:
        lines.append("Change:")
        lines.append(f"- {subject}")
        lines.append("")

    if suite:
        lines.append("Suite:")
        lines.append(f"- `{suite}`")
        lines.append("")

    if command:
        lines.append("Command:")
        lines.append("```powershell")
        lines.append(command.rstrip())
        lines.append("```")
        lines.append("")

    if stdout:
        lines.append("Stdout:")
        lines.append(f"- `{stdout}`")
        lines.append("")

    lines.append("Results (us):")
    lines.append(
        "| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint | p95 dispatch | p95 hit_test |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for r in rows:
        lines.append(
            "| {script} | {p50_total} | {p95_total} | {max_total} | {p95_layout} | {p95_solve} | {p95_prepaint} | {p95_paint} | {p95_dispatch} | {p95_hit_test} |".format(
                script=r["script"],
                p50_total=r["total"]["p50"],
                p95_total=r["total"]["p95"],
                max_total=r["total"]["max"],
                p95_layout=r["layout"]["p95"],
                p95_solve=r["solve"]["p95"],
                p95_prepaint=r["prepaint"]["p95"],
                p95_paint=r["paint"]["p95"],
                p95_dispatch=r["dispatch"]["p95"],
                p95_hit_test=r["hit_test"]["p95"],
            )
        )
    lines.append("")

    # Derived “pointer move” (or “dispatch frame”) costs. This avoids the common distortion where the
    # worst top frame is a non-dispatch settle/selector frame, making p95 dispatch/hit_test appear as 0.
    for r in rows:
        bundles = r.get("bundles") or []
        if not bundles:
            continue

        derived: List[Dict[str, Any]] = []
        for b in bundles:
            try:
                bp = Path(b)
                if not bp.is_absolute():
                    bp = repo_root / bp
            except Exception:
                continue
            m = _derive_pointer_move_metrics_from_bundle(bundle_path=bp, repo_root=repo_root)
            if m:
                derived.append(m)

        if not derived:
            continue

        dispatch_maxes = [int(x.get("max_dispatch_time_us", 0) or 0) for x in derived]
        hit_test_maxes = [int(x.get("max_hit_test_time_us", 0) or 0) for x in derived]
        global_change_counts = [int(x.get("snapshots_with_global_changes", 0) or 0) for x in derived]

        p50_d, p95_d, max_d = _p50_p95_max(dispatch_maxes)
        p50_h, p95_h, max_h = _p50_p95_max(hit_test_maxes)
        p50_gc, p95_gc, max_gc = _p50_p95_max(global_change_counts)

        worst_dispatch = max(derived, key=lambda x: int(x.get("max_dispatch_time_us", 0) or 0))
        worst_hit_test = max(derived, key=lambda x: int(x.get("max_hit_test_time_us", 0) or 0))

        mode = (
            "Pointer-move frames"
            if any(x.get("pointer_move_frames_present") for x in derived)
            else "Dispatch frames"
        )

        lines.append("Notes:")
        lines.append(
            f"- {mode} (derived from bundle snapshots; per-run **max** over frames where `dispatch_events > 0`; us):"
        )
        lines.append(f"  - `dispatch_time_us`: `{p50_d} / {p95_d} / {max_d}` (p50 / p95 / max)")
        lines.append(f"  - `hit_test_time_us`: `{p50_h} / {p95_h} / {max_h}` (p50 / p95 / max)")
        lines.append(
            f"  - `snapshots_with_global_changes` (within that frame set): `{p50_gc} / {p95_gc} / {max_gc}` (p50 / p95 / max)"
        )
        lines.append(f"  - Worst dispatch bundle: `{worst_dispatch.get('bundle','')}`")
        lines.append(f"  - Worst hit-test bundle: `{worst_hit_test.get('bundle','')}`")
        lines.append("")
        break

    # Derived “text prepare” cost + reason counters on the per-bundle worst frame. This is
    # particularly important for resize-drag probes where width jitter can trigger expensive wrap
    # recompute even when most UI chrome is cached.
    lines.append("Text prepare signals (worst frame in each bundle; p95/max):")
    lines.append(
        "| script | p95 prepare_us | max prepare_us | p95 width_changed | max width_changed | p95 calls | max calls |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for r in rows:
        bundles = r.get("bundles") or []
        if not bundles:
            continue

        derived: List[Dict[str, Any]] = []
        for b in bundles[:20]:
            try:
                bp = Path(b)
                if not bp.is_absolute():
                    bp = repo_root / bp
            except Exception:
                continue
            m = _derive_worst_frame_text_prepare_metrics_from_bundle(
                bundle_path=bp, repo_root=repo_root
            )
            if m:
                derived.append(m)

        if not derived:
            continue

        prepare_maxes = [int(x.get("paint_text_prepare_time_us", 0) or 0) for x in derived]
        width_changed = [
            int(x.get("paint_text_prepare_reason_width_changed", 0) or 0) for x in derived
        ]
        calls = [int(x.get("paint_text_prepare_calls", 0) or 0) for x in derived]

        _p50_p, p95_p, max_p = _p50_p95_max(prepare_maxes)
        _p50_w, p95_w, max_w = _p50_p95_max(width_changed)
        _p50_c, p95_c, max_c = _p50_p95_max(calls)

        lines.append(
            (
                "| {script} | {p95_prepare} | {max_prepare} | {p95_width} | {max_width} | "
                "{p95_calls} | {max_calls} |"
            ).format(
                script=r["script"],
                p95_prepare=p95_p,
                max_prepare=max_p,
                p95_width=p95_w,
                max_width=max_w,
                p95_calls=p95_c,
                max_calls=max_c,
            )
        )
    lines.append("")

    lines.append("Churn signals (top frame; p95/max):")
    lines.append(
        "| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |"
    )
    lines.append(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    )
    for r in rows:
        churn = r.get("churn", {}) or {}
        atlas_upload = churn.get("text_atlas_upload_bytes", {}) or {}
        atlas_evicted_pages = churn.get("text_atlas_evicted_pages", {}) or {}
        svg_upload = churn.get("svg_upload_bytes", {}) or {}
        image_upload = churn.get("image_upload_bytes", {}) or {}
        svg_cache_misses = churn.get("svg_raster_cache_misses", {}) or {}
        svg_evictions = churn.get("svg_raster_budget_evictions", {}) or {}
        intermediate_peak = churn.get("intermediate_peak_in_use_bytes", {}) or {}
        pool_evictions = churn.get("intermediate_pool_evictions", {}) or {}
        lines.append(
            "| {script} | {p95_upload} | {max_upload} | {p95_evict} | {max_evict} | {p95_svg_upload} | {max_svg_upload} | {p95_image_upload} | {max_image_upload} | {p95_svg_misses} | {max_svg_misses} | {p95_svg_evictions} | {max_svg_evictions} | {p95_peak} | {max_peak} | {p95_evictions} | {max_evictions} |".format(
                script=r["script"],
                p95_upload=int(atlas_upload.get("p95", 0) or 0),
                max_upload=int(atlas_upload.get("max", 0) or 0),
                p95_evict=int(atlas_evicted_pages.get("p95", 0) or 0),
                max_evict=int(atlas_evicted_pages.get("max", 0) or 0),
                p95_svg_upload=int(svg_upload.get("p95", 0) or 0),
                max_svg_upload=int(svg_upload.get("max", 0) or 0),
                p95_image_upload=int(image_upload.get("p95", 0) or 0),
                max_image_upload=int(image_upload.get("max", 0) or 0),
                p95_svg_misses=int(svg_cache_misses.get("p95", 0) or 0),
                max_svg_misses=int(svg_cache_misses.get("max", 0) or 0),
                p95_svg_evictions=int(svg_evictions.get("p95", 0) or 0),
                max_svg_evictions=int(svg_evictions.get("max", 0) or 0),
                p95_peak=int(intermediate_peak.get("p95", 0) or 0),
                max_peak=int(intermediate_peak.get("max", 0) or 0),
                p95_evictions=int(pool_evictions.get("p95", 0) or 0),
                max_evictions=int(pool_evictions.get("max", 0) or 0),
            )
        )
    lines.append("")

    lines.append("Intermediate pool signals (top frame; p95/max):")
    lines.append(
        "| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |"
    )
    lines.append(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    )
    for r in rows:
        churn = r.get("churn", {}) or {}
        budget = churn.get("intermediate_budget_bytes", {}) or {}
        in_use = churn.get("intermediate_in_use_bytes", {}) or {}
        peak = churn.get("intermediate_peak_in_use_bytes", {}) or {}
        release_targets = churn.get("intermediate_release_targets", {}) or {}
        allocations = churn.get("intermediate_pool_allocations", {}) or {}
        reuses = churn.get("intermediate_pool_reuses", {}) or {}
        releases = churn.get("intermediate_pool_releases", {}) or {}
        evictions = churn.get("intermediate_pool_evictions", {}) or {}
        free_bytes = churn.get("intermediate_pool_free_bytes", {}) or {}
        free_textures = churn.get("intermediate_pool_free_textures", {}) or {}
        lines.append(
            "| {script} | {p95_budget} | {max_budget} | {p95_in_use} | {max_in_use} | {p95_peak} | {max_peak} | {p95_release_targets} | {max_release_targets} | {p95_alloc} | {max_alloc} | {p95_reuse} | {max_reuse} | {p95_release} | {max_release} | {p95_evict} | {max_evict} | {p95_free_bytes} | {max_free_bytes} | {p95_free_tex} | {max_free_tex} |".format(
                script=r["script"],
                p95_budget=int(budget.get("p95", 0) or 0),
                max_budget=int(budget.get("max", 0) or 0),
                p95_in_use=int(in_use.get("p95", 0) or 0),
                max_in_use=int(in_use.get("max", 0) or 0),
                p95_peak=int(peak.get("p95", 0) or 0),
                max_peak=int(peak.get("max", 0) or 0),
                p95_release_targets=int(release_targets.get("p95", 0) or 0),
                max_release_targets=int(release_targets.get("max", 0) or 0),
                p95_alloc=int(allocations.get("p95", 0) or 0),
                max_alloc=int(allocations.get("max", 0) or 0),
                p95_reuse=int(reuses.get("p95", 0) or 0),
                max_reuse=int(reuses.get("max", 0) or 0),
                p95_release=int(releases.get("p95", 0) or 0),
                max_release=int(releases.get("max", 0) or 0),
                p95_evict=int(evictions.get("p95", 0) or 0),
                max_evict=int(evictions.get("max", 0) or 0),
                p95_free_bytes=int(free_bytes.get("p95", 0) or 0),
                max_free_bytes=int(free_bytes.get("max", 0) or 0),
                p95_free_tex=int(free_textures.get("p95", 0) or 0),
                max_free_tex=int(free_textures.get("max", 0) or 0),
            )
        )
    lines.append("")

    if worst_overall:
        worst_script = _rel_script_path(str(worst_overall.get("script", "")), repo_root)
        worst_us = int(worst_overall.get("top_total_time_us", 0) or 0)
        worst_bundle = str(worst_overall.get("bundle", "") or "")
        if worst_bundle:
            worst_bundle = _rel_script_path(worst_bundle, repo_root)

        lines.append("Worst overall:")
        lines.append(f"- script: `{worst_script}`")
        lines.append(f"- top_total_time_us: `{worst_us}`")
        if worst_bundle:
            lines.append(f"- bundle: `{worst_bundle}`")
        lines.append("")

    return "\n".join(lines)


def append_cmd(args: argparse.Namespace) -> int:
    repo_root = _repo_root()
    stdout_path = Path(args.stdout)
    log_path = Path(args.log)

    stdout_text = stdout_path.read_text(encoding="utf-8", errors="replace")
    perf = _extract_perf_json_from_stdout(stdout_text)

    commit = args.commit or _run_git(["rev-parse", "HEAD"]) or "UNKNOWN"
    subject = _run_git(["show", "-s", "--format=%s", commit]) or ""

    rows = _summarize_rows(perf, repo_root)
    worst_overall = perf.get("worst_overall") or None

    timestamp = args.timestamp or dt.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    change = args.change or ""
    command = args.command or ""
    suite = args.suite or ""

    entry = _format_entry_markdown(
        timestamp=timestamp,
        commit=commit,
        subject=subject,
        change=change,
        suite=suite,
        command=command,
        stdout=_rel_script_path(str(stdout_path), repo_root),
        worst_overall=worst_overall if isinstance(worst_overall, dict) else None,
        rows=rows,
        repo_root=repo_root,
    )

    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("a", encoding="utf-8") as f:
        if log_path.stat().st_size > 0:
            f.write("\n")
        f.write(entry)

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Append `fretboard diag perf --json` results to a Markdown log.")
    sub = parser.add_subparsers(dest="cmd", required=True)

    append = sub.add_parser("append", help="Append a new perf entry by parsing a captured stdout file.")
    append.add_argument("--stdout", required=True, help="Path to captured stdout from `fretboard diag perf ... --json`.")
    append.add_argument("--log", required=True, help="Markdown log file to append to.")
    append.add_argument("--suite", default="", help="Suite name (e.g. ui-gallery).")
    append.add_argument("--command", default="", help="Exact command used to capture the stdout.")
    append.add_argument("--commit", default="", help="Commit hash to attribute the run to (default: HEAD).")
    append.add_argument("--change", default="", help="Short human description of the change.")
    append.add_argument("--timestamp", default="", help="Timestamp override (default: now).")
    append.set_defaults(fn=append_cmd)

    args = parser.parse_args()
    return int(args.fn(args))


if __name__ == "__main__":
    raise SystemExit(main())
