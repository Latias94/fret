# imui authoring vocabulary closure v1 - closeout audit

Status: closed
Last updated: 2026-03-31

Related:

- `docs/workstreams/imui-authoring-vocabulary-closure-v1/DESIGN.md`
- `docs/workstreams/imui-authoring-vocabulary-closure-v1/TODO.md`
- `docs/workstreams/imui-authoring-vocabulary-closure-v1/MILESTONES.md`
- `docs/workstreams/imui-authoring-vocabulary-closure-v1/GAP_AUDIT_2026-03-31.md`

## Scope closed

This workstream set out to close the remaining high-frequency immediate authoring nouns that still
made day-to-day editor UI more awkward than Dear ImGui / egui:

- `selectable`
- generic `combo`
- immediate `table`
- generic `virtual_list`
- `separator_text`

All of those slices are now landed in `ecosystem/fret-ui-kit::imui`.

## Final conclusion

The repo direction was correct.
The missing work was vocabulary closure, not another runtime rewrite.

The closeout result is:

- one canonical generic selection-row helper,
- one canonical generic combo path,
- one bounded immediate table helper distinct from layout grid,
- one canonical virtualized-list helper built directly on the runtime virtual-list substrate,
- and one narrow section-label chrome helper.

No compatibility aliases were added for the superseded path that this lane intentionally replaced.

## Ownership check

The owner split remains aligned with repository architecture:

- generic immediate helpers live in `ecosystem/fret-ui-kit::imui`
- editor-only composites remain in `ecosystem/fret-ui-editor::imui`
- shell/docking/workspace policy did not leak into generic `imui`
- no new mechanism/policy pressure was pushed into `crates/fret-ui`

## Evidence anchors

Implementation:

- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/virtual_list_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/separator_text_controls.rs`

Focused gates:

- `ecosystem/fret-ui-kit/tests/imui_selectable_smoke.rs`
- `ecosystem/fret-ui-kit/tests/imui_combo_smoke.rs`
- `ecosystem/fret-ui-kit/tests/imui_table_smoke.rs`
- `ecosystem/fret-ui-kit/tests/imui_virtual_list_smoke.rs`
- `ecosystem/fret-ui-kit/tests/imui_separator_text_smoke.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`

Proof surface:

- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`

## Recommended follow-up

Do not reopen this workstream for broader widget accretion.

Future `imui` work should only open a new lane if there is new evidence for:

- a genuinely missing high-frequency immediate noun,
- a boundary mistake between `imui` and editor/shell layers,
- or a regression that invalidates one of the gates listed above.
