# M3 Syntax Cache Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move syntax cache invalidation, population, materialization, and prefetch orchestration out of
`ecosystem/fret-code-editor/src/editor/paint/mod.rs` into
`ecosystem/fret-code-editor/src/editor/syntax.rs`.

The new owner contains:

- `SyntaxRowCacheLookup`
- `SYNTAX_CACHE_LOOKBACK_ROWS`
- `SYNTAX_CACHE_LOOKAHEAD_ROWS`
- `SYNTAX_PREFETCH_CHUNK_ROWS`
- `SYNTAX_PREFETCH_AHEAD_ROWS`
- `syntax_prefetch_chunk_for_row`
- `syntax_row_cache_chunk_is_ready`
- `syntax_prefetch_visible_line_window`
- `syntax_row_cache_store_rows`
- `syntax_rows_from_highlight_spans`
- `ensure_syntax_row_cache_fresh`
- `lookup_row_syntax_spans`
- `populate_row_syntax_spans_after_miss`
- `cached_row_syntax_spans`
- `populate_syntax_row_cache_for_chunk`
- `invalidate_syntax_row_cache_for_delta`
- `rebuild_syntax_row_cache_queue`
- `schedule_syntax_prefetch_for_frame`

`paint/mod.rs` now consumes the syntax owner for row-scene replay and row-rich prefetching, but it
no longer owns syntax cache policy itself. `editor/mod.rs` dispatches syntax prefetch through the
syntax module, and `input/edit.rs` invalidates the syntax cache through the syntax owner.

## Rationale

The syntax cache hot path was the last remaining large syntax-related owner inside `paint/mod.rs`.
Leaving invalidation, cache fill, and highlight materialization there made the paint module absorb
both rendering and syntax state-policy responsibilities. Moving the owner into `syntax.rs` makes
the syntax contract visible as an editor subsystem boundary instead of a paint implementation
detail.

## Non-Goals

This slice does not change syntax highlighting behavior, cache keys, prefetch candidate selection,
pending/ready queue limits, background dispatch priority, or public editor APIs.

`paint/mod.rs` still owns row-rich cache policy, row-scene replay, and row-level rendering.

## Evidence

- Syntax owner: `ecosystem/fret-code-editor/src/editor/syntax.rs`
- Paint consumer: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Surface integration: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Cache invalidation caller: `ecosystem/fret-code-editor/src/editor/input/edit.rs`
- Syntax regression tests: `ecosystem/fret-code-editor/src/editor/tests/syntax.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
python tools/check_layering.py
git diff --check
```
