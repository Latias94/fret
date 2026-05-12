# M3 Test Keyboard Commands Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move focused keyboard command and interaction-mode tests out of
`ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/keyboard_commands.rs`.

The extracted tests cover Ctrl+PageDown bubbling, Ctrl+A select-all, and read-only mutation gates.

## Rationale

Keyboard command handling and interaction-mode gating are input keyboard/command owner concerns.
They are part of the public command/keymap/undo boundary tracked by this lane, but they do not need
to live beside paint, geometry, pointer, or platform text-input tests.

This split preserves behavior while making command/interaction tests easier to audit and extend.

## Non-Goals

This slice does not change command ids, keymap resolution, interaction semantics, or read-only
behavior.

## Evidence

- Keyboard command test owner: `ecosystem/fret-code-editor/src/editor/tests/keyboard_commands.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Keyboard dispatch implementation: `ecosystem/fret-code-editor/src/editor/input/keyboard.rs`
- Command/keymap/undo boundary note:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M1_COMMAND_KEYMAP_UNDO_BOUNDARY_2026-05-12.md`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor ctrl_ --no-fail-fast
cargo nextest run -p fret-code-editor read_only --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
