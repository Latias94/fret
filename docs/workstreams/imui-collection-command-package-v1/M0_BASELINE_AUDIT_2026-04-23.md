# ImUi Collection Command Package v1 - M0 Baseline Audit

Date: 2026-04-23
Status: baseline frozen

## What we re-read

- `docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/tests/imui_editor_collection_context_menu_surface.rs`
- `apps/fret-examples/tests/imui_editor_collection_rename_surface.rs`

## Assumptions-first restatement

1. The closed collection modularization lane already reset the default next non-multi-window priority to broader app-owned command-package depth.
2. The current collection proof already has enough local substrate for a first duplicate-selected slice: stable ids, selection owner, context menu, button affordance, and status readouts.
3. A proof-local command status model is sufficient for this lane; system clipboard, platform reveal, or generic command buses are unnecessary.
4. The frozen proof-budget rule still blocks shared helper growth from one proof surface.

## Owner split check

- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
  - should own the first command-package slice.
- `apps/fret-examples/tests/imui_editor_collection_command_package_surface.rs`
  - should freeze the new command-package boundary explicitly.
- `fret-imui`, `fret-ui-kit::imui`, and `crates/fret-ui`
  - should remain unchanged.

## Narrow target

The correct M1 slice is:

1. add one app-owned duplicate-selected command,
2. route it through keyboard, explicit button, and context menu,
3. publish one proof-local command status line,
4. keep selection/active-tile follow-up explicit and app-owned,
5. and leave closeout vs one-more-verb as a later lane decision.
