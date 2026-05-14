# M4C Boundary Hint API Slice - 2026-05-14

Status: landed as an authoring-contract slice; no new perf claim.

## Summary

This slice replaces first-party direct `contained_layout` authoring with a boundary-hint API.

Before this slice:

- `ViewCacheProps` exposed `contained_layout` as a standalone mechanism flag.
- `CachedSubtreeProps::contained_layout(true)` was the ecosystem authoring surface used by first
  party examples and torture pages.
- UI Gallery page metadata used the same contained-layout wording for code-editor content-cache
  hints.

After this slice:

- `ViewCacheProps` carries `ViewBoundaryHints`.
- The layout-containment hint is named
  `contain_layout_when_bounds_known`, matching ADR 0327's dependency-contract wording.
- `CachedSubtreeProps` exposes `boundary_hints(...)` and
  `contain_layout_when_bounds_known(...)`.
- First-party `CachedSubtreeProps` and direct `ViewCacheProps` call sites use the boundary-hint
  API.
- The internal retained `ViewCacheFlags::contained_layout` bit remains as the low-level runtime
  implementation detail used by existing contained-relayout code and diagnostics.

## Code Changes

- `crates/fret-ui/src/element.rs`
  - adds `ViewBoundaryHints`,
  - replaces public `ViewCacheProps::contained_layout` with `ViewCacheProps::boundary_hints`,
  - adds `ViewCacheProps::contain_layout_when_bounds_known(...)`.
- `crates/fret-ui/src/declarative/mount.rs`
  - maps `ViewCacheProps::boundary_hints.contain_layout_when_bounds_known` into the existing
    low-level view-cache flags and debug records.
- `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs`
  - replaces `CachedSubtreeProps::contained_layout(...)` with `boundary_hints(...)` and
    `contain_layout_when_bounds_known(...)`.
- First-party examples, UI Gallery pages, docking/workspace/facade code, and cookbook snippets now
  use `contain_layout_when_bounds_known(...)` for boundary authoring.

## Correctness Gates

Boundary hint runtime gate:

```bash
cargo nextest run -p fret-ui \
  view_cache_boundary_hints_drive_boundary_layout_dependency \
  view_cache_runs_contained_relayout_for_invalidated_boundaries \
  view_cache_contained_relayout_does_not_force_next_frame_rerender \
  --no-fail-fast
```

Observed result:

- `3 passed, 930 skipped`.

Ecosystem authoring API gate:

```bash
cargo nextest run -p fret-ui-kit \
  cached_subtree_props_boundary_hint_replaces_direct_contained_layout_authoring \
  --no-fail-fast
```

Observed result:

- `1 passed, 520 skipped`.

Compile gate:

```bash
cargo check -p fret-ui -p fret-ui-kit -p fret-ui-gallery -p fret-docking -p fret-workspace -p fret --all-targets
```

Observed result:

- passed.

Source drift check:

```bash
rg -n "contained_layout\\(|page_content_cache_contained_layout|ViewCacheProps \\{[^\\n]*contained_layout|contained_layout:" \
  apps/fret-cookbook apps/fret-examples apps/fret-ui-gallery ecosystem/fret-ui-kit/src/declarative \
  crates/fret-ui/src/declarative crates/fret-ui/src/element.rs ecosystem/fret-docking \
  ecosystem/fret-workspace ecosystem/fret/src --glob '*.rs'
```

Observed result:

- no matches.

## Deletion Note

Deleted or retired in this slice:

- public `ViewCacheProps::contained_layout`,
- `CachedSubtreeProps::contained_layout(...)`,
- first-party direct `.contained_layout(true)` authoring,
- UI Gallery `page_content_cache_contained_layout(...)` wording.

Still intentionally not deleted:

- internal `ViewCacheFlags::contained_layout`,
- debug/cache-root fields named `contained_layout`,
- lower-level contained-relayout function names and test names.

Those remaining names describe the current runtime implementation or diagnostic schema. They should
be reconsidered during the broader view-cache/build-boundary consolidation, not during this
authoring-contract slice.

## Perf Evidence

This slice changes authoring/API shape only. It does not make a new optimization claim.

The current code-editor closeout perf evidence remains the latest perf evidence:

- `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/check.perf_thresholds.json`
- `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/1778700500609/bundle.schema2.json`

## Remaining Gaps

- Broader view-cache rendered/next maps are still not boundary-owned.
- Broader paint-cache replay stores are still not boundary-owned.
- Internal low-level `contained_layout` flags and diagnostics remain until the runtime owner path is
  consolidated.
- A second non-code-editor proof surface is still required before global closeout.
