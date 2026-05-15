# Code Editor Row Content Snapshot Cache TODO

Status: Closed after M2.

- [x] Rename the row text cache payload into an explicit row content snapshot owner.
- [x] Route `cached_row_text_with_range` through the snapshot helper for compatibility callers.
- [x] Store row content snapshots on row scene cache entries.
- [x] Carry row content snapshots through row scene replay-plan payloads.
- [x] Remove prepaint replay planning calls to `cached_row_text_with_range` for scene-cache hits.
- [x] Strengthen the focused replay-plan test so prepaint planning no longer increments row text get
  calls for planned replay rows.
- [x] Convert snapshot payload movement to `Arc<RowContentSnapshot>` so cache/replay/paint clone only
  stable payload handles.
- [x] Run focused and package `fret-code-editor` gates.
- [x] Run `ui-code-editor-resize-probes` with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`.
- [x] Record residual follow-on: edge-row full path can still dominate the worst bundle.

Future work should start a new narrow lane for edge-row full-path replay/prefetch behavior if more
resize paint budget is needed.
