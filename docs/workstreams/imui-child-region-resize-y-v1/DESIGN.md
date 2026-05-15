# ImUi Child Region ResizeY v1

Status: closed execution follow-on
Last updated: 2026-05-15

Status note (2026-05-15): this lane closed after the vertical resize proof landed. Keep future
child-region behavior growth in a new proof-led follow-on instead of widening this folder.

## Scope

This lane owns the first generic child-region resize behavior admitted after the closed
`imui-child-region-depth-v1` chrome slice.

The target is intentionally narrow:

- add vertical resize-handle policy for `fret-ui-kit::imui` child regions,
- keep height state app-owned through a response helper,
- reuse existing pointer-region drag mechanics and row-resize cursor behavior,
- keep `fret-imui` thin and unchanged as a runtime owner,
- avoid copying Dear ImGui's broad `BeginChild()` flag set.

## Assumptions

- Confident: the old child-region depth lane is closed and should not be reopened.
  Evidence: `docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`.
  Consequence if wrong: this follow-on would duplicate scope, so closeout/readme state must be
  refreshed before landing.
- Confident: `ResizeY` is the strongest first child-region behavior candidate.
  Evidence: `docs/workstreams/imui-imgui-gap-closure-v1/P3_CHILD_REGION_READINESS_2026-05-06.md`.
  Consequence if wrong: this lane should stop at documentation and not widen the API.
- Likely: app-owned height state is the correct Fret translation.
  Evidence: Fret's declarative element tree already externalizes cross-frame state, and table
  column resize responses expose drag deltas instead of mutating caller data.
  Consequence if wrong: introduce a separate model-backed helper in a later lane, not hidden
  runtime state in `fret-imui`.

## Target API

`ChildRegionOptions` gains an optional `resize_y` field. When absent, behavior is unchanged.

The resize option carries:

- optional minimum height,
- optional maximum height,
- optional handle `test_id`.

`child_region_with_options(...)` returns a `ChildRegionResponse` that can be ignored by existing
callers. When `resize_y` is enabled, the response exposes:

- whether vertical resize is enabled,
- drag started/dragging/stopped,
- delta and total Y movement,
- `height_from_start(start_height)` with min/max clamping.

The helper does not persist height. Callers apply the returned height to their next frame through
their own model/state.

## Non-Goals

- No `ResizeX`.
- No auto-resize or always-auto-resize.
- No `BeginChild() -> bool` visibility-return contract.
- No focus-boundary flattening.
- No new dependency from `fret-imui` to `fret-ui-kit`.
