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
- [x] Converge editor-local and document fallback history on the canonical `edit.undo` /
      `edit.redo` route; the temporary `text.undo` / `text.redo` aliases are retired.
- [x] Extend focused command availability beyond `select_all` for the editor-handled undo, redo,
      copy, cut, paste, and word movement commands.
- [x] Define how hover/completion/code-action overlays compose with Fret overlay/focus policy
      without putting overlay policy into `fret-code-editor`.

## P1 - Implementation Slices

- [x] Land the first diagnostics/decorations/gutter API slice with tests.
  - [x] Diagnostic span + logical-line summary view-layer APIs and tests.
  - [x] Gutter marker payload API and tests.
  - [x] Range decoration payload API and tests.
- [x] Land the widget-facing feature payload store with public setters/readouts, buffer-revision
      clearing, display-map gutter validation, and row scene cache epoch wiring.
- [x] Land the first completion/hover/code-action assist request and payload contract without
      adding overlay/focus/listbox policy to `fret-code-editor`.
- [x] Add UI Gallery or example coverage that combines diagnostics, gutter markers, syntax, folds,
      inlays, soft wrap, selection, and an ecosystem-owned overlay-style hook.
  - [x] Add an anchored text-assist overlay proof on `code_editor_torture`.
- [x] Add a diagnostics bundle assertion for feature payload stability.
- [x] Split monolithic editor tests into feature-owned test modules or fixture-driven runners.
  - [x] Move shared scroll-audit/test-telemetry helpers into `ecosystem/fret-code-editor/src/editor/tests/support.rs`.
  - [x] Split at least one more feature-owned behavior cluster out of `tests/mod.rs` (`syntax-rust` cache regression tests into `ecosystem/fret-code-editor/src/editor/tests/syntax.rs`).
  - [x] Extract feature payload API tests into
        `ecosystem/fret-code-editor/src/editor/tests/feature_payloads.rs`.
  - [x] Extract geometry helper/keying tests into
        `ecosystem/fret-code-editor/src/editor/tests/geometry.rs`.
  - [x] Extract row-text cache and paint-frame cache-floor tests into
        `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`.
  - [x] Extract accessibility composed-window and mapping tests into
        `ecosystem/fret-code-editor/src/editor/tests/accessibility.rs`.
  - [x] Extract platform text input and IME preedit semantic tests into
        `ecosystem/fret-code-editor/src/editor/tests/platform_text_input.rs`.
  - [x] Extract platform text input bounds/index roundtrip tests into
        `ecosystem/fret-code-editor/src/editor/tests/platform_text_input_roundtrip.rs`.
  - [x] Extract pointer helper tests into
        `ecosystem/fret-code-editor/src/editor/tests/pointer_helpers.rs`.
  - [x] Extract pointer selection tests into
        `ecosystem/fret-code-editor/src/editor/tests/pointer_selection.rs`.
  - [x] Extract word navigation tests into
        `ecosystem/fret-code-editor/src/editor/tests/word_navigation.rs`.
  - [x] Extract caret navigation tests into
        `ecosystem/fret-code-editor/src/editor/tests/caret_navigation.rs`.
  - [x] Extract keyboard command/interaction tests into
        `ecosystem/fret-code-editor/src/editor/tests/keyboard_commands.rs`.
  - [x] Extract preedit rich-text paint tests into
        `ecosystem/fret-code-editor/src/editor/tests/preedit_paint.rs`.
  - [x] Extract display-row navigation tests into
        `ecosystem/fret-code-editor/src/editor/tests/display_navigation.rs`.
  - [x] Extract row geometry cache tests into
        `ecosystem/fret-code-editor/src/editor/tests/row_geom_cache.rs`.
  - [x] Extract state lifecycle tests into
        `ecosystem/fret-code-editor/src/editor/tests/state_lifecycle.rs`.
  - [x] Extract editor scroll window test into
        `ecosystem/fret-code-editor/src/editor/tests/scroll_window.rs`.
  - [x] Extract residual syntax-window, paint-guard, fold-lifecycle, and edit-refresh tests into
        feature-owned modules.
  - [ ] Split large internal owners only when the public API or test ownership benefits:
  - [x] feature payload store/snapshot,
  - [x] diagnostics/perf snapshots,
  - [x] state schema,
  - [x] state methods,
  - [x] state initializer,
  - [x] handle module,
  - [x] handle method boundary,
  - [x] a11y,
  - [x] input,
    - [x] edit transactions / IME delete-surrounding / undo-redo / row-geom cache shift,
    - [x] keyboard and command dispatch,
    - [x] caret navigation and pointer selection,
    - [x] clipboard effects,
  - [ ] paint,
    - [x] row-scene cache freshness / replay / store,
    - [x] row-text cache materialization / freshness / store,
    - [x] row-geom cache freshness / touch / store,
    - [x] row-rich cache / syntax mapping / prefetch / materialization,
  - [x] syntax,
    - [x] syntax prefetch runtime types,
    - [x] syntax cache invalidation/population/materialization,
  - [x] diagnostics/decorations.

## P2 - Performance and Diagnostics

- [x] Require p50/p95/max and renderer payload evidence for any hot-path editor change.
  - [x] Close the lane-level rule in
        `M5_PERF_CONTRACT_CLOSURE_2026-05-12.md`; future hot-path editor changes must use
        existing editor perf contracts or add/reseed a scoped contract with p50/p95/max and
        relevant payload fields.
- [x] Add feature-payload counters once diagnostics/decorations are implemented.
- [x] Compare feature-heavy editor stressors against the existing complex wheel and autoscroll
      contracts.
  - [x] Record resize, autoscroll steady, autoscroll typical, complex wheel, and row-scene
        replay/store contract roles in `M5_PERF_CONTRACT_CLOSURE_2026-05-12.md`.
- [ ] Add Linux evidence when a Linux runner/profile is available; do not infer it from Windows
      RTX 4090 baselines.

## Deferred

- Full LSP client integration.
- Multi-cursor editing implementation.
- Rectangular/block selection.
- Minimap and sticky scroll.
- Embedded widgets/composable rows beyond what measured evidence requires.
