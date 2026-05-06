# M1 Command Model Slice - 2026-05-06

Status: command-model owner split landed; lane remains active for paint/path owner splits.

## What Changed

- Added `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands.rs`.
- Moved the command-model owner into that file:
  - `DebugDrawCommandKind`
  - `DebugDrawCommandSummary`
  - `DebugDrawListSummary`
  - private `DebugDrawCommand`
  - command-to-summary conversion and aggregate list summary accounting.
- Kept `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs` as the public helper and writer
  extension owner.
- Kept `ecosystem/fret-ui-kit/src/imui.rs` re-export paths unchanged through the parent module.

## Public Surface Verdict

No public API names, defaults, method names, or re-export paths changed.

The private command enum now uses `pub(super)` visibility so the parent module can continue to
record, summarize, paint, and test commands without exposing raw command variants to downstream
users.

## Size Movement

- `debug_draw_controls.rs`: 4519 lines before M1, 3994 lines after M1.
- `debug_draw_controls/commands.rs`: 539 lines after M1.

This does not finish the owner split. The paint and path helpers still dominate the parent file and
remain the next structural candidates.

## Gates Run Before This Note

```bash
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
```

Result: both passed locally on 2026-05-06.

## Post-note Gates

```bash
cargo fmt --package fret-ui-kit -- --check
python tools/check_workstream_catalog.py
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
python -m json.tool docs/workstreams/imui-debug-draw-owner-split-v1/WORKSTREAM.json
git diff --check
```

Result: all passed locally on 2026-05-06.

## Next Slice

Split canvas painting into a private `paint.rs` owner only after the current source/workstream gates
pass with the command model owner in place.
