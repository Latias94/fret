# M3 Input Keyboard Split

Status: Landed
Date: 2026-05-12

## Decision

The editor input module now keeps keyboard dispatch in
`ecosystem/fret-code-editor/src/editor/input/keyboard.rs`.

`ecosystem/fret-code-editor/src/editor/input/mod.rs` remains the public owner boundary for current
callers, so the existing `input::handle_key_down` call path is unchanged.

## Rationale

Keyboard handling is a dispatch layer: it translates key/modifier combinations into editor actions
such as navigation, selection, edit transactions, clipboard requests, and preedit cancellation. It
should stay separate from the lower-level action owners so future command/keymap routing work can
review dispatch behavior without mixing it with buffer mutation or pointer geometry.

This split keeps the current compatibility path intact while making the next architecture step
clear: route command-facing editor actions through Fret's command/action infrastructure without
turning `input/mod.rs` into a policy sink.

## Non-Goals

This slice does not change keybindings, command ids, preedit cancellation behavior, read-only
gating, navigation semantics, edit transactions, clipboard effects, or public editor APIs.

## Evidence

- Input owner boundary: `ecosystem/fret-code-editor/src/editor/input/mod.rs`
- Keyboard dispatch owner: `ecosystem/fret-code-editor/src/editor/input/keyboard.rs`
- Edit transaction owner: `ecosystem/fret-code-editor/src/editor/input/edit.rs`
- Clipboard effect owner: `ecosystem/fret-code-editor/src/editor/input/clipboard.rs`
- Region key hook integration: `ecosystem/fret-code-editor/src/editor/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python tools/check_layering.py
git diff --check
```
