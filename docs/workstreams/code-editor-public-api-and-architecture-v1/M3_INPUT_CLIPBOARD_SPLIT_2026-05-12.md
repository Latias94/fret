# M3 Input Clipboard Split

Status: Landed
Date: 2026-05-12

## Decision

The editor input module now keeps clipboard effects in
`ecosystem/fret-code-editor/src/editor/input/clipboard.rs`.

`ecosystem/fret-code-editor/src/editor/input/mod.rs` remains the public owner boundary for current
callers, so existing `input::copy_selection`, `input::request_paste`, and `input::cut_selection`
call paths are unchanged.

## Rationale

Clipboard integration is an effect boundary, not a keyboard-navigation concern. Splitting it away
from `input/mod.rs` keeps copy, paste-request, and cut behavior close to the `UiActionHost` effect
surface while leaving command/key dispatch and caret movement reviewable on their own.

`cut_selection` still uses the edit transaction owner for buffer mutation, so undo grouping and
feature-payload invalidation remain centralized in `input/edit.rs`.

## Non-Goals

This slice does not change clipboard token allocation, copy/cut selection semantics, paste command
routing, text insertion, undo grouping, or public editor APIs.

## Evidence

- Input owner boundary: `ecosystem/fret-code-editor/src/editor/input/mod.rs`
- Clipboard effect owner: `ecosystem/fret-code-editor/src/editor/input/clipboard.rs`
- Edit transaction owner used by cut: `ecosystem/fret-code-editor/src/editor/input/edit.rs`
- Command integration: `ecosystem/fret-code-editor/src/editor/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python tools/check_layering.py
git diff --check
```
