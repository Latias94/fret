# M3 Input Edit Transaction Split

Status: Landed
Date: 2026-05-12

## Decision

The editor input module now keeps edit transaction, IME delete-surrounding, undo/redo, and row-geom
cache-shift logic in `ecosystem/fret-code-editor/src/editor/input/edit.rs`.

`ecosystem/fret-code-editor/src/editor/input/mod.rs` remains the public owner boundary for current
callers, so existing `input::insert_text`, `input::apply_and_record_edit`, `input::undo`,
`input::redo`, and `input::apply_ime_delete_surrounding` call paths are unchanged.

## Rationale

Editing is the part of input that touches the most cross-cutting editor contracts:

- buffer transactions and undo grouping,
- IME delete-surrounding and preedit preservation,
- `DisplayMap` refresh and feature-payload invalidation,
- syntax-row cache invalidation,
- row-geometry cache shifting or reset after edits.

Keeping this logic behind a focused `input/edit.rs` owner makes future command/keymap, IME, and
cache-invalidation work easier to review without mixing it into keyboard dispatch or pointer
selection behavior.

## Non-Goals

This slice does not change keyboard routing, command ids, pointer selection, clipboard effects,
caret navigation, undo semantics, cache thresholds, feature-payload invalidation, or public editor
APIs.

The broader `input` owner split remains open. Follow-up slices should separate keyboard/command
dispatch, caret navigation, pointer selection, and clipboard effects only when tests or API review
benefit from that separation.

## Evidence

- Input owner boundary: `ecosystem/fret-code-editor/src/editor/input/mod.rs`
- Edit transaction owner: `ecosystem/fret-code-editor/src/editor/input/edit.rs`
- Syntax invalidation owner consumed by edit transactions:
  `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Input/cache regression tests: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Syntax cache regression tests: `ecosystem/fret-code-editor/src/editor/tests/syntax.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python tools/check_layering.py
git diff --check
```
