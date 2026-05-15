# Closeout Audit

Date: 2026-05-15
Status: Closed

## Objective

Close the narrow follow-on after `code-editor-resize-paint-cache-replay-v1` by making row content a
stable snapshot payload shared by the row text cache, row scene cache, replay plan, and paint path.

Explicit scope:

- reduce repeated `cached_row_text_with_range` work in prepaint replay planning,
- reduce tuple-style movement of row range, row text, fold map, preedit range, and row spans,
- keep replay-plan hits on a stable row content payload,
- verify with `ui-code-editor-resize-probes` and `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`,
- avoid global `fret-ui` layout/view-cache refactors.

## Shipped Change

- `ecosystem/fret-code-editor/src/editor/state.rs`
  - Replaced the old row text cache entry shape with `RowContentSnapshot`.
  - Stored row text cache entries as `Arc<RowContentSnapshot>`.
  - Added snapshot payloads to row scene cache entries and row scene replay-plan payloads.
- `ecosystem/fret-code-editor/src/editor/paint/text.rs`
  - Added `cached_row_content_snapshot`.
  - Kept `cached_row_text_with_range` as a compatibility wrapper over snapshot parts.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
  - Reused row scene cache snapshots while building prepaint replay plans.
  - Stopped calling `cached_row_text_with_range` for scene-cache replay candidates.
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
  - Resolved row content once from either replay-plan payload or row text cache.
  - Stored the same snapshot handle into the row scene cache.
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
  - Strengthened the replay-plan test so prepaint planning must reuse row scene cache snapshots.

No `crates/fret-ui` layout, view-cache, scroll, virtual-list, text prepare, or scene replay
infrastructure files were changed.

## Evidence

Focused and package gates:

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint --features syntax-rust --no-fail-fast`
  - Result: passed (`1` test).
- `cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast`
  - Result: passed (`129` tests).
- `cargo check -p fret-code-editor --features syntax-rust --all-targets`
  - Result: passed.

Perf gate:

- command: see `EVIDENCE_AND_GATES.md`
- worst bundle:
  `target/fret-diag/code-editor-row-content-snapshot-cache-v1-after-m2-20260515/1778827921081/bundle.schema2.json`
- aggregate p95: total `1418us`, paint `866us`, prepaint `347us`

Prior comparison bundle:

- `target/fret-diag/code-editor-resize-paint-cache-replay-v1-after-m1-20260515/1778822452927/bundle.schema2.json`
- prior aggregate p95: total `1469us`, paint `848us`, prepaint `335us`

Row-content result:

- M2 repeated-run p95 values: `110us`, `116us`, `305us`
- prior after-M1 worst-bundle p95: `283us`

The median/repeated replay-hit path improved materially. The worst M2 bundle still has an edge-row
full-path spike (`us_row_text=12us`, `us_row_rich_cache_compare=23us`, `us_row_geom_key=55us`).

## Prompt-To-Artifact Checklist

- Row content snapshot has a clear owner:
  `CodeEditorState.row_text_cache`, `RowSceneCacheEntry`, and `RowSceneFragmentPayload` all carry
  `Arc<RowContentSnapshot>`.
- Repeated `cached_row_text_with_range` work is reduced:
  prepaint replay planning reuses `cached.content`, and the focused test asserts row text get calls
  do not increase for planned replay rows.
- Replay-plan/cache entry carries stable payload:
  replay payload stores the snapshot handle from row scene cache.
- Replay hit only does necessary overlay/geometry touch:
  paint derives local row content from the replay payload and skips syntax/rich probing on planned
  replay rows.
- Required proof surface:
  `ui-code-editor-resize-probes` ran with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`.
- Priority guidance:
  no global layout/view-cache refactor was done; remaining work is a narrow edge-row full-path lane.

## Residual Risk And Follow-On

This lane should not continue into global architecture work. The next useful optimization, if more
resize paint budget is required, is a narrow follow-on for edge-row full-path behavior during resize:
prefetch or carry the one newly exposed row's rich/geom payload so it does not dominate the worst
frame.
