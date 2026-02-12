#!/usr/bin/env python3
"""
Cross-platform perf gate triage helper.

Companion to:
  - tools/perf/diag_resize_probes_gate.sh
  - tools/perf/diag_resize_probes_gate.py

This is a Python equivalent of `triage_gate.sh`, intended to work on Windows/macOS/Linux
without requiring bash/awk/jq.
"""

from __future__ import annotations

import argparse
import json
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class GateSummary:
    out_dir: str
    suite: str
    attempts: int
    pass_attempts: int
    fail_attempts: int
    selected_attempt_dir: str
    attempt_summaries: list[dict[str, Any]]


def _read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _is_abs(p: str) -> bool:
    return os.path.isabs(p)


def _resolve_attempt_dir(*, raw: str, out_dir_abs: Path, summary_out_dir_rel: str) -> Path:
    if not raw:
        return Path(raw)

    # 1) absolute path
    if _is_abs(raw):
        return Path(raw)

    # 2) relative to cwd
    cwd_rel = Path(raw)
    if cwd_rel.is_dir():
        return cwd_rel

    # 3) relative to out_dir (common when summary stores workspace-relative paths)
    if summary_out_dir_rel and raw.startswith(summary_out_dir_rel):
        suffix = raw[len(summary_out_dir_rel) :].lstrip("/\\")
        return out_dir_abs / suffix

    return Path(raw)


def _extract_json_payload(stdout_path: Path) -> dict[str, Any]:
    text = stdout_path.read_text(encoding="utf-8", errors="replace")

    # Most gate outputs put logs before the JSON. Look for a line that starts with '{'.
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if line.startswith("{"):
            payload = "\n".join(lines[i:])
            return json.loads(payload)

    # Fallback: find the first '{' character (rare; but helps if the JSON is not line-aligned).
    idx = text.find("{")
    if idx >= 0:
        return json.loads(text[idx:])

    raise ValueError("no JSON payload found in stdout.json")


def _unique(items: list[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for item in items:
        if item in seen:
            continue
        seen.add(item)
        out.append(item)
    return out


def _endswith(haystack: str, suffix: str) -> bool:
    # Normalize path separators so suffix matching works on Windows.
    return haystack.replace("\\", "/").endswith(suffix.replace("\\", "/"))


def _resolve_bundle_path(bundle: str, *, out_dir_abs: Path, attempt_dir: Path) -> Path:
    if not bundle:
        return Path(bundle)
    if _is_abs(bundle):
        return Path(bundle)

    p = Path(bundle)
    if p.exists():
        return p

    attempt_rel = attempt_dir / p
    if attempt_rel.exists():
        return attempt_rel

    out_rel = out_dir_abs / p
    if out_rel.exists():
        return out_rel

    return p


def _snapshot_total_us(snap: dict[str, Any]) -> int:
    debug = snap.get("debug", {})
    stats = debug.get("stats", {})
    return int(stats.get("dispatch_time_us", 0) or 0) + int(stats.get("layout_time_us", 0) or 0) + int(
        stats.get("prepaint_time_us", 0) or 0
    ) + int(stats.get("paint_time_us", 0) or 0)


def _bundle_snapshot_summary(bundle_json: Path) -> list[str]:
    if not bundle_json.is_file():
        return []

    try:
        bundle = _read_json(bundle_json)
    except Exception:
        return []

    windows = bundle.get("windows") or []
    if not windows:
        return []

    snapshots = (windows[0] or {}).get("snapshots") or []
    if not snapshots:
        return []

    best = max(snapshots, key=_snapshot_total_us)
    debug = (best.get("debug") or {})
    stats = (debug.get("stats") or {})

    lines: list[str] = []
    total = _snapshot_total_us(best)
    lines.append(
        f"  - max frame: total={total}us paint={stats.get('paint_time_us')}us "
        f"layout={stats.get('layout_time_us')}us prepaint={stats.get('prepaint_time_us')}us"
    )

    app_snapshot = best.get("app_snapshot") or {}
    code_editor = (app_snapshot.get("code_editor") or {}).get("torture") or {}

    paint_perf = code_editor.get("paint_perf")
    if isinstance(paint_perf, dict):
        lines.append(
            "  - paint_perf: us_total={us_total}us us_syntax_spans={us_syntax_spans}us us_text_draw={us_text_draw}us rows={rows_painted}".format(
                us_total=paint_perf.get("us_total"),
                us_syntax_spans=paint_perf.get("us_syntax_spans"),
                us_text_draw=paint_perf.get("us_text_draw"),
                rows_painted=paint_perf.get("rows_painted"),
            )
        )

    cache_stats = code_editor.get("cache_stats")
    if isinstance(cache_stats, dict):
        lines.append(
            "  - cache_stats: syntax_resets={syntax_resets} row_rich_hits={row_rich_hits} row_rich_misses={row_rich_misses}".format(
                syntax_resets=cache_stats.get("syntax_resets"),
                row_rich_hits=cache_stats.get("row_rich_hits"),
                row_rich_misses=cache_stats.get("row_rich_misses"),
            )
        )

    return lines


def _load_summary(summary_path: Path) -> GateSummary:
    v = _read_json(summary_path)
    return GateSummary(
        out_dir=str(v.get("out_dir") or ""),
        suite=str(v.get("suite") or ""),
        attempts=int(v.get("attempts") or 0),
        pass_attempts=int(v.get("pass_attempts") or 0),
        fail_attempts=int(v.get("fail_attempts") or 0),
        selected_attempt_dir=str(v.get("selected_attempt_dir") or ""),
        attempt_summaries=list(v.get("attempt_summaries") or []),
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="triage_gate.py",
        description=(
            "Print a compact triage summary for perf gate output produced by tools/perf/diag_resize_probes_gate.sh"
        ),
    )
    parser.add_argument("out_dir", help="Gate out-dir (contains summary.json + attempt-N dirs)")
    parser.add_argument("--all", action="store_true", help="Also print worst bundles for passing attempts")
    parser.add_argument(
        "--script",
        dest="script_suffix",
        default="",
        help='Filter to scripts whose path ends with the given suffix (e.g. "ui-code-editor-resize-probes.json")',
    )
    parser.add_argument(
        "--app-snapshot",
        action="store_true",
        help="Print a small bundle-derived summary (worst frame stats + optional app_snapshot attribution)",
    )
    args = parser.parse_args()

    out_dir = Path(args.out_dir)
    summary_path = out_dir / "summary.json"
    if not summary_path.is_file():
        raise SystemExit(f"error: missing summary: {summary_path}")

    summary = _load_summary(summary_path)
    out_dir_abs = out_dir.resolve()
    summary_out_dir_rel = summary.out_dir

    print("== gate summary ==")
    print(
        f"pass={summary.fail_attempts == 0} suite={summary.suite} attempts={summary.attempts} "
        f"pass_attempts={summary.pass_attempts} fail_attempts={summary.fail_attempts} selected={summary.selected_attempt_dir}"
    )
    print()
    print("== attempts ==")
    for a in summary.attempt_summaries:
        attempt_dir = str(a.get("attempt_dir") or "")
        attempt_pass = bool(a.get("pass"))
        failures = ((a.get("check") or {}).get("failures"))
        failures_str = failures if failures is not None else "missing"
        rc = a.get("rc")
        print(f"{attempt_dir}\tpass={attempt_pass}\tfailures={failures_str}\trc={rc}")

    print()

    for a in summary.attempt_summaries:
        attempt_dir_raw = str(a.get("attempt_dir") or "")
        attempt_dir = _resolve_attempt_dir(raw=attempt_dir_raw, out_dir_abs=out_dir_abs, summary_out_dir_rel=summary_out_dir_rel)
        check_path = attempt_dir / "check.perf_thresholds.json"
        if not check_path.is_file():
            continue

        check = _read_json(check_path)
        failures = list(check.get("failures") or [])

        if not failures and not args.all:
            continue

        if failures:
            print(f"== FAIL: {attempt_dir_raw} ({len(failures)} threshold(s)) ==")
            for f in failures:
                print(
                    f"- {f.get('script')} :: {f.get('metric')} actual={f.get('actual_us')}us threshold={f.get('threshold_us')}us"
                )
        else:
            print(f"== PASS: {attempt_dir_raw} ==")

        stdout_path = attempt_dir / "stdout.json"
        if not stdout_path.is_file():
            print("- note: missing stdout.json (cannot resolve worst bundles)")
            print()
            continue

        try:
            payload = _extract_json_payload(stdout_path)
        except Exception as e:
            print(f"- note: failed to parse stdout.json JSON payload: {e}")
            print()
            continue

        rows = list(payload.get("rows") or [])
        if not rows:
            print("- note: no rows in JSON payload (cannot resolve worst bundles)")
            print()
            continue

        scripts: list[str]
        if failures:
            scripts = _unique([str(f.get("script") or "") for f in failures if (f.get("script") or "")])
        else:
            scripts = _unique([str(r.get("script") or "") for r in rows if (r.get("script") or "")])

        for script in scripts:
            if args.script_suffix and not _endswith(script, args.script_suffix):
                continue

            row = next((r for r in rows if _endswith(str(r.get("script") or ""), script)), None)
            if row is None:
                row = next((r for r in rows if _endswith(str(r.get("script") or ""), Path(script).name)), None)
            if row is None:
                print(f"- note: could not resolve row for {Path(script).name}")
                continue

            runs = list(row.get("runs") or [])
            if not runs:
                print(f"- note: no runs for {Path(script).name}")
                continue

            def run_key(run: dict[str, Any]) -> int:
                return int(run.get("top_total_time_us") or 0)

            worst = max(runs, key=run_key)
            worst_bundle_raw = str(worst.get("bundle") or "")
            worst_total = worst.get("top_total_time_us")
            if not worst_bundle_raw or worst_bundle_raw == "null":
                print(f"- note: could not resolve worst bundle for {Path(script).name}")
                continue

            print(f"- worst bundle ({Path(script).name}): {worst_bundle_raw} (top_total_time_us={worst_total})")

            if args.app_snapshot:
                bundle_path = _resolve_bundle_path(worst_bundle_raw, out_dir_abs=out_dir_abs, attempt_dir=attempt_dir)
                for line in _bundle_snapshot_summary(bundle_path):
                    print(line)

        print()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
