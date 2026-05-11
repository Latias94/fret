# M2 Range Decoration Contract - 2026-05-12

Status: Fourth extension-model code slice

This slice adds the first explicit range decoration payload contract. It is a view-layer data model
for feature overlays such as diagnostics, search matches, bracket matches, and transient highlights.
The contract is intentionally independent from concrete colors, renderer primitives, hover popup
composition, and command execution.

## Contract Surface

New view-layer items:

- `RangeDecorationLayer`
- `RangeDecorationHitTest`
- `RangeDecoration`
- `RangeDecorationError`
- `validate_range_decorations`
- `normalized_range_decorations`

Owner layer:

- crate: `fret-code-editor-view`
- coordinate space: `TextBuffer` UTF-8 byte ranges
- visual identity: semantic `class` string, not a paint color
- UI policy: explicitly out of scope

## v1 Semantics

- Decorations attach to buffer byte ranges.
- Empty ranges are valid for point-style or transient decorations.
- Overlapping decorations are valid and are not merged, dropped, or de-duplicated.
- Ranges must be in bounds and on UTF-8 char boundaries.
- `class` must be non-empty and represents semantic styling, not direct paint values.
- `layer` and `z_index` provide deterministic ordering hints without choosing renderer behavior.
- `hover_id` is a data identifier; the widget/app layer owns hover content and overlay policy.
- `hit_test` is an intent flag only; the widget/app layer owns pointer handling.
- `normalized_range_decorations` validates first, then sorts deterministically by range, layer,
  z-index, class, hover id, and hit-test policy.

## Why This Is Still View-Layer

Decoration payloads need to share the same buffer/display vocabulary as diagnostics, folds, inlays,
preedit, and future semantic tokens. Keeping this contract in `fret-code-editor-view` makes it
testable without a renderer and prevents widget paint policy from becoming the source of truth for
feature coordinates.

## Evidence

- Implementation: `ecosystem/fret-code-editor-view/src/decorations.rs`
- Public export: `ecosystem/fret-code-editor-view/src/lib.rs`
- Unit tests:
  - empty and overlapping ranges are accepted,
  - reversed/out-of-bounds/non-UTF-8-boundary ranges are rejected,
  - empty classes are rejected,
  - normalization is deterministic without dropping overlaps,
  - hover and hit-test policy remain data-only.

## Gates

```powershell
cargo fmt -p fret-code-editor-view --check
cargo check -p fret-code-editor-view
cargo nextest run -p fret-code-editor-view --lib --no-fail-fast
```

## Follow-ups

1. Use `M2_SEMANTIC_TOKEN_CONTRACT_2026-05-12.md` for source-language token inputs before mapping
   to paint spans.
2. Wire diagnostics, gutter markers, and range decorations into a UI Gallery/editor proof.
3. Add feature-payload counters once these payloads enter the paint path.
