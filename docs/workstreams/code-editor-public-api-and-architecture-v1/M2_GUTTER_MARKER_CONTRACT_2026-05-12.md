# M2 Gutter Marker Contract - 2026-05-12

Status: Third extension-model code slice

This slice adds the first explicit gutter marker payload contract. It is intentionally view-layer
data only: the contract says where a marker attaches and which lightweight payload it carries, but
it does not decide icon rendering, colors, hover surfaces, command dispatch, or hit-test behavior.

## Contract Surface

New view-layer items:

- `GutterMarkerAnchor`
- `GutterMarkerKind`
- `GutterMarkerVisual`
- `GutterMarkerHitTarget`
- `GutterMarker`
- `GutterMarkerError`
- `validate_gutter_markers`
- `normalized_gutter_markers`

Owner layer:

- crate: `fret-code-editor-view`
- logical-line anchors validate against `TextBuffer::line_count`
- display-row anchors validate against `DisplayMap::row_count`
- UI policy: explicitly out of scope

## v1 Semantics

- A marker may attach to a logical line or display row.
- Logical-line markers validate against the buffer line count.
- Display-row markers require a `DisplayMap` during validation.
- Marker `kind` is semantic, not visual policy: diagnostic, breakpoint, bookmark, runnable, diff,
  or custom.
- Marker `visual` is an optional symbolic payload (`None`, `Icon`, or `Text`), not a concrete
  renderer asset.
- `tooltip` and `action_id` are data identifiers only; the widget/app layer owns popup composition
  and command execution.
- `normalized_gutter_markers` sorts deterministically by anchor, descending priority, semantic kind,
  visual payload, action id, tooltip, and hit target.

## Why This Is Not in `fret-code-editor`

The editor widget will decide line-number layout, hover affordances, clickable targets, and icon
painting. Those are UI-surface concerns. The hard-to-change part is the payload and coordinate
contract, so this lives in `fret-code-editor-view` next to `DisplayMap` and diagnostic summaries.

## Evidence

- Implementation: `ecosystem/fret-code-editor-view/src/gutter.rs`
- Public export: `ecosystem/fret-code-editor-view/src/lib.rs`
- Unit tests:
  - logical line bounds validation,
  - display row bounds validation with `DisplayMap`,
  - display row validation requiring a display map,
  - deterministic marker sorting,
  - action hooks represented as data identifiers, not callbacks.

## Gates

```powershell
cargo fmt -p fret-code-editor-view --check
cargo check -p fret-code-editor-view
cargo nextest run -p fret-code-editor-view --lib --no-fail-fast
```

## Follow-ups

1. Use `M2_RANGE_DECORATION_CONTRACT_2026-05-12.md` for range-based editor feature overlays.
2. Add a widget-level gutter proof after diagnostics, markers, and decorations can be passed as
   explicit extension inputs.
3. Add feature-payload counters once these payloads enter the paint path.
