# M3 Paths Slice - 2026-05-06

Status: path/shape sampling owner split landed; lane remains active for any later geometry owner
split.

## What Changed

- Added `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths.rs`.
- Moved the path sampling and shape-conversion helpers into that owner.
- Kept the low-level image/mesh helpers and geometry rounding helpers parent-local for now.
- Kept public debug draw names, defaults, and re-export paths unchanged.

## Public Surface Verdict

No public API names, defaults, method names, or re-export paths changed.

`paths.rs` stays private and only exposes `pub(super)` helpers to the parent module and its
children.

## Size Movement

- `debug_draw_controls.rs`: 3431 lines before M3, 3057 lines after M3.
- `debug_draw_controls/commands.rs`: 539 lines after M3.
- `debug_draw_controls/paint.rs`: 583 lines after M3.
- `debug_draw_controls/paths.rs`: 401 lines after M3.

## Gates Run

```bash
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
cargo fmt --package fret-ui-kit -- --check
```

Result: all passed locally on 2026-05-06.

## Next Slice

Decide whether the remaining low-level geometry helpers should stay parent-local or move into a
small private `geometry.rs` owner. For now, keep that pressure separate so the path split stays
reviewable.
