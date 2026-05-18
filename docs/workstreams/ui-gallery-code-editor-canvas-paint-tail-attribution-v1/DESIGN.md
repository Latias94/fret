# UI Gallery Code Editor Canvas Paint Tail Attribution v1

Status: Closed
Last updated: 2026-05-18

Status note (2026-05-18): this lane closed with a `fret-ui` positioned-container layout fix. The
code-editor `Canvas` tail was real row paint work, but the root owner was a wrong inner windowed
scroll viewport caused by final child sizing under an outer scroll overflow probe. See
`CPT_030_CPT_040_OWNER_PROOF_2026-05-18.md` and `CLOSEOUT_AUDIT_2026-05-18.md`.

## Why This Lane Exists

`view-cache-resize-jitter-attribution-v1` closed without a `ViewCache` runtime change. Its fresh
VCRJ-030 bundle shows the current UI Gallery code-editor resize-jitter worst frame is dominated by
`Canvas` paint work, not layout:

```text
total=362814us layout=1070us prepaint=1349us paint=360395us
Canvas paint_time_us=360009 inclusive_us=360009 scene_ops_delta=20009
```

The `Canvas` hotspot comes from the code-editor windowed rows surface path:

```text
ecosystem/fret-code-editor/src/editor/mod.rs:1390
ecosystem/fret-code-editor/src/editor/mod.rs:2079
ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs:783
ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs:859
```

This lane exists to decide whether that tail is real painting work, a code-editor row-surface cache
miss, a diagnostics attribution gap, or a renderer/canvas API limitation.

## Target State

Produce a source-backed owner verdict before changing runtime behavior:

- If the `Canvas` tail is real app/component work, identify the smallest code-editor or
  windowed-rows-surface optimization with a focused proof.
- If it is a diagnostics attribution gap, add the missing counters before optimizing.
- If it is renderer/canvas mechanism work, split the renderer follow-on with a clear boundary.
- If it is a one-off local scheduling artifact, record a no-change verdict with reproducible
  evidence.

## In Scope

- `ecosystem/fret-code-editor/src/editor/mod.rs`
  - torture editor canvas/root surface composition,
  - row paint callbacks,
  - row scene/cache interaction.
- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs`
  - canvas callback shape,
  - row iteration and visible-row bounds,
  - diagnostics hooks.
- `crates/fret-ui`
  - Canvas paint diagnostics and paint hotspot attribution.
- `crates/fret-diag`
  - stats/reporting only when needed to distinguish real canvas work from attribution gaps.

## Out Of Scope

- `ViewCache` runtime changes.
- `Scroll` layout optimization.
- Windows RTX 4090 performance baselines.
- Broad renderer redesign before the owner verdict is known.
- Recipe-level visual changes unless the source audit proves UI Gallery composition owns the tail.

## Starting Evidence

Fresh bundle:

- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json`

Layout summary:

- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/layout.perf.summary.v1.json`

Relevant observed fields:

- `paint_widget.hotspots canvas.top_exclusive_us(p50/p95/max)=354464/360009/360009`
- `paint_widget.hotspots canvas.sampled_sum_exclusive_us(p50/p95/max)=354464/360009/360009`
- `paint_widget.hotspots canvas.gap_p95_us(code_editor_total/surface_callback)=360009/360009`
- `code_editor.paint_perf` counters are zero in the same stats output, so the first audit must
  explain whether the callback is missing instrumentation or bypassing the code-editor paint perf
  path.

## First Questions

1. Which `Canvas` callback owns the 360ms exclusive paint time?
2. Why do `code_editor.paint_perf` counters report zero while the `Canvas` node reports the tail?
3. Is the tail caused by row iteration, row text/scene cache lookup, full scene capture, or
   diagnostics attribution around a callback?
4. Does a second fresh run reproduce the same `Canvas` signature, or is it a local outlier?

## Closeout Condition

Close this lane only after it records one of these outcomes:

- a focused optimization with tests/diag evidence;
- a diagnostics-first change with proof that the original owner was ambiguous;
- a renderer follow-on with a source-backed owner boundary;
- or a no-change verdict if the evidence does not reproduce.

Closeout result:

- Closed with a focused `fret-ui` runtime mechanism fix and regression tests.
- No renderer, `ViewCache`, or code-editor row-surface follow-on is split from this evidence set.
