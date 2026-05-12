# M3 Test Platform Text Input Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move platform text input and IME preedit semantic tests out of
`ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/platform_text_input.rs`.

The extracted tests cover preedit caret rect offsets, IME cursor area projection, UTF-16 marked
range to UTF-8 cursor mapping, replace-and-mark composition semantics, empty-composition cancel,
marked-none replace, IME delete-surrounding, and the staged single-line clamp for multi-line
composition ranges.

## Rationale

Platform text input is one of the editor hot paths named by this lane: it binds IME/preedit state,
a11y composed windows, display mapping, row text materialization, and edit semantics. Keeping these
tests inside the monolithic editor test module made the input contract harder to review alongside
`editor/input/*` and the platform callback implementation in `editor/mod.rs`.

This split gives the platform text input contract a focused test owner without changing behavior,
public APIs, or performance thresholds.

## Non-Goals

This slice does not move the larger bounds/index roundtrip tests, pointer-selection tests, keyboard
tests, or preedit rich-text paint tests. Those remain follow-up slices.

## Evidence

- Platform text input test owner:
  `ecosystem/fret-code-editor/src/editor/tests/platform_text_input.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Platform callback implementation: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Input edit owner: `ecosystem/fret-code-editor/src/editor/input/edit.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo nextest run -p fret-code-editor platform --no-fail-fast
cargo nextest run -p fret-code-editor ime --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
