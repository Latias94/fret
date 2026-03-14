# Editor component system and visual baseline v1

Tracking doc:
`docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`

Related docs:

- `docs/workstreams/ui-editor-v1.md`
- `docs/workstreams/ui-editor-egui-imgui-gap-v1.md`
- `docs/adr/0316-editor-ecosystem-token-namespaces-and-skinning-boundary-v1.md`

Status: Active design note (workstream contract, not an ADR)

Last updated: 2026-03-14

## Purpose

This note answers two practical questions for the editor ecosystem:

1. How should reusable editor components be designed?
2. How should their default look and feel be defined without binding the ecosystem to one visual
   brand?

The answer is not "copy Dear ImGui" and not "wrap shadcn".
The answer is: define a Fret-native editor component system that borrows the right outcomes from
ImGui, egui, Radix/Base UI, and design-system adapters.

## Design stance

### 1) Contract before cosmetics

For editor-grade UI, the hard part is not the component count.
The hard part is keeping these aligned across the whole stack:

- identity and state ownership,
- interaction-state visuals,
- edit-session semantics,
- keyboard/focus behavior,
- density and responsive minimums.

If those contracts are weak, the library will feel inconsistent even with many widgets.

### 2) One widget implementation, many authoring syntaxes

- Reusable editor widgets live in `ecosystem/fret-ui-editor`.
- `imui` is an authoring frontend, not a second implementation tree.
- The same widget should render the same semantics from declarative and `imui` entrypoints.

### 3) Default baseline: neutral engineering, not branded softness

The default Fret editor look should be:

- compact,
- readable,
- border-defined,
- low-ornament,
- visually stable under frequent editing.

That means:

- less decorative softness than shadcn defaults,
- less pixel-brand imitation than a strict ImGui clone,
- and more emphasis on predictable field chrome, focus, and dense information layout.

### 4) Density in visuals, safety in hit targets

Editor apps need compact rows and toolbars, but compact visuals must not imply tiny hit regions.

Keep these separate:

- visual density: row height, padding, track thickness, icon size,
- usable targets: pointer hit thickness, close buttons, drag handles, tabs, menu rows.

### 5) Presets are skins, not forks

We should support:

- a neutral default editor baseline,
- `imgui_like_dense` as a proof-oriented preset,
- shadcn/material/custom adapters as one-way skins.

We should not support:

- `ImGuiButton`, `ImGuiPropertyGrid`, or other branded widget forks,
- a separate `imui` token vocabulary,
- reverse dependencies from editor/workspace crates into skin crates.

## Component stack and ownership

| Layer | Owner | Examples | Rule |
| --- | --- | --- | --- |
| Authoring frontend | `fret-imui`, `fret-ui-kit::imui` | `ui.drag_value(...)`, response helpers, keyed helpers | Syntax and facade ergonomics only. |
| Primitives | `fret-ui-editor::primitives` | widget visuals, edit session helpers, chrome, input groups | Shared editor behaviors and chrome classes. |
| Controls | `fret-ui-editor::controls` | `TextField`, `DragValue`, `Slider`, `Checkbox`, `EnumSelect`, `ColorEdit`, `VecNEdit` | One declarative implementation per reusable control. |
| Composites | `fret-ui-editor::composites` | `PropertyRow`, `PropertyGrid`, `InspectorPanel`, small editor toolbars | Compose primitives/controls into reusable editor surfaces. |
| Shell chrome | `fret-workspace` | frame, top bar, status bar, shell tabstrip, pane chrome | Shell-level layout and chrome, not dock-graph policy. |
| Dock-aware chrome | `fret-docking` | drop overlays, dock tabs, insert markers, split previews | Dock-graph-aware interactions stay docking-owned. |
| Specialized editor surfaces | dedicated ecosystem crates | code editor, viewport tooling, gizmos, node graphs | Do not force all editor surfaces into one crate. |
| Protocols and services | app layer today; future dedicated crates | inspector/property protocol, edit services, project/asset services | Extract only after ownership and reuse are real. |

## Visual grammar

### Default editor baseline

The default editor baseline should optimize for inspection, tuning, and repeated edits:

- clear control frames,
- small radii,
- moderate contrast,
- tight spacing,
- explicit focus and invalid states,
- minimal decorative motion.

This is the recommended default target for `fret-ui-editor`.
Presets may move away from it, but reusable controls should first look correct in this baseline.

### What to borrow from upstreams

#### Dear ImGui / egui

Borrow:

- dense but readable spacing,
- unified state visuals,
- explicit identity,
- strong edit-session semantics,
- "editor starter set" completeness.

Do not borrow:

- API shape as a goal,
- label-hash identity tricks as the only identity story,
- frame-perfect theme cloning.

#### shadcn / Base UI / Radix

Borrow:

- composition discipline,
- headless/policy separation,
- overlay/menu semantics,
- recipe-level skinning.

Do not borrow:

- app-style softness as the default editor feel,
- recipe decisions as framework/runtime contracts.

### Surface classes

We should style editor surfaces by class, not one-off widget whim.

#### 1) Field-like controls

Examples:

- `TextField`
- `NumericInput`
- `DragValue` typing mode
- `EnumSelect` trigger

Default direction:

- stable frame,
- compact padding,
- clear border ownership,
- focus ring separate from hover/active fill changes.

#### 2) Discrete selectors and toggles

Examples:

- `Checkbox`
- segmented value toggles
- enum/list options

Default direction:

- selection state should be stronger than hover,
- disabled and mixed states must remain legible,
- icons/checkmarks should be semantic and consistent in size.

#### 3) Continuous value controls

Examples:

- `DragValue`
- `Slider`
- color/vec/transform axis controls

Default direction:

- interaction feedback should read immediately,
- scrubbing affordances need larger hit areas than they look,
- typed-edit mode must still visually belong to the same control family,
- and affixed numeric variants should keep prefix/suffix chrome outside the editable draft so
  typed replacement stays fast in dense inspector workflows.

Recommended authoring rule:

- when a first-party editor surface needs a reusable numeric text story plus optional control
  chrome affixes, prefer `NumericPresentation<T>` over open-coded `(format, parse, prefix, suffix)`
  bundles,
- keep unit-bearing text that is part of the editable representation inside the formatter/parser
  pair (for example `90°`),
- and reserve control chrome affixes for joined non-editable segments such as currency or timing
  labels.

#### 4) Composite/editor sections

Examples:

- `PropertyRow`
- `PropertyGrid`
- `PropertyGroup`
- `InspectorPanel`

Default direction:

- prioritize alignment and scanability over decoration,
- group hierarchy should be readable with spacing and headers, not heavy card nesting,
- outer panel framing and inner group framing should use distinct token slots when they need
  different visual weight; do not force both levels through one shared border tone,
- likewise, the top inspector header band should not be forced to share the exact same header tone
  as repeated property-group headers when those two levels serve different hierarchy roles,
- reset/help/status affordances must align to a consistent slot model.

#### 5) Popup/list surfaces

Examples:

- `TextAssistField` inline / anchored panels
- `InspectorPanel` search-history assist
- `EnumSelect` popup/list surface
- `ColorEdit` popover surface

Default direction:

- popup shells should resolve their surface tone from editor-owned popup tokens before falling back
  to host semantic palettes,
- overlay and inline variants may differ in elevation/shadow, but not in their basic background /
  border grammar,
- list/assist/select/color popovers should read as one editor family even when their row
  interaction policies differ,
- and screenshot proof surfaces should make popup geometry and chrome reviewable without a manual
  launch.

#### 6) Shell chrome

Examples:

- top bar,
- status bar,
- shell tabstrip,
- pane headers.

Default direction:

- keep shell chrome visually adjacent to the editor baseline,
- but do not force shell recipes into `fret-ui-editor`,
- align shell and docking via preset seeding, not shared ownership.

## State model

Every reusable editor control should align to a common state vocabulary.

### Core interactive states

These should be resolved centrally through shared visuals rather than ad-hoc paint logic:

- `inactive`
- `hovered`
- `focused`
- `active` / `pressed`
- `open`
- `disabled`

`EditorWidgetVisuals` should remain the main owner of this class-level state model.
Field-like controls should not hand-paint their own "editing" and "invalid" frames; they should
route those semantics through the shared visuals policy so text fields, numeric inputs, slider
typing paths, and select triggers continue to read as one control family.

### Semantic overlays

These are not separate component families.
They are semantic overlays applied on top of the core widget state:

- `invalid`
- `mixed`
- `selected`
- `dirty`
- `drag_preview`

Rule:

- keep core widget-state visuals centralized,
- layer semantic meanings on top with small, explicit cues,
- do not fork a new style system for every semantic variation.

Current baseline:

- `typing` and `invalid` are shared field-level semantic overlays owned by `EditorWidgetVisuals`,
- editor text-like controls should prefer shared `editor.control.invalid.*` tokens before
  introducing widget-local error chrome,
- and per-control overrides should be a last resort rather than the primary styling path.

### Focus treatment

Focus must be visible and not collapse into hover.

Requirements:

- keyboard focus is distinct from pointer hover,
- focus treatment must survive clipping/overflow decisions,
- focus ring or focus border should remain visible on compact controls.

## Density and layout rules

### Baseline metric ranges (non-normative)

Use these as a smell test, not a frozen pixel spec:

- standard editor row height: roughly 28-32 px equivalent
- compact tool row height: roughly 24-28 px equivalent
- horizontal field padding: roughly 6-10 px equivalent
- vertical field padding: roughly 4-6 px equivalent
- icon size: roughly 14-16 px equivalent
- slider/thumb visuals: compact, but with larger hit thickness than visible chrome

### Responsive minimums

The component system should prefer stacking or layout mode changes over micro-shrinking.

Rules:

- property surfaces should stack below a width threshold rather than crush label/value readability,
- vec/transform editors should preserve per-axis minimum widths,
- icon-only affordances should not become the only visible label in narrow layouts,
- popup/list max heights should preserve scrolling and keyboard usability.

### Inspector lane grammar

Reusable inspector/property surfaces should default to one shared lane model:

- a scan-aligned label lane,
- a value lane that can fill but stays capped to a readable maximum,
- an optional reset slot that preserves ordering but collapses when absent,
- and an optional status/actions slot that preserves ordering but collapses when absent.

Implications:

- `PropertyRow` owns the lane grammar,
- `PropertyGrid`, `PropertyGroup`, and `InspectorPanel` should forward shared metrics rather than
  restating their own hardcoded spacing,
- reset/clear/remove/icon affordances should default to a shared row-height-square target even when
  the visible chrome remains denser,
- rows that genuinely do not want trailing affordances should collapse those slots by default rather
  than reserving dead width,
- proof-only outcome instrumentation should follow the same rule whether it lives in a trailing
  lane or a whole readout row: idle/empty state should not reserve right-lane width or idle row
  height, and only material commit/cancel/error states should claim that space,
- compact non-edit readout text styling may be shared as an editor primitive when real reuse
  appears, but proof-local readout layout/container geometry should stay local until a second
  genuine layout family exists,
- and wide inspectors should first let the shared value lane grow toward a readable cap before
  accepting large pools of empty panel slack.

### Visual density vs usability

Use `editor.density.*` to tune the visual system, but do not encode unsafe defaults:

- dense rows are good,
- tiny close buttons are not,
- compact tabs are good,
- fragile hit targets are not.

## Interaction rules

| Concern | Rule | Owner |
| --- | --- | --- |
| Identity | Prefer explicit `id_source`; otherwise derive from stable model identity. Never use `test_id` as widget identity. | `fret-ui-editor`, `fret-imui` |
| Response semantics | Expose predictable response objects across declarative and `imui` entrypoints. | `fret-ui-editor`, facade layers |
| Numeric edit session | Start, live update, commit, and cancel semantics must be consistent across scrub and typed-edit flows. | `fret-ui-editor` |
| Keyboard defaults | Tab order, Enter/Escape, arrow adjustments, and popup navigation should be explicit editor defaults. | `fret-ui-editor` with `fret-ui-kit` |
| Undo boundaries | Scrub sessions should coalesce into one logical edit when undo integration is enabled. | `fret-ui-editor` + optional integration |
| Menus and popups | Use shared overlay/menu substrates plus editor-owned popup-surface tokens/chrome; keep editor-specific recipes in policy crates. | `fret-ui-kit`, `fret-ui-editor` |

## Token and preset strategy

### Stable ownership

- `editor.*` belongs to `fret-ui-editor`
- `workspace.*` belongs to `fret-workspace`
- dock-aware drop/insert/split visuals remain docking-owned

`imui` and declarative surfaces must share the same vocabulary.

### Recommended expansion path

Prefer extending shared editor families before inventing new per-component token trees.

Good first choices:

- `editor.control.*`
- `editor.density.*`
- `editor.numeric.*`
- `editor.property.*`
- `editor.popup.*`
- `editor.slider.*`
- `editor.checkbox.*`
- `editor.color.*`
- `editor.vec.*`
- `editor.axis.*`

If new style pressure appears, prefer cross-control families before widget-brand families.

### Preset layers

#### Recommended default

Use a neutral engineering baseline as the default reusable editor target.
This is the baseline new controls should first satisfy.

#### Imgui-like preset

Keep `imgui_like_dense` as a density/feel preset that proves:

- compact spacing,
- stronger frame boundaries,
- lower radius,
- fast/direct scrub feel,
- shell and docking alignment through seeding.

#### Adapter skins

shadcn/material/custom-app skins should:

- seed or alias editor/workspace namespaces,
- remain one-way dependencies,
- not force design-system decisions back into editor crates.

## Definition of done for a new reusable editor component

Before a component is considered a stable part of the editor ecosystem, it should have:

1. one declarative implementation in the correct crate,
2. a clear place in the component taxonomy,
3. shared widget-state visuals instead of one-off state paint logic,
4. responsive minimums and density behavior,
5. explicit keyboard/edit-session rules,
6. stable `id_source` / `test_id` conventions,
7. a proof surface plus at least one regression gate.

If a component cannot satisfy these, it should stay app-local or experimental longer.
