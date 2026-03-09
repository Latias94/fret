# Editor Ecosystem Fearless Refactor v1 - Imgui-like Preset Draft

Tracking doc: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`
Related matrix: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/PARITY_MATRIX.md`

Status: Draft
Last updated: 2026-03-09

## Purpose

This note defines how Fret should support an imgui-like editor look and authoring feel without:

- creating a second widget implementation tree,
- copying Dear ImGui APIs literally,
- or binding editor widgets to one design system.

The key rule is:

- authoring ergonomics come from `fret-imui`,
- editor widgets come from `fret-ui-editor`,
- workspace shell chrome comes from `fret-workspace`,
- and the imgui-like appearance is a preset / skin layered on top of `editor.*` and `workspace.*`.

## Non-goals

- API compatibility with Dear ImGui.
- Pixel-identical reproduction of Dear ImGui's default theme.
- Replacing shadcn/material skins as the only supported visual direction.

## Layering decision

| Concern | Owner | Why |
| --- | --- | --- |
| Immediate-style syntax (`ui.xxx(...)`, `Response`) | `ecosystem/fret-imui` | This is authoring ergonomics, not widget ownership. |
| Editor controls and composites | `ecosystem/fret-ui-editor` | Single source of truth for editor-grade widgets. |
| Workspace shell chrome | `ecosystem/fret-workspace` | Shell chrome is distinct from editor controls. |
| Dock-aware tabs and drop surfaces | `ecosystem/fret-docking` | Dock-graph-aware behavior remains docking-owned. |
| Imgui-like visual preset | adapter / preset layer | This is a skin, not a new core widget crate. |

## Preset shape

Recommended landing path:

1. Start as a preset module or theme patch document that targets `editor.*` and `workspace.*`.
2. Validate it in proof demos.
3. Only create a dedicated adapter crate if there is repeated reuse pressure.

Recommended naming direction:

- `imgui_like_dense`
- `imgui_like_classic`

Avoid naming the core widgets after ImGui.

## What "imgui-like" means in Fret

This preset should target outcomes, not branding:

- dense layout with low padding and compact row height,
- strong frame boundaries around interactive fields,
- minimal corner radius,
- clear hovered / active / disabled deltas,
- direct numeric editing feel,
- compact tab and toolbar chrome,
- low-ceremony visual noise compared to shadcn-style softer surfaces.

## Token plan

The preset should not introduce a new token namespace.
It should seed or alias existing namespaces.

### Editor token families

| Token family | Imgui-like direction | Notes |
| --- | --- | --- |
| `editor.density.row_height` | compact | Keep rows visibly dense but still pointer-usable. |
| `editor.density.padding_x` | compact | Smaller than shadcn-style field padding. |
| `editor.density.padding_y` | compact | Tight vertical packing. |
| `editor.density.hit_thickness` | moderate | Keep handles usable even when visuals are compact. |
| `editor.density.icon_size` | compact | Typically small toolbar and row icons. |
| `editor.numeric.scrub_speed` | direct | Numeric drag should feel immediate, not soft. |
| `editor.numeric.scrub_drag_threshold` | low | Start scrubbing quickly. |
| `editor.property.column_gap` | compact | Inspector columns should feel dense. |
| `editor.property.group_header_height` | compact | Keep group headers tight. |
| `editor.checkbox.size` | compact | Visual checkbox square stays dense. |
| `editor.checkbox.radius` | near-zero or small | Prefer squarer frames. |
| `editor.slider.track_height` | compact | Thin but readable track. |
| `editor.slider.thumb_diameter` | compact | Slightly smaller than app-style sliders. |
| `editor.axis.*` | strong semantic axis colors | X/Y/Z should stay immediately scannable. |

### Workspace token families

| Token family | Imgui-like direction | Notes |
| --- | --- | --- |
| `workspace.frame.*` | flat / low-ornament | Avoid decorative shell treatment. |
| `workspace.top_bar.*` | dense | Toolbar/title region should feel tool-like. |
| `workspace.status_bar.*` | dense | Small but readable footer chrome. |
| `workspace.pane.*` | subtle separation | Pane borders matter more than rounded cards. |
| `workspace.tabstrip.*` | compact / segmented | Tabs should read as tooling chrome, not app pills. |

### Docking token alignment

Docking should not get a separate imgui-only implementation.
Instead:

- docking-owned tokens can be seeded to match the same compact preset,
- `workspace.tabstrip.*` and docking tab chrome should visually align by preset aliasing,
- ownership still remains separate.

## Visual state mapping

| Visual state | Imgui-like expectation | Fret implementation direction |
| --- | --- | --- |
| Idle | clear frame, low radius, muted fill | `EditorWidgetVisuals::inactive` with compact chrome |
| Hovered | stronger border / slightly brighter fill | `EditorWidgetVisuals::hovered` |
| Active / pressed | visibly engaged, not animated fluff | `EditorWidgetVisuals::active` |
| Open | popup/select trigger clearly active | `EditorWidgetVisuals::open` |
| Disabled | reduced contrast but still readable | `EditorWidgetVisuals::disabled` |
| Error | direct border and foreground cue | `editor.numeric.error_*` and shared field-status visuals |

## Authoring model

The preset must work with both authoring frontends below and produce the same widget semantics.

### Declarative-style example

```rust,ignore
controls::DragValue::new(model)
    .preset(editor_presets::imgui_like_dense())
```

### `imui`-style example

```rust,ignore
ui.drag_value_model_ex(
    "Position X",
    &model,
    DragValueOptions::default().preset(editor_presets::imgui_like_dense()),
);
```

The important point is that both calls still end up in the same underlying widget implementation.

## Component implications

The preset assumes these reusable components exist and stay Fret-native:

- `DragValue`
- `NumericInput` / `TextField`
- `Slider`
- `Checkbox`
- `EnumSelect`
- `ColorEdit`
- `Vec2Edit` / `Vec3Edit` / `Vec4Edit`
- `TransformEdit`
- `PropertyRow`
- `PropertyGroup`
- `PropertyGrid`
- `InspectorPanel`

It does not require a separate `ImGuiButton`, `ImGuiSlider`, or `ImGuiPropertyGrid`.

## Recommended rollout

### Phase 1

- finish token inventory for `editor.*` and `workspace.*`
- draft the preset values
- apply them to `imui_editor_proof_demo`

## Current proof surface

The first landing surface is:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`

Current switch behavior:

- default preset: leave `FRET_IMUI_EDITOR_PRESET` unset or set it to `default`
- dense imgui-like preset: set `FRET_IMUI_EDITOR_PRESET=imgui_like_dense` before launching the demo

Current implementation note:

- the preset surface lives in `ecosystem/fret-ui-editor/src/theme.rs`
- the proof demo applies `EditorThemePresetV1` during app init
- the demo header renders the resolved preset name so screenshots / diag captures can prove which
  preset was active
- the proof demo also includes a shared-model parity section:
  declarative editor controls on the left, `fret-ui-editor::imui` adapters on the right
- the proof demo now exposes shared-state readouts with stable `test_id`s so scripted gates can
  prove both authoring frontends mutate the same underlying models
- the current smoke gate for the adapter seam lives in
  `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- the current scripted parity gate lives in
  `tools/diag-scripts/ui-editor/imui/imui-editor-proof-authoring-parity-shared-models.json`
- the promoted launched diagnostics run now passes with
  `FRET_IMUI_EDITOR_PRESET=imgui_like_dense` against `target/debug/imui_editor_proof_demo.exe`
- the current `imui` enum-select proof path uses per-item `test_id`s and `click_stable` with
  `stable_frames: 1`, because the popup item can stop being hit-testable after an extra
  stabilization frame in this proof surface

### Phase 2

- add one or two focused proof pages:
  - editor controls under the preset
  - workspace shell under the preset
- add screenshots / diag gates for high-signal surfaces

### Phase 3

- decide whether this stays a preset module or becomes a dedicated adapter crate

## Exit criteria

This preset is ready to move from draft to active implementation when:

1. at least one editor proof surface can switch between the default editor preset and the imgui-like preset,
2. the switch affects visuals and density without requiring different widget code,
3. both declarative and `imui` authoring paths render the same behaviors,
4. docking and workspace tab chrome remain visually aligned without collapsing crate ownership.
