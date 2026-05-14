# M4H Previous-Frame Paint Replay Span Slice

Date: 2026-05-14
Status: Landed as a replay-ownership narrowing step; final paint-cache recording owner still open

## Why

M4G named the previous-frame paint recording as `PreviousFramePaintRecording`, but `paint_node`
still owned too much of the replay contract:

- it validated `PaintCacheEntry` op ranges against the previous-frame op buffer directly;
- it sliced `previous_frame.ops[start..end]` itself;
- it replayed through the generic scene replay path, which rescanned replayed ops to rebuild the
  text blob side index;
- `PaintCacheEntry` only described op spans, not the matching precomputed text blob side-index span.

That kept the hot replay path correct for geometry, but left the previous-frame recording carrier as
storage rather than the owner of its own replay invariants. M4H narrows that owner boundary before
the final decision about whether the recording moves into `ViewBoundaryState`, feeds
boundary-owned scene fragments, or remains a retained per-tree recording mechanism.

## Change

- Added `text_blob_start` / `text_blob_end` to `PaintCacheEntry`.
- Made `PreviousFramePaintRecording::ops` private.
- Added `PreviousFramePaintRecording::is_entry_replayable(...)` and
  `PreviousFramePaintRecording::replay_entry_translated(...)`.
- Moved previous-frame entry range validation and replay slicing into `PreviousFramePaintRecording`.
- Replayed cached paint entries with `Scene::replay_ops_translated_with_text_blob_ids(...)`, so
  cache hits preserve the precomputed text blob side index instead of rescanning retained ops.
- Updated normal paint and replayed paint to carry the emitted text blob side-index span into the
  next `PaintCacheEntry`.
- Added a focused test proving previous-frame paint replay preserves the text blob side index.

## What This Deletes Or Avoids

Deleted from the caller path:

- direct reads of `PreviousFramePaintRecording::ops` from `paint_node`;
- direct op-range validation in `paint_node`;
- direct op-slice replay in `paint_node`.

Avoided:

- moving `PreviousFramePaintRecording` into `ViewBoundaryState` before the plain-entry side-store
  and second proof-surface decisions are complete;
- restoring node-owned paint-cache entry storage;
- making `paint_node` the long-term owner of previous-frame text side-index span integrity.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/tree/paint_cache.rs`
- `crates/fret-ui/src/tree/paint/node.rs`
- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/tests/paint_cache.rs`

Correctness gates:

```bash
cargo fmt
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null
git diff --check
```

Observed results:

- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- `tree::tests::paint_cache`: `12 passed, 929 skipped`.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `WORKSTREAM.json` JSON validation: passed.
- `git diff --check`: passed.

## Remaining Work

- Decide whether `PreviousFramePaintRecording` migrates into `ViewBoundaryState`, becomes a
  boundary-owned scene-fragment source, or remains an explicitly retained per-tree recording
  mechanism.
- Decide whether `UiTree::boundary_paint_cache_entries` remains the final plain retained
  paint-cache entry store or is replaced by a broader boundary table.
- Add a second non-code-editor proof surface before global closeout.
