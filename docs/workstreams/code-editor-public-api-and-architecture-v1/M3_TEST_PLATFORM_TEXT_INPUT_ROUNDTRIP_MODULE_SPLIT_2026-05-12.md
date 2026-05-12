# M3 Test Platform Text Input Roundtrip Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move the platform text input bounds/index roundtrip tests out of
`ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/platform_text_input_roundtrip.rs`.

The extracted tests cover platform-style bounds-for-range and character-index-for-point
roundtrips across replacing preedit, inline preedit, soft wrap, folds, inlays, and composed
decorations.

## Rationale

These tests exercise a deeper integration path than the basic platform text input semantic tests:
a11y offset mapping, display-map projection, row-text materialization, row-geometry cache seeding,
caret rect lookup, and pointer hit-testing all need to agree. Keeping this roundtrip set isolated
makes the platform query contract easier to review and keeps future performance/resource changes
close to the test owner that proves them.

## Non-Goals

This slice does not change mapping behavior, row-geometry cache semantics, platform callback
behavior, public APIs, or perf thresholds. It only moves tests.

## Evidence

- Platform text input roundtrip test owner:
  `ecosystem/fret-code-editor/src/editor/tests/platform_text_input_roundtrip.rs`
- Basic platform text input test owner:
  `ecosystem/fret-code-editor/src/editor/tests/platform_text_input.rs`
- Row geometry owner: `ecosystem/fret-code-editor/src/editor/geom/mod.rs`
- A11y window/mapping owners:
  - `ecosystem/fret-code-editor/src/editor/a11y/window.rs`
  - `ecosystem/fret-code-editor/src/editor/a11y/mapping.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo nextest run -p fret-code-editor roundtrip --no-fail-fast
cargo nextest run -p fret-code-editor platform --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
