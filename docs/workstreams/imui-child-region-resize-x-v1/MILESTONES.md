# ImUi Child Region ResizeX Milestones

Status: closed
Last updated: 2026-05-16

Status note (2026-05-16): M0-M2 are complete; see `CLOSEOUT_AUDIT_2026-05-16.md`.

## M0 - Lane Open

Exit criteria:

- Workstream docs exist.
- Scope names only horizontal resize and axis composability.
- Gates name the existing child-region proof floor.

## M1 - API And Behavior

Exit criteria:

- `ChildRegionOptions::resize_x` is optional and defaults to disabled.
- `child_region_with_options(...)` returns a response without forcing existing callers to consume
  it.
- The resize handle uses column-resize cursor and existing pointer-region drag response plumbing.
- Width calculation remains app-owned through `width_from_start(...)`.
- Enabling `resize_x` does not duplicate root `test_id` ownership between the scroll area and the
  outer resize stack.

## M2 - Gates And Evidence

Exit criteria:

- Focused `fret-ui-kit` child-region smoke passes.
- Focused resize helper tests pass.
- IMUI source gate and workstream catalog pass.
