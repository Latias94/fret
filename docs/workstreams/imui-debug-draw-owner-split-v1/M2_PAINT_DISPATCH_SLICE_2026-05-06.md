# M2 Paint Dispatch Slice - 2026-05-06

Status: paint command-dispatch owner split landed; lane remains active for path/geometry owner
split.

## What Changed

- Added `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint.rs`.
- Moved `paint_debug_draw_commands(...)` into that owner.
- Kept low-level image/mesh/path helper functions in `debug_draw_controls.rs` for this slice so
  the move stayed bounded.
- Kept public draw-list recording methods, response types, options, and re-export paths unchanged.

## Public Surface Verdict

No public API names, defaults, method names, or re-export paths changed.

The paint owner stays private to `debug_draw_controls` and only exposes
`paint_debug_draw_commands(...)` with `pub(super)` visibility.

## Size Movement

- `debug_draw_controls.rs`: 3994 lines before M2, 3431 lines after M2.
- `debug_draw_controls/commands.rs`: 539 lines after M2.
- `debug_draw_controls/paint.rs`: 583 lines after M2.

## Gates Run

```bash
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
cargo fmt --package fret-ui-kit -- --check
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
python -m json.tool docs/workstreams/imui-debug-draw-owner-split-v1/WORKSTREAM.json
git diff --check
```

Result: all passed locally on 2026-05-06.

## Next Slice

Split path sampling and shape conversion helpers into a private `paths.rs` owner. Keep low-level
image/mesh helpers in place until the path/geometry split shows whether they need a dedicated
`geometry.rs` owner or can remain parent-local.
