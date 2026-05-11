# TODO

Status: Active checklist
Last updated: 2026-05-12

## P0 - Public API and Model Boundaries

- [x] Create the narrow follow-on lane and first-open state file.
- [x] Capture a baseline public-surface audit for `fret-code-editor`.
- [x] Update the workstream catalog for the new dedicated directory.
- [x] Classify current public exports as stable, experimental, or internal-by-accident.
- [x] Audit every `CodeEditorHandle` method and group it by owner:
  - model mutation,
  - view configuration,
  - interaction policy,
  - diagnostics/perf readout,
  - debug-only hook.
- [x] Re-export `fret-code-editor` public-signature types that were only reachable through the
      private `editor` module.
- [x] Decide whether `Selection` remains in `fret-code-editor` or moves toward buffer/view model
      ownership.
- [x] Add a public surface diff note for the first ownership move (`Selection`).
- [x] Define the minimum public docs for app authors who want a code editor surface without reading
      internal modules.

## P0 - Extension Model

- [x] Specify the first diagnostics data contract as a view-layer buffer-range model.
- [x] Extend the first diagnostics contract into decorations/gutter data contracts.
  - [x] Add diagnostic logical-line summaries for gutter/overview consumers.
  - [x] Add explicit gutter marker payloads.
  - [x] Add range decoration payloads.
- [x] Decide the coordinate vocabulary for feature payloads: buffer byte range, display point,
      logical line, or display row.
  - [x] Diagnostics v1 uses `TextBuffer` UTF-8 byte ranges; line summaries use logical line indexes.
  - [x] Display-row projection is view-owned, requires `DisplayMap` validation, and is allowed for
        wrapped-row gutter attachments or current visible-row evidence.
- [x] Define semantic-token inputs separately from paint colors.
- [x] Define command/keymap/undo grouping boundaries for editor actions.
- [x] Define how hover/completion/code-action overlays compose with Fret overlay/focus policy
      without putting overlay policy into `fret-code-editor`.

## P1 - Implementation Slices

- [x] Land the first diagnostics/decorations/gutter API slice with tests.
  - [x] Diagnostic span + logical-line summary view-layer APIs and tests.
  - [x] Gutter marker payload API and tests.
  - [x] Range decoration payload API and tests.
- [x] Land the widget-facing feature payload store with public setters/readouts, buffer-revision
      clearing, display-map gutter validation, and row scene cache epoch wiring.
- [x] Add UI Gallery or example coverage that combines diagnostics, gutter markers, syntax, folds,
      inlays, soft wrap, and selection.
- [x] Add a diagnostics bundle assertion for feature payload stability.
- [ ] Split monolithic editor tests into feature-owned test modules or fixture-driven runners.
- [ ] Split large internal owners only when the public API or test ownership benefits:
  - state/handle,
  - input,
  - paint,
  - syntax,
  - a11y,
  - diagnostics/decorations.

## P2 - Performance and Diagnostics

- [ ] Require p50/p95/max and renderer payload evidence for any hot-path editor change.
- [x] Add feature-payload counters once diagnostics/decorations are implemented.
- [ ] Compare feature-heavy editor stressors against the existing complex wheel and autoscroll
      contracts.
- [ ] Add Linux evidence when a Linux runner/profile is available; do not infer it from Windows
      RTX 4090 baselines.

## Deferred

- Full LSP client integration.
- Multi-cursor editing implementation.
- Rectangular/block selection.
- Minimap and sticky scroll.
- Embedded widgets/composable rows beyond what measured evidence requires.
