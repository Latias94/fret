# Code Editor Row Content Snapshot Cache Milestones

## M1 - Snapshot Owner Split

Status: Done

Done criteria:

- Row text cache payload is represented as `RowContentSnapshot`.
- Row scene cache entries and replay-plan payloads carry row content snapshots.
- Prepaint replay planning reuses row scene cache content and does not call row text materialization
  for planned rows.
- Focused test asserts zero row text get calls during prepaint replay planning.

Evidence:

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint --features syntax-rust --no-fail-fast`
- Result: passed.

## M2 - Stable Snapshot Handle

Status: Done

Done criteria:

- `CodeEditorState.row_text_cache` stores `Arc<RowContentSnapshot>`.
- `RowSceneCacheEntry` stores `Arc<RowContentSnapshot>`.
- `RowSceneFragmentPayload` stores `Arc<RowContentSnapshot>`.
- Paint/replay paths clone only the snapshot handle when moving between cache, replay plan, and
  paint orchestration.

Evidence:

- `cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast`
- `cargo check -p fret-code-editor --features syntax-rust --all-targets`
- `ui-code-editor-resize-probes` after bundle:
  `target/fret-diag/code-editor-row-content-snapshot-cache-v1-after-m2-20260515/1778827921081/bundle.schema2.json`

## M3 - Edge-Row Full Path

Status: Future follow-on, not part of this lane

Rationale:

- After M2, the first two perf runs had `us_row_content_resolve.p95` at `110us` and `116us`.
- The worst run had `305us`, correlated with nonzero `us_row_text`, `us_row_rich_cache_compare`,
  and `us_row_geom_key`.
- That is an edge-row miss/full-path problem, not a reason to broaden global layout or view-cache
  architecture.
