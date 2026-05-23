# Editor Canvas Paint Replay Row Setup v1

Status: Closed on 2026-05-24 after target-machine validation and attribution. This lane added
diagnostics-only planned replay setup attribution; remaining optimization work is split to a new
bounded Canvas replay follow-on.

## Problem

The r63 resource-touch closeout kept the parent `ui-perf-zed-smoothness-v1` owner at
`canvas-paint-replay`. Hosted-resource touch aggregation reduced one repeated hot-path operation,
but the target-machine attribution still reports Canvas replay and windowed row paint as the
dominant owner:

- complex-wheel: `paint_widget_p95=516us`, `canvas_exclusive_p95=370us`,
  `code_editor_total_p95=314us`, `row_paint_p95=335us`.
- typical-autoscroll: `paint_widget_p95=482us`, `canvas_exclusive_p95=356us`,
  `code_editor_total_p95=309us`, `row_paint_p95=327us`.
- resize-jitter: `paint_widget_p95=511us`, `canvas_exclusive_p95=309us`,
  `code_editor_total_p95=237us`, `row_paint_p95=254us`.

After r63, existing counters split replay touch, replay ops, prepaint planning, geom cache, text,
and overlay work. They do not isolate the planned-replay setup performed by `paint_row` before
`scene::replay_row_scene_plan_entry`, so the remaining row-paint overhead is under-attributed.

## Target State

- Code-editor paint diagnostics expose `us_row_scene_replay_setup` and
  `ns_row_scene_replay_setup`.
- `fret-ui-gallery` app snapshots include the field in the code-editor torture paint-perf payload.
- `fret-diag stats` extracts, aggregates, percentiles, and prints the new metric.
- The new field is diagnostics-only and does not change row replay order, overlay behavior,
  retained fragment semantics, or checked-in perf baselines.

## Scope

- `ecosystem/fret-code-editor/src/editor/diagnostics.rs`
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`
- `crates/fret-diag/src/stats.rs`
- `crates/fret-diag/src/stats/bundle_stats_compute.inc.rs`
- `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`
- Parent workstream accounting under `docs/workstreams/ui-perf-zed-smoothness-v1/`

## Non-Goals

- No row replay batching.
- No Canvas hosted-resource policy change.
- No renderer encode/upload change.
- No checked-in baseline change from this diagnostics slice.
- No reopening of the closed plan-cache or resource-touch lanes unless fresh evidence proves a
  mechanism regression.

## Design

`paint_row` already records a row-local `us_total` and several sub-counters. The planned replay
path still performs setup before replaying retained ops: plan entry lookup, retained row-content
resolution, row range clone, scale and stable width setup, baseline cache check, origin/key/constraint
preparation, overlay touch check, and plan-resource acquisition.

The first slice adds a narrow diagnostics counter around that setup and records it only when a
matching planned replay entry is actually replayed. This keeps old bundles readable through
`fret-diag`'s existing ns/us fallback logic while making new target-machine attribution clear
enough to decide the next optimization owner.
