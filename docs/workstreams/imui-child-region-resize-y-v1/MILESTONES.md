# ImUi Child Region ResizeY Milestones

Status: closed
Last updated: 2026-05-15

Status note (2026-05-15): M0-M2 are complete; see `CLOSEOUT_AUDIT_2026-05-15.md`.

## M0 - Lane Open

Exit criteria:

- Workstream docs exist.
- Scope names only vertical resize.
- Gates name the existing child-region floor.

## M1 - API And Behavior

Exit criteria:

- `ChildRegionOptions::resize_y` is optional and defaults to disabled.
- `child_region_with_options(...)` returns a response without forcing existing callers to consume
  it.
- The resize handle uses row-resize cursor and existing pointer-region drag response plumbing.
- Height calculation remains app-owned through `height_from_start(...)`.

## M2 - Gates And Evidence

Exit criteria:

- Focused `fret-ui-kit` child-region smoke passes.
- Focused `fret-imui` child-region composition tests pass.
- IMUI source gate and workstream catalog pass.
