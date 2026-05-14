# M4G Previous-Frame Paint Recording Slice

Date: 2026-05-14
Status: Landed as a local ownership split; global paint-cache replay owner still open

## Why

M4F deleted the node-owned `PaintCacheEntry` fallback, but ordinary retained paint-cache replay
still depended on `PaintCacheState` fields that mixed two responsibilities:

- frame/replay control: generation, source/target generation, hit/miss counters, replay counters;
- previous-frame recording storage: scene ops, text blob side index, and scene fingerprint.

That made the next owner decision harder to review because moving `PaintCacheState` wholesale would
also move unrelated control counters. M4G narrows the next step by making the previous-frame
recording a named owner.

## Change

- Added `PreviousFramePaintRecording` in `crates/fret-ui/src/tree/paint_cache.rs`.
- Moved previous-frame scene ops, text blob ids, and fingerprint into that recording carrier.
- Changed `UiTree::ingest_paint_cache_source(...)` to delegate scene storage ingestion to the
  recording carrier.
- Changed paint replay to read previous ops through `self.paint_cache.previous_frame.ops`.
- Added a focused test proving scene ingestion moves ops into the recording carrier and disabling
  paint cache clears that carrier.

## What This Deletes Or Avoids

- Deletes the anonymous `PaintCacheState::prev_ops`, `prev_text_blob_ids`, and `prev_fingerprint`
  fields.
- Avoids treating the remaining previous-frame recording as final `ViewBoundaryState` ownership
  before the replay-key and plain-entry side-store decisions are finished.
- Avoids reintroducing node-owned paint-cache entry storage.

## Gates

Focused gates:

```bash
cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast
cargo check -p fret-ui --features diagnostics --all-targets
```

Observed result on 2026-05-14:

- `tree::tests::paint_cache`: `11 passed, 929 skipped`.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.

Conflict-resolution gates rerun after pulling upstream:

```bash
python3 tools/check_diag_scripts_registry.py
python3 tools/check_workstream_catalog.py
cargo check -p fret-ui --all-targets
git diff --check
```

Observed result:

- diagnostics script registry: up to date;
- workstream catalog: `370 dedicated directories, 47 standalone markdown files`;
- `cargo check -p fret-ui --all-targets`: passed;
- `git diff --check`: passed.

## Remaining Work

- Decide whether `PreviousFramePaintRecording` migrates into `ViewBoundaryState` directly, becomes a
  boundary-owned scene-fragment source, or remains an explicitly retained per-tree recording
  mechanism with an accepted reason.
- Decide whether `UiTree::boundary_paint_cache_entries` is the final shape for plain retained
  paint-cache entries or a transition before a broader boundary table.
- Add the second non-code-editor proof surface before global closeout.
