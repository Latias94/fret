# Authoring Ergonomics — Fluent Builder TODOs (v1)

Status: Active

This tracker focuses on authoring ergonomics improvements that stay within Fret’s layering rules.

Related:

- Design note: `docs/workstreams/authoring-ergonomics-fluent-builder.md`
- Shadcn progress: `docs/shadcn-declarative-progress.md`

## Tracking Format

- ID: `AUE-{area}-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

## A. Fluent “Edges” Helpers

- [x] AUE-helpers-001 Add `UiBuilder::paddings(...)` that accepts a single token-aware 4-edge value.
  - Evidence: `ecosystem/fret-ui-kit/src/edges4.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-helpers-002 Add `UiBuilder::margins(...)` that accepts a single token-aware 4-edge value (supports `auto`).
  - Evidence: `ecosystem/fret-ui-kit/src/edges4.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-helpers-003 Add `UiBuilder::insets(...)` (positioning) that accepts a token-aware 4-edge value and supports
  signed/negative values via `SignedMetricRef`.
  - Evidence: `ecosystem/fret-ui-kit/src/edges4.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`

## B. Chrome Presets (Discoverable Recipes)

- [x] AUE-chrome-010 Add debug-only builder helpers (debug border) consistent with shadcn token names.
  - Evidence: `ecosystem/fret-ui-kit/src/style/chrome.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-chrome-011 Add a kit-level “focused border/ring” preset usable by multiple shadcn components.
  - Evidence: `ecosystem/fret-ui-kit/src/style/chrome.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-chrome-012 Add per-corner radius refinement support (`corner_radii(...)` or `rounded_tl/...`).
  - Evidence: `ecosystem/fret-ui-kit/src/corners4.rs`, `ecosystem/fret-ui-kit/src/style/chrome.rs`
  - Builder: `ecosystem/fret-ui-kit/src/ui_builder.rs`
  - Resolution: `ecosystem/fret-ui-kit/src/declarative/style.rs`
- [x] AUE-chrome-013 Add shadow shorthands to the `ui()` chain (e.g. `shadow_sm/md/lg`).
  - Evidence: `ecosystem/fret-ui-kit/src/style/chrome.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
  - Resolution: `ecosystem/fret-ui-kit/src/declarative/style.rs`
- [x] AUE-chrome-014 Add `border_width(...)` / `radius(...)` setters that accept `Px` / `MetricRef` (avoid struct literal noise).
  - Evidence: `ecosystem/fret-ui-kit/src/style/chrome.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`

## C. Layout Constructors (Reduce Props Noise)

- [x] AUE-layout-020 Add `ui::h_flex(...)` / `ui::v_flex(...)` constructors in `fret-ui-kit` that return a patchable builder.
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-layout-021 Add a minimal `ui::stack(...)` constructor (overlay composition helper; optional).
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-layout-022 Add “gap” and alignment shorthands on the layout constructor path (not only on components).
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs`

## D. Surface Consolidation

- [x] AUE-surface-030 Decide whether `ecosystem/fret-ui-kit/src/styled.rs` should be:
  - deprecated in favor of `ui()`, or
  - expanded to be a thin alias over `UiBuilder`, or
  - kept intentionally tiny (and documented as such).
  - Decision: keep intentionally tiny + chrome-only; do not expand.
  - Evidence: `docs/workstreams/authoring-ergonomics-fluent-builder.md`
- [x] AUE-surface-031 Ensure `fret-ui-shadcn` prelude re-exports the single recommended authoring chain.
  - Evidence: `ecosystem/fret-ui-shadcn/src/lib.rs`

## E. Documentation / Adoption

- [x] AUE-docs-040 Add an “Authoring Golden Path” section with before/after examples in `docs/shadcn-declarative-progress.md`.
  - Evidence: `docs/shadcn-declarative-progress.md`
- [x] AUE-docs-041 Add a short cookbook for layout-only authoring (flex/grid/stack) using the new constructors.
  - Evidence: `docs/shadcn-declarative-progress.md`
- [x] AUE-docs-042 Prefer direct `Px(...)` arguments for `*_px(...)` APIs (they accept `impl Into<MetricRef>`).
  - Evidence: `apps/fret-ui-gallery/src/ui.rs`
- [x] AUE-docs-043 Prefer direct `Px(...)` arguments in web parity tests to reduce noise.
  - Evidence: `ecosystem/fret-ui-shadcn/tests/{web_vs_fret_*,radix_web_*}.rs`
- [x] AUE-docs-044 Prefer direct `Px(...)` arguments in ecosystem components (avoid `MetricRef::Px(...)` boilerplate).
  - Evidence: `ecosystem/fret-ui-shadcn/src/{badge,button,checkbox,combobox,command,drawer,input_group,pagination,progress,radio_group,select,switch,tabs,toggle,toggle_group}.rs`
  - Evidence: `ecosystem/fret-ui-material3/src/{radio_group,select}.rs`
- [x] AUE-docs-045 Prefer direct `Px(...)` arguments in kit-level declarative helpers and goldens.
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/{icon,text_field}.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
- [x] AUE-docs-046 Remove remaining `MetricRef::Px(Px(...))` noise in apps/examples and shadcn surfaces.
  - Evidence: `apps/fret-examples/src/{assets_demo,markdown_demo}.rs`
  - Evidence: `apps/fret-ui-gallery/src/ui.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/src/{command,hover_card}.rs`

## F. (Future) Proc-macro / Derive

- [ ] AUE-macro-050 Re-audit `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md` and decide the minimal derive set:
  - `IntoElement` for props structs
  - `RenderOnce` boilerplate reduction
  - ergonomics for children slots

## G. Parity / Presets (High ROI)

- [x] AUE-parity-060 Add a shadcn-level `popover_style` preset that can be applied from the `ui()` chain (policy layer).
  - Evidence: `ecosystem/fret-ui-shadcn/src/ui_builder_ext/surfaces.rs`, `ecosystem/fret-ui-shadcn/src/popover.rs`
- [x] AUE-parity-061 Add a shadcn-level `dialog_style` preset that can be applied from the `ui()` chain (policy layer).
  - Evidence: `ecosystem/fret-ui-shadcn/src/ui_builder_ext/surfaces.rs`, `ecosystem/fret-ui-shadcn/src/dialog.rs`
- [x] AUE-parity-062 Add shadcn-level `menu_style` / `menu_sub_style` presets that can be applied from the `ui()` chain (policy layer).
  - Evidence: `ecosystem/fret-ui-shadcn/src/ui_builder_ext/surfaces.rs`, `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- [x] AUE-layout-023 Add a patchable `ui::container(...)` constructor as the default “box” layout node.
  - Intent: reduce raw `cx.container(...)` usage in cookbook/recipes.
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-layout-024 Add `*_px(...)` positioning/margin shorthands on the fluent builder surface.
  - Motivation: reduce verbosity when authoring pixel-precise overlays/popovers (avoid `Edges4::all(Px(...))` for the common case).
  - Evidence: `ecosystem/fret-ui-kit/src/style/layout.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] AUE-text-070 Add a minimal patchable `ui::text(...)` / `ui::label(...)` authoring constructor with a small typed refinement surface.
  - Scope: size/weight/color + a shadcn-aligned default line-height.
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`, `ecosystem/fret-ui-kit/src/declarative/text.rs`

## I. Iterable Children / Render Callbacks

- [x] AUE-iter-090 Make high-frequency row/cell render callbacks iterator-friendly (avoid forcing `Vec<AnyElement>`).
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/{list,table}.rs`
- [x] AUE-iter-091 Make sortable list recipes accept iterable row children.
  - Evidence: `ecosystem/fret-ui-kit/src/recipes/sortable_dnd.rs`
- [x] AUE-iter-092 Make Material3 dialog/tooltip surfaces accept iterable content closures.
  - Evidence: `ecosystem/fret-ui-material3/src/{dialog,tooltip}.rs`
- [x] AUE-iter-093 Add `Elements`-returning overlay layer assembly helpers (avoid forcing `Vec<AnyElement>`).
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/{alert_dialog,dialog,popover,select}.rs`
- [x] AUE-iter-094 Add `with_elements` helpers for tooltip providers; make tooltip requests accept iterable children.
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`, `ecosystem/fret-ui-shadcn/src/tooltip.rs`, `ecosystem/fret-ui-material3/src/tooltip.rs`
- [x] AUE-iter-095 Add `AnyElementIterExt::elements_owned()` for collecting into `Elements`.
  - Evidence: `crates/fret-ui/src/element.rs`
- [x] AUE-iter-108 Make `Elements` implement `IntoIterator` (so it can be passed to any `IntoIterator<Item = AnyElement>` API).
  - Evidence: `crates/fret-ui/src/element.rs`
- [x] AUE-iter-096 Audit overlay request constructors: already accept `IntoIterator<Item = AnyElement>`; keep `children: Vec<AnyElement>` as the stable storage boundary.
  - Evidence: `ecosystem/fret-ui-kit/src/overlay_controller.rs`, `ecosystem/fret-ui-kit/src/window_overlays/requests.rs`
- [x] AUE-iter-097 Make shadcn list-style builder inputs accept `IntoIterator` (avoid forcing `Vec<T>`).
  - Evidence: `ecosystem/fret-ui-shadcn/src/{accordion,button_group,command,context_menu,data_table_controls,drawer,dropdown_menu,menubar,navigation_menu,resizable,select,toggle_group}.rs`
  - Evidence: `ecosystem/fret-ui-kit/src/{overlay_controller,tree,viewport_tooling}.rs`
- [x] AUE-iter-098 Make menu/item authoring closures accept iterable outputs (avoid forcing `Vec<T>`).
  - Evidence: `ecosystem/fret-ui-shadcn/src/{accordion,context_menu,dropdown_menu,menubar,navigation_menu,resizable,tabs,toggle_group}.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/src/ui_builder_ext/menus.rs`
- [x] AUE-iter-099 Add `Elements` helpers for select pointer-up guard assembly; remove legacy `*_layer_children*` wrappers.
  - Motivation: keep `#![deny(deprecated)]` crates clean while moving callsites to iterator-friendly APIs.
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/select.rs`, `ecosystem/fret-ui-kit/src/primitives/{dialog,alert_dialog,popover}.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/src/{dialog,popover,select,sheet}.rs`, `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- [x] AUE-iter-100 Validate iterator-friendly layer helpers via `todo_demo` and ecosystem overlays (reduce `vec![...]` boilerplate).
  - Evidence: `apps/fret-examples/src/todo_demo.rs`, `ecosystem/fret-ui-shadcn/src/{alert_dialog,select}.rs`, `ecosystem/fret-ui-material3/src/select.rs`
- [x] AUE-iter-101 Add sink-based `stack::{hstack_build,vstack_build}` helpers to avoid `.elements()` borrow pitfalls.
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/stack.rs`
  - Evidence: `apps/fret-examples/src/todo_demo.rs`
- [x] AUE-iter-102 Add sink-based `ui::{h_flex_build,v_flex_build}` helpers that preserve the fluent patch surface.
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`
  - Evidence: `apps/fret-examples/src/todo_mvu_demo.rs`
- [x] AUE-iter-103 Remove remaining `.elements()` hotspots in `fret-examples` by using sink-based builders or eager `Vec` assembly.
  - Evidence: `apps/fret-examples/src/{assets_demo,todo_interop_demo,todo_interop_kit_demo,todo_mvu_interop_demo}.rs`
- [x] AUE-iter-104 Adopt iterable/sink-based patterns in `fret-ui-gallery` to reduce boilerplate in real-world authoring code.
  - Evidence: `apps/fret-ui-gallery/src/ui.rs`
- [x] AUE-iter-105 Adopt iterable children patterns in demo shells (reduce `vec![...]` where APIs accept `IntoIterator`).
  - Evidence: `apps/fret-demo/src/bin/hotpatch_smoke_demo.rs`
- [x] AUE-iter-106 Remove redundant `stack::{hstack_iter,vstack_iter}` wrappers (base functions already accept `IntoIterator`).
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/stack.rs`, `ecosystem/fret-ui-kit/src/declarative/scroll.rs`
  - Evidence: `apps/fret-examples/src/{todo_interop_kit_demo,todo_mvu_interop_demo}.rs`
- [x] AUE-iter-107 Remove redundant `TooltipProvider::with` wrappers (keep `with_elements` as the single surface).
  - Evidence: `ecosystem/fret-ui-shadcn/src/tooltip.rs`, `ecosystem/fret-ui-material3/src/tooltip.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`
  - Evidence: `apps/fret-ui-gallery/src/{ui,docs}.rs`
- [x] AUE-iter-109 Make render harness callbacks accept iterable output (avoid forcing `-> Vec<AnyElement>`).
  - Evidence: `apps/fret-ui-gallery/src/ui.rs`
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/popper_content.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/src/command.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/tests/{snapshots,web_vs_fret_overlay_chrome,web_vs_fret_overlay_placement,radix_web_overlay_geometry,radix_web_primitives_state}.rs`
- [x] AUE-iter-110 Add `ui::container_build(...)` sink constructor to avoid iterator borrow pitfalls.
  - Evidence: `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`

## H. Adoption Audit — `ui::text` in `fret-ui-shadcn`

Goal: replace ad-hoc `cx.text_props(TextProps { ... })` callsites with `fret_ui_kit::ui::text(...)` / `ui::label(...)` builders,
while keeping geometry/overflow semantics stable (verified via web goldens).

Current state (as of 2026-01-21):

- Adopted: `ecosystem/fret-ui-shadcn/src/accordion.rs`, `ecosystem/fret-ui-shadcn/src/alert.rs`, `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`, `ecosystem/fret-ui-shadcn/src/avatar.rs`, `ecosystem/fret-ui-shadcn/src/badge.rs`, `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`, `ecosystem/fret-ui-shadcn/src/button.rs`, `ecosystem/fret-ui-shadcn/src/calendar.rs`, `ecosystem/fret-ui-shadcn/src/calendar_range.rs`, `ecosystem/fret-ui-shadcn/src/card.rs`, `ecosystem/fret-ui-shadcn/src/combobox.rs`, `ecosystem/fret-ui-shadcn/src/command.rs`, `ecosystem/fret-ui-shadcn/src/context_menu.rs`, `ecosystem/fret-ui-shadcn/src/data_table.rs`, `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`, `ecosystem/fret-ui-shadcn/src/dialog.rs`, `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`, `ecosystem/fret-ui-shadcn/src/empty.rs`, `ecosystem/fret-ui-shadcn/src/field.rs`, `ecosystem/fret-ui-shadcn/src/hover_card.rs`, `ecosystem/fret-ui-shadcn/src/input_otp.rs`, `ecosystem/fret-ui-shadcn/src/item.rs`, `ecosystem/fret-ui-shadcn/src/kbd.rs`, `ecosystem/fret-ui-shadcn/src/menubar.rs`, `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`, `ecosystem/fret-ui-shadcn/src/popover.rs`, `ecosystem/fret-ui-shadcn/src/select.rs`, `ecosystem/fret-ui-shadcn/src/sheet.rs`, `ecosystem/fret-ui-shadcn/src/sidebar.rs`, `ecosystem/fret-ui-shadcn/src/table.rs`, `ecosystem/fret-ui-shadcn/src/tabs.rs`, `ecosystem/fret-ui-shadcn/src/toggle.rs`, `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- Remaining: 0 `cx.text_props(TextProps { ... })` callsites under `ecosystem/fret-ui-shadcn/src`

Top remaining hotspots (by callsite count):

| Count | File |
| ---: | --- |
| 0 | _none_ |

Migration guidelines:

- Prefer `ui::label(cx, ...)` for 1-line UI labels (defaults: `nowrap + clip`); prefer `ui::text(cx, ...)` for multi-line/body text.
- If the old code used `TextProps::new(...)` (unstyled), prefer `ui::raw_text(cx, ...)` to preserve `style: None` and keep inherited styling behavior.
- When the old code set explicit layout height (e.g. badge), keep it with `.h_px(...)` plus `.line_height_px(...)`.
- When the old code set wrap/overflow, keep it explicit with `.wrap(...)`, `.nowrap()`, `.truncate()` (avoid semantic drift).
- Land changes component-by-component, gated by the relevant web-golden tests (avoid “big bang” refactors).

Next TODOs (suggested order: low-risk → high-risk):

- [x] AUE-adopt-text-080 Migrate `CardTitle` / `CardDescription` text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/card.rs`
- [x] AUE-adopt-text-081 Migrate `Breadcrumb*` text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`
- [x] AUE-adopt-text-082 Migrate `Button` label text callsites (ensure alignment/height matches web goldens).
  - Evidence: `ecosystem/fret-ui-shadcn/src/button.rs`
- [x] AUE-adopt-text-083 Migrate `Kbd` text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/kbd.rs`
- [x] AUE-adopt-text-084 Migrate `Empty` text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/empty.rs`
- [x] AUE-adopt-text-085 Migrate `Command` (command palette) text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/command.rs`
- [x] AUE-adopt-text-090 Migrate menu family text callsites after surface presets settle.
  - Done: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
  - Done: `ecosystem/fret-ui-shadcn/src/context_menu.rs`
  - Done: `ecosystem/fret-ui-shadcn/src/menubar.rs`
- [x] AUE-adopt-text-091 Migrate `Tooltip` text callsites (highest density; beware of placement + masking).
  - Evidence: `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- [x] AUE-adopt-text-092 Migrate `Field*` text callsites (keep layout/line-height stable).
  - Evidence: `ecosystem/fret-ui-shadcn/src/field.rs`
- [x] AUE-adopt-text-093 Migrate `Select` text callsites (trigger + listbox options; verify overlay goldens).
  - Evidence: `ecosystem/fret-ui-shadcn/src/select.rs`
- [x] AUE-adopt-text-094 Migrate `HoverCard` text callsites (unstyled content uses `ui::raw_text`).
  - Evidence: `ecosystem/fret-ui-shadcn/src/hover_card.rs`
- [x] AUE-adopt-text-095 Migrate `Sidebar` text callsites (menu buttons use `ui::text` + `truncate`).
  - Evidence: `ecosystem/fret-ui-shadcn/src/sidebar.rs`
- [x] AUE-adopt-text-096 Migrate `Table` text callsites (head + caption; verify layout goldens).
  - Evidence: `ecosystem/fret-ui-shadcn/src/table.rs`
- [x] AUE-adopt-text-097 Migrate `Sheet` text callsites (title + description; verify overlay goldens).
  - Evidence: `ecosystem/fret-ui-shadcn/src/sheet.rs`
- [x] AUE-adopt-text-098 Migrate `Popover` text callsites (title + description; verify overlay goldens).
  - Evidence: `ecosystem/fret-ui-shadcn/src/popover.rs`
- [x] AUE-adopt-text-099 Migrate `Item` text callsites (title uses `truncate`).
  - Evidence: `ecosystem/fret-ui-shadcn/src/item.rs`
- [x] AUE-adopt-text-100 Migrate `Accordion` fallback trigger label text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/accordion.rs`
- [x] AUE-adopt-text-101 Migrate `AlertDialogTitle` / `AlertDialogDescription` text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
- [x] AUE-adopt-text-102 Migrate `Calendar` day + navigation button text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/calendar.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs`
- [x] AUE-adopt-text-103 Migrate `CalendarRange` day + navigation button text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/calendar_range.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs`
- [x] AUE-adopt-text-104 Migrate `Combobox` trigger + option label text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/combobox.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`, `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs`
- [x] AUE-adopt-text-105 Migrate `DataTable` header label + sort indicator text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/data_table.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
- [x] AUE-adopt-text-106 Migrate `DialogTitle` / `DialogDescription` text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/dialog.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
- [x] AUE-adopt-text-107 Migrate `InputOtp` separator + slot glyph text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/input_otp.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs`
- [x] AUE-adopt-text-108 Migrate `Toggle` label text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/toggle.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
- [x] AUE-adopt-text-109 Migrate `NavigationMenuTrigger` fallback label text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
- [x] AUE-adopt-text-110 Migrate `TabsTrigger` fallback label text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/tabs.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
- [x] AUE-adopt-text-111 Migrate `AvatarFallback` text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/avatar.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
- [x] AUE-adopt-text-112 Migrate `DataTable` recipe text callsites.
  - Evidence: `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`
  - Tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
