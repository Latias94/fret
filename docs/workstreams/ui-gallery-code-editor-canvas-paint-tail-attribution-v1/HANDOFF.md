# UI Gallery Code Editor Canvas Paint Tail Attribution v1 - Handoff

Status: Active
Last updated: 2026-05-18

## Current State

The lane is open as the follow-on to
`docs/workstreams/view-cache-resize-jitter-attribution-v1/`.

Starting evidence:

- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json`
- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/layout.perf.summary.v1.json`

Starting verdict:

- The current resize-jitter worst frame is paint-dominated:
  `total=362814us`, `paint=360395us`.
- The top paint hotspot is a `Canvas` node:
  `paint_time_us=360009`, `scene_ops_delta=20009`.
- The source path points through code-editor and windowed rows surface code.
- `code_editor.paint_perf` counters are zero in the same stats output, so source attribution comes
  before runtime changes.

CPT-020 source attribution is complete:

- `CPT_020_SOURCE_ATTRIBUTION_2026-05-18.md`
- The `Canvas` owner is the code-editor windowed rows surface callback.
- The VCRJ-030 bundle has `app_snapshot.code_editor.torture.paint_perf = null` for every snapshot
  because `FRET_CODE_EDITOR_DIAG_PAINT_PERF` was not enabled.
- The all-zero `code_editor.paint_perf frames=10` stats lines are a reporting artifact, not proof
  that row paint did no work.

## Next Task

Run CPT-030.

Goal:

- Capture a fresh same-script bundle with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`.
- Verify whether the `Canvas` paint tail repeats.
- If it repeats, split it by `us_windowed_surface_paint_callback`,
  `us_windowed_surface_row_paint`, `us_windowed_surface_non_row`, and row-scene/text counters.

Use:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json \
  --repeat 1 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_RENDERER_PERF=1 \
  --env FRET_LAYOUT_NODE_PROFILE=1 \
  --env FRET_LAYOUT_NODE_PROFILE_TOP=20 \
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=1 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

## Guardrails

- Keep `ViewCache` out of this lane.
- Keep `Scroll` layout out of this lane.
- Keep renderer redesign out until source evidence proves renderer ownership.
- Treat VCRJ-030 as local attribution evidence, not a portable performance baseline.
