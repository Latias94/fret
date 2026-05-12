# M3 Paint Row-Scene Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move row-scene cache freshness, replay, syntax replay-key refresh, and store/eviction policy out of
`ecosystem/fret-code-editor/src/editor/paint/mod.rs` into
`ecosystem/fret-code-editor/src/editor/paint/scene.rs`.

The new owner contains:

- `ensure_row_scene_cache_fresh`
- `refresh_row_scene_syntax_replay_key`
- `try_replay_row_scene_cache_fast_syntax`
- `try_replay_row_scene_cache`
- `store_row_scene_cache`

`paint/mod.rs` now consumes row-scene behavior through `scene::...` call sites and no longer keeps a
second local implementation of the replay/store chain.

## Rationale

The row-scene cache is a paint hot-path owner with its own invalidation epochs, hosted-resource
touching, replay translation, store pressure, and diagnostics counters. Keeping that logic inline
inside the main row paint function made `paint/mod.rs` absorb rendering orchestration and cache
resource policy at the same time.

Splitting the row-scene owner makes future cache or replay changes easier to review against the
existing p50/p95/max and renderer payload contracts without turning every paint change into a large
monolithic diff.

## Non-Goals

This slice does not change row-scene cache keys, LRU behavior, syntax replay-key matching, hosted
resource retention, paint attribution fields, public editor APIs, renderer payload thresholds, or
windowed-row behavior.

`paint/mod.rs` still owns row text/rich cache helpers, row materialization, and the main row painting
orchestration. Broader paint cleanup remains open.

## Evidence

- Row-scene owner: `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- Paint consumer: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Perf contract note: `docs/workstreams/code-editor-public-api-and-architecture-v1/M5_PERF_CONTRACT_CLOSURE_2026-05-12.md`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
python tools/check_layering.py
```
