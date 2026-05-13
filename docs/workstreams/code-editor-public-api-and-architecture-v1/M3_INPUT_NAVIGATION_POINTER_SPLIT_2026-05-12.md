# M3 Input Navigation Pointer Split

Status: Landed
Date: 2026-05-12

## Decision

The editor input module now uses feature-owned child modules for the remaining interaction owners:

- `input/navigation.rs`: caret scrolling, page/home/end/word/vertical navigation, fold clamping,
  and viewport movement for page navigation.
- `input/pointer.rs`: pointer-down selection behavior for single, double, triple, and shift-click.
- `input/edit.rs`: text insertion, deletion commands, IME delete-surrounding, edit transactions,
  undo/redo, and row-geometry cache shifting.

`input/mod.rs` is now a narrow owner boundary and re-export surface for existing internal callers.
Current `input::*` call paths are unchanged.

## Rationale

Navigation, pointer selection, and edit mutation have different invariants:

- navigation owns caret movement through `DisplayMap`, row geometry, folds, and viewport scroll,
- pointer selection owns click-count and shift-selection behavior,
- edit mutation owns buffer changes, undo grouping, feature-payload invalidation, and cache updates.

Separating these owners makes future command/keymap routing and pointer/drag changes reviewable
without reopening edit transaction or cache invalidation code.

## Non-Goals

This slice does not change navigation semantics, pointer selection semantics, fold snapping,
delete/backspace behavior, undo semantics, cache thresholds, command ids, or public editor APIs.

## Evidence

- Input boundary: `ecosystem/fret-code-editor/src/editor/input/mod.rs`
- Navigation owner: `ecosystem/fret-code-editor/src/editor/input/navigation.rs`
- Pointer selection owner: `ecosystem/fret-code-editor/src/editor/input/pointer.rs`
- Edit mutation owner: `ecosystem/fret-code-editor/src/editor/input/edit.rs`
- Keyboard dispatch owner: `ecosystem/fret-code-editor/src/editor/input/keyboard.rs`
- Input regression coverage: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python tools/check_layering.py
git diff --check
```
