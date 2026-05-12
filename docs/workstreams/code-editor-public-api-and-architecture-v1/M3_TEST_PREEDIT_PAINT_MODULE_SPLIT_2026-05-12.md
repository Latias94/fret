# M3 Test Preedit Paint Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move preedit rich-text paint tests out of `ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/preedit_paint.rs`.

The extracted tests cover inline preedit insertion/underline materialization and propagation of
code shaping feature policy to all rich-text spans.

## Rationale

Preedit rich-text materialization is a paint/text-composition owner concern. It is distinct from
platform IME event semantics, a11y offset mapping, keyboard command dispatch, and pointer
selection. Keeping these tests in a dedicated module makes the paint boundary easier to audit while
preserving the existing behavior unchanged.

## Non-Goals

This slice does not change IME semantics, preedit state transitions, text shaping policy, or public
APIs.

## Evidence

- Preedit paint test owner: `ecosystem/fret-code-editor/src/editor/tests/preedit_paint.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Paint rich-text owner: `ecosystem/fret-code-editor/src/editor/paint/rich.rs`
- Paint row owner: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor preedit_rich_text --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
