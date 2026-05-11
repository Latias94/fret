# M2 Diagnostic Span Contract - 2026-05-12

Status: First extension-model code slice

This slice starts M2 with the smallest view-layer contract needed by diagnostics/decorations/gutter
work: a buffer-local diagnostic span model with deterministic validation and ordering.

## Reference Read

Zed keeps diagnostics as buffer-associated entries and lets display mapping/presentation consume
them later:

- `repo-ref/zed/crates/language_core/src/diagnostic.rs`
- `repo-ref/zed/crates/language/src/diagnostic_set.rs`
- `repo-ref/zed/crates/editor/src/display_map.rs`

The relevant architectural lesson is the separation, not the exact implementation:

- diagnostics are data tied to buffer ranges,
- display chunks can carry current diagnostic severity,
- rendering chooses colors/underline/gutter affordances later.

## Fret Contract

New view-layer items:

- `DiagnosticSeverity`
- `DiagnosticSourceKind`
- `DiagnosticSpan`
- `DiagnosticSpanError`
- `validate_diagnostic_spans`
- `normalized_diagnostic_spans`

Owner layer:

- crate: `fret-code-editor-view`
- coordinate space: `TextBuffer` UTF-8 byte ranges
- UI policy: explicitly out of scope

## v1 Semantics

- Ranges are buffer byte ranges, not display rows and not UTF-16 ranges.
- Empty ranges are valid because point diagnostics are common.
- Ranges must be in bounds and on UTF-8 char boundaries.
- Overlaps are valid because multiple sources can annotate the same text.
- Normalization sorts deterministically and does not merge, drop, or de-duplicate diagnostics.
- Severity order is `Error < Warning < Information < Hint`, matching max-severity filter style.

## Why This Is Not in `fret-code-editor`

The widget will eventually render diagnostics, expose hover affordances, and connect commands/code
actions. Those are surface-layer concerns. The first hard-to-change contract is lower: what
coordinate system a diagnostic payload uses and how it stays valid against a buffer.

## Follow-ups

1. Add a gutter marker projection that can derive line/display-row summaries from diagnostic spans.
2. Add decoration payloads that can share the same range validation.
3. Wire a feature-heavy UI Gallery proof only after the data model can be tested without rendering.
4. Add perf counters when diagnostic payloads become part of the paint path.
