# Layout perf summary v1 (tooling contract)

Status: Draft

Last updated: 2026-03-03

This document defines a **tooling-facing** (not runtime) contract for a bounded “layout perf summary”
viewer. The goal is to make a perf regression explainable without opening or grepping a huge
`bundle.json`.

This is intentionally derived from existing bundle stats output and is safe to extend additively.

---

## What it is

`layout perf summary v1` is a bounded JSON payload derived from a single bundle artifact
(`bundle.schema2.json` preferred; `bundle.json` as fallback). It focuses on:

- top frame identification (window/tick/frame),
- layout + layout-engine solve cost,
- the most relevant layout hotspots captured in the runtime debug snapshot.

It is intended for:

- “why did layout get slower?” debugging,
- capturing a small evidence artifact for CI or bug reports,
- comparing two bundles manually (diff the JSON).

---

## CLI viewer (source of truth)

The v1 viewer lives in:

- `crates/fret-diag/src/commands/layout_perf_summary.rs`

Usage:

- Human summary (default):
  - `fretboard diag layout-perf-summary <bundle_or_out_dir>`
- JSON (bounded) output:
  - `fretboard diag layout-perf-summary <bundle_or_out_dir> --json`
- Write to a file:
  - `fretboard diag layout-perf-summary <bundle_or_out_dir> --json --out exported.layout.perf.summary.v1.json`

Options:

- `--top <n>` controls how many entries are kept in each “top list”.
- `--warmup-frames <n>` (global) skips early frames when selecting the “top” frame.

---

## Output payload shape (v1)

Top-level object:

- `schema_version: 1`
- `kind: "layout_perf_summary"`
- `bundle_artifact: string` (path label)
- `bundle_dir: string` (resolved bundle root dir)
- `warmup_frames: u64`
- `top: u64` (viewer limit; applies to all lists)
- `frame: { window, tick_id, frame_id, window_snapshot_seq?, timestamp_unix_ms? }`
- `stats: { total_time_us, layout_time_us, layout_engine_solve_time_us, layout_engine_solves }`
- `top_layout_engine_solves: [ ... ]` (bounded to `top`)
- `layout_hotspots: [ ... ]` (bounded to `top`)
- `widget_measure_hotspots: [ ... ]` (bounded to `top`)

### `top_layout_engine_solves` entries

Each entry is derived from `debug.layout_engine_solves`:

- `solve_time_us: u64`
- `root_node: u64`
- `root_element?: u64`
- `root_element_kind?: string`
- `root_element_path?: string`
- `root_role?: string` (best-effort via semantics lookup)
- `root_test_id?: string` (best-effort via semantics lookup)
- `measure_calls: u64`
- `measure_cache_hits: u64`
- `measure_time_us: u64`
- `top_measures: [ ... ]` (bounded; currently a small fixed cap in stats extraction)

### `layout_hotspots` entries

Each entry is derived from `debug.layout_hotspots`:

- `layout_time_us: u64` (exclusive)
- `inclusive_time_us: u64`
- `node: u64`
- `element?: u64`
- `element_kind?: string`
- `element_path?: string`
- `widget_type?: string`
- `role?: string` (best-effort)
- `test_id?: string` (best-effort)

### `widget_measure_hotspots` entries

Each entry is derived from `debug.widget_measure_hotspots`:

- `measure_time_us: u64` (exclusive)
- `inclusive_time_us: u64`
- `node: u64`
- `element?: u64`
- `element_kind?: string`
- `element_path?: string`
- `widget_type?: string`
- `role?: string` (best-effort)
- `test_id?: string` (best-effort)

---

## Bounding rules (v1)

- The viewer MUST bound list outputs:
  - `top_layout_engine_solves.len() <= top`
  - `layout_hotspots.len() <= top`
  - `widget_measure_hotspots.len() <= top`
- The viewer MUST be safe to print to stdout by default:
  - default output is a small human summary,
  - JSON is bounded and can be redirected to a file with `--out`.

---

## Non-goals (v1)

- Not a runtime schema contract (no new on-wire fields required).
- Not a perf gate by itself (gating remains in `diag perf` thresholds/baselines).
- Not a “perfect correlation” tool: semantics ↔ layout-node correlation is best-effort.

