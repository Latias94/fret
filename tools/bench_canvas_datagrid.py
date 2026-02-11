#!/usr/bin/env python3
"""
Run the canvas datagrid stress demo and emit simple performance summaries.

Cross-platform replacement for `tools/bench_canvas_datagrid.ps1`.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TextIO


_STATS_RE = re.compile(r"datagrid_canvas_stats: samples=(\d+) total_avg=([\d\.]+)ms total_p95=([\d\.]+)ms")
_RENDERER_RE = re.compile(r"renderer_perf: frames=(\d+) encode=([\d\.]+)ms prepare_text=([\d\.]+)ms draws=(\d+)")
_VISIBLE_RE = re.compile(r"datagrid_canvas: visible_rows=(\d+) visible_cols=(\d+) visible_cells=(\d+)")


@dataclass(frozen=True)
class Stats:
    samples: int
    total_avg_ms: float
    total_p95_ms: float


@dataclass(frozen=True)
class Renderer:
    frames: int
    encode_ms: float
    prepare_text_ms: float
    draws: int


@dataclass(frozen=True)
class Visible:
    visible_rows: int
    visible_cols: int
    visible_cells: int


@dataclass(frozen=True)
class CaseResult:
    name: str
    rows: int
    cols: int
    variable: bool
    log: Path
    full_log: Path
    stats: Stats | None
    renderer: Renderer | None
    visible: Visible | None


def _timestamp_folder() -> str:
    return _dt.datetime.now().strftime("%Y%m%d-%H%M%S")


def _repo_root() -> Path:
    return Path(".").resolve()


def _resolve_cargo_target_dir(repo_root: Path) -> Path | None:
    cargo_target_dir = os.environ.get("CARGO_TARGET_DIR", "").strip()
    if cargo_target_dir:
        return Path(cargo_target_dir).expanduser().resolve()

    sccache_dir = os.environ.get("SCCACHE_DIR", "").strip()
    if sccache_dir:
        parent = Path(sccache_dir).expanduser().resolve().parent
        return parent / "target" / repo_root.name

    return None


def _pick_run_dir(repo_root: Path, out_dir: str) -> Path:
    if out_dir.strip():
        return Path(out_dir).expanduser().resolve()

    env_out = os.environ.get("FRET_BENCH_OUT_DIR", "").strip()
    if env_out:
        return Path(env_out).expanduser().resolve()

    sccache_dir = os.environ.get("SCCACHE_DIR", "").strip()
    if sccache_dir:
        return Path(sccache_dir).expanduser().resolve().parent / "bench"

    temp = os.environ.get("TEMP", "").strip()
    if temp:
        return Path(temp).expanduser().resolve() / "fret-bench"

    tmpdir = os.environ.get("TMPDIR", "").strip()
    if tmpdir:
        return Path(tmpdir).expanduser().resolve() / "fret-bench"

    return repo_root / ".bench"


def _ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def _parse_last_match(path: Path, regex: re.Pattern[str]) -> re.Match[str] | None:
    last: re.Match[str] | None = None
    with path.open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            m = regex.search(line)
            if m is not None:
                last = m
    return last


def _parse_last_stats(path: Path) -> Stats | None:
    m = _parse_last_match(path, _STATS_RE)
    if m is None:
        return None
    return Stats(samples=int(m.group(1)), total_avg_ms=float(m.group(2)), total_p95_ms=float(m.group(3)))


def _parse_last_renderer(path: Path) -> Renderer | None:
    m = _parse_last_match(path, _RENDERER_RE)
    if m is None:
        return None
    return Renderer(
        frames=int(m.group(1)),
        encode_ms=float(m.group(2)),
        prepare_text_ms=float(m.group(3)),
        draws=int(m.group(4)),
    )


def _parse_last_visible(path: Path) -> Visible | None:
    m = _parse_last_match(path, _VISIBLE_RE)
    if m is None:
        return None
    return Visible(visible_rows=int(m.group(1)), visible_cols=int(m.group(2)), visible_cells=int(m.group(3)))


def _stream_process(proc: subprocess.Popen[str], *, log: TextIO, full_log: TextIO | None) -> int:
    assert proc.stdout is not None
    for line in proc.stdout:
        log.write(line)
        log.flush()
        if full_log is not None:
            full_log.write(line)
            full_log.flush()
        sys.stdout.write(line)
        sys.stdout.flush()
    return proc.wait()


def _median(values: list[float]) -> float | None:
    if not values:
        return None
    sorted_values = sorted(values)
    n = len(sorted_values)
    if n % 2 == 1:
        return float(sorted_values[(n - 1) // 2])
    a = float(sorted_values[(n // 2) - 1])
    b = float(sorted_values[n // 2])
    return (a + b) / 2.0


def _invoke_case(
    *,
    repo_root: Path,
    run_dir: Path,
    cargo_target_dir: Path | None,
    name: str,
    rows: int,
    cols: int,
    variable: bool,
    iteration: int,
    release: bool,
    exit_after_frames: int,
    stats_window: int,
    auto_scroll: bool,
    full_log: bool,
) -> CaseResult:
    case_dir = run_dir / name
    _ensure_dir(case_dir)

    log_base = case_dir / f"run_iter{iteration}"
    log_path = Path(f"{log_base}.log")
    full_log_path = Path(f"{log_base}.full.log")

    env = os.environ.copy()
    if cargo_target_dir is not None:
        env["CARGO_TARGET_DIR"] = str(cargo_target_dir)
    env["FRET_CANVAS_GRID_ROWS"] = str(rows)
    env["FRET_CANVAS_GRID_COLS"] = str(cols)
    env["FRET_CANVAS_GRID_VARIABLE"] = "1" if variable else "0"
    env["FRET_CANVAS_GRID_STATS_WINDOW"] = str(stats_window)
    env["FRET_CANVAS_GRID_EXIT_AFTER_FRAMES"] = str(exit_after_frames)
    env["FRET_CANVAS_GRID_AUTO_SCROLL"] = "1" if auto_scroll else "0"

    cargo_args: list[str] = ["cargo", "run"]
    if not full_log:
        cargo_args.append("-q")
    cargo_args += ["-p", "fret-demo", "--bin", "canvas_datagrid_stress_demo"]
    if release:
        cargo_args.append("--release")

    header = (
        f"case={name} rows={rows} cols={cols} variable={variable} release={release} "
        f"frames={exit_after_frames} window={stats_window} autoscroll={auto_scroll}\n"
    )
    log_path.write_text(header, encoding="utf-8")

    with log_path.open("a", encoding="utf-8") as log_f:
        full_f: TextIO | None = None
        try:
            if full_log:
                full_f = full_log_path.open("w", encoding="utf-8")
            proc = subprocess.Popen(
                cargo_args,
                cwd=str(repo_root),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
            code = _stream_process(proc, log=log_f, full_log=full_f)
        finally:
            if full_f is not None:
                full_f.close()

    if code != 0:
        raise RuntimeError(f"cargo exited with code {code}")

    return CaseResult(
        name=name,
        rows=rows,
        cols=cols,
        variable=variable,
        log=log_path,
        full_log=full_log_path,
        stats=_parse_last_stats(log_path),
        renderer=_parse_last_renderer(log_path),
        visible=_parse_last_visible(log_path),
    )


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--exit-after-frames", type=int, default=600)
    parser.add_argument("--stats-window", type=int, default=240)
    parser.add_argument("--auto-scroll", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument("--iterations", type=int, default=1)
    parser.add_argument("--full-log", action="store_true")
    parser.add_argument("--out-dir", default="")
    args = parser.parse_args(argv)

    if args.iterations < 1:
        raise RuntimeError("--iterations must be >= 1")

    repo_root = _repo_root()
    ts = _timestamp_folder()
    cargo_target_dir = _resolve_cargo_target_dir(repo_root)

    run_dir_base = _pick_run_dir(repo_root, args.out_dir)
    run_dir = run_dir_base / "canvas-datagrid" / ts
    _ensure_dir(run_dir)

    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=str(repo_root), check=False, stdout=subprocess.PIPE, text=True).stdout.strip()
    rustc = subprocess.run(["rustc", "-V"], cwd=str(repo_root), check=False, stdout=subprocess.PIPE, text=True).stdout.strip()
    cargo = subprocess.run(["cargo", "-V"], cwd=str(repo_root), check=False, stdout=subprocess.PIPE, text=True).stdout.strip()

    meta_path = run_dir / "meta.txt"
    meta_lines = [
        f"commit={commit}",
        f"rustc={rustc}",
        f"cargo={cargo}",
        f"sccache={os.environ.get('SCCACHE_DIR', '')}",
        f"cargo_target_dir={str(cargo_target_dir) if cargo_target_dir is not None else ''}",
        "",
    ]
    meta_path.write_text("\n".join(meta_lines), encoding="utf-8")

    summary_path = run_dir / "summary.csv"
    summary_path.write_text(
        "case,iteration,rows,cols,variable,profile,exit_after_frames,stats_window,auto_scroll,"
        "visible_rows,visible_cols,visible_cells,samples,total_avg_ms,total_p95_ms,encode_ms,prepare_text_ms,draws,log,full_log\n",
        encoding="utf-8",
    )

    cases = [
        {"name": "200k_x_200_fixed", "rows": 200000, "cols": 200, "variable": False},
        {"name": "200k_x_200_variable", "rows": 200000, "cols": 200, "variable": True},
        {"name": "1m_x_200_fixed", "rows": 1000000, "cols": 200, "variable": False},
        {"name": "1m_x_200_variable", "rows": 1000000, "cols": 200, "variable": True},
    ]

    profile = "release" if args.release else "debug"
    all_rows: list[dict[str, object]] = []

    for c in cases:
        for iteration in range(1, args.iterations + 1):
            result = _invoke_case(
                repo_root=repo_root,
                run_dir=run_dir,
                cargo_target_dir=cargo_target_dir,
                name=str(c["name"]),
                rows=int(c["rows"]),
                cols=int(c["cols"]),
                variable=bool(c["variable"]),
                iteration=iteration,
                release=args.release,
                exit_after_frames=args.exit_after_frames,
                stats_window=args.stats_window,
                auto_scroll=bool(args.auto_scroll),
                full_log=args.full_log,
            )

            vrows = str(result.visible.visible_rows) if result.visible is not None else ""
            vcols = str(result.visible.visible_cols) if result.visible is not None else ""
            vcells = str(result.visible.visible_cells) if result.visible is not None else ""

            samples = str(result.stats.samples) if result.stats is not None else ""
            avg = f"{result.stats.total_avg_ms:.3f}" if result.stats is not None else ""
            p95 = f"{result.stats.total_p95_ms:.3f}" if result.stats is not None else ""

            encode = f"{result.renderer.encode_ms:.3f}" if result.renderer is not None else ""
            prepare_text = f"{result.renderer.prepare_text_ms:.3f}" if result.renderer is not None else ""
            draws = str(result.renderer.draws) if result.renderer is not None else ""

            line = (
                f"{result.name},{iteration},{result.rows},{result.cols},{str(result.variable).lower()},{profile},"
                f"{args.exit_after_frames},{args.stats_window},{str(bool(args.auto_scroll)).lower()},"
                f"{vrows},{vcols},{vcells},{samples},{avg},{p95},{encode},{prepare_text},{draws},"
                f"{result.log},{result.full_log}\n"
            )
            with summary_path.open("a", encoding="utf-8") as f:
                f.write(line)

            all_rows.append(
                {
                    "case": result.name,
                    "rows": result.rows,
                    "cols": result.cols,
                    "variable": result.variable,
                    "visible_rows": result.visible.visible_rows if result.visible else None,
                    "visible_cols": result.visible.visible_cols if result.visible else None,
                    "visible_cells": result.visible.visible_cells if result.visible else None,
                    "total_avg_ms": result.stats.total_avg_ms if result.stats else None,
                    "total_p95_ms": result.stats.total_p95_ms if result.stats else None,
                    "prepare_text_ms": result.renderer.prepare_text_ms if result.renderer else None,
                    "draws": result.renderer.draws if result.renderer else None,
                }
            )

    if args.iterations > 1:
        agg_path = run_dir / "summary_agg.csv"
        agg_path.write_text(
            "case,rows,cols,variable,profile,iterations,visible_rows_median,visible_cols_median,visible_cells_median,"
            "total_avg_median_ms,total_p95_median_ms,prepare_text_median_ms,draws_median\n",
            encoding="utf-8",
        )

        by_case: dict[str, list[dict[str, object]]] = {}
        for row in all_rows:
            by_case.setdefault(str(row["case"]), []).append(row)

        with agg_path.open("a", encoding="utf-8") as f:
            for case_name, rows in sorted(by_case.items(), key=lambda kv: kv[0]):
                first = rows[0]
                vr_med = _median([float(r["visible_rows"]) for r in rows if r["visible_rows"] is not None])
                vc_med = _median([float(r["visible_cols"]) for r in rows if r["visible_cols"] is not None])
                vcells_med = _median([float(r["visible_cells"]) for r in rows if r["visible_cells"] is not None])
                avg_med = _median([float(r["total_avg_ms"]) for r in rows if r["total_avg_ms"] is not None])
                p95_med = _median([float(r["total_p95_ms"]) for r in rows if r["total_p95_ms"] is not None])
                prep_med = _median([float(r["prepare_text_ms"]) for r in rows if r["prepare_text_ms"] is not None])
                draws_med = _median([float(r["draws"]) for r in rows if r["draws"] is not None])

                def fmt_int(v: float | None) -> str:
                    return f"{int(v):d}" if v is not None else ""

                def fmt_float(v: float | None) -> str:
                    return f"{v:.3f}" if v is not None else ""

                line = (
                    f"{case_name},{first['rows']},{first['cols']},{str(first['variable']).lower()},{profile},{args.iterations},"
                    f"{fmt_int(vr_med)},{fmt_int(vc_med)},{fmt_int(vcells_med)},"
                    f"{fmt_float(avg_med)},{fmt_float(p95_med)},{fmt_float(prep_med)},{fmt_int(draws_med)}\n"
                )
                f.write(line)

    print("\nWrote:")
    print(f"  {summary_path}")
    if args.iterations > 1:
        print(f"  {run_dir / 'summary_agg.csv'}")
    print(f"  {meta_path}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
