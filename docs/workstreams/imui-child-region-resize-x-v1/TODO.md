# ImUi Child Region ResizeX TODO

Status: closed
Last updated: 2026-05-16

Status note (2026-05-16): all tasks below landed in the closeout slice. Start a new narrow
follow-on for broader child-region behavior.

- [x] Resolve that this is a new narrow follow-on, not a reopen of
  `imui-child-region-resize-y-v1`.
- [x] Add `ChildRegionResizeXOptions` and `ChildRegionResizeXResponse` in `fret-ui-kit::imui`.
- [x] Render a right-edge column-resize pointer-region handle when `resize_x` is enabled.
- [x] Keep `resize_x` and `resize_y` composable on the same child region.
- [x] Add public smoke coverage for defaults, options, response accessors, and width clamping.
- [x] Run focused gates and record results in `EVIDENCE_AND_GATES.md`.
