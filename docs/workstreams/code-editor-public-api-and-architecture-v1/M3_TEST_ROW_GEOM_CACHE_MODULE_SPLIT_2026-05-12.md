# M3 Test Row Geom Cache Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move row geometry cache tests out of `ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/row_geom_cache.rs`.

The extracted tests cover row geometry cache byte shifting, soft-wrap row shifting, font-stack
invalidation, stale preedit geometry rejection, and code-wrap policy invalidation.

## Rationale

Row geometry cache behavior is a paint/geometry cache owner concern with input-edit invalidation
hooks. Keeping these tests together makes cache freshness and invalidation semantics easier to
audit without mixing them with buffer lifecycle, display navigation, platform text input, or scroll
window tests.

## Non-Goals

This slice does not change row geometry cache keys, cache invalidation behavior, edit handling,
font stack semantics, or public APIs.

## Evidence

- Row geometry cache test owner: `ecosystem/fret-code-editor/src/editor/tests/row_geom_cache.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Paint row-geom cache owner: `ecosystem/fret-code-editor/src/editor/paint/geom_cache.rs`
- Input edit transaction owner: `ecosystem/fret-code-editor/src/editor/input/edit.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor row_geom_cache --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
