# M2 Widget Feature Payload Surface

Status: Landed
Date: 2026-05-12

## Decision

`fret-code-editor` now exposes a widget-facing v1 feature payload surface instead of leaving
diagnostics, range decorations, gutter markers, and semantic tokens only in `fret-code-editor-view`.

The public surface is intentionally data-only:

- `CodeEditorHandle::set_diagnostic_spans(...)`
- `CodeEditorHandle::set_range_decorations(...)`
- `CodeEditorHandle::set_gutter_markers(...)`
- `CodeEditorHandle::set_semantic_tokens(...)`
- matching clear methods
- `CodeEditorHandle::diagnostic_line_summaries()`
- `CodeEditorHandle::feature_payload_snapshot()`

The crate root also re-exports the view feature payload types used by these signatures so app
authors can use `fret-code-editor` without importing private implementation modules.

## Resource And Invalidation Semantics

Feature payloads are normalized before storage and are attached to the current buffer revision or
display map epoch:

- diagnostics, range decorations, and semantic tokens are source-backed UTF-8 byte ranges;
- diagnostic line summaries are derived from the normalized diagnostic spans;
- gutter markers may attach to logical lines or display rows, and display-row markers are validated
  against the current `DisplayMap`.

The v1 editor does not remap feature payloads across text edits. Any buffer mutation clears all
stored feature payloads; language services and workspace shells must re-publish payloads for the new
`buffer_revision()`.

Display-map changes revalidate gutter markers and prune out-of-bounds display-row anchors. This
keeps the display-row coordinate vocabulary explicit without pretending that display-row anchors are
source-stable.

Row scene cache freshness now includes `feature_payload_epoch`, so future paint integration cannot
replay stale scenes after payload changes. Syntax/rich-row caches are cleared when semantic/payload
updates may affect paint.

## Diagnostics Surface

UI Gallery diagnostics snapshots now include:

- feature payload schema version,
- feature payload epoch,
- buffer revision,
- display map epoch,
- counts for diagnostic spans, diagnostic line summaries, range decorations, gutter markers, and
  semantic tokens.

This is the first bundle-facing hook for future feature-heavy editor gates. It does not replace the
still-open scripted assertion for feature payload stability.

## Non-Goals

This slice does not implement:

- final visual styling for diagnostics/decorations/gutter/semantic tokens,
- LSP ownership or payload remapping across edits,
- hover/completion/signature/code-action overlay policy,
- the combined UI Gallery feature proof.

Those remain separate slices so the public contract, rendering policy, and performance evidence can
be reviewed independently.

## Evidence

- Public re-exports and handle surface: `ecosystem/fret-code-editor/src/lib.rs`,
  `ecosystem/fret-code-editor/src/editor/mod.rs`
- Cache freshness: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Buffer-edit invalidation: `ecosystem/fret-code-editor/src/editor/input/mod.rs`
- Diagnostics snapshot: `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`
- Public authoring guide: `docs/code-editor.md`
- Public-surface gate: `ecosystem/fret-code-editor/tests/public_surface.rs`
- Focused unit gates: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor -p fret-ui-gallery
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax-rust --no-fail-fast
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
cargo check -p fret-ui-gallery
```
