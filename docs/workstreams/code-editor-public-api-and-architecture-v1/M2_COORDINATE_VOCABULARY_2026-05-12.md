# M2 Coordinate Vocabulary - 2026-05-12

Status: Boundary decision

This slice closes the open coordinate question for editor feature payloads. The intent is to keep
source-backed data stable across view changes while still allowing view-owned features to attach to
wrapped display rows when the UI needs that projection.

## Decision

Use buffer byte ranges as the default storage contract. Use logical lines for aggregate line facts.
Use display points, display rows, and geometry only as view projections.

## Coordinate Types

### Buffer UTF-8 byte ranges

Use for source-backed feature payloads:

- diagnostics,
- range decorations,
- semantic tokens,
- search highlights,
- bracket matches,
- code-action ranges,
- hover/signature source ranges.

Rules:

- ranges are half-open UTF-8 byte ranges in `TextBuffer`,
- ranges must be validated against the current `TextBuffer` and char boundaries,
- payloads should carry a `Revision` or be invalidated by owner policy when the buffer changes,
- LSP UTF-16 conversion belongs in a provider/adapter layer before data enters
  `fret-code-editor-view`.

Current sources:

- `DiagnosticSpan`
- `RangeDecoration`
- `SemanticToken`

### Logical line indexes

Use for stable line-level aggregates that should not change when soft wrap or display composition
changes:

- diagnostic line summaries,
- overview ruler summaries,
- logical-line gutter markers,
- line-level commands such as run/test/bookmark affordances.

Rules:

- indexes are `TextBuffer` logical line indexes,
- line summaries should be derived from buffer ranges instead of copied from display rows,
- visual row placement is decided by the widget using the current `DisplayMap`.

Current sources:

- `DiagnosticLineSummary`
- `GutterMarkerAnchor::LogicalLine`

### Display points

Use for caret, selection, hit-test, and overlay-anchor projections in the current view:

- caret position,
- pointer-hit result,
- hover/completion anchor request,
- a11y cursor/range projection,
- current display-row-local placement decisions.

Rules:

- `DisplayPoint` is a projection through `DisplayMap`,
- it is valid only for the buffer revision plus current wrap/fold/inlay/preedit inputs that built
  the map,
- long-lived feature payloads should not store display points as their primary identity.

Current sources:

- `DisplayMap::byte_to_display_point`
- `DisplayMap::display_point_to_byte`
- `MaterializedDisplayRow`

### Display rows

Use only for view-owned row attachments that intentionally follow wrapped/materialized rows:

- wrapped-row gutter markers,
- current visible-window diagnostics/debug snapshots,
- row-local paint, a11y, and perf attribution,
- overlay anchors after display projection.

Rules:

- display-row payloads require a `DisplayMap` for validation,
- display-row indexes must be re-derived when wrap/fold/inlay/preedit inputs change,
- display-row indexes are not document storage ids,
- if a payload can be expressed as a buffer range or logical line, prefer that primary identity.

Current source:

- `GutterMarkerAnchor::DisplayRow`, validated by `validate_gutter_markers(..., Some(&DisplayMap),
  ...)`.

### Window-space geometry

Use only at the UI surface/overlay layer:

- anchored popover rects,
- hover cards,
- completion panels,
- code-action menus,
- IME/caret-adjacent placement.

Rules:

- geometry is produced after layout,
- geometry is not a model or persistence coordinate,
- placement policy remains in overlay/component layers.

## Owner Matrix

| Coordinate | Primary owner | Use it for | Do not use it for |
| --- | --- | --- | --- |
| Buffer byte range | `fret-code-editor-buffer` / `fret-code-editor-view` | source-backed payloads | overlay placement or row UI ids |
| Logical line | `fret-code-editor-view` | line aggregates and logical gutters | wrapped-row placement |
| Display point | `DisplayMap` projection | caret/hit-test/current anchors | persistent feature identity |
| Display row | `DisplayMap` projection | view-owned row attachments | source storage |
| Window rect | UI surface / overlay layer | placement | model data |

## Consequences

- Diagnostics, decorations, and semantic tokens stay stable when the user toggles soft wrap.
- Gutter markers can choose logical-line attachment for source features or display-row attachment
  for wrapped-row UI features.
- Hover/completion/code-action overlays can ask the editor for current anchor facts without forcing
  overlay policy into `fret-code-editor`.
- Performance counters can distinguish source payload count from visible-row/rendered payload
  count.

## Evidence Anchors

- Buffer model: `ecosystem/fret-code-editor-buffer/src/lib.rs`
- Display map and display points: `ecosystem/fret-code-editor-view/src/lib.rs`
- Diagnostics: `ecosystem/fret-code-editor-view/src/diagnostics.rs`
- Gutter markers: `ecosystem/fret-code-editor-view/src/gutter.rs`
- Range decorations: `ecosystem/fret-code-editor-view/src/decorations.rs`
- Semantic tokens: `ecosystem/fret-code-editor-view/src/semantic_tokens.rs`
- Overlay feature boundary:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M2_OVERLAY_FEATURE_BOUNDARY_2026-05-12.md`

## Follow-Up

1. Keep concrete completion/hover/code-action structs revision-aware and data-first.
2. Add UI proof coverage that exercises logical-line and display-row gutter attachments under soft
   wrap.
3. Add diagnostics payload counters that report source payloads separately from visible-row
   projected payloads.
