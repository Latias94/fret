# IMUI vs Dear ImGui Gap Audit — 2026-04-22

Status: Snapshot audit (current-code evidence). This note is intended to complement, and in a few
places correct, older pre-reset parity notes.

Local Dear ImGui snapshot used for reference: `repo-ref/imgui` @ `d7b40ab9a`

## Scope

- Compare the current Fret immediate-mode lane against the local Dear ImGui snapshot.
- Focus on current code, current first-party teaching surfaces, and current executable proofs.
- Exclude:
  - the compatibility-only retained bridge lane (`imui_node_graph_demo`),
  - user-owned in-progress menu/tab work,
  - platform-specific multi-viewport validation that cannot currently be exercised on this machine.

## Inputs Reviewed

- `ecosystem/fret-imui/src/lib.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-imui/src/tests/{floating.rs,interaction.rs,popup_hover.rs,models.rs,models_text_basic.rs,models_text_lifecycle.rs,models_text_identity.rs,models_text_picker.rs,models_text_filters.rs,models_text_modes.rs,models_text_commands.rs,models_text_area.rs}`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-examples/src/{imui_hello_demo.rs,imui_response_signals_demo.rs,imui_shadcn_adapter_demo.rs,imui_floating_windows_demo.rs,imui_editor_proof_demo.rs,workspace_shell_demo.rs,imui_node_graph_demo.rs}`
- `docs/examples/README.md`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`
- `docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`

## Findings

### 1. The current layer split is correct and should not be "fearlessly refactored" away

`fret-imui` is already the thin frontend that Fret needs: it exports the immediate-mode mounting
primitives (`imui`, `imui_in`, `imui_build`, `imui_raw`, etc.) and intentionally keeps policy out
of the crate.

The imgui-like surface is intentionally hosted in `fret-ui-kit::imui`, while the app-facing root
lane is `fret::imui::{prelude::*, kit, editor, docking}`.

Conclusion:

- Do not fatten `fret-imui` into a widget/policy crate.
- Do not move interaction policy from `fret-ui-kit::imui` into `crates/*` or `fret-imui`.

Evidence anchors:

- `ecosystem/fret-imui/src/lib.rs`
- `ecosystem/fret/src/lib.rs`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`

### 2. Current Fret IMUI parity is materially broader than some older parity notes still imply

The current code already covers a meaningful Dear ImGui subset:

- floating window behavior knobs, including explicit analogs for `NoBringToFrontOnFocus` and
  `NoInputs`,
- hovered query flags and nav-aware `hovered_like_imgui()`,
- `disabled_scope(...)`,
- the ImGui-aligned drag threshold default (`6px`),
- immediate wrappers for menus, tab bars, tables, virtual lists, combos, text input, and tooltips,
- typed drag-source / drop-target payload seams,
- keyed identity helpers (`ui.id(...)`, `ui.push_id(...)`, `for_each_keyed(...)`),
- multi-select helpers and editor-grade proof surfaces.

This means the current gap is no longer "basic immediate widgets are missing". Older notes that
still say "no immediate tables/tab bars/drag-and-drop API yet" are stale relative to the current
tree.

Evidence anchors:

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs`
- `ecosystem/fret-imui/src/tests/floating.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`

### 3. ID ergonomics are not the same as ID capability

Dear ImGui exposes `PushID()` / `GetID()` plus label suffix conventions such as `"##"` and
`"###"`.

Fret does not mirror the label-suffix parsing model, but it does already provide the core identity
capability through explicit keyed scopes:

- `ui.id(...)`
- `ui.push_id(...)`
- `ui.for_each_keyed(...)`
- `ui.for_each_unkeyed(...)` as an explicit opt-in for static-order collections

So the real gap is ergonomic sugar for ports from raw ImGui code, not missing stable identity
mechanics.

Conclusion:

- Do not invent a second hashing / ID runtime.
- If friction remains high for ImGui ports, add narrow sugar on top of the existing keyed story
  instead of copying the raw label-suffix model into the whole lane.

Evidence anchors:

- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `docs/examples/README.md`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`

### 4. The biggest remaining parity gaps are now deeper than buttons, sliders, or menus

#### 4.1 Input-text parity is still shallow

Update (2026-05-04): partially superseded by
`docs/workstreams/imui-text-input-policy-depth-v1/`, the command-oriented
`docs/workstreams/imui-text-input-history-completion-policy-v1/`, and the public cookbook proof in
`docs/workstreams/imui-editor-cookbook-proof-v1/`, plus the named filter policy slice in
`docs/workstreams/imui-text-input-filter-policy-v1/` and the custom insertion-filter slice in
`docs/workstreams/imui-text-input-custom-filter-policy-v1/`, and the undo command policy slice in
`docs/workstreams/imui-text-input-undo-command-policy-v1/`, plus the visible picker recipe in
`docs/workstreams/imui-text-input-picker-recipe-v1/`.

Current `InputTextOptions` now includes:

- `read_only`,
- `select_all_on_focus`,
- `mode: InputTextMode` with `PlainText` / `Password`,
- command-oriented completion/history routing for unmodified Tab/Up/Down,
- `input_text_completion_model(_with_options)` and `input_text_history_model(_with_options)` picker
  recipes for visible app-owned candidates,
- `filters: InputTextFilters` for Dear ImGui-style decimal, hexadecimal, scientific, uppercase,
  and no-blank named filters,
- `custom_filter: Option<InputTextCustomFilter>` as a Fret-native insertion-filter equivalent of
  `CallbackCharFilter`,
- app-owned undo/redo command routing for Ctrl+Z, Ctrl+Y, and Ctrl+Shift+Z,
- `TextAreaOptions::allow_tab_input`,
- accessibility labels/roles,
- placeholder,
- submit/cancel commands.

Dear ImGui exposes a wide `ImGuiInputTextFlags_*` family (`ReadOnly`, `Password`,
`AutoSelectAll`, `NoUndoRedo`, completion/history callbacks, `AllowTabInput`, multiline-specific
flags, etc.).

Conclusion:

- The remaining serious "imgui-level editor UX" gap is deeper text editing policy, not generic
  button chrome.
- Completion/history now has a Fret-native command routing slice; named character filters and
  custom insertion filters are covered by `InputTextFilters` / `InputTextCustomFilter`; undo/redo
  shortcut routing is covered by app-owned commands where the unset default is the Fret-native
  `NoUndoRedo` behavior. A visible completion/history picker recipe now covers the first reusable UI
  layer, the picker keyboard-navigation follow-on covers ArrowUp/ArrowDown active movement plus
  Enter/NumpadEnter commit, and the picker accessibility follow-on covers generic combobox-style
  expanded / controls / active-descendant semantics. The still-open pieces are editor-owned ranking
  / storage, richer platform accessibility announcement checks, popup role refinement, and deeper
  multiline-specific behavior beyond the landed Tab-input opt-in.
- These should stay in `fret-ui-kit::imui` and `fret-ui-editor`, not by bloating `fret-imui`.

Evidence anchors:

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`
- `crates/fret-ui/src/text/input/widget.rs`
- `ecosystem/fret-imui/src/tests/models_text_basic.rs`
- `ecosystem/fret-imui/src/tests/models_text_lifecycle.rs`
- `ecosystem/fret-imui/src/tests/models_text_identity.rs`
- `ecosystem/fret-imui/src/tests/models_text_picker.rs`
- `ecosystem/fret-imui/src/tests/models_text_filters.rs`
- `ecosystem/fret-imui/src/tests/models_text_modes.rs`
- `ecosystem/fret-imui/src/tests/models_text_commands.rs`
- `ecosystem/fret-imui/src/tests/models_text_area.rs`
- `repo-ref/imgui/imgui.h`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

#### 4.1a Color-edit parity is past the stub stage, but full picker depth remains separate

Update (2026-05-04): partially superseded by
`docs/workstreams/imui-color-edit-popup-depth-v1/`,
`docs/workstreams/imui-color-edit-alpha-policy-v1/`,
`docs/workstreams/imui-color-edit-alpha-preview-v1/`,
`docs/workstreams/imui-color-edit-alpha-preview-options-v1/`,
`docs/workstreams/imui-color-edit-drag-drop-payload-v1/`,
`docs/workstreams/imui-color-edit-alpha-bar-v1/`,
`docs/workstreams/imui-color-edit-hsv-picker-v1/`,
`docs/workstreams/imui-color-edit-numeric-readout-v1/`, and
`docs/workstreams/imui-color-edit-numeric-input-v1/`. Update (2026-05-05): popup option/default
depth is now partially superseded by `docs/workstreams/imui-color-edit-popup-options-v1/`; the
post-depth file-size hazard is partially superseded by
`docs/workstreams/imui-color-edit-model-split-v1/` and
`docs/workstreams/imui-color-edit-popup-split-v1/`; editable numeric row ownership is partially
superseded by `docs/workstreams/imui-color-edit-popup-numeric-split-v1/`; HSV/SV/Hue and
AlphaBar picker ownership is partially superseded by
`docs/workstreams/imui-color-edit-popup-picker-split-v1/`; shared preview helper ownership is
partially superseded by `docs/workstreams/imui-color-edit-popup-preview-split-v1/`; preset swatch
ownership is partially superseded by
`docs/workstreams/imui-color-edit-popup-swatches-split-v1/`; color payload source/target behavior
is partially superseded by `docs/workstreams/imui-color-edit-drag-drop-payload-v1/`; current and
original popup reference previews are partially superseded by
`docs/workstreams/imui-color-edit-reference-preview-v1/`; vertical PickerHueBar shape is partially
superseded by `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/`; vertical AlphaBar shape is
partially superseded by `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/`; HueWheel picker
shape is partially superseded by `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/`; picker
options popup behavior is partially superseded by
`docs/workstreams/imui-color-edit-picker-options-popup-v1/`; app-owned palette source
customization is partially superseded by
`docs/workstreams/imui-color-edit-palette-customization-v1/`; editable palette slot drag/drop is
partially superseded by `docs/workstreams/imui-color-edit-editable-palette-slots-v1/`; app-owned
recent color history swatches are partially superseded by
`docs/workstreams/imui-color-edit-history-swatches-v1/`; hover tooltip previews are partially
superseded by `docs/workstreams/imui-color-edit-tooltip-preview-v1/`; copy-as context menu payloads
are partially superseded by `docs/workstreams/imui-color-edit-copy-as-context-menu-v1/`; picker
options thumbnails are partially superseded by
`docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/`.

Current editor `ColorEdit` now has:

- hex input,
- a preset swatch popup instead of a visible placeholder,
- RGB-only hex and preset behavior that preserves the current alpha channel,
- per-control alpha preview modes for the main and preset swatches: checkerboard, opaque,
  no-background, and half-alpha preview,
- typed RGB/RGBA color drag/drop payloads on the root swatch, preserving target alpha for RGB
  payloads and RGB-only targets,
- current/original popup reference previews with original restore behavior matching Dear ImGui's
  RGB/RGBA component-count rules,
- Dear ImGui-shaped side-preview layout: current/original previews sit beside the picker as a
  vertical column with 3:2 preview swatches,
- a bounded AlphaBar-style popup control when `show_alpha=true`,
- bounded HSV picker controls in the popup: RGB/HSV conversion, saturation/value picking, and a
  Dear ImGui-shaped vertical HueBar,
- Dear ImGui-shaped vertical AlphaBar in the `HsvHueBar` picker when alpha editing is visible,
- an opt-in Dear ImGui-shaped `HsvHueWheel` picker with hue ring angle mapping, rotated SV triangle
  mapping, Canvas rendering, and optional vertical AlphaBar composition,
- a popup-local picker options surface for switching between `HsvHueBar` and `HsvHueWheel` and
  toggling AlphaBar visibility without global `SetColorEditOptions()` state,
- Dear ImGui-style picker type thumbnails inside the popup-local picker options surface,
- an app-owned eyedropper request hook and popup command through
  `ColorEditOptions::on_eyedropper`, without pretending runtime/platform screen sampling exists,
- app-owned palette entries through `ColorEditOptions::palette`, preserving the built-in palette
  and alpha-preserving palette activation,
- editable popup palette slots through `OnColorEditPaletteSlotDrop`, with palette swatches acting
  as RGB drag sources and optional app-owned drop targets,
- app-owned recent color history swatches through `ColorEditOptions::history`,
- Dear ImGui-style hover tooltip previews on root swatches through `ColorEditOptions::tooltip`,
  including hex, RGB, and HSV text,
- Dear ImGui-style copy-as context menus on root swatches through `ColorEditOptions::copy`, with
  float tuple, integer tuple, RGB hex, and visible-alpha RGBA hex clipboard payloads,
- RGB and HSV numeric readouts, with alpha percent shown when alpha is visible,
- editable RGB/HSV numeric popup rows with editor-owned validation,
- per-control popup defaults for HueBar picker, RGB/HSV numeric rows, preset palette, and AlphaBar
  visibility,
- an internal model module for color parsing, formatting, HSV/RGB conversion, coordinate math,
  sanitization, and a11y helper text,
- an internal popup module for overlay composition and content ordering,
- an internal popup numeric module for editable RGB/HSV row composition and commit handling,
- an internal popup picker module for HSV/SV/Hue and AlphaBar composition, gradient/thumb preview
  helpers, and picker-local pointer handlers,
- an internal popup preview module for checkerboard, fill-layout, and color preview stack helpers,
- an internal popup swatches module for preset row composition and alpha-preserving preset
  activation,
- and an app-facing cookbook proof through `fret::imui::editor`.

#### 4.1b Debug draw is no longer missing at the first baseline level

Update (2026-05-04): partially superseded by the canvas-backed
`docs/workstreams/imui-debug-draw-baseline-v1/`. Update (2026-05-05): richer debug-draw shape
depth is partially superseded by `docs/workstreams/imui-debug-draw-shape-primitives-v1/`,
`docs/workstreams/imui-debug-draw-stroke-style-v1/`,
`docs/workstreams/imui-debug-draw-clip-stack-v1/`,
`docs/workstreams/imui-debug-draw-image-overlay-v1/`,
`docs/workstreams/imui-debug-draw-bezier-primitives-v1/`,
`docs/workstreams/imui-debug-draw-convex-poly-fill-v1/`,
`docs/workstreams/imui-debug-draw-quad-primitives-v1/`, and
`docs/workstreams/imui-debug-draw-ngon-primitives-v1/`.

Current IMUI now exposes:

- a thin immediate-mode `debug_draw` facade in `fret-ui-kit::imui`,
- line, rect, filled rect, and text primitives,
- polyline, stroked/filled quad, stroked/filled triangle, stroked/filled circle, stroked/filled
  regular polygon, quadratic Bezier, and cubic Bezier primitives,
- an `AddConvexPolyFilled`-style filled convex polygon command,
- explicit stroke width/cap/join/miter/dash policy,
- clip rect stack commands with paint-end auto-balancing,
- registered image, image-region, SVG image, and SVG mask icon overlay commands,
- declarative lowering into `Canvas`,
- and smoke tests that keep the facade boundary clean.

Dear ImGui still goes much deeper through full `DrawList` parity, richer stroke styles, dashed
paths, path-builder ergonomics, channel splitting, per-command metadata, and hit-test-aware debug
interaction.

Conclusion:

- The remaining color gap is deeper picker/editor affordances, not "visible popup is a stub".
- Keep the current alpha-preserving RGB policy in `fret-ui-editor`.
- Start separate narrow follow-ons for platform-owned screen sampling or screenshot-backed full
  picker visual polish.
- The remaining debug-draw gap is richer DrawList parity, not "no debug-draw surface exists".

Evidence anchors:

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/drag_drop.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/model.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/options.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/eyedropper.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/numeric.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/swatches.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
- `repo-ref/imgui/imgui_demo.cpp`
- `repo-ref/imgui/imgui_widgets.cpp`

#### 4.2 There is still no immediate style-stack lane, and that is mostly the right decision

Older parity notes already framed style-stack APIs (`PushStyleVar`, `PushStyleColor`,
`SetNextItemWidth`, historical `same_line`) as intentionally not mirrored by the current lane.

That remains the correct default posture: Fret expresses visual policy through tokens, explicit
layout, and recipe/component layers rather than a parallel immediate styling runtime.

Conclusion:

- Do not reopen a generic style-stack API on `fret-imui`.
- If a repeated editor/tooling use case emerges, solve it with narrow policy helpers on
  `fret::imui::kit` or `fret-ui-editor`, not a global push/pop styling world.

Evidence anchors:

- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/DESIGN.md`

#### 4.3 Immediate draw-list parity is still absent from the IMUI lane

The repo has explicit draw-list concepts, but they live in specialized domains such as gizmos and
renderer overlays, not in a generic IMUI `DrawList` surface.

Conclusion:

- If Fret needs imgui-like debug overlays / custom immediate drawing, add a dedicated ecosystem
  adapter or debug-draw lane.
- Do not overload `fret-imui` with a generic drawing API just because Dear ImGui has one.

Evidence anchors:

- `ecosystem/fret-gizmo/src/gizmo/types.rs`
- `apps/fret-examples/src/gizmo3d_demo.rs`
- `docs/audits/gizmo-imguizmo-transform-gizmo-alignment.md`

#### 4.4 Multi-viewport / OS-window parity remains the hardest unresolved area

Fret already has meaningful in-window floating and docking proofs:

- `imui_floating_windows_demo`
- `imui_editor_proof_demo`
- `workspace_shell_demo`

But Dear ImGui's multi-viewport behavior goes further into OS-window lifecycle, viewport flags, and
backend cooperation. The current repo still treats that as a dedicated docking / multi-window lane,
not a solved general IMUI claim.

Conclusion:

- Keep this work docking-owned.
- Do not try to "finish imgui parity" by adding windowing policy into `fret-imui`.

Evidence anchors:

- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`
- `repo-ref/imgui/imgui.cpp`

### 5. Test architecture is now a bigger refactor hazard than missing public API

`fret-imui` itself is tiny, but its verification surface is concentrated in very large test files:

- `src/tests/interaction.rs`
- `src/tests/models.rs`
- `src/tests/floating.rs`
- `src/tests/popup_hover.rs`

This is now one of the main reasons IMUI refactors stay risky: behavior coverage exists, but it is
expensive to navigate and review.

The same shape appeared in editor `ColorEdit` after the popup, alpha, HSV, numeric, and option
slices: the public control surface became useful, but the implementation file mixed pure color
model helpers with UI composition.

Update (2026-05-04): the first mechanical splits have landed in
`docs/workstreams/imui-models-text-picker-test-split-v1/` and
`docs/workstreams/imui-models-text-filter-test-split-v1/`, and
`docs/workstreams/imui-models-text-mode-test-split-v1/`, and
`docs/workstreams/imui-models-text-command-test-split-v1/`, and
`docs/workstreams/imui-models-text-area-test-split-v1/`, and
`docs/workstreams/imui-models-text-final-test-split-v1/`. Completion/history picker tests now live
in `src/tests/models_text_picker.rs`; named/custom filter tests now live in
`src/tests/models_text_filters.rs`; single-line read-only, select-all-on-focus, and password-mode
tests now live in `src/tests/models_text_modes.rs`; completion/history/undo command-policy tests
now live in `src/tests/models_text_commands.rs`; multiline textarea tests now live in
`src/tests/models_text_area.rs`; basic changed-signal, single-line lifecycle/bounds, and push-id
identity tests now live in `src/tests/models_text_basic.rs`, `src/tests/models_text_lifecycle.rs`,
and `src/tests/models_text_identity.rs`. The legacy `src/tests/models_text.rs` aggregate is
retired.

Update (2026-05-05): `docs/workstreams/imui-color-edit-model-split-v1/` moves the editor
`ColorEdit` pure model helpers into `src/controls/color_edit/model.rs` and moves focused tests into
`src/controls/color_edit/tests.rs`. `docs/workstreams/imui-color-edit-popup-split-v1/` then moves
popup UI composition and popup-local pointer helpers into `src/controls/color_edit/popup.rs`.
`docs/workstreams/imui-color-edit-popup-numeric-split-v1/` moves editable numeric row composition
and commit handling into `src/controls/color_edit/popup/numeric.rs`.
`docs/workstreams/imui-color-edit-popup-picker-split-v1/` moves HSV/SV/Hue and AlphaBar picker
composition, gradient/thumb helpers, and picker-local pointer handlers into
`src/controls/color_edit/popup/picker.rs`.
`docs/workstreams/imui-color-edit-popup-preview-split-v1/` moves shared checkerboard, fill-layout,
and color preview stack helpers into `src/controls/color_edit/popup/preview.rs`.
`docs/workstreams/imui-color-edit-popup-swatches-split-v1/` moves preset row composition and
alpha-preserving preset activation into `src/controls/color_edit/popup/swatches.rs`.
`docs/workstreams/imui-color-edit-alpha-preview-options-v1/` adds per-control alpha preview modes
matching Dear ImGui's ColorButton preview policy axis without global color edit option state.

Conclusion:

- Continue prioritizing test decomposition over more top-level helper growth.
- Continue splitting large editor controls along model/composition boundaries before adding more
  popup features.
- Use fixture-driven splits only where the test cases are data-shaped. The current picker tests stay
  as Rust interaction tests because they exercise multi-frame focus, popup, keyboard, and semantics
  behavior.

Evidence anchors:

- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/model.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/numeric.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-imui/src/tests/models_text_basic.rs`
- `ecosystem/fret-imui/src/tests/models_text_lifecycle.rs`
- `ecosystem/fret-imui/src/tests/models_text_identity.rs`
- `ecosystem/fret-imui/src/tests/models_text_picker.rs`
- `ecosystem/fret-imui/src/tests/models_text_filters.rs`
- `ecosystem/fret-imui/src/tests/models_text_modes.rs`
- `ecosystem/fret-imui/src/tests/models_text_commands.rs`
- `ecosystem/fret-imui/src/tests/models_text_area.rs`
- `ecosystem/fret-imui/src/tests/floating.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `tools/audit_crate.py --crate fret-imui`

### 6. First-party teaching surfaces are finally mostly aligned; compatibility exceptions should stay explicit

The first-party immediate-mode story now teaches the root `fret::imui` lane across cookbook/examples,
while `imui_node_graph_demo` remains the explicit compatibility-only retained-bridge exception.

This is a good place to stop API churn and use the examples as regression anchors instead of
continuing to move teaching surfaces around.

Evidence anchors:

- `docs/examples/README.md`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`

## What Not To Refactor Next

- Do not turn `fret-imui` into a fat widget crate.
- Do not move menu/tab/hover/floating policy down into `crates/*`.
- Do not reopen retained-bridge authoring as a normal first-party IMUI lane.
- Do not copy Dear ImGui's style-stack model into Fret just for API familiarity.
- Do not add a second identity/hash story when `ui.id(...)` / `ui.push_id(...)` already exists.

## Recommended Next Steps

1. Continue splitting the `fret-imui` mega-tests by capability family.
   - Status: completion/history picker tests have moved to `models_text_picker.rs` under
     `docs/workstreams/imui-models-text-picker-test-split-v1/`; named/custom filter tests have moved
     to `models_text_filters.rs` under `docs/workstreams/imui-models-text-filter-test-split-v1/`;
     single-line read-only/select-all/password tests have moved to `models_text_modes.rs` under
     `docs/workstreams/imui-models-text-mode-test-split-v1/`; completion/history/undo
     command-policy tests have moved to `models_text_commands.rs` under
     `docs/workstreams/imui-models-text-command-test-split-v1/`; multiline textarea tests have
     moved to `models_text_area.rs` under
     `docs/workstreams/imui-models-text-area-test-split-v1/`; the remaining basic/lifecycle/
     identity tests have moved to `models_text_basic.rs`, `models_text_lifecycle.rs`, and
     `models_text_identity.rs` under `docs/workstreams/imui-models-text-final-test-split-v1/`.
   - Outcome: safer refactors for hover/floating/menu/tab/text/drag lanes.
   - Likely tool: fixture-driven harnesses for repetitive response/state matrices; keep procedural
     interaction tests in Rust.

2. Continue text-input parity in narrower `fret-ui-kit::imui` / `fret-ui-editor` lanes.
   - Status: read-only, password mode, auto-select-all, and multiline `AllowTabInput` are covered by
     `docs/workstreams/imui-text-input-policy-depth-v1/`; command-oriented completion/history key
     routing is covered by `docs/workstreams/imui-text-input-history-completion-policy-v1/`; named
     character filters are covered by `docs/workstreams/imui-text-input-filter-policy-v1/`; custom
     insertion filters are covered by `docs/workstreams/imui-text-input-custom-filter-policy-v1/`;
     undo/redo command policy is covered by
     `docs/workstreams/imui-text-input-undo-command-policy-v1/`; visible completion/history picker
     UI is covered by `docs/workstreams/imui-text-input-picker-recipe-v1/`; picker keyboard
     navigation is covered by `docs/workstreams/imui-text-input-picker-keyboard-nav-v1/`; and
     generic picker active-descendant semantics are covered by
     `docs/workstreams/imui-text-input-picker-a11y-v1/`.
   - Remaining focus: editor-owned completion/history ranking/storage, richer platform
     accessibility announcement checks, popup role refinement, and deeper multiline behavior.

3. Treat menu/tab depth as an explicit `fret-ui-kit::imui` policy lane.
   - Outcome: finish the difficult part of IMUI parity in the correct layer.
   - Constraint: do not route this through `fret-imui` or `crates/fret-ui`.

4. Refresh older parity notes after this audit, not before.
   - Outcome: prevent stale documents from driving the wrong refactor.
   - Specifically: archived notes that still claim immediate tables/tab bars/drag-drop are absent
     should be marked historical or updated with current evidence.

5. Decide separately whether Fret wants a dedicated immediate debug-draw adapter.
   - Outcome: either a clear non-goal or a narrow ecosystem lane for custom draw-list-like tools.
   - Constraint: keep it out of the minimal frontend unless the adapter story is proven.

## Bottom Line

Fret IMUI is no longer blocked by "missing the obvious imgui basics". The architecture is mostly in
the right place already:

- `fret-imui` is thin,
- `fret-ui-kit::imui` owns policy,
- first-party teaching surfaces now route through `fret::imui`,
- and the remaining parity work is concentrated in deeper policy and editor-grade UX.

If the goal is "reach imgui-level usefulness", the next wins are:

- the remaining text/input parity after read-only/password/auto-select-all/AllowTabInput,
- menu/tab depth,
- continued test/control architecture decomposition after the first picker, color-model, popup,
  popup-numeric, popup-picker, popup-preview, and popup-swatches splits,
- and a deliberate decision on whether immediate debug-draw belongs in the ecosystem.

Not the right next move:

- another broad IMUI surface reset,
- or making `fret-imui` itself a large, stateful widget framework.
