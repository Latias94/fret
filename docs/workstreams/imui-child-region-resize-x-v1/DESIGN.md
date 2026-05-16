# ImUi Child Region ResizeX v1

Status: closed execution follow-on
Last updated: 2026-05-16

Status note (2026-05-16): this lane closed after the horizontal resize proof landed. Keep future
child-region behavior growth in new proof-led follow-ons instead of widening this folder.

## Scope

This lane owns the horizontal counterpart to the closed
`imui-child-region-resize-y-v1` proof.

The target is intentionally narrow:

- add horizontal resize-handle policy for `fret-ui-kit::imui` child regions,
- keep width state app-owned through a response helper,
- reuse existing pointer-region drag mechanics and column-resize cursor behavior,
- allow `resize_x` and `resize_y` to be enabled together without duplicate root `test_id`s,
- keep `fret-imui` thin and unchanged as a runtime owner,
- avoid copying Dear ImGui's broad `BeginChild()` flag set.

## Assumptions

- Confident: the old child-region depth lane and the ResizeY lane are closed and should not be
  reopened.
  Evidence: `docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md` and
  `docs/workstreams/imui-child-region-resize-y-v1/CLOSEOUT_AUDIT_2026-05-15.md`.
  Consequence if wrong: this follow-on would duplicate scope, so closeout/readme state must be
  refreshed before landing.
- Confident: `ResizeX` is a legitimate bounded follow-on because Dear ImGui exposes axis-specific
  child resize flags and Fret already proved the vertical response pattern.
  Evidence: `repo-ref/imgui/imgui.h` and the closed ResizeY workstream.
  Consequence if wrong: keep only the docs and do not widen the public options surface.
- Likely: app-owned width state is the correct Fret translation.
  Evidence: Fret's declarative element tree externalizes cross-frame state, and table/child resize
  responses expose drag deltas instead of mutating caller data.
  Consequence if wrong: introduce a separate model-backed helper in a later lane, not hidden
  runtime state in `fret-imui`.

## Target API

`ChildRegionOptions` gains an optional `resize_x` field. When absent, behavior is unchanged.

The resize option carries:

- optional minimum width,
- optional maximum width,
- optional handle `test_id`.

`child_region_with_options(...)` returns a `ChildRegionResponse` that can be ignored by existing
callers. When `resize_x` is enabled, the response exposes:

- whether horizontal resize is enabled,
- drag started/dragging/stopped,
- delta and total X movement,
- `width_from_start(start_width)` with min/max clamping.

The helper does not persist width. Callers apply the returned width to their next frame through
their own model/state.

## Non-Goals

- No auto-resize or always-auto-resize.
- No `BeginChild() -> bool` visibility-return contract.
- No focus-boundary flattening.
- No diagnostics pointer-drag script in this slice.
- No new dependency from `fret-imui` to `fret-ui-kit`.
