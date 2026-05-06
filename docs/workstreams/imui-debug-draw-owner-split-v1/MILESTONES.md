# ImUi Debug Draw Owner Split v1 - Milestones

Status: closed
Last updated: 2026-05-06

## M0 - Baseline and lane scaffold

Exit criteria:

- Dedicated workstream folder exists.
- `WORKSTREAM.json` names the owner, repro, gates, and continue policy.
- Baseline audit records the file-size and ownership pressure.

Status: done.

## M1 - Command model owner

Exit criteria:

- `DebugDrawCommandKind`, `DebugDrawCommandSummary`, `DebugDrawListSummary`, and the private
  `DebugDrawCommand` enum live in `debug_draw_controls/commands.rs`.
- `debug_draw_controls.rs` remains the public helper and writer-extension owner.
- Focused debug draw tests pass.
- No public API names, defaults, or re-export paths change.

Status: done. See `M1_COMMAND_MODEL_SLICE_2026-05-06.md`.

## M2 - Paint owner

Exit criteria:

- Canvas painting and scene-op emission live in a private owner file.
- Debug draw list recording methods stay public-surface owned by `debug_draw_controls.rs`.
- Existing image/SVG/mesh/clip tests and smoke coverage still pass.

Status: done. See `M2_PAINT_DISPATCH_SLICE_2026-05-06.md`.

## M3 - Path/geometry owner

Exit criteria:

- Arc, bezier, ellipse, rect, polyline, and polygon sampling helpers live in a private owner file.
- Any shared finite/rounding helpers either stay local to paint/path or move into a small private
  geometry owner with no public exposure.

Status: done. See `M3_PATHS_SLICE_2026-05-06.md`.

## M4 - Geometry and paint helper owners

Exit criteria:

- Rect/point finiteness, triangle/index helpers, rect quad math, and rounding math live in
  `geometry.rs`.
- Opacity/UV validation, rounded-corner helpers, and low-level image/mesh paint emission live in
  `paint.rs`.
- Public debug draw names, defaults, and re-export paths remain stable.
- Focused debug draw tests and the compile floor still pass.

Status: done. See `M4_GEOMETRY_AND_PAINT_HELPERS_SLICE_2026-05-06.md`.

## Closeout

Exit criteria:

- The monolithic file is no longer the primary debug draw refactor hotspot.
- Future additive capabilities have a clear rule to start separate follow-ons.
- The lane has a closeout audit with commands run and residual gaps.

Status: done. See `CLOSEOUT_AUDIT_2026-05-06.md`.
