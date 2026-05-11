# Code Editor Public API and Architecture v1

Status: Active follow-on
Last updated: 2026-05-12

This lane is a narrow follow-on to `docs/workstreams/code-editor-ecosystem-v1/`. The older lane
proved the editor ecosystem can exist as buffer/view/surface crates; this lane decides what public
API and extension boundaries are stable enough for editor-grade applications to build on.

## Problem

`fret-code-editor` is currently a usable editor widget with strong internal evidence: windowed
rows, per-row caches, syntax feature gates, IME/preedit integration, a11y projection, and perf
counters. It is not yet a fully stabilized editor subsystem surface.

The highest-risk gap is not one missing feature or one hot path. The risk is that feature work
keeps adding ad-hoc methods to `CodeEditorHandle` or paint/input internals before the model
boundaries are explicit. That would make later LSP-style features, diagnostics, completion, hover,
multi-cursor, and gutter integration harder to evolve without breaking users.

## Decision

Prioritize the editor work in this order:

1. Public architecture and model boundaries.
2. Feature extension surfaces.
3. Performance tuning only when gates or attribution show a real regression or near-threshold
   stressor.

The existing ADR 0185 split remains the foundation:

- `fret-code-editor-buffer`: document identity, rope-backed text, edit transactions, revision
  semantics, undo-friendly deltas.
- `fret-code-editor-view`: display rows, buffer/display mappings, folds, inlays, inline preedit,
  and view-owned fragment composition.
- `fret-code-editor`: Fret UI integration, windowed surface behavior, input/IME/a11y, paint,
  caches, diagnostics/perf hooks.

## Scope

This lane owns:

- classifying current public exports as stable, experimental, or internal-by-accident,
- designing the target public API for editor-grade apps,
- defining feature extension contracts for diagnostics, decorations, gutter markers, semantic
  tokens, hover, completion, code actions, commands/keymaps, and multi-cursor follow-ups,
- keeping diagnostics/perf gate expectations close to the public API,
- identifying module splits that reduce merge conflicts without changing behavior.

## Non-goals

- Do not move editor policy into `crates/fret-ui`.
- Do not rewrite the renderer or windowed-row engine without measured evidence.
- Do not implement a full LSP client in this lane.
- Do not freeze every candidate type as stable API before a second consumer exists.
- Do not make Linux parity a blocker for public API work; keep platform assumptions explicit and
  add Linux evidence later when it is testable.

## Architecture Stance

The editor should behave like a subsystem with a small stable waist:

- buffer/edit APIs are data-model contracts,
- view/display APIs are deterministic projection contracts,
- UI surface APIs are widget/controller contracts,
- language/editor features are extension inputs, not hard-coded widget methods,
- diagnostics/perf hooks are first-class because editor-grade regressions need bundle evidence.

This is aligned with Zed/GPUI as an architecture reference, but the target is not to copy Zed code.
The target is the same separation of model, display map, visible-window work, and measured cache
behavior.

## Primary Hazards

1. `CodeEditorHandle` can become a dumping ground for every feature knob.
2. Diagnostics, decorations, folds, inlays, and semantic tokens can grow separate coordinate
   systems if they are not normalized under the view/display map layer.
3. Completion and hover can accidentally encode overlay/focus policy inside the editor widget
   instead of composing with Fret's overlay and command layers.
4. Performance work can optimize around today's demos while missing the public extension payloads
   that real editors add.
5. Large monolithic implementation/test files can hide ownership boundaries and make parallel
   refactors risky.

## Success Criteria

- A maintainer can tell which editor APIs are stable, experimental, or internal.
- New editor features have an obvious data contract and owner layer before code is written.
- At least one editor feature surface beyond folds/inlays is specified with repro/gate/evidence.
- Public API changes come with a surface-diff note and focused tests.
- Perf changes remain tied to p50/p95/max and renderer payload evidence, not intuition.
