# M3 Test Caret Navigation Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move basic caret navigation tests out of `ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/caret_navigation.rs`.

The extracted tests cover fold-aware left/right movement and vertical movement preferred-x
preservation.

## Rationale

Caret movement is an input navigation owner concern. These tests depend on selection state,
display-row geometry, and fold-aware caret mapping, but not on the larger editor render or platform
text-input fixtures. A dedicated module makes the navigation boundary easier to audit.

## Non-Goals

This slice does not change movement behavior, command routing, home/end/page movement, or public
APIs.

## Evidence

- Caret navigation test owner: `ecosystem/fret-code-editor/src/editor/tests/caret_navigation.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Input navigation implementation: `ecosystem/fret-code-editor/src/editor/input/navigation.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor caret_ --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
