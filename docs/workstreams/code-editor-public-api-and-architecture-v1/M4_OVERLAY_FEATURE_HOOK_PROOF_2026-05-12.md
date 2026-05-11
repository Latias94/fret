# M4 Overlay Feature Hook Proof

Status: Landed
Date: 2026-05-12

## Decision

The `code_editor_torture` UI Gallery page now includes a recipe-owned anchored text-assist overlay
hook. The hook is intentionally outside `fret-code-editor`:

- `fret-code-editor` continues to own editor state, feature payloads, diagnostics, and paint/cache
  readouts.
- `fret-ui-editor` owns the concrete text-assist field recipe.
- `fret-ui-kit` owns overlay request, placement, dismissal, and focus policy infrastructure.
- `fret-ui-gallery` composes the proof surface and exposes stable `test_id` anchors.

The overlay is closed by default and opens only through the `Open actions` trigger or user input.
This proves the overlay composition path without perturbing existing code-editor scroll/perf
baselines by default.

## Coverage

The same `code_editor_torture` page now combines:

- syntax highlighting,
- diagnostics/decorations,
- gutter markers,
- semantic tokens,
- folds/inlays,
- soft wrap,
- selection/preedit evidence paths,
- an ecosystem-owned anchored overlay hook.

The overlay proof uses `TextAssistFieldSurface::AnchoredOverlay` with stable anchors:

- trigger: `ui-gallery-code-editor-torture-assist-open`,
- field: `ui-gallery-code-editor-torture-assist-field`,
- listbox: `ui-gallery-code-editor-torture-assist-list`,
- first row: `ui-gallery-code-editor-torture-assist.item.feature-overlay-hook`.

## Non-Goals

This does not stabilize final hover, completion, signature-help, or code-action request structs.
Those should still be introduced only when a concrete app-facing feature needs the data shape.

This also does not move overlay policy into `fret-code-editor`.

## Evidence

- Gallery dependency and feature wiring: `apps/fret-ui-gallery/Cargo.toml`
- Overlay proof surface:
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/torture.rs`
- Render-flow gate: `apps/fret-ui-gallery/src/driver/render_flow.rs`
- Existing recipe owner: `ecosystem/fret-ui-editor/src/controls/text_assist_field.rs`
- Overlay infrastructure: `ecosystem/fret-ui-kit/src/overlay_controller.rs`

## Gates

```powershell
cargo fmt -p fret-ui-gallery --check
cargo check -p fret-ui-gallery --features gallery-dev
cargo nextest run -p fret-ui-gallery --features gallery-dev code_editor_torture_assist_overlay_hook_stays_within_window --no-fail-fast
```
