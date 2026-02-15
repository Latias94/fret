# `fret-ui-editor` v1 — Egui / ImGui Capability Gap Matrix

Status: Active gap analysis (workstream note; not an ADR)  
Last updated: 2026-02-15

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
| Interaction-state visuals | `egui::Visuals::widgets` (`inactive/hovered/active/open/noninteractive`) | `ImGuiStyle` + PushStyleColor/Var | Many controls hand-roll hover/active/disabled chrome | Need a shared editor “widget visuals” resolver so components don’t drift | `fret-ui-editor` (policy) | P0 |
| Spacing & density knobs | `egui::Spacing` (interact_size, item_spacing, slider widths, icon sizes) | `ImGuiStyle` (FramePadding, ItemSpacing, ScrollbarSize, etc.) | `editor.density.*` exists; some controls apply it, some don’t | Define/centralize: row height, hit targets, padding, icon size, gaps; apply across all controls/composites | `fret-ui-editor` (policy) | P0 |
| Icon strategy | Egui paints vector-ish primitives; optional icon crates | Fonts/icons (e.g. FontAwesome) commonly used | SVG icon pipeline exists (`fret-icons` + packs + `SvgIcon`) | Ensure all chrome affordances (chevrons, carets, close, check, minus) use semantic SVG icons (no tofu glyphs) | `fret-ui-editor` (policy) | P0 |
| State identity / internal widget state | `egui::Id` / `Response::id`-driven widget identity | `ImGuiID` (label hashing) + `PushID` | Some editor controls have stateful internals (`with_state`) and are sensitive to identity/keying | Document and enforce a keying rule so multiple widgets never share drag/typing/open state (prefer `test_id`, otherwise `model.id()`) | `fret-ui-editor` (policy) | P0 |
| Checkbox / tri-state visuals | `Spacing::icon_width*`, `WidgetVisuals` | Checkbox uses frame + check mark | Checkbox existed but used glyphs; now SVG icons | Still needs “hover/active/disabled” tuning, mixed state clarity, and consistent sizing | `fret-ui-editor` (policy) | P1 |
| DragValue feature completeness | `egui::DragValue` (speed/range/prefix/suffix/formatter/parser) | `DragFloat*` flags/range/format | `DragValueCore` + `NumericInput` exist; tokens for speed/modifiers | Missing: prefix/suffix, explicit range clamp policy, step, value-decimals policy, unit formatting helpers | `fret-ui-editor` (policy) | P1 |
| Slider widgets | `egui::Slider` (clamping, step, log, vertical, show_value) | `SliderFloat*`, `VSliderFloat*` | `Slider<T>` exists (horizontal, clamp+step, value+typing) | Missing: vertical/log variants, range labeling, and richer unit formatting (beyond `percent_0_1_*`) | `fret-ui-editor` (policy) | P1 |
| Text input richness | `egui::TextEdit` (multiline/password/IME/cursor/selection/undo) | `InputText*` + flags (password, undo, completion/history) | `crates/fret-ui` has TextInput; editor has `MiniSearchBox` + `NumericInput` | Need a reusable `TextField` control surface (single/multi, password, selection defaults, clear buttons, completion hooks) | `fret-ui-editor` + maybe `crates/fret-ui` (mechanism gaps) | P0/P1 |
| Menus / popups / context menu | `MenuButton`, `Area`, tooltips | `BeginMenu`, `BeginPopup*`, context popup helpers | `fret-ui-kit` has Menu/ContextMenu/OverlayController/TooltipProvider | Editor layer needs concrete recipes (inspector row menu, enum select menu, right-click row menu) + consistent chrome | `fret-ui-editor` (policy) | P1 |
| Damage / invalidation correctness (overlays) | full repaint each frame (cheap CPU, predictable visuals) | full repaint each frame | Some cases show stale pixels when a non-modal overlay closes (ghosting artifacts) | Ensure overlay mount/unmount and close transitions invalidate underlying regions (or provide a safe fallback clear policy while the damage system is hardened) | `crates/fret-ui` (mechanism) | P0 |
| Scroll areas & scrollbars | `ScrollArea` + `ScrollStyle` | Child windows + scrollbars | `fret-ui-kit` has scroll area policy & visibility helpers | Editor widgets should expose scroll affordances (thin/auto, hover reveal) and apply consistent tokens | `fret-ui-editor` (policy) | P1 |
| Tables / grids | `Grid` + `egui_extras` tables | `BeginTable` | `fret-ui` has Grid; `fret-ui-kit` has a declarative Table + VirtualList infrastructure | Editor needs “property table” and “list/table” recipes that look/feel editor-grade (striping, header, resize, sort) | `fret-ui-editor` (policy) | P1 |
| Docking UX polish | ImGui docking demo parity | `DockSpace`, docking flags | Tab labels + close/overflow icons are painted; close hit target is larger | Still needs: more consistent hit targets (tabs/overflow), overflow menu polish, and tab chrome parity | `fret-docking` (policy) | P0/P1 |
| Keyboard navigation & focus | Egui has focusable responses; ImGui has nav config | Nav, focus rules, Escape behaviors | Mechanisms exist; policies vary per control | Define editor defaults: Tab order, Enter/Escape semantics, focus ring consistency | `fret-ui-editor` (policy) | P1 |
| Drag & drop (payload UX) | Egui has dnd patterns in ecosystem | ImGui DragDrop API | Docking has DnD; editor controls mostly don’t | Add editor-grade DnD affordances for lists/trees (future) | `fret-ui-editor` + kit | P2 |
| “Editor feel” composites | egui demo apps & inspectors | ImGui demo + ecosystem patterns | `PropertyGrid/Group/Row`, `TransformEdit` exist | Still missing: InspectorPanel recipe, consistent section spacing, inline help/tooltips, status badges alignment | `fret-ui-editor` (policy) | P1 |

## Recommended workstream adjustments (high-signal)

1) **Define `EditorWidgetVisuals`** (policy) analogous to `egui::Widgets`:
   - Resolve from theme tokens (no new runtime contracts)
   - Provide state palette: inactive/hovered/active/open/disabled
   - Make all editor controls consume this (instead of bespoke colors)

2) **Harden `Slider<T>` and extend parity** (policy):
   - Add value display, optional typing path, unit formatting hooks
   - Consider vertical and/or log variants only if demanded by demos
   - Required for parity with common inspectors and for “coarse vs fine” editing

3) **Elevate `TextField` as a first-class editor control**:
   - Single-line + multi-line
   - Password mode (masking + copy policy)
   - Optional clear button + completion/history hooks (policy first; mechanism later)

4) **Docking polish continues as a parallel track**:
   - Tab labels + close/overflow affordances
   - Icons (close, overflow, pin/lock later)
   - Stable hit targets and alignment

5) **Fix overlay close ghosting as a mechanism gate**:
   - Add a minimal scripted repro (open/close select overlay repeatedly)
   - Make the underlying region repaint deterministically on close (no stale pixels)
