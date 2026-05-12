# M3 Paint Row-Rich Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move row-rich text materialization, syntax span mapping/normalization, row-rich cache store/evict,
and row-rich prefetch scheduling out of
`ecosystem/fret-code-editor/src/editor/paint/mod.rs` into
`ecosystem/fret-code-editor/src/editor/paint/rich.rs`.

`paint/mod.rs` keeps the row-level paint orchestration and imports the focused row-rich helpers it
needs for syntax-highlighted rows and preedit rich text.

## Rationale

The row-rich path combines several responsibilities that are distinct from row paint orchestration:
mapping source syntax spans into display-row text, building attributed text, deciding whether a
row-rich cache entry is fresh, accounting row-rich cache memory, and scheduling background prefetch
jobs.

Keeping all of that inline in `paint/mod.rs` made hot-path review harder because row content
painting, syntax mapping, cache resource accounting, and background prefetch policy were interleaved.
The split creates one owner for rich row materialization and prefetch while keeping renderer and
windowed-row behavior unchanged.

## Non-Goals

This slice does not change syntax cache ownership, row-rich cache keys, prefetch candidate policy,
background dispatch priority, preedit rendering semantics, row-scene replay/store, or public editor
APIs.

`paint/mod.rs` still owns row-level paint orchestration and selection/caret painting.

## Evidence

- Row-rich owner: `ecosystem/fret-code-editor/src/editor/paint/rich.rs`
- Paint consumer/call sites: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Syntax runtime/cache owner: `ecosystem/fret-code-editor/src/editor/syntax.rs`
- Row-scene owner: `ecosystem/fret-code-editor/src/editor/paint/scene.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
