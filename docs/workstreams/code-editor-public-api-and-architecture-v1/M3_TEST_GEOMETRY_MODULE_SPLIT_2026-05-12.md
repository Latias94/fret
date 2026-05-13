# M3 Test Geometry Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move geometry helper/keying tests out of
`ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/geometry.rs`.

The extracted tests cover caret-stop hit-testing, row display-to-buffer mapping with preedit,
`RowFoldMap` mapping, and `RowGeomKey` key stability.

## Rationale

The editor test module remains large, but geometry helper/keying tests are a low-coupling cluster:
they do not need the UI render harness, platform text-input fixtures, or pointer interaction setup.
Moving them gives geometry tests a clear owner without changing behavior or public API.

## Non-Goals

This slice does not change geometry behavior, row-geometry cache policy, input/edit tests, a11y
tests, or platform text-input tests. The broader monolithic test split remains open.

## Evidence

- Geometry test owner: `ecosystem/fret-code-editor/src/editor/tests/geometry.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Geometry implementation owner: `ecosystem/fret-code-editor/src/editor/geom/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo nextest run -p fret-code-editor geometry --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
