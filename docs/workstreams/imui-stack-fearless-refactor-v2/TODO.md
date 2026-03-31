# imui stack fearless refactor v2 - TODO

Tracking doc: `docs/workstreams/imui-stack-fearless-refactor-v2/DESIGN.md`

Milestones: `docs/workstreams/imui-stack-fearless-refactor-v2/MILESTONES.md`

Baseline audit: `docs/workstreams/imui-stack-fearless-refactor-v2/BASELINE_AUDIT_2026-03-31.md`

This board assumes a workspace-wide breaking migration.
Compatibility shims are explicitly out of scope.

## M0 - Source-of-truth reset

- [x] Create a new v2 workstream directory with `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and a
      baseline audit.
- [x] Repoint top-level docs entrypoints to the v2 lane.
- [x] Repoint the immediate-mode workstream map to the v2 lane.
- [x] Record the current shipped-vs-missing surface in a baseline audit.
- [x] Add historical or partially superseded status notes to older `imui` workstreams that still
      read like active guidance.

## M1 - Editor adapter closure freeze

- [x] Audit all public `fret-ui-editor` declarative exports against `fret-ui-editor::imui`.
- [x] Add a thin `field_status_badge(...)` adapter.
- [x] Add a thin `gradient_editor(...)` adapter.
- [x] Decide whether `property_row(...)` is part of the official immediate editor surface.
- [x] Reject `property_row(...)` promotion for this lane and keep it declarative-only.
- [x] If `property_row(...)` stays declarative-only, document that decision explicitly in this
      lane and remove ambiguity from proof/demo code.
- [ ] Delete or rewrite any competing first-party helper path that becomes redundant once the
      adapter closure is in place.

Decision note (2026-03-31):

- `property_row(...)` is intentionally not part of the official `fret-ui-editor::imui` surface.
- Keep `PropertyRow` declarative-only because it is already the row primitive consumed by
  `PropertyGrid`, `PropertyGridVirtualized`, and `GradientEditor`, and a `UiWriter` adapter would
  not remove the nested proof/demo `.into_element(cx)` call sites that still need migration.

Audit result (2026-03-31):

- Top-level editor control/composite nouns are now closed in `fret-ui-editor::imui`:
  `TextField`, `Checkbox`, `ColorEdit`, `DragValue`, `AxisDragValue`, `NumericInput`, `Slider`,
  `EnumSelect`, `MiniSearchBox`, `TextAssistField`, `IconButton`, `FieldStatusBadge`, `Vec2Edit`,
  `Vec3Edit`, `Vec4Edit`, `TransformEdit`, `PropertyGroup`, `PropertyGrid`, `GradientEditor`,
  `PropertyGridVirtualized`, and `InspectorPanel`.
- The remaining declarative exports without immediate adapters are intentional subordinate pieces,
  not missing top-level immediate nouns:
  `PropertyRow`, `PropertyRowReset`, row/context carrier types, callback aliases, binding structs,
  options/outcome enums, and similar support types.

## M2 - Proof/demo migration

- [x] Update the immediate side of `imui_editor_proof_demo` to use promoted editor adapters.
- [x] Keep the declarative comparison side explicit and do not mix declarative direct calls into the
      immediate column when an official adapter exists.
- [x] Remove direct `.into_element(cx)` immediate-side usage for editor surfaces that now have an
      official adapter.
- [x] Preserve or improve `test_id` stability while moving proof/demo call sites.

Clarification (2026-03-31):

- The remaining direct `FieldStatusBadge::new(...).into_element(cx)` and
  `GradientEditor::new(...).into_element(cx)` usage in `imui_editor_proof_demo` lives inside the
  declarative `InspectorPanel` proof subtree and is intentionally not treated as immediate-side
  bypass.

## M3 - Generic surface and ownership cleanup

- [ ] Re-audit `fret-ui-kit::imui` shipped nouns against the current code, not stale docs.
- [ ] Delete stale doc claims that still describe already-shipped generic helpers as missing.
- [ ] Decide whether any remaining `fret-ui-kit::imui` helper has become redundant after proof/demo
      migration.
- [ ] Split remaining large `fret-ui-kit::imui` files only when the split sharpens ownership or
      reviewability.
- [ ] Keep official generic/editor adapters on `&mut impl fret_authoring::UiWriter<H>` and avoid
      concrete `ImUi` coupling.

## M4 - Gates and evidence closure

- [x] Extend `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs` or an equivalent policy gate
      for any newly promoted editor adapter.
- [x] Extend `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs` to compile the newly promoted
      editor adapters.
- [x] Add or extend one runtime smoke/proof path that exercises the new adapters from a real
      immediate authoring surface.
- [x] Keep `cargo nextest run -p fret-imui --lib` green.
- [x] Keep
      `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy`
      green.
- [x] Keep
      `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke`
      green.

## M5 - Delete-ready closeout

- [ ] Verify docs and proof surfaces no longer teach bypasses or stale gap statements.
- [ ] Verify each surviving helper family has one clear owner.
- [ ] Capture a final audit of:
      - what survived,
      - what was newly promoted,
      - what remains intentionally declarative-only,
      - and what was deleted.
