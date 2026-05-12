# M3 Paint Row-Text Cache Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move row-text cache materialization, freshness, LRU touch/eviction, and size accounting out of
`ecosystem/fret-code-editor/src/editor/paint/mod.rs` into
`ecosystem/fret-code-editor/src/editor/paint/text.rs`.

`paint/mod.rs` keeps the existing internal call surface by re-exporting:

- `cached_row_text_with_range`
- `cached_row_text` for tests

## Rationale

The row-text cache is a hot-path bridge between `DisplayMap` materialization and row paint. It owns
revision/display-map epoch freshness, row span retention, text byte accounting, and the syntax-only
row-rich cache invalidation that follows row-text invalidation.

Keeping that policy inline in the main paint module made row paint, text materialization, cache
resource accounting, and syntax-rich invalidation harder to audit independently. The split makes the
row-text cache a focused owner while preserving the existing `paint::cached_row_text_with_range`
call sites used by paint, a11y, debug hooks, and tests.

## Non-Goals

This slice does not change `DisplayMap` materialization, row span composition, cache keys, LRU
policy, row-rich cache behavior, a11y text-window behavior, or public editor APIs.

`paint/mod.rs` still owns row-level paint orchestration, row-rich cache/prefetch, and
selection/caret painting. Row-geometry cache updates later moved to
`M3_PAINT_ROW_GEOM_CACHE_MODULE_SPLIT_2026-05-12.md`.

## Evidence

- Row-text owner: `ecosystem/fret-code-editor/src/editor/paint/text.rs`
- Paint consumer/re-export: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- A11y consumer: `ecosystem/fret-code-editor/src/editor/a11y/window.rs`
- Debug consumer: `ecosystem/fret-code-editor/src/editor/handle/debug.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
