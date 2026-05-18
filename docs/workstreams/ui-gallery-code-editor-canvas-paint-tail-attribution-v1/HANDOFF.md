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

## Next Task

Run CPT-020.

Goal:

- Map the code-editor `Canvas` callback, windowed rows surface, row scene/cache paths, and
  diagnostics counters to source owners.
- Explain why `code_editor.paint_perf` is zero while `Canvas` paint owns the tail.
- Decide whether CPT-030 needs extra instrumentation, a direct fresh repro, or a focused runtime
  proof.

Start with:

```bash
rg -n "windowed_rows_surface|Canvas|paint_perf|row_scene|surface_callback|torture" \
  ecosystem/fret-code-editor \
  ecosystem/fret-ui-kit/src/declarative \
  crates/fret-ui \
  crates/fret-diag \
  -S
```

## Guardrails

- Keep `ViewCache` out of this lane.
- Keep `Scroll` layout out of this lane.
- Keep renderer redesign out until source evidence proves renderer ownership.
- Treat VCRJ-030 as local attribution evidence, not a portable performance baseline.
