# ImUi Debug Draw Baseline v1 TODO

Status: Closed
Last updated: 2026-05-04

## M1 - Debug Draw Baseline

- [x] Add a canvas-backed debug draw facade helper.
- [x] Support line, rect, filled rect, and text primitives.
- [x] Add smoke coverage for the new facade API.
- [x] Keep the implementation in `fret-ui-kit::imui`, not `fret-imui`.

## Future Follow-Ons

- [x] Add dashed/path styles and richer line rendering. See
  `docs/workstreams/imui-debug-draw-stroke-style-v1/CLOSEOUT_AUDIT_2026-05-04.md`.
- [x] Add image overlay follow-on. See
  `docs/workstreams/imui-debug-draw-image-overlay-v1/CLOSEOUT_AUDIT_2026-05-04.md`.
- [ ] Add custom paint metadata, channel splitting, hit-test, or image loading recipe follow-ons
  only when a real first-party consumer needs them.
