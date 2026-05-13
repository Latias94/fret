# M3 Test Accessibility Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move accessibility composed-window and mapping tests out of
`ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/accessibility.rs`.

The extracted tests cover bounded a11y window materialization, platform composition selection
mapping, preedit offset mapping, composed decoration mapping, UTF-8 scalar clamping, newline
mapping, and direction-preserving selection projection.

## Rationale

Accessibility is already an implementation owner under `editor/a11y/*`, but many of its regression
tests still lived in the monolithic editor test module. Moving the window/mapping tests gives the
a11y contract a matching test owner and makes large-document/windowed text-input regressions easier
to find.

This remains a pure ownership split: it does not change platform text input behavior, a11y mapping
semantics, preedit semantics, or public editor APIs.

## Non-Goals

This slice does not move platform text-input roundtrip tests, pointer/input tests, or the internal
unit tests already owned by `editor/a11y/*`.

## Evidence

- Accessibility test owner: `ecosystem/fret-code-editor/src/editor/tests/accessibility.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Accessibility implementation owners:
  - `ecosystem/fret-code-editor/src/editor/a11y/mod.rs`
  - `ecosystem/fret-code-editor/src/editor/a11y/window.rs`
  - `ecosystem/fret-code-editor/src/editor/a11y/mapping.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo nextest run -p fret-code-editor a11y --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
