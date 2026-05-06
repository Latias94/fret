# M4 Geometry And Paint Helpers Slice - 2026-05-06

Status: geometry and low-level paint helpers split landed; lane remains active for the private test
colocation decision.

## What Changed

- Added `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/geometry.rs`.
- Moved rect/point finiteness checks, triangle/index helpers, rect quad math, and effective
  rounded-rect math into the geometry owner.
- Moved normalized opacity, UV validation, rounded-corner visibility, rounded image clipping
  helpers, and low-level image/mesh scene emission into `paint.rs`.
- Removed the old helper block from `debug_draw_controls.rs`, leaving the parent file focused on
  public helper types, draw-list recording, path-builder glue, element construction, and tests.
- Kept private tests colocated in `debug_draw_controls.rs` for now.

## Public Surface Verdict

No public API names, defaults, method names, or re-export paths changed.

The new helper owner is private. `geometry.rs` and the new paint helpers only expose `pub(super)`
items to the debug draw implementation and its tests.

## Size Movement

- `debug_draw_controls.rs`: 3057 lines before M4, 2548 lines after M4.
- `debug_draw_controls/paint.rs`: 583 lines before M4, 716 lines after M4.
- `debug_draw_controls/paths.rs`: 401 lines before M4, 363 lines after M4.
- `debug_draw_controls/geometry.rs`: 114 lines after M4.

## Gates Run

```bash
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
cargo fmt --package fret-ui-kit
```

Result: all passed locally on 2026-05-06.

## Next Slice

Decide whether private tests should remain colocated in `debug_draw_controls.rs` or move into
owner-specific test modules. Keep additive draw-list capabilities out of this lane.
