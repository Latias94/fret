# imui stack fearless refactor v2 - TODO

Tracking doc: `docs/workstreams/imui-stack-fearless-refactor-v2/DESIGN.md`

Milestones: `docs/workstreams/imui-stack-fearless-refactor-v2/MILESTONES.md`

Baseline audit: `docs/workstreams/imui-stack-fearless-refactor-v2/BASELINE_AUDIT_2026-03-31.md`

Teaching-surface audit:
`docs/workstreams/imui-stack-fearless-refactor-v2/TEACHING_SURFACE_AUDIT_2026-03-31.md`

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

- [x] Re-audit `fret-ui-kit::imui` shipped nouns against the current code, not stale docs.
- [x] Delete stale doc claims that still describe already-shipped generic helpers as missing.
- [x] Decide whether any remaining `fret-ui-kit::imui` helper has become redundant after proof/demo
      migration.
- [x] Do not split the remaining `fret-ui-kit::imui` root surface yet because the current split
      already matches ownership better than another coordination-layer shuffle would.
- [x] Keep official adapter seams generic over authoring traits and avoid concrete `ImUi` coupling.

Generic audit result (2026-03-31):

- `fret-ui-kit::imui` already ships the generic vocabulary that older notes historically opened as
  gaps: `selectable`, `combo`, `combo_model`, `table`, `virtual_list`, `separator_text`,
  `collapsing_header`, `tree_node`, tooltip helpers, typed `drag_source` / `drop_target`, and the
  floating surface family (`floating_layer`, `floating_area`, `window_with_options`).
- The main doc cleanup need was not current v2 drift; it was older historical notes whose headings
  and opening paragraphs still read like a live gap board.
- Historical `imui-authoring-vocabulary-closure-v1` notes should remain as archive evidence, but
  must now read as closed historical gap snapshots rather than current missing-surface claims.
- The only clearly redundant first-party helper path found in this audit was the built-in sample
  wrapper pair in `fret_ui_kit::imui::adapters`; those examples should live in tests or external
  crates, while the public module stays contract-only (`AdapterSignal*` + `report_adapter_signal`).
- `ecosystem/fret-ui-kit/src/imui.rs` still acts as the coordination surface for options,
  responses, `ImUiFacade`, and `UiWriterImUiFacadeExt`, while behavior-heavy logic already lives in
  dedicated submodules (`combo_controls`, `drag_drop`, `popup_overlay`, `tooltip_overlay`, etc.).
  Splitting the root file again in this lane would mostly move tightly coupled coordination code
  across files without improving owner clarity.
- The final surface rule for this lane is:
  editor-owned declarative forwarders stay on `&mut impl fret_authoring::UiWriter<H>`, while
  generic immediate adapter seams compile against `UiWriterImUiFacadeExt<H>` and remain free of any
  concrete `fret_imui::ImUi` dependency.

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

- [x] Verify docs and proof surfaces no longer teach bypasses or stale gap statements.
- [x] Verify each surviving helper family has one clear owner.
- [ ] Capture a final audit of:
      - what survived,
      - what was newly promoted,
      - what remains intentionally declarative-only,
      - and what was deleted.

Teaching-surface audit result (2026-03-31):

- Active first-party `imui` teaching surfaces now lock the current facade imports and official
  adapter entrypoints in source-policy tests under `apps/fret-examples` and `apps/fret-cookbook`.
- No active example or cookbook surface reintroduces deleted historical names such as
  `select_model_ex`, `window_ex`, `window_open_ex`, `floating_area_show_ex`, or
  `begin_disabled`, and none of them teach the contract-only `fret_ui_kit::imui::adapters`
  module.
- The surviving owner split is now explicit in code and docs:
  generic immediate teaching uses `fret_ui_kit::imui::UiWriterImUiFacadeExt`,
  editor-owned nouns use `fret_ui_editor::imui`,
  and `imui_node_graph_demo` remains explicitly labeled as a retained-bridge compatibility example
  rather than the default downstream authoring path.
