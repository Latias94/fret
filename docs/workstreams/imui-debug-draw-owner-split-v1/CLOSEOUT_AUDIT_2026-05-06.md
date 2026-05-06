# ImUi Debug Draw Owner Split v1 - Closeout Audit - 2026-05-06

Status: closed
Last updated: 2026-05-06

## Objective

Close `imui-debug-draw-owner-split-v1` by:

1. deciding whether private debug draw tests should be split into owner-specific test modules,
2. performing the split only if it is worth the added boundary cost,
3. running the focused debug draw tests plus compile/format/source-policy gates,
4. updating the workstream evidence, and
5. leaving a clear closeout record.

## Completion checklist

| Requirement | Evidence |
| --- | --- |
| Private owner split completed without public API widening | `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`, `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands.rs`, `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint.rs`, `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths.rs`, `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/geometry.rs` |
| Command, paint, path, and geometry helper ownership separated | same files as above, plus M1-M4 slice notes |
| Private tests reviewed and given an explicit ownership verdict | `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests.rs`, this audit |
| Focused debug draw tests passed | `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast` |
| Compile floor passed | `cargo check -p fret-ui-kit --features imui` |
| Format floor passed | `cargo fmt --package fret-ui-kit -- --check` |
| Workstream and source-policy gates passed | `python tools/check_workstream_catalog.py`, `python tools/gate_imui_workstream_source.py`, `python tools/gate_imui_facade_teaching_source.py`, `python -m json.tool docs/workstreams/imui-debug-draw-owner-split-v1/WORKSTREAM.json`, `git diff --check` |
| Workstream metadata updated for closeout | `docs/workstreams/imui-debug-draw-owner-split-v1/WORKSTREAM.json`, `docs/workstreams/imui-debug-draw-owner-split-v1/TODO.md`, `docs/workstreams/imui-debug-draw-owner-split-v1/MILESTONES.md`, `docs/workstreams/imui-debug-draw-owner-split-v1/EVIDENCE_AND_GATES.md`, `docs/workstreams/imui-debug-draw-owner-split-v1/DESIGN.md` |

## Decision on tests

The tests were moved out of `debug_draw_controls.rs` into a private `debug_draw_controls/tests.rs`
owner. A further split into `commands.rs`, `paint.rs`, or `paths.rs` test files was judged not worth
the extra boundary noise, because the current test suite intentionally validates the parent façade
and multiple private owners together.

This is the explicit no-split verdict for source-owner-specific test modules.

## Residual gaps

- None for this lane.
- Later additive draw-list capabilities, renderer attribution, raw buffers, callbacks, and
  per-geometry picking still belong in separate follow-on lanes.

## Outcome

The monolithic file is no longer the primary debug draw refactor hotspot, and the lane is now
closed with clear evidence and a durable ownership map.

Implementation note: `WORKSTREAM.json` is now marked `closed` and `stay_closed` so the
machine-readable lane state matches this closeout audit. Additive debug draw work must start as a
separate follow-on with its own proof and gates.
