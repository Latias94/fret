# M1 Source Model Result - 2026-04-28

Status: landed

## Result

M1 now has a browser-ready identity diagnostics source model in `fret-diag`.

The model consumes schema2 bundle snapshots from:

- `windows[*].window`
- `windows[*].snapshots[*]`
- `snapshots[*].debug.element_runtime.identity_warnings[]`

It maps each warning into an owned row with:

- window, tick, snapshot frame, snapshot sequence, and timestamp anchors,
- warning kind,
- element id and element path,
- list id,
- duplicate-key hash and duplicate indices,
- unkeyed-list previous/next lengths,
- source file/line/column,
- a grouping key over kind, window/frame, source file, list id, key hash, and element path.

The existing `diag query identity-warnings` JSON output remains compatible with the prior query
shape. The command now uses the shared browser model instead of walking raw bundle JSON itself.

## Field Sufficiency

Existing capture fields are enough for the first post-run browser workflow:

- duplicate keyed-list warnings can navigate to source file, element path, list id, key hash, and
  first/second duplicate indices,
- unkeyed reorder warnings can navigate to source file, element path, list id, and previous/next
  list lengths,
- default dedup still keeps the latest matching observation,
- `--timeline` still preserves repeated observations across snapshots.

No capture-side field is blocking M1.

## Evidence

- `crates/fret-diag/src/identity_browser.rs`
- `crates/fret-diag/src/commands/query.rs`
- `crates/fret-diag/src/lib.rs`

## Gates

- `cargo nextest run -p fret-diag identity_browser --no-fail-fast`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`
- `cargo fmt --package fret-diag --check`
