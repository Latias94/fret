# ImUi Child Region ResizeY TODO

Status: closed
Last updated: 2026-05-15

Status note (2026-05-15): all tasks below landed in the closeout slice. Start a new narrow
follow-on for broader child-region behavior.

- [x] Resolve that this is a new narrow follow-on, not a reopen of
  `imui-child-region-depth-v1`.
- [x] Add `ChildRegionResizeYOptions` and `ChildRegionResponse` in `fret-ui-kit::imui`.
- [x] Render a bottom row-resize pointer-region handle when `resize_y` is enabled.
- [x] Add public smoke coverage for defaults, options, response accessors, and height clamping.
- [x] Add composition coverage proving the handle renders without breaking existing child-region
  scroll/chrome behavior.
- [x] Run focused gates and record results in `EVIDENCE_AND_GATES.md`.
