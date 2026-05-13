# M3 Test Row-Text Cache Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move row-text cache and paint-frame cache-floor tests out of
`ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`.

The extracted tests cover row-text cache hits, revision invalidation, LRU eviction, cache stats,
code-wrap invalidation, and visible-window cache-floor retention.

## Rationale

The row-text cache is a hot-path resource owner for display-row materialization, row span retention,
and row-paint cache residency. Keeping its tests in the monolithic editor test module made the
performance/resource contract harder to find after the implementation moved into
`editor/paint/text.rs`.

This split gives row-text cache behavior a matching test owner without changing behavior, public
API, cache policy, or perf thresholds.

## Non-Goals

This slice does not change row-text cache behavior, row-rich cache behavior, row-geometry cache
behavior, renderer payload thresholds, or public editor APIs. Broader paint/input/a11y tests remain
in follow-up slices.

## Evidence

- Row-text cache test owner: `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Row-text cache implementation owner: `ecosystem/fret-code-editor/src/editor/paint/text.rs`
- Paint-frame cache floor owner: `ecosystem/fret-code-editor/src/editor/state.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo nextest run -p fret-code-editor row_text_cache --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
