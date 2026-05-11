# M2 Semantic Token Contract - 2026-05-12

Status: Fifth extension-model code slice

This slice adds the first semantic-token input contract. The intent is to keep syntax/LSP semantic
facts separate from paint colors and text span styling, so later theme changes can map classes to
paint without changing the token input model.

## Contract Surface

New view-layer items:

- `SemanticToken`
- `SemanticTokenError`
- `validate_semantic_tokens`
- `normalized_semantic_tokens`

Owner layer:

- crate: `fret-code-editor-view`
- coordinate space: `TextBuffer` UTF-8 byte ranges
- semantic identity: token `class` plus unordered `modifiers`
- paint policy: explicitly out of scope

## v1 Semantics

- Tokens attach to non-empty buffer byte ranges.
- Ranges must be in bounds and on UTF-8 char boundaries.
- Overlapping tokens are valid and are not merged, dropped, or de-duplicated.
- `class` must be non-empty and represents semantic identity, not a color.
- `modifiers` are unordered semantic flags; normalization sorts and de-duplicates them.
- Empty modifiers are rejected.
- `normalized_semantic_tokens` validates first, normalizes modifier sets, and sorts
  deterministically by range, class, and modifiers.

## Why This Is Separate From Decorations

Range decorations model editor feature overlays such as diagnostics, search matches, and transient
highlights. Semantic tokens model source-language facts. Keeping them separate prevents the editor
from treating syntax classes as paint spans too early and preserves the ADR 0185 goal that theme-only
changes should remain paint-only where possible.

## Evidence

- Implementation: `ecosystem/fret-code-editor-view/src/semantic_tokens.rs`
- Public export: `ecosystem/fret-code-editor-view/src/lib.rs`
- Unit tests:
  - overlaps are accepted,
  - reversed/empty/out-of-bounds/non-UTF-8-boundary ranges are rejected,
  - empty classes and modifiers are rejected,
  - modifiers are sorted and de-duplicated during normalization,
  - tokens carry semantic class/modifier data but no paint colors.

## Gates

```powershell
cargo fmt -p fret-code-editor-view --check
cargo check -p fret-code-editor-view
cargo nextest run -p fret-code-editor-view --lib --no-fail-fast
```

## Follow-ups

1. Map `fret-syntax::HighlightSpan` into `SemanticToken` before converting to paint spans.
2. Wire diagnostics, gutter markers, range decorations, and semantic tokens into a UI
   Gallery/editor proof.
3. Add feature-payload counters once semantic/decorative payloads enter the paint path.
