# ImUi Debug Draw Owner Split v1 - Evidence & Gates

Goal: make the debug draw owner split reviewable without changing public API or behavior.

Status: closed
Last updated: 2026-05-06

## Evidence anchors

- `docs/workstreams/imui-debug-draw-owner-split-v1/WORKSTREAM.json`
- `docs/workstreams/imui-debug-draw-owner-split-v1/DESIGN.md`
- `docs/workstreams/imui-debug-draw-owner-split-v1/M0_BASELINE_AUDIT_2026-05-06.md`
- `docs/workstreams/imui-debug-draw-owner-split-v1/M1_COMMAND_MODEL_SLICE_2026-05-06.md`
- `docs/workstreams/imui-debug-draw-owner-split-v1/M2_PAINT_DISPATCH_SLICE_2026-05-06.md`
- `docs/workstreams/imui-debug-draw-owner-split-v1/M3_PATHS_SLICE_2026-05-06.md`
- `docs/workstreams/imui-debug-draw-owner-split-v1/M4_GEOMETRY_AND_PAINT_HELPERS_SLICE_2026-05-06.md`
- `docs/workstreams/imui-debug-draw-owner-split-v1/CLOSEOUT_AUDIT_2026-05-06.md`
- `docs/workstreams/imui-debug-draw-owner-split-v1/TODO.md`
- `docs/workstreams/imui-debug-draw-owner-split-v1/MILESTONES.md`
- `docs/workstreams/imui-debug-draw-owner-split-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P1_CLOSEOUT_AUDIT_2026-05-06.md`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/geometry.rs`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint.rs`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests.rs`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
- `docs/workstreams/README.md`
- `docs/roadmap.md`
- `docs/todo-tracker.md`

## First-open repro

Use this as the smallest behavior-preservation proof:

```bash
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
```

This catches the public smoke test plus private debug draw command/path/summary tests whose names
include `debug_draw`.
The private test owner now lives in `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests.rs`.

## Current gates

### Focused debug draw test floor

```bash
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
```

### Compile floor

```bash
cargo check -p fret-ui-kit --features imui
```

### Format floor

```bash
cargo fmt --package fret-ui-kit -- --check
```

### Workstream and source-policy floors

```bash
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
python -m json.tool docs/workstreams/imui-debug-draw-owner-split-v1/WORKSTREAM.json
git diff --check
```

## Non-gates

Do not use this lane to validate renderer draw-call attribution, raw buffer access, callbacks, or
per-geometry picking. Those require separate follow-ons because they change the behavioral surface.
