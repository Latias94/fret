# shadcn Part Surface Alignment v1 — Inventory

This document is a **workstream-local** inventory of upstream shadcn/ui v4 **radix base** components
and the corresponding `ecosystem/fret-ui-shadcn` modules.

Goal: make it easy to answer “which components are still missing the upstream part split / naming?”
without having to grep the whole crate.

## Legend

- **Surface kind**
  - `parts`: exported as upstream-named part structs/functions
  - `recipe`: higher-level builder / closure API (still may re-export parts)
  - `adapter`: a `into_element_parts(...)` bridge that makes call sites look like upstream even when
    the underlying structure is not literally nested parts
- **Audited** means the component has an explicit entry in `TODO.md` with a gate anchor.

## Recommended next audit order (dev sequence)

1. `button` / `toggle` (variants helpers parity; ensure copy/paste authoring stays stable)
2. **Defer last**: `select` / `combobox` deeper redesign (structural drift is known and deeper than “just names”)

## Inventory table (upstream radix base → Fret module)

| Upstream component | Upstream base file | Fret module | Surface kind (today) | `into_element_parts` | Audited in `TODO.md` | Notes |
|---|---|---|---|---:|---:|---|
| `accordion` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/accordion.tsx` | `ecosystem/fret-ui-shadcn/src/accordion.rs` | parts | No | Yes | Uses measured-height motion wrapper; trigger/content are Rust-native (no DOM slots). |
| `alert` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/alert.tsx` | `ecosystem/fret-ui-shadcn/src/alert.rs` | parts | No | Yes | Role stamping + title truncation are gated by unit tests. |
| `alert-dialog` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/alert-dialog.tsx` | `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` | recipe + adapter | Yes | Yes | Trigger/Portal/Overlay are adapters; default “open on activate” when trigger is pressable. |
| `aspect-ratio` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/aspect-ratio.tsx` | `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs` | parts | No | Yes | Re-exports the Radix-aligned primitive; ratio stamping is gated in the primitive tests. |
| `avatar` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/avatar.tsx` | `ecosystem/fret-ui-shadcn/src/avatar.rs` | parts | No | Yes | Size scope footgun candidates. |
| `badge` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/badge.tsx` | `ecosystem/fret-ui-shadcn/src/badge.rs` | parts | No | Yes | Variant fg + shrink-0 + overflow clip are gated by unit tests. |
| `breadcrumb` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/breadcrumb.tsx` | `ecosystem/fret-ui-shadcn/src/breadcrumb.rs` | recipe (+ primitive aliases) | No | Yes | Root name conflict (`Breadcrumb` recipe vs upstream root part). |
| `button` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/button.tsx` | `ecosystem/fret-ui-shadcn/src/button.rs` | parts | No | No | Has `buttonVariants(...)` mapping; not tracked as a component row (yet). |
| `button-group` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/button-group.tsx` | `ecosystem/fret-ui-shadcn/src/button_group.rs` | parts | No | Yes | Unit tests lock merged borders/corners and separator margin defaults. |
| `calendar` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/calendar.tsx` | `ecosystem/fret-ui-shadcn/src/calendar.rs` | parts | No | No | Calendar variants live in multiple modules (`calendar_*`). |
| `card` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/card.tsx` | `ecosystem/fret-ui-shadcn/src/card.rs` | parts | No | Yes | Includes `CardSize` support. |
| `carousel` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/carousel.tsx` | `ecosystem/fret-ui-shadcn/src/carousel.rs` | parts + adapter | Yes | Yes | `into_element_parts` exists; audit/gates live in carousel workstreams. |
| `chart` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/chart.tsx` | `ecosystem/fret-ui-shadcn/src/chart.rs` | parts | No | Yes | Part surface exists but engine wiring is incomplete; tracked as “Done (with known gaps)”. |
| `checkbox` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/checkbox.tsx` | `ecosystem/fret-ui-shadcn/src/checkbox.rs` | parts | No | Yes | Keyboard/pointer outcomes are gated by unit tests. |
| `collapsible` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/collapsible.tsx` | `ecosystem/fret-ui-shadcn/src/collapsible.rs` | parts | No | Yes | Trigger requires an explicit `open: Model<bool>`; content uses measured-height motion wrapper. |
| `combobox` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx` | `ecosystem/fret-ui-shadcn/src/combobox.rs` | recipe + adapter | Yes | Yes | Structural drift vs Base UI expectations remains (defer deeper redesign). |
| `command` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/command.tsx` | `ecosystem/fret-ui-shadcn/src/command.rs` | parts | No | Yes | Cmdk-style listbox semantics + keyboard outcomes are gated by unit tests. |
| `context-menu` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/context-menu.tsx` | `ecosystem/fret-ui-shadcn/src/context_menu.rs` | recipe + adapter | Yes | Yes | Submenu parts are helpers over the closure API. |
| `dialog` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/dialog.tsx` | `ecosystem/fret-ui-shadcn/src/dialog.rs` | recipe + adapter | Yes | Yes | Trigger/Portal/Overlay are adapters; default “open on activate” when trigger is pressable. |
| `direction` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/direction.tsx` | `ecosystem/fret-ui-shadcn/src/direction.rs` | parts | No | Yes | Includes `use_direction` alias for copy/paste parity. |
| `drawer` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/drawer.tsx` | `ecosystem/fret-ui-shadcn/src/drawer.rs` | recipe + adapter | Yes | Yes | Trigger/Portal/Overlay are adapters; default “open on activate” when trigger is pressable. |
| `dropdown-menu` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/dropdown-menu.tsx` | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` | recipe + adapter | Yes | Yes | Portal is a no-op wrapper (overlay root owns portals). |
| `empty` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/empty.tsx` | `ecosystem/fret-ui-shadcn/src/empty.rs` | parts | No | Yes | Uses container queries intentionally. |
| `field` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/field.tsx` | `ecosystem/fret-ui-shadcn/src/field.rs` | parts | No | Yes | Association is modeled via `ControlId` + `ControlRegistry` (`Input`/`Textarea` `.control_id(...)`; helper text `.for_control(...)`; `described-by` prefers error over description). |
| `hover-card` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/hover-card.tsx` | `ecosystem/fret-ui-shadcn/src/hover_card.rs` | parts | No | Yes | Trigger/content composition + hover intent are gated by unit tests. |
| `input` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/input.tsx` | `ecosystem/fret-ui-shadcn/src/input.rs` | parts | No | Yes | Default constraints + selection color token mapping are gated by unit tests. |
| `input-group` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/input-group.tsx` | `ecosystem/fret-ui-shadcn/src/input_group.rs` | recipe + adapter | Yes | Yes | Addon click-to-focus needs explicit hints for interactive descendants. |
| `input-otp` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/input-otp.tsx` | `ecosystem/fret-ui-shadcn/src/input_otp.rs` | recipe + adapter | Yes | Yes | `aria-invalid` is global, not per-slot. |
| `item` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/item.tsx` | `ecosystem/fret-ui-shadcn/src/item.rs` | parts | Yes | Yes | Adds `ItemSize::Xs` + `item_sized(...)` to model `group-data-[size=...]/item:*` outcomes; unit tests lock padding/gap + size-scoped media/content defaults. |
| `kbd` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/kbd.tsx` | `ecosystem/fret-ui-shadcn/src/kbd.rs` | parts | No | Yes | Unit tests lock `h-5` (20px) + `text-xs` (12px) defaults and tooltip slot alpha mapping. |
| `label` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/label.tsx` | `ecosystem/fret-ui-shadcn/src/label.rs` | parts | No | Yes | Re-exports the primitive; association uses `labelled_by_element(...)` on controls; defaults are gated in `fret-ui-kit` tests. |
| `menubar` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/menubar.tsx` | `ecosystem/fret-ui-shadcn/src/menubar.rs` | recipe + adapter | Yes | Yes | Portal is a no-op wrapper; trigger/content are adapters. |
| `native-select` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/native-select.tsx` | `ecosystem/fret-ui-shadcn/src/native_select.rs` | parts | No | Yes | Models a shadcn-aligned select surface (trigger + listbox) and stamps deterministic `test_id`s; gated by unit tests. |
| `navigation-menu` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/navigation-menu.tsx` | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` | parts (+ style helper) | No | Yes | `navigation_menu_trigger_style(...)` is a typed refinement helper. |
| `pagination` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/pagination.tsx` | `ecosystem/fret-ui-shadcn/src/pagination.rs` | parts | No | Yes | Root label + link active semantics + ellipsis hidden semantics are gated by unit tests. |
| `popover` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/popover.tsx` | `ecosystem/fret-ui-shadcn/src/popover.rs` | parts | No | Yes | Placement + focus outcomes are gated by unit tests. |
| `progress` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/progress.tsx` | `ecosystem/fret-ui-shadcn/src/progress.rs` | parts | No | Yes | Optional value `None` maps to 0% and semantics are gated by unit tests. |
| `radio-group` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/radio-group.tsx` | `ecosystem/fret-ui-shadcn/src/radio_group.rs` | parts | No | Yes | Selection + a11y metadata are gated by unit tests. |
| `resizable` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/resizable.tsx` | `ecosystem/fret-ui-shadcn/src/resizable.rs` | parts | No | Yes | Unit tests lock splitter semantics, `test_id_prefix`, and `with_handle` hit thickness. |
| `scroll-area` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/scroll-area.tsx` | `ecosystem/fret-ui-shadcn/src/scroll_area.rs` | parts | No | Yes | Exposes `ScrollBar` alias for copy/paste parity. |
| `select` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/select.tsx` | `ecosystem/fret-ui-shadcn/src/select.rs` | config wrappers + adapter | Yes | Yes | Trigger/Value/Content are config wrappers; `into_element_parts` is callsite-only parity. |
| `separator` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/separator.tsx` | `ecosystem/fret-ui-shadcn/src/separator.rs` | parts | No | Yes | Adds shadcn defaults (`shrink-0`, `h-px w-full`) + a vertical stretch approximation; gated by unit tests. |
| `sheet` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/sheet.tsx` | `ecosystem/fret-ui-shadcn/src/sheet.rs` | recipe + adapter | Yes | Yes | Trigger/Portal/Overlay are adapters; default “open on activate” when trigger is pressable. |
| `sidebar` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/sidebar.tsx` | `ecosystem/fret-ui-shadcn/src/sidebar.rs` | parts | No | Yes | Motion + width invariants are gated by unit tests; includes `useSidebar` + `use_sidebar` alias. |
| `skeleton` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/skeleton.tsx` | `ecosystem/fret-ui-shadcn/src/skeleton.rs` | parts | No | Yes | Default layout + pulse stability policy are gated by unit tests. |
| `slider` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/slider.tsx` | `ecosystem/fret-ui-shadcn/src/slider.rs` | parts | No | Yes | Keyboard/pointer + a11y SetValue outcomes are gated by unit tests. |
| `sonner` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/sonner.tsx` | `ecosystem/fret-ui-shadcn/src/sonner.rs` | parts | No | Yes | Unit tests lock Toaster layout neutrality and toast layer visibility gating. |
| `spinner` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/spinner.tsx` | `ecosystem/fret-ui-shadcn/src/spinner.rs` | parts | No | Yes | Default `size-4` + loading semantics stamping are gated by unit tests. |
| `switch` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/switch.tsx` | `ecosystem/fret-ui-shadcn/src/switch.rs` | parts | No | Yes | Thumb centering + semantics role outcomes are gated by unit tests. |
| `table` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/table.tsx` | `ecosystem/fret-ui-shadcn/src/table.rs` | parts | No | Yes | `ScrollArea(axis=X)` wrapper is best-effort; unit tests lock width defaults + border clearing. |
| `tabs` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/tabs.tsx` | `ecosystem/fret-ui-shadcn/src/tabs.rs` | parts (+ helper) | No | Yes | Part surface + style helper are tracked as “Done (with known gaps)”. |
| `textarea` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/textarea.tsx` | `ecosystem/fret-ui-shadcn/src/textarea.rs` | parts | No | Yes | Shadow wrapper + resize handle policy are gated by unit tests. |
| `toggle` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/toggle.tsx` | `ecosystem/fret-ui-shadcn/src/toggle.rs` | parts (+ helper) | No | No | Has `toggleVariants(...)` mapping; not audited yet. |
| `toggle-group` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/toggle-group.tsx` | `ecosystem/fret-ui-shadcn/src/toggle_group.rs` | parts | No | Yes | Unit tests lock shadcn-aligned root sizing + gap and vertical stretch behavior. |
| `tooltip` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/tooltip.tsx` | `ecosystem/fret-ui-shadcn/src/tooltip.rs` | parts | No | Yes | Inherited defaults + max width are gated by unit tests. |

## Per-component audit template (quick)

For each component we mark “Done”, lock at least one of these:

1. **Part surface parity:** upstream names available for copy/paste authoring (unit test gates for
   type construction + a couple of default refinements).
2. **Default constraints parity:** `w-full` / `flex-1` / `min-w-0` / truncate behavior applied where
   upstream implies it.
3. **Overlay semantics parity (if applicable):** trigger opens, outside press dismisses, focus is
   restored, and keyboard outcomes match (prefer unit tests + optionally a diag script).
