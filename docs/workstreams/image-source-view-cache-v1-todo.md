# Image Source + ViewCache Correctness v1 (Tracking)

Last updated: 2026-02-13

This file tracks concrete work for:

- `docs/workstreams/image-source-view-cache-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done
- `[!]` blocked

ID convention:

- `IMGVC-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script name

## Phase 0 — Decide naming + dependency boundaries

- [ ] IMGVC-feat-001 Add `ui` feature to `ecosystem/fret-ui-assets`.
  - Rationale: matches other ecosystem crates (`fret-query`, `fret-selector`).

- [ ] IMGVC-feat-002 Add `query-integration` feature to `ecosystem/fret-ui-assets`.
  - Rationale: matches `ecosystem/fret-router` naming.
  - Non-goal: do not make `fret-query` a required dependency for image loading.

## Phase 1 — Make image loading observable (ViewCache-safe)

- [ ] IMGVC-core-100 Introduce an app-global signal model per request (`Model<ImageSourceUiSignal>`).
  - Must track last-use for GC.
  - Must avoid holding `ModelStore` borrows across user code.
  - Evidence:
    - `ecosystem/fret-ui-assets/src/...`

- [ ] IMGVC-core-110 Ensure async decode completion updates the observed model.
  - Evidence:
    - `ecosystem/fret-ui-assets/src/image_source.rs` (inbox drainer)

- [ ] IMGVC-core-120 Ensure GPU-ready events trigger invalidation under ViewCache.
  - v1: `UiAssets::handle_event(...)` notifies ImageSource by `ImageAssetKey` and bumps per-request models.

## Phase 2 — UI API (no query required)

- [ ] IMGVC-ui-200 Add `ImageSourceElementContextExt::use_image_source(...)` behind `fret-ui-assets/ui`.
  - Must observe the model with `Invalidation::Paint` (default) and return a data-only state.
  - Evidence:
    - `ecosystem/fret-ui-assets/src/ui.rs`

- [ ] IMGVC-ui-210 Update UI Gallery Card cover demo to use the new API (remove any continuous-frame hacks).
  - Evidence:
    - `apps/fret-ui-gallery/src/ui/previews/gallery/atoms/card.rs`

## Phase 3 — Optional query integration (ergonomics)

- [ ] IMGVC-query-300 Add helpers behind `fret-ui-assets/query-integration` that use `fret-query` to drive decode/fetch.
  - Must not require `fret-query` for the baseline UI API.
  - Evidence:
    - `ecosystem/fret-ui-assets/src/query_integration.rs` (or similar)

## Phase 4 — Regression gates

- [ ] IMGVC-diag-400 Add a UI Gallery diag screenshot script for the Card cover image case.
  - Evidence:
    - `tools/diag-scripts/ui-gallery-card-image-event-cover-screenshot.json`

- [ ] IMGVC-test-410 Add at least one unit/integration test ensuring ViewCache does not stall image completion.
  - Acceptable: a focused harness test that asserts the observed model changes after inbox/event drive.
  - Evidence:
    - `ecosystem/fret-ui-assets/tests/...`
