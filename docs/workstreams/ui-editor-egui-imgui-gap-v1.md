# `fret-ui-editor` v1 — Egui / ImGui Capability Gap Matrix


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- egui: https://github.com/emilk/egui
- Dear ImGui: https://github.com/ocornut/imgui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Active gap analysis (workstream note; not an ADR)  
Last updated: 2026-02-16

## Purpose

This document answers: **“Compared to egui and Dear ImGui, what capabilities are we still missing to ship a credible editor-grade component surface?”**

Scope:

- Compare against:
  - `repo-ref/egui` (Rust immediate-mode UI with a mature widget/style system)
  - `repo-ref/imgui` (Dear ImGui; editor UX baseline and taxonomy reference)
- Focus on the editor surface we are building:
  - `ecosystem/fret-ui-editor` (controls + composites)
  - `ecosystem/fret-ui-kit` (policy/infra: menu/select/scroll/table/tooltips/icons)
  - `crates/fret-ui` (mechanisms: layout, focus, text input, overlays, painting)

Non-goals:

- API-compatibility with egui/imgui
- Moving policy into `crates/fret-ui`

Related:

- Workstream summary: `docs/workstreams/ui-editor-v1.md`
- Tracker: `docs/workstreams/ui-editor-v1-todo.md`
- ImGui/precision inventory: `docs/workstreams/ui-editor-imgui-alignment-v1.md`

## Baseline: what egui/imgui give “for free”

Both egui and Dear ImGui ship with (or assume) a coherent “widget system”:

- A *single* place to define **interaction-state visuals** (inactive/hovered/active/open/disabled)
- A *single* place to define **spacing & density** knobs (frame padding, item spacing, min hit size)
- Mature **text editing** behavior (selection, caret, IME, undo/redo, multiline, password, etc.)
- A complete “editor starter set” of widgets: button, checkbox/radio/toggle, drag value, slider,
  combo/select, collapsing headers, menus/popups/tooltips, scroll areas, tables/grids

Fret already has strong mechanisms, but our editor layer still needs to *compose them into a consistent,
token-driven, editor-grade widget surface*.

## Matrix

Legend:

- **Owner**: where the fix should land (policy vs mechanism)
- **Priority**: P0 = blocks usability, P1 = blocks “editor feel”, P2 = longer-tail parity

| Area | Egui reference | ImGui reference | Fret status (today) | Gap / missing capability | Owner | Priority |
| --- | --- | --- | --- | --- | --- | --- |
| Interaction-state visuals | `egui::Visuals::widgets` (`inactive/hovered/active/open/noninteractive`) | `ImGuiStyle` + PushStyleColor/Var | `EditorWidgetVisuals` exists; most editor controls consume it | Expand coverage (remaining ad-hoc controls) and add regression gates so visuals don’t drift | `fret-ui-editor` (policy) | P1 |
| Spacing & density knobs | `egui::Spacing` (interact_size, item_spacing, slider widths, icon sizes) | `ImGuiStyle` (FramePadding, ItemSpacing, ScrollbarSize, etc.) | `EditorStyle` resolves `editor.density.*` and centralizes defaults | Keep applying density + responsive width policies consistently across all controls/composites (avoid “too tiny to use” layouts) | `fret-ui-editor` (policy) | P1 |
| Icon strategy | Egui paints vector-ish primitives; optional icon crates | Fonts/icons (e.g. FontAwesome) commonly used | SVG icon pipeline exists (`fret-icons` + packs + `SvgIcon`) and is used across most chrome | Ensure remaining chrome affordances (including reset-to-default) use semantic SVG icons (`ui.reset`, etc.) | `fret-ui-editor` (policy) | P1 |
| State identity / internal widget state | `egui::Id` / `Response::id`-driven widget identity | `ImGuiID` (label hashing) + `PushID` | Implemented per-instance keying for stateful editor controls and keyed demo model helpers by stable names | Audit remaining stateful controls and enforce a keying rule so multiple widgets never share drag/typing/open state (prefer explicit `id_source`, otherwise `(callsite, model.id())`; never use `test_id`) | `fret-ui-editor` (policy) | P0 |
| Checkbox / tri-state visuals | `Spacing::icon_width*`, `WidgetVisuals` | Checkbox uses frame + check mark | Checkbox existed but used glyphs; now SVG icons | Still needs “hover/active/disabled” tuning, mixed state clarity, and consistent sizing | `fret-ui-editor` (policy) | P1 |
| DragValue feature completeness | `egui::DragValue` (speed/range/prefix/suffix/formatter/parser) | `DragFloat*` flags/range/format | `DragValueCore` + `NumericInput` exist; tokens for speed/modifiers | Missing: prefix/suffix, explicit range clamp policy, step, value-decimals policy, unit formatting helpers | `fret-ui-editor` (policy) | P1 |
| Slider widgets | `egui::Slider` (clamping, step, log, vertical, show_value) | `SliderFloat*`, `VSliderFloat*` | `Slider<T>` exists (horizontal, clamp+step, value+typing) | Missing: vertical/log variants, range labeling, and richer unit formatting (beyond `percent_0_1_*`) | `fret-ui-editor` (policy) | P1 |
| Text input richness | `egui::TextEdit` (multiline/password/IME/cursor/selection/undo) | `InputText*` + flags (password, undo, completion/history) | `TextField` exists (single/multi) and powers `MiniSearchBox` + `NumericInput` | Finish editor-grade text input parity: password mode, completion/history hook placeholders, and richer selection defaults | `fret-ui-editor` + maybe `crates/fret-ui` (mechanism gaps) | P0/P1 |
| Menus / popups / context menu | `MenuButton`, `Area`, tooltips | `BeginMenu`, `BeginPopup*`, context popup helpers | `fret-ui-kit` has Menu/ContextMenu/OverlayController/TooltipProvider | Editor layer needs concrete recipes (inspector row menu, enum select menu, right-click row menu) + consistent chrome | `fret-ui-editor` (policy) | P1 |
| Damage / invalidation correctness (overlays) | full repaint each frame (cheap CPU, predictable visuals) | full repaint each frame | Mitigated for popover-like overlays by invalidating the base root on hide; still treat as a mechanism gate | Ensure overlay mount/unmount and close transitions invalidate underlying regions with correct damage tracking (avoid “repaint everything forever” while keeping correctness) | `crates/fret-ui` (mechanism) | P0 |
| Scroll areas & scrollbars | `ScrollArea` + `ScrollStyle` | Child windows + scrollbars | `fret-ui-kit` has scroll area policy & visibility helpers | Editor widgets should expose scroll affordances (thin/auto, hover reveal) and apply consistent tokens | `fret-ui-editor` (policy) | P1 |
| Tables / grids | `Grid` + `egui_extras` tables | `BeginTable` | `fret-ui` has Grid; `fret-ui-kit` has a declarative Table + VirtualList infrastructure | Editor needs “property table” and “list/table” recipes that look/feel editor-grade (striping, header, resize, sort) | `fret-ui-editor` (policy) | P1 |
| Docking UX polish | ImGui docking demo parity | `DockSpace`, docking flags | Tab labels + close/overflow icons are painted; close hit target is larger | Still needs: more consistent hit targets (tabs/overflow), overflow menu polish, and tab chrome parity (track separately from editor controls) | `fret-docking` (policy) | P2 |
| Keyboard navigation & focus | Egui has focusable responses; ImGui has nav config | Nav, focus rules, Escape behaviors | Mechanisms exist; policies vary per control | Define editor defaults: Tab order, Enter/Escape semantics, focus ring consistency | `fret-ui-editor` (policy) | P1 |
| Drag & drop (payload UX) | Egui has dnd patterns in ecosystem | ImGui DragDrop API | Docking has DnD; editor controls mostly don’t | Add editor-grade DnD affordances for lists/trees (future) | `fret-ui-editor` + kit | P2 |
| “Editor feel” composites | egui demo apps & inspectors | ImGui demo + ecosystem patterns | `PropertyGrid/Group/Row`, `TransformEdit` exist | Still missing: InspectorPanel recipe, consistent section spacing, inline help/tooltips, status badges alignment | `fret-ui-editor` (policy) | P1 |

## Recommended workstream adjustments (high-signal)

1) **Finish convergence on editor widget visuals** (policy):
   - Keep moving remaining ad-hoc controls onto `EditorWidgetVisuals`
   - Add/extend scripted repro gates so hover/active/open/disabled visuals can’t drift silently

2) **Complete editor-grade text input**:
   - Add password mode (masking + copy policy)
   - Add completion/history hook placeholders (policy first; mechanism later)

3) **Keep responsive minimums explicit** (policy):
   - Define width/stacking knobs (e.g. `editor.vec.axis_min_width`) so narrow inspectors stay usable
   - Prefer “Auto” layout variants that degrade gracefully rather than shrinking controls below readability

4) **Treat overlay close/damage as a mechanism gate**:
   - Keep minimal scripted repros (open/close select overlay repeatedly + screenshots)
   - Ensure underlying regions repaint deterministically on close (no stale pixels)
