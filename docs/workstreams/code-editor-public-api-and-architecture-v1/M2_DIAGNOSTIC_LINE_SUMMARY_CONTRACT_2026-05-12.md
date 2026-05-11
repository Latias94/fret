# M2 Diagnostic Line Summary Contract - 2026-05-12

Status: Second extension-model code slice

This slice extends the diagnostic span contract into the first logical-line projection that gutter,
overview, and future display-row consumers can share without depending on widget paint policy.

## Contract Surface

New view-layer items:

- `DiagnosticLineSummary`
- `diagnostic_line_summaries`

Owner layer:

- crate: `fret-code-editor-view`
- input coordinate space: `TextBuffer` UTF-8 byte ranges from `DiagnosticSpan`
- output coordinate space: `TextBuffer` logical line indexes
- UI policy: explicitly out of scope

## v1 Semantics

- Input spans reuse `validate_diagnostic_spans`; invalid byte ranges, out-of-bounds ranges, and
  non-UTF-8-boundary ranges bubble the same `DiagnosticSpanError`.
- Empty diagnostic ranges map to `TextBuffer::line_index_at_byte(start)`.
- Non-empty ranges are half-open: `start..end` covers the line containing the previous character
  boundary before `end`.
- A range ending exactly at the start of the next line does not include that next line.
- Output summaries are sorted by logical line.
- Overlapping diagnostics are counted; they are not merged, dropped, or de-duplicated.
- `most_severe` uses the existing diagnostic severity order where `Error` sorts before
  `Warning`, `Information`, and `Hint`.
- Boolean flags are line-level rollups only: `has_primary`, `has_unnecessary`, `has_deprecated`,
  and `has_underline`.

## Why This Is Still View-Layer

Line summaries are closer to gutter and overview rendering, but they still express only stable
coordinate/data facts. The widget layer will choose icons, colors, hover text, code-action affordance
policy, and display-row projection. Keeping this in `fret-code-editor-view` lets tests validate the
hard coordinate behavior without creating a UI dependency.

## Evidence

- Implementation: `ecosystem/fret-code-editor-view/src/diagnostics.rs`
- Public export: `ecosystem/fret-code-editor-view/src/lib.rs`
- Unit tests:
  - point diagnostics map to their logical line,
  - multi-line ranges cover all touched logical lines,
  - half-open range ends do not include the next line,
  - overlapping diagnostics count and retain the most severe value,
  - validation errors bubble unchanged.

## Gates

```powershell
cargo fmt -p fret-code-editor-view --check
cargo check -p fret-code-editor-view
cargo nextest run -p fret-code-editor-view --lib --no-fail-fast
```

## Follow-ups

1. Use `M2_GUTTER_MARKER_CONTRACT_2026-05-12.md` when wiring diagnostic summaries into gutter
   markers.
2. Add range decorations that share the same buffer-range validation vocabulary.
3. Wire a UI Gallery proof after diagnostics, gutter markers, and decorations have stable inputs.
