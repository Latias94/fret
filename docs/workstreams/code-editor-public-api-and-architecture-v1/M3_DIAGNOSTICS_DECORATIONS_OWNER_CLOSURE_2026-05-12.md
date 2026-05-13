# M3 Diagnostics/Decorations Owner Closure

Status: Landed
Date: 2026-05-12

## Decision

Close the M3 `diagnostics/decorations` owner item without adding another editor module.

The owner boundary is already explicit across the existing layers:

- `fret-code-editor-view` owns validation, normalization, and coordinate contracts for diagnostics,
  range decorations, gutter markers, and semantic tokens.
- `fret-code-editor/src/editor/feature_payloads.rs` owns widget-local payload storage, epochs,
  buffer-change clearing, display-map gutter pruning, and diagnostics snapshot counts.
- `fret-code-editor/src/editor/handle/feature_payloads.rs` owns the public handle call sites and
  converts raw app inputs into normalized store payloads.
- `fret-code-editor/src/editor/tests/feature_payloads.rs` owns the focused editor-side behavior
  tests for idempotence, clearing, summaries, and display-map pruning.

`fret-code-editor/src/editor/diagnostics.rs` remains the diagnostics/perf snapshot owner. It should
not also become the feature diagnostics/decorations payload owner.

## Rationale

The remaining TODO entry was stale after the view contracts, feature payload store, handle method
split, and focused feature payload tests landed. Adding a new diagnostics/decorations module inside
`fret-code-editor` would create a second policy owner rather than clarifying the current boundary.

Closing this item keeps the architecture split aligned with the v1 public surface: feature payloads
are data contracts and storage facts, while visual styling, overlay behavior, and LSP production stay
outside the core editor widget.

## Non-Goals

This closure does not implement final visual styling for diagnostics or decorations, LSP payload
remapping across edits, hover/completion/code-action request structs, or overlay policy.

Those remain feature-roadmap items, not M3 internal owner splits.

## Evidence

- View diagnostics contract: `ecosystem/fret-code-editor-view/src/diagnostics.rs`
- View range decoration contract: `ecosystem/fret-code-editor-view/src/decorations.rs`
- View gutter marker contract: `ecosystem/fret-code-editor-view/src/gutter.rs`
- View semantic token contract: `ecosystem/fret-code-editor-view/src/semantic_tokens.rs`
- Widget payload owner: `ecosystem/fret-code-editor/src/editor/feature_payloads.rs`
- Handle payload owner: `ecosystem/fret-code-editor/src/editor/handle/feature_payloads.rs`
- Focused editor tests: `ecosystem/fret-code-editor/src/editor/tests/feature_payloads.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor-view --check
cargo check -p fret-code-editor-view
cargo nextest run -p fret-code-editor-view --lib --no-fail-fast
cargo nextest run -p fret-code-editor feature_payload --no-fail-fast
```
