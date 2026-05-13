# M4 UI Gallery Feature Payload Fixture

Status: Landed
Date: 2026-05-12

## Decision

The `code_editor_torture` UI Gallery page now applies a deterministic feature payload fixture to its
large editor document:

- one diagnostic span,
- one range decoration,
- one logical-line gutter marker,
- one display-row gutter marker,
- semantic tokens for the first generated row.

The fixture is re-applied only when the editor buffer revision changes. This exercises the public
`CodeEditorHandle` feature payload setters without adding per-frame setter churn.

## Coverage

The existing torture page already exercises:

- optional Rust syntax highlighting,
- soft wrap,
- folds,
- inlays,
- preedit composition modes,
- selection/caret state in diagnostics snapshots,
- windowed row scrolling/autoscroll.

With the feature payload fixture present, the existing code-editor torture scripts now capture a
single scenario that combines the first editor feature package with the established scroll, wrap,
fold/inlay, preedit, and selection evidence paths.

## Non-Goals

This slice does not make diagnostics/decorations/gutter/semantic tokens visually complete. It only
proves that the public payload surface is exercised by a first-party example and exported through
diagnostics snapshots.

The explicit bundle assertion for payload stability remains the next gate slice.

## Evidence

- Fixture wiring:
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/torture.rs`
- Diagnostics snapshot counts:
  `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`
- Public API surface:
  `ecosystem/fret-code-editor/src/editor/mod.rs`

## Gates

```powershell
cargo fmt -p fret-ui-gallery
cargo check -p fret-ui-gallery
```
