# `fret-ui-editor` — ImGui / `fret-ui-precision` Alignment Inventory v1

Status: Active inventory (workstream note; not an ADR)  
Last updated: 2026-02-16

## Purpose

This document inventories the initial editor-grade component targets for `ecosystem/fret-ui-editor` and
tracks alignment against:

- ImGui (outcome + taxonomy reference; not an API compatibility goal): `repo-ref/imgui/`
- The `fret-ui-precision` demo set (visual density / layout inspiration; not authoritative): `repo-ref/fret-ui-precision/`

Non-goals:

- This does not bind the project to ImGui naming or implementation details.
- This does not define new runtime contracts (`crates/fret-ui`). If a gap is discovered, open/update an ADR first.

Related:

- Design / constraints: `docs/workstreams/ui-editor-v1.md`
- TODO tracker: `docs/workstreams/ui-editor-v1-todo.md`

## Layering shorthand

- **A (Primitives)**: editor-grade interaction primitives (sessions, scrubbing, density).
- **B (Controls)**: concrete controls (numeric, color, vec/transform, selects).
- **C (Composites)**: composed surfaces (property grid/panel, palettes).

## Inventory table (v1)

Legend:

- **Milestone**: M1–M5 per `ui-editor-v1-todo.md`
- **ImGui reference**: function name(s) and/or flags (non-normative)
- **Precision reference**: demo file (non-normative)

| Target | Layer | Milestone | ImGui reference (non-normative) | Precision reference (non-normative) | Tokens touched (initial) | Slots / hooks (initial) | Notes / gaps |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `EditSession` | A | M1 | “edit commit/cancel” patterns across widgets | (N/A) | `editor.numeric.*` (settings-adjacent) | `on_commit`, `on_cancel`, `format/parse` hooks | Define commit/cancel + pre-edit restore; used by multiple controls. |
| `DragValueCore` | A | M1 | `DragFloat*`, `DragInt*` (see `repo-ref/imgui/imgui.h`) | `repo-ref/fret-ui-precision/src/components/showcase/demos/DragValueDemo.tsx` | `editor.numeric.scrub_*`, `editor.density.hit_thickness` | `on_change_live`, `on_commit`, `on_cancel` | Modifier semantics: Shift slow, Alt fast; token-tunable multipliers. |
| `NumericInput` | B | M1 | `InputFloat*`, `InputInt*` | `repo-ref/fret-ui-precision/src/components/showcase/demos/NumberInputDemo.tsx` | `editor.density.*` | `parse`, `format`, `on_validate`, `error_slot` | Parse/format hooks; validation affordance; Escape cancels edit. |
| `DragValue<T>` | B | M1 | `DragFloat*`, `DragInt*` | `repo-ref/fret-ui-precision/src/components/showcase/demos/DragValueDemo.tsx` | `editor.numeric.*`, `editor.density.*` | `format/units`, `min/max/step` | Must support double-click-to-type and unit formatting hooks. |
| `Slider<T>` | B | M2.6 | `SliderFloat*`, `VSliderFloat*` | (N/A) | `editor.slider.*`, `editor.density.*` | `format/parse` hooks, `min/max/step`, optional `show_value` | Horizontal slider exists (value display + typing). Missing: vertical/log variants and richer labeling/unit formatting. Per-instance state keying is a regression class: use explicit `id_source` (or default `(callsite, model.id())`) and ensure proof demo model helpers are named/keyed. |
| `PropertyRow` | C | M1 | “2-column item layout” (common inspector pattern) | `repo-ref/fret-ui-precision/src/components/showcase/demos/BuildSettingsDemo.tsx` | `editor.property.column_gap`, `editor.density.row_height` | `label_slot`, `value_slot`, `actions_slot`, `reset_hook` | Slots: label/value/actions; reset-to-default affordance. |
| `EditorDensity` | A | M2 | `ImGuiStyle` metrics (frame padding, item spacing, etc.) | `repo-ref/fret-ui-precision/docs/DESIGN_PHILOSOPHY.md` | `editor.density.*` | (N/A) | Namespaced density tokens; used across composites. |
| `FieldStatus` | A | M2 | Disabled/hovered variants; status text patterns | (N/A) | `editor.*` + theme semantic colors | `status_slot`, query/selector adapters | Optional `query/selector` glue for loading/error/mixed/dirty. |
| `MiniSearchBox` | B | M2 | `InputTextWithHint` | `repo-ref/fret-ui-precision/src/components/showcase/demos/CommandPaletteDemo.tsx` | `editor.density.*` | `on_change`, `on_submit`, `leading_icon_slot` | Used by palette + inspector filtering; clear affordance uses semantic `ui.close` icon (SVG). |
| `PropertyGroup` | C | M2 | `CollapsingHeader` / `TreeNodeEx` patterns | `repo-ref/fret-ui-precision/src/components/showcase/demos/CollapsingHeaderDemo.tsx` | `editor.property.group_header_height` | `header_slot`, `default_open` | Collapsible group header + section; search anchors. |
| `PropertyGrid` | C | M2 | “inspector layout” + child scrolling | `repo-ref/fret-ui-precision/src/components/showcase/demos/BuildSettingsDemo.tsx` | `editor.property.*`, `editor.density.*` | `row_builder`, `virtualization_policy` | Decide virtualization strategy; avoid forcing runtime changes. |
| `Checkbox` | B | M3 | `Checkbox`, mixed/indeterminate patterns | (N/A) | `editor.checkbox.*`, `editor.density.*` | `a11y_label`, `test_id` | Supports `Model<bool>` and `Model<Option<bool>>` (`None` = indeterminate). |
| `EnumSelect` | B | M3 | `Combo` / listbox patterns | `repo-ref/fret-ui-precision/src/components/showcase/demos/AutocompleteDemo.tsx` | `editor.density.*` | `item_renderer`, `filter_policy` | Filterable select; item render slot. |
| `ColorEdit` | B | M3 | `ColorEdit3/4`, `ColorPicker3/4` | `repo-ref/fret-ui-precision/src/components/showcase/demos/ColorPickerDemo.tsx` | `editor.color.*`, `editor.density.*` | `popup_content_slot`, `format_policy` | Start minimal: swatch + hex; popup picker can be incremental. |
| `VecNEdit` | B | M3 | `DragFloat2/3/4` | `repo-ref/fret-ui-precision/src/components/showcase/demos/MatrixInputDemo.tsx` | `editor.numeric.*`, axis color tokens | `axis_label_slot`, `axis_reset_hook` | Implemented as `Vec2Edit/Vec3Edit/Vec4Edit` (axis labels + tokens + optional per-axis reset hooks). Axis groups grow evenly in-row; axis labels use a tinted background for stronger affordance. Evidence: `ecosystem/fret-ui-editor/src/controls/vec_edit.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`. |
| `TransformEdit` | C | M3 | common editor composite | `repo-ref/fret-ui-precision/src/components/showcase/demos/MaterialEditorDemo.tsx` | `editor.density.*` | `layout_variant`, per-section slots | Implemented as `TransformEdit` (position/rotation/scale, optional link-scale). Evidence: `ecosystem/fret-ui-editor/src/controls/transform_edit.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`. |
| `AssetRefField` | B | M3 | “combo + preview” patterns | `repo-ref/fret-ui-precision/src/components/showcase/demos/AssetPickerDemo.tsx` | `editor.density.*` | `data_source`, `preview_slot`, query adapters | UI-only shell; caller injects data; query integration optional. |
| `InspectorPanel` | C | M3 | “left inspector” pattern | `repo-ref/fret-ui-precision/src/components/showcase/demos/BuildSettingsDemo.tsx` | `editor.density.*`, `editor.property.*` | `toolbar_slot`, `sections_slot` | Search + toolbar slots + grid; should look “real”. |
| `GradientEditor` (spike) | C | M4 | gradient editor patterns (not core ImGui) | `repo-ref/fret-ui-precision/src/components/showcase/demos/GradientEditorDemo.tsx` | `editor.color.*`, `editor.numeric.*` | `stop_renderer`, `stop_menu_slot` | Composition proof: reuse ColorEdit + DragValue. Evidence: `ecosystem/fret-ui-editor/src/composites/gradient_editor.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`. |
| `CurveEditor` (candidate) | C | M5 | curves are usually custom (canvas) | `repo-ref/fret-ui-precision/src/components/showcase/demos/CurveEditorDemo.tsx` | `editor.density.*` | `snapping_policy`, `grid_policy` | Defer until M4 spike identifies substrate gaps. |
| `Timeline` (future) | C | P2 | (custom) | `repo-ref/fret-ui-precision/src/components/showcase/demos/KeyframeEditorDemo.tsx` | `editor.density.*` | `virtualization_policy` | Post-v1; heavy state + perf constraints. |

## Composition notes (shadcn-like “small → big”)

The intended composition direction is:

- `DragValueCore` + `NumericInput` → `DragValue<T>`
- `DragValue<T>` → `VecNEdit` / angle/percent controls
- `PropertyRow` + `PropertyGroup` + `PropertyGrid` → `InspectorPanel`
- `ColorEdit` + `DragValue<T>` → `GradientEditor` spike

Authoring direction (Plan A):

- Declarative widgets are the single source of truth.
- imui is a façade authoring frontend that delegates to the same widgets (no parallel runtime).

This mirrors the “small primitives + recipes” approach used elsewhere in the ecosystem:

- `fret-ui` (mechanisms) → `fret-ui-kit` (infra) → design-system/taxonomy crates (recipes)

## Cross-ecosystem integration pressure (intentional)

This workstream is expected to “pull up” customization surfaces across other ecosystem crates:

- Node/plot/chart should be able to adopt editor density and theming without hardcoding styles.
- Minimum expectation for integration readiness:
  - tokens are namespaced and overrideable,
  - key sizes are injectable (row height, padding, hit thickness),
  - render/tooltip/context-menu slots exist for app-specific composition.

### Integration readiness checklist (use for ecosystem follow-ups)

- [ ] No hardcoded colors/spacing that block theming (use tokens / theme aliases).
- [ ] Density-sensitive sizes are injectable or token-driven (row height, padding, hit thickness).
- [ ] Key visuals are overrideable (icons, badges, hover/active chrome).
- [ ] Context menu / tooltip hooks exist where expected (app-owned policy).
- [ ] Rendering is decomposed into slots where domain apps need control (row renderers, item renderers, previews).
