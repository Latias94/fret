# M1 Plain Cached Replay

Date: 2026-05-15
Status: Landed slice

## Objective

Reduce resize paint work for cached plain rows that already have row scene cache entries but were
excluded from prepaint replay planning because they do not carry syntax replay keys.

## Shipped Change

- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
  - `replay_row_scene_plan_candidates_for_frame` now accepts cached plain row scene entries in
    addition to syntax replay entries.
  - Syntax rows still validate through syntax replay keys and syntax spans.
  - Plain rows validate with `RowSceneKey::plain(cached_row_geom_key, fg)`.
  - The validation intentionally reuses the cached entry's `RowGeomKey` so pointer-identity fields
    are not rebuilt during prepaint planning.
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
  - Added `prepaint_row_scene_replay_plan_handles_plain_cached_rows`.
  - The test seeds plain row scene cache entries, resizes the editor, and asserts that planned rows
    are consumed in paint without row text work.

## Evidence

Gates:

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan --features syntax-rust --no-fail-fast`
  - Result: passed (`2` tests).
- `cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast`
  - Result: passed (`130` tests).
- `cargo check -p fret-code-editor --features syntax-rust --all-targets`
  - Result: passed.
- `cargo fmt --check`
  - Result: passed.
- `git diff --check`
  - Result: passed.

Perf:

- after M1 worst bundle:
  `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m1-20260515/1778830062977/bundle.schema2.json`
- previous comparison bundle:
  `target/fret-diag/code-editor-row-content-snapshot-cache-v1-after-m2-20260515/1778827921081/bundle.schema2.json`

Key comparison:

- `code_editor_paint_perf.p95.us_total`: `394us` -> `371us`
- `us_row_content_resolve`: `305us` -> `281us`
- `us_row_text`: `12us` -> `6us`
- `us_row_rich_cache_compare`: `23us` -> `20us`
- `us_row_geom_key`: `55us` -> `53us`
- `us_row_scene_prepaint_plan`: `70us` -> `111us`

## Verdict

M1 is worth landing as a bounded improvement: it moves more cached rows into the replay-hit shape
and reduces code-editor-owned paint work. It does not complete the full lane objective because rows
with no row scene cache entry still cannot get a complete scene payload during prepaint.

The next slice should classify remaining edge-row misses and reduce replay-plan candidate cost
before adding a broader payload prebuild mechanism.
