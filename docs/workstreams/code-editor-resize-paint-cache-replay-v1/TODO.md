# Code Editor Resize Paint/Cache Replay TODO

Status: Closed after M1.

- [x] Capture baseline `ui-code-editor-resize-probes` with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`.
- [x] Attribute baseline with `diag stats` and record `code_editor_paint_perf` subfields.
- [x] Add M1 replay-plan short path so planned replay rows skip syntax/rich content probes in paint.
- [x] Add a focused regression assertion for replay-plan paint counters.
- [x] Run after-change `ui-code-editor-resize-probes` with the same command shape.
- [x] Compare baseline and after bundle `code_editor_paint_perf` fields.
- [x] Decide whether M2 should target row geom cache touch cost, prepaint-plan cost, or scene replay ops.
- [x] Add closeout audit when the lane has either a proven improvement or a documented next bottleneck.
