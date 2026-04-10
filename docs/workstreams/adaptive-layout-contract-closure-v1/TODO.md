# Adaptive Layout Contract Closure v1 — TODO

Status: Active
Last updated: 2026-04-10

## Lane opening

- [x] ALC-001 Open a new adaptive execution lane instead of silently widening the older
  `container-queries-v1` or `environment-queries-v1` implementation trackers.
- [x] ALC-002 Record the assumptions, boundaries, and first proof surfaces for adaptive work.

## M0 — Baseline and inventory freeze

- [x] ALC-010 Inventory the current adaptive surface across:
  - `fret::env`,
  - `fret-ui-kit` query helpers,
  - `fret-ui-shadcn` recipe APIs,
  - UI Gallery snippets/pages,
  - and GenUI adaptive strategy components.
- [x] ALC-011 Classify current responsive behavior into:
  - container-driven,
  - environment-driven,
  - caller-owned shell sizing,
  - or current debt / ambiguous ownership.
- [x] ALC-012 Record the first drift list for targeted refactors, including:
  - duplicated breakpoint seams,
  - raw viewport reads,
  - and page-level fixed-width shells that make gallery proofs brittle.

## M1 — Adaptive taxonomy freeze

- [x] ALC-020 Freeze the v1 adaptive feature set for the public story:
  - breakpoint tokens,
  - container queries,
  - environment/capability queries,
  - safe-area/occlusion helpers,
  - strategy-layer adaptive components,
  - and caller-owned shell sizing rules.
- [x] ALC-021 Decide whether the current `fret::env` export surface is sufficient or needs a
  narrower cleanup pass without broadening the default prelude.
- [x] ALC-022 Decide where composable `children(...)` APIs are genuinely needed for adaptive
  authoring and where recipe-specific builders should remain the default.

## M2 — Proof surfaces and gates

- [x] ALC-030 Keep the existing popup/menu narrow-surface proof green and treat it as this lane's
  first gallery-level regression anchor.
- [ ] ALC-031 Add or refresh one explicit Gallery proof surface that compares container-driven and
  viewport-driven behavior without mixing them.
- [x] ALC-032 Promote one panel-resize proof surface that validates container-first behavior while
  the window size stays fixed.

## M3 — First fearless-refactor slices

- [x] ALC-040 Choose the first bounded drift sweep from the initial inventory, likely among:
  - dialog / alert-dialog responsive alignment fallbacks,
  - drawer / sheet mobile-shell thresholds,
  - carousel breakpoint vocabulary and page copy,
  - data-table / pagination / navigation-menu responsive teaching surfaces.
- [x] ALC-041 Land the smallest slice with tests or diag evidence before widening scope.
- [x] ALC-042 Pin the editor rail / inspector sidebar owner layer so app-shell `Sidebar` does not
  silently widen into the editor-panel adaptive story.
- [x] ALC-043 Decide whether the next reusable editor rail slice belongs in `fret-ui-editor`, a
  workspace-shell layer, or can stay app-local after the panel-resize proof is promoted.
- [ ] ALC-044 Promote one reviewable editor-rail composition that uses the existing
  `WorkspaceFrame.left/right` shell seam plus editor-owned inner panel content before extracting a
  new public rail primitive.

## M4 — Docs, closeout, or follow-on split

- [ ] ALC-050 Refresh roadmap, todo tracker, known issues, and usage docs as the adaptive story
  gets frozen.
- [ ] ALC-051 Decide whether older workstream docs need status notes or can remain pure mechanism
  references.
- [ ] ALC-052 Close this lane explicitly or split a narrower follow-on once the first fearless
  slices reveal the remaining backlog shape.

## Boundaries to protect

- Do not merge container and environment queries into one ambiguous helper path.
- Do not move responsive policy into `crates/fret-ui`.
- Do not accept raw `cx.bounds` magic numbers as the default story when an existing query helper
  fits.
- Do not widen generic `children(...)` APIs without source-aligned proof.

Completed M0 evidence:

- `docs/workstreams/adaptive-layout-contract-closure-v1/BASELINE_AUDIT_2026-04-10.md`

Completed M1 evidence:

- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M1_CONTRACT_FREEZE_2026-04-10.md`

Completed narrow-width slice evidence:

- `apps/fret-ui-gallery/src/ui/snippets/dialog/demo.rs`
- `apps/fret-ui-gallery/tests/dialog_docs_surface.rs`
- `apps/fret-ui-gallery/tests/popup_menu_narrow_surface.rs`
- `apps/fret-ui-gallery/tests/combobox_docs_surface.rs`
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-demo-narrow-sweep.json`
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json`

Completed adaptive naming slice evidence:

- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `ecosystem/fret-ui-shadcn/src/field.rs`
- `ecosystem/fret-ui-shadcn/tests/combobox_responsive_breakpoint.rs`
- `ecosystem/fret-ui-shadcn/tests/field_responsive_orientation.rs`
- `apps/fret-ui-gallery/tests/combobox_docs_surface.rs`
- `apps/fret-ui-gallery/tests/field_docs_surface.rs`

Completed sidebar boundary-pin slice evidence:

- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
- `docs/audits/shadcn-sidebar.md`

Completed editor-panel owner audit evidence:

- `docs/workstreams/adaptive-layout-contract-closure-v1/EDITOR_PANEL_SURFACE_AUDIT_2026-04-10.md`

Completed workspace-rail seam audit evidence:

- `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`

Completed panel-resize gate promotion evidence:

- `docs/workstreams/adaptive-layout-contract-closure-v1/M2_PANEL_RESIZE_GATE_PROMOTION_2026-04-10.md`
- `tools/diag-scripts/container-queries-docking-panel-resize.json`
- `tools/diag-scripts/docking/container-queries/container-queries-docking-panel-resize.json`
