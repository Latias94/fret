# M3 A11y Module Split

Status: Landed
Date: 2026-05-12

## Decision

The editor accessibility helpers are split into feature-owned modules under
`ecosystem/fret-code-editor/src/editor/a11y/`:

- `mod.rs`: small owner boundary and test-facing re-exports.
- `window.rs`: accessibility text-window construction, composed display-row windows, and buffer
  offset projection into the current accessibility value.
- `mapping.rs`: accessibility offset-to-buffer and buffer-to-accessibility offset mapping,
  including inline preedit and replacement-range cases.

This is a source-ownership split only. Public APIs, semantics values, selection/composition offset
behavior, and editor runtime behavior are unchanged.

## Rationale

Accessibility text export is one of the editor's high-risk cross-boundary surfaces because it ties
together buffer byte ranges, `DisplayMap` row composition, inline preedit, folds, and semantics
selection offsets. Keeping the window builder and offset mapping in one large module made it harder
to review future IME, display-fragment, and accessibility fixes independently.

The split keeps the owner lines explicit:

- window construction owns the bounded text value exposed to semantics,
- mapping owns conversion between semantics offsets and buffer byte offsets,
- `mod.rs` owns the narrow integration surface consumed by the editor.

## Non-Goals

This slice does not change accessibility semantics shape, the 4096-byte before/after text-window
policy, inline preedit composition behavior, fold/inlay mapping behavior, or public exports.

## Evidence

- A11y module boundary: `ecosystem/fret-code-editor/src/editor/a11y/mod.rs`
- Text-window owner: `ecosystem/fret-code-editor/src/editor/a11y/window.rs`
- Offset-mapping owner: `ecosystem/fret-code-editor/src/editor/a11y/mapping.rs`
- Surface integration: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Accessibility and syntax regression tests: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor a11y --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python tools/check_layering.py
git diff --check
```
