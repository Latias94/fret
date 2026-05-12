# M3 Paint Row-Geom Cache Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move row-geometry cache freshness, touch/store, eviction, and caret-stop size accounting out of
`ecosystem/fret-code-editor/src/editor/paint/mod.rs` into
`ecosystem/fret-code-editor/src/editor/paint/geom_cache.rs`.

`paint_row` keeps producing fresh geometry from the actual shaped text blob. The new module only
owns committing that paint-produced geometry into the cache used by pointer hit-testing, caret
navigation, and IME cursor-area anchoring.

## Rationale

Row geometry is a hot-path bridge between paint and later input/IME queries. The cache policy has
different responsibilities from row content painting: revision/display-map freshness, row touch,
LRU eviction, and approximate caret-stop memory accounting.

Keeping this cache write policy inline in `paint_row` made the row painter harder to audit and made
it less obvious which part of paint owns geometry persistence. The split keeps the existing perf
field `us_row_geom_cache` / `ns_row_geom_cache` while making the cache owner explicit.

## Non-Goals

This slice does not change row geometry keying, caret-stop generation, row-scene replay/store,
hit-testing, IME anchoring, or public editor APIs.

`paint/mod.rs` still owns row-level paint orchestration and selection/caret painting. Row-rich
cache/prefetch later moved to `M3_PAINT_ROW_RICH_MODULE_SPLIT_2026-05-12.md`.

## Evidence

- Row-geom cache owner: `ecosystem/fret-code-editor/src/editor/paint/geom_cache.rs`
- Paint producer/call site: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Geometry consumers: `ecosystem/fret-code-editor/src/editor/geom/mod.rs`
- Input cache shift policy: `ecosystem/fret-code-editor/src/editor/input/edit.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
