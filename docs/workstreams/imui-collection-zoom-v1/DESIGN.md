# ImUi Collection Zoom v1

Status: closed closeout reference
Last updated: 2026-04-23

Status note (2026-04-23): this document remains the lane-opening rationale. The shipped verdict now
lives in `M1_APP_OWNED_ZOOM_LAYOUT_SLICE_2026-04-23.md` and `CLOSEOUT_AUDIT_2026-04-23.md`.
References below to broader collection/product depth should be read as lane-opening rationale
rather than an invitation to reopen either the collection context-menu folder or generic layout
ownership questions.

Related:

- `M0_BASELINE_AUDIT_2026-04-23.md`
- `M1_APP_OWNED_ZOOM_LAYOUT_SLICE_2026-04-23.md`
- `CLOSEOUT_AUDIT_2026-04-23.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_zoom_surface.rs`
- `repo-ref/imgui/imgui_demo.cpp`

This lane exists because the closed collection context-menu follow-on already proved the current
asset-browser proof can own deeper collection action policy locally, but it explicitly deferred
zoom/layout depth.

The narrow remaining question is now:

> land one app-owned collection zoom/layout slice on the existing proof surface, keep generic
> layout/helper widening closed, and avoid turning one demo's zoom affordance into a new public
> `fret-ui-kit::imui` collection helper or runtime contract.

## Why this is a new lane

This work should not be forced back into `imui-collection-context-menu-v1`.

That folder is already closed on a bounded quick-actions verdict. Reopening it would blur:

- collection depth already shipped
  - background marquee / box-select, collection keyboard owner, delete-selected, context-menu;
- collection depth still open
  - zoom/layout derivation, select-all / rename breadth, and any second proof surface strong
    enough to justify shared helper growth.

This work also should not widen `crates/fret-ui` or generic `fret-ui-kit::imui` collection policy.

## Assumptions-first baseline

### 1) The closed collection context-menu lane already deferred collection zoom/layout depth.

- Evidence:
  - `docs/workstreams/imui-collection-context-menu-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this folder would duplicate or muddy the context-menu closeout instead of owning the next
    narrow collection slice.

### 2) The current proof surface already has the right ingredients for a narrow app-owned collection zoom slice:

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `ecosystem/fret-ui-kit/src/imui/child_region.rs`
  - `crates/fret-ui/src/elements/cx.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would invent extra helper surface instead of exercising the already-shipped wheel and
    scroll-handle seams from one real proof surface.

### 3) The scroll handle and wheel hooks already exist generically, so this lane is not a justification to widen shared helper ownership.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
  - `crates/fret-ui/src/action.rs`
  - `crates/fret-ui/src/scroll/mod.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would accidentally reopen generic runtime/layout questions for a product-owned asset
    browser depth issue.

### 4) Dear ImGui keeps asset-browser zoom and layout recomputation at the proof surface instead of turning them into a generic runtime contract.

- Evidence:
  - `repo-ref/imgui/imgui_demo.cpp`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane could overfit local demo structure rather than following the upstream parity posture.

### 5) The lane should land and close narrowly.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the folder would expand into select-all, rename, or generic helper growth instead of one
    reviewable zoom/layout slice.

## Goals

1. Land one app-owned collection zoom/layout slice on the current collection-first proof.
2. Replace the frozen constant column count with viewport-plus-zoom-derived layout metrics.
3. Reuse the existing scroll-handle and wheel seams instead of inventing a second runtime path.
4. Leave one repro, one gate package, and one evidence set for the slice.

## Non-goals

- Widening `crates/fret-ui`.
- Widening public `fret-ui-kit::imui` with a collection zoom helper.
- Reopening the generic key-owner or menu-policy questions.
- Solving select-all, rename, or broader command palette integration.
- Adding a second proof surface.

## Initial target surface

The current collection proof already has the ingredients this slice needs:

- stable item ids,
- a collection-scoped child region with scroll state,
- wheel hooks at the pointer-region layer,
- keyboard navigation that currently reads a frozen column constant,
- and a closed proof-budget rule that still blocks shared helper growth.

The first landable target is therefore narrow:

1. derive collection layout metrics from viewport width plus an app-owned zoom model,
2. route primary+wheel through one collection-scope zoom policy,
3. keep hovered rows anchored by reusing the existing child-region scroll handle,
4. update keyboard navigation to read derived layout columns instead of a frozen constant,
5. and leave select-all / rename / broader product breadth for different follow-ons.

## Default owner split

### `apps/fret-examples`

Owns:

- the zoom model and collection-local scroll handle,
- the viewport-plus-zoom-derived layout metrics,
- primary+wheel zoom policy and hovered-row anchor math,
- keyboard navigation reading derived columns,
- and the source-policy teaching surface for this lane.

### `fret-ui-kit::imui`

Owns:

- the existing child-region scroll-handle seam and wheel hooks this proof builds on,
- but not a new public collection zoom helper in this lane.

### Not owned here

- `docs/workstreams/imui-collection-context-menu-v1/`
  - remains the closed quick-actions verdict.
- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- select-all / rename / second proof surface
  - remain separate follow-ons if stronger first-party proof appears later.

## First landable target

Do not start by designing a shared collection zoom helper or runtime-owned layout contract.

The first correct target is:

- one proof-local zoom model plus one proof-local scroll handle in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- a derived layout-metrics helper that feeds both grid rendering and keyboard navigation,
- primary+wheel zoom that keeps hovered rows anchored,
- source-policy and unit-test gates that keep the slice visible,
- and a closeout that freezes generic layout/helper widening as still unjustified.
