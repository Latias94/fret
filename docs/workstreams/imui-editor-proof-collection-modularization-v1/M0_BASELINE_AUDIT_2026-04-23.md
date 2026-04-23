# ImUi Editor Proof Collection Modularization v1 - M0 Baseline Audit

Date: 2026-04-23
Status: baseline frozen

## What we re-read

- `docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_box_select_surface.rs`
- `apps/fret-examples/tests/imui_editor_collection_rename_surface.rs`

## Assumptions-first restatement

1. The closed collection inline rename lane already landed the current app-owned collection product depth.
2. The current collection proof now spans enough owner-local helpers and models that host-file shape is a real maintenance concern.
3. A demo-local `collection.rs` module is sufficient to reduce that pressure without widening any public surface.
4. The frozen proof-budget rule still blocks shared helper growth from one proof surface.

## Owner split check

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - should become the slim host that routes into the collection module.
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
  - should own collection assets, models, render logic, and unit tests.
- `fret-imui`, `fret-ui-kit::imui`, and `crates/fret-ui`
  - should remain unchanged.

## Narrow target

The correct M1 slice is:

1. move collection implementation into a demo-local module,
2. keep the host on explicit routing only,
3. move unit tests beside the module,
4. add a modularization surface/source-policy floor,
5. and close the lane again once the structural slice is reviewable.
