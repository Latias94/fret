# Shadcn Declarative Implementation Progress

Tracks the ongoing work to rebuild Fret's shadcn-aligned component surface as a declarative-only API.

## Source of Truth

This file is the canonical tracker for shadcn/ui v4 parity and the declarative-only migration.

Historical documents under `docs/archive/` are kept for context only and may be stale:

- `docs/archive/backlog/shadcn-v4-component-parity-todo.md` (archived)

Related trackers:

- Cross-repo priorities: `docs/roadmap.md`, `docs/todo-tracker.md`
- Web conformance harness: `docs/shadcn-web-goldens.md`, `docs/audits/shadcn-web-layout-conformance.md`
- new-york-v4 coverage snapshot: `docs/audits/shadcn-new-york-v4-coverage.md`
- new-york-v4 alignment notes: `docs/audits/shadcn-new-york-v4-alignment.md`

Current golden parity snapshot (new-york-v4):

- Keys referenced by tests: `250/448` (`55.8%`)
- Next P0 gaps: open-state date pickers with calendar dropdown caption (`calendar-27..29`) and `calendar-hijri`, then forms/field validation chrome and carousel.

## Scope

- `ecosystem/fret-ui-shadcn`: shadcn/ui v4 naming + taxonomy surface (recipes).
- `ecosystem/fret-ui-kit`: reusable infra (tokens/recipes/headless helpers).
- `crates/fret-ui`: runtime substrate (contracts/mechanisms only).

## Layering & Ownership

This repo intentionally splits responsibilities across three layers (similar to Tailwind + headless + Radix/RSC composition, but in Rust):

- `fret-ui` (**mechanisms/contracts**): element tree, hit-test, focus, semantics/a11y, overlay roots/layers, outside-press observers, layout, paint.
- `fret-ui-kit` (**design-system + infra**, Tailwind-ish): token-driven styling (`Theme` keys + refinements), reusable declarative helpers (`scroll`, `text_field`, etc), and headless state machines (`roving_focus`, hover intent, menu navigation).
- `fret-ui-shadcn` (**taxonomy + recipes**): shadcn/ui v4 naming surface and component composition; no retained widgets, no renderer/platform deps.

App/editor-specific composition belongs in `fret-editor` and ecosystem app layers (e.g. `fret-bootstrap`) (app toolbars, menu bars, command palette wiring).

### Interaction Policy (Action Hooks)

Cross-cutting interaction policies (toggle models, close overlays, selection writes, "dismiss on escape/outside press", etc.) are *component-owned*:

- `fret-ui` provides hook plumbing (`on_activate`, `on_dismiss_request`) as a mechanism-only substrate (ADR 0074).
- `fret-ui-kit` and `fret-ui-shadcn` register handlers to implement policies for each component.
- Legacy runtime shortcuts on `PressableProps` / dismissible roots have been removed from `crates/fret-ui`.
  Use component-owned action hooks (`fret-ui-kit::declarative::action_hooks::ActionHooksExt`) instead.

## Hard Boundary (Enforced in code)

Retained-widget authoring is runtime-internal only:

- `crates/fret-ui`: `widget` module is `pub(crate)`.
- `crates/fret-ui`: `UiTree::create_node` is `pub(crate)`.
- Component crates author via declarative elements (`RenderOnce` / `Render` / `IntoElement`), not `Widget`.

Exception (explicitly gated):

- A dedicated docking crate may depend on a feature-gated, unstable retained-widget substrate for migration purposes.
  This must remain **off by default** and must not be used by shadcn/tailwind component crates.
  - Current gate: `fret-ui/unstable-retained-bridge` (ADR 0075).

---

## Authoring Ergonomics: `ui()` Fluent Builder (P1)

Goal (P1): make shadcn components feel like gpui-component by providing **one** fluent, discoverable
chain for layout + chrome overrides:

- `Button::new("OK").ui().px_2().w_full().rounded_md().into_element(cx)`

This is ecosystem-only (no runtime contract changes). The builder holds a merged `UiPatch`:

- `ChromeRefinement` (control chrome: padding/border/radius/colors, etc)
- `LayoutRefinement` (layout: size, min/max, margins/insets, etc)

Implementation anchors:

- Builder substrate: `ecosystem/fret-ui-kit/src/ui_builder.rs`
- shadcn opt-in glue: `ecosystem/fret-ui-shadcn/src/ui_ext/mod.rs`
- ADR: `docs/adr/0175-unified-authoring-builder-surface-v1.md`
- Coverage audit note: `docs/workstreams/authoring-ergonomics-fluent-builder.md`
- TODO tracker: `docs/workstreams/authoring-ergonomics-fluent-builder-todo.md`

### Status

- `fret-ui-kit`: `ui()` is available for any type that implements `UiPatchTarget`.
- `fret-ui-shadcn`: coverage is incremental via `ui_ext/*` (no component internals required unless a
  component does not yet support chrome/layout refinements).
- Dev note (Windows worktrees): if incremental builds pick up stale artifacts from another worktree,
  run `cargo clean -p fret-ui-kit -p fret-ui-shadcn` (or set a per-worktree `CARGO_TARGET_DIR`).

### Authoring Golden Path

Recommended imports for app code:

```rust
use fret_ui_shadcn::prelude::*;
```

Guidelines:

- Prefer `ui()` for all authoring (chrome + layout + debug helpers).
- Prefer composing shadcn components over introducing new wrapper nodes.
- `StyledExt` exists in `fret-ui-kit` but is intentionally not part of the shadcn prelude to avoid splitting the
  ecosystem into competing patterns.

Before (low density; props structs + wrappers):

```rust
let ok = Button::new("OK").into_element(cx);
let root = cx.container(Default::default(), move |_cx| vec![ok]);
```

After (golden path; fluent patch chain):

```rust
let ok = Button::new("OK")
    .ui()
    .paddings(Edges4::symmetric(Space::N3, Space::N2))
    .shadow_md()
    .focused_border()
    .into_element(cx);
```

### Layout-Only Cookbook (Stack / Flex)

Layout-only code can use `ui::h_flex` / `ui::v_flex` for a patchable builder (so layout nodes participate in the
same fluent `ui()` vocabulary as components). `stack::hstack/vstack` remain available as a lower-level helper.

Horizontal row:

```rust
let row = ui::h_flex(cx, move |cx| {
    vec![
        Button::new("Cancel").ui().into_element(cx),
        Button::new("OK").ui().into_element(cx),
    ]
})
.gap(Space::N2)
.w_full()
.into_element(cx);
```

Vertical column:

```rust
let col = ui::v_flex(cx, move |cx| {
    vec![
        Input::new().ui().w_full().into_element(cx),
        Textarea::new().ui().w_full().into_element(cx),
    ]
})
.gap(Space::N2)
.into_element(cx);
```

Overlay stack (layered children):

```rust
let overlay = ui::stack(cx, move |cx| {
    vec![
        // Underlay (e.g. modal barrier)
        ui::container(cx, |_cx| Vec::new())
            .absolute()
            .inset(Space::N0)
            .into_element(cx),
        // Foreground content
        DialogContent::new(vec![]).ui().into_element(cx),
    ]
})
.into_element(cx);
```

Text (patchable):

```rust
let title = ui::text(cx, "Settings")
    .text_base()
    .font_semibold()
    .truncate()
    .into_element(cx);

let field_label = ui::label(cx, "Username").into_element(cx);
```

### Coverage Tracker (Update as we proceed)

Legend:

- `Chrome+Layout`: supports both style and layout fluent methods (`UiSupportsChrome + UiSupportsLayout`).
- `Layout-only`: supports only layout fluent methods (`UiSupportsLayout`); chrome methods are gated.
- `Patch-only`: supports `ui().build()`. Components that require extra args/closures may still support
  `ui().into_element(cx, ...)` via `fret-ui-shadcn` builder extension traits (re-exported by the prelude).
- `Pass-through`: supports `ui().into_element(cx)` but does not accept chrome/layout patches (no fluent style/layout methods; patch is ignored).

| Module | Type | Status | Notes |
| --- | --- | --- | --- |
| `button` | `Button` | Chrome+Layout |  |
| `alert` | `Alert` | Chrome+Layout |  |
| `badge` | `Badge` | Chrome+Layout |  |
| `kbd` | `Kbd` | Chrome+Layout |  |
| `breadcrumb` | `Breadcrumb` | Chrome+Layout |  |
| `empty` | `Empty` | Chrome+Layout |  |
| `combobox` | `Combobox` | Chrome+Layout |  |
| `checkbox` | `Checkbox` | Chrome+Layout |  |
| `radio_group` | `RadioGroup` | Chrome+Layout |  |
| `calendar` | `Calendar` | Chrome+Layout | Single-date variants are gated, including multi-month + locale + bounded month windows (`calendar-02/05/09/11/12`). |
| `calendar_range` | `CalendarRange` | Chrome+Layout | Range selection variants are gated, including multi-month (`calendar-07`). |
| `calendar_multiple` | `CalendarMultiple` | Chrome+Layout | Multiple selection variants are gated, including multi-month `required` + `max` policy (`calendar-03`). |
| `date_picker` | `DatePicker` | Chrome+Layout |  |
| `date_range_picker` | `DateRangePicker` | Chrome+Layout |  |
| `input` | `Input` | Chrome+Layout |  |
| `textarea` | `Textarea` | Chrome+Layout |  |
| `switch` | `Switch` | Chrome+Layout |  |
| `card` | `Card` | Chrome+Layout |  |
| `popover` | `PopoverContent` | Chrome+Layout |  |
| `popover` | `Popover` | Patch-only | `ui().into_element(cx, trigger, content)` (extra args) |
| `tooltip` | `TooltipContent` | Chrome+Layout |  |
| `dialog` | `DialogContent` | Chrome+Layout |  |
| `dialog` | `Dialog` | Patch-only | `ui().into_element(cx, trigger, content)` (extra args) |
| `alert_dialog` | `AlertDialogContent` | Chrome+Layout |  |
| `alert_dialog` | `AlertDialog` | Patch-only | `ui().into_element(cx, trigger, content)` (extra args) |
| `sheet` | `SheetContent` | Chrome+Layout |  |
| `sheet` | `Sheet` | Patch-only | `ui().into_element(cx, trigger, content)` (extra args) |
| `hover_card` | `HoverCardContent` | Chrome+Layout |  |
| `drawer` | `DrawerContent` | Chrome+Layout |  |
| `drawer` | `Drawer` | Patch-only | `ui().into_element(cx, trigger, content)` (extra args) |
| `dropdown_menu` | `DropdownMenu` | Patch-only | `ui().into_element(cx, trigger, entries)` (extra args) |
| `context_menu` | `ContextMenu` | Patch-only | `ui().into_element(cx, trigger, entries)` (extra args) |
| `menubar` | `Menubar` | Chrome+Layout |  |
| `select` | `Select` | Chrome+Layout | Layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (select-scrollable listbox width/height + option row height + scroll buttons). |
| `slider` | `Slider` | Chrome+Layout |  |
| `accordion` | `AccordionTrigger` | Chrome+Layout (Patch-only) | `into_element` requires root/value args |
| `accordion` | `AccordionContent` | Chrome+Layout (Patch-only) | Rendered via `Accordion` |
| `accordion` | `AccordionItem` | Chrome+Layout (Patch-only) | Rendered via `Accordion` |
| `accordion` | `Accordion` | Layout-only | Needs chrome support for full parity |
| `avatar` | `Avatar` | Chrome+Layout |  |
| `avatar` | `AvatarFallback` | Chrome+Layout |  |
| `avatar` | `AvatarImage` | Chrome+Layout |  |
| `progress` | `Progress` | Chrome+Layout |  |
| `skeleton` | `Skeleton` | Chrome+Layout |  |
| `tabs` | `Tabs` | Chrome+Layout |  |
| `tabs` | `TabsRoot` | Chrome+Layout |  |
| `toggle` | `Toggle` | Chrome+Layout |  |
| `toggle_group` | `ToggleGroup` | Chrome+Layout |  |
| `table` | `Table` | Chrome+Layout |  |
| `table` | `TableCell` | Chrome+Layout |  |
| `command` | `Command` | Chrome+Layout |  |
| `command` | `CommandPalette` | Chrome+Layout |  |
| `command` | `CommandInput` | Chrome+Layout |  |
| `input_group` | `InputGroup` | Chrome+Layout |  |
| `input_otp` | `InputOtp` | Chrome+Layout |  |
| `sidebar` | `Sidebar` | Chrome+Layout |  |
| `data_table` | `DataTable` | Chrome+Layout (Patch-only) | `ui().into_element(cx, data, data_revision, state, columns, ...)` (extra args) |
| `data_grid` | `DataGridElement` | Chrome+Layout (Patch-only) | Exported as `experimental::DataGridElement`; `ui().into_element(cx, rows_revision, cols_revision, ...)` (extra args) |
| `data_grid_canvas` | `DataGridCanvas` | Chrome+Layout (Patch-only) | `ui().into_element(cx, cell_text_at)` (extra args) |
| `collapsible` | `Collapsible` | Chrome+Layout (Patch-only) | `ui().into_element(cx, trigger, content)` (extra args) |
| `collapsible` | `CollapsibleContent` | Chrome+Layout |  |
| `field` | `Field` | Chrome+Layout |  |
| `item` | `Item` | Chrome+Layout |  |
| `pagination` | `Pagination` | Layout-only |  |
| `navigation_menu` | `NavigationMenu` | Layout-only |  |
| `scroll_area` | `ScrollArea` | Layout-only |  |
| `scroll_area` | `ScrollAreaRoot` | Layout-only |  |
| `resizable` | `ResizablePanelGroup` | Layout-only |  |
| `resizable` | `ResizablePanel` | Layout-only (Patch-only) | `into_element` is not public; used via panel group |
| `spinner` | `Spinner` | Layout-only |  |
| `tooltip` | `Tooltip` | Layout-only |  |
| `hover_card` | `HoverCard` | Layout-only |  |
| `dialog` | `DialogClose` | Chrome+Layout |  |
| `alert_dialog` | `AlertDialogTrigger` | Pass-through | Non-visual wrapper; apply chrome/layout patches on the child trigger element (e.g. `Button::ui()`) to avoid extra container nodes |
| `drawer` | `DrawerClose` | Chrome+Layout |  |

Additional pass-through subcomponents also opt into `ui()` (not tracked individually in the table):

- Alert/Dialog/Sheet: `*Header/*Footer/*Title/*Description` (+ `AlertDialogAction/AlertDialogCancel`).
- Card: `CardHeader/CardContent/CardFooter/CardTitle/CardDescription`.
- Command: `CommandEmpty/CommandList/CommandShortcut` (plus `CommandDialog` as Patch-only).
- Field/Item/Pagination: `FieldSet/FieldLegend/FieldLabel/FieldTitle/FieldDescription/FieldError/FieldSeparator/FieldGroup/FieldContent`, `ItemGroup/ItemHeader/ItemContent/ItemTitle/ItemDescription/ItemMedia/ItemActions/ItemFooter/ItemSeparator`, `PaginationContent/PaginationItem/PaginationLink/PaginationPrevious/PaginationNext/PaginationEllipsis`.
- Data table controls: `DataTableGlobalFilterInput/DataTableViewOptions`.
- Sonner: `Toaster`.
- Table: `TableHeader/TableBody/TableFooter/TableRow/TableHead/TableCaption`.
- Wrappers: `PopoverTrigger/PopoverAnchor/TooltipTrigger/TooltipAnchor/HoverCardTrigger/HoverCardAnchor/DrawerTrigger`.

## shadcn/ui v4 Registry Baseline

The upstream reference in `repo-ref/ui` defines 54 `registry:ui` components (`repo-ref/ui/apps/v4/registry.json`).

Status below uses Rust module naming (hyphenated names normalized to `_`).

Audit column is a lightweight review marker for shadcn parity against `repo-ref/ui` docs/examples:
`Unreviewed` -> `In review` -> `Pass` (or `Defer`/`Skip` when applicable).

| Registry name | Rust module | Status | Audit | Notes |
| --- | --- | --- | --- | --- |
| accordion | `accordion` | Present | In review | Selection model drives open/close; measured-height motion is wired; shadcn-web layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`accordion-demo`, light+dark) |
| alert | `alert` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`alert-demo`) |
| alert-dialog | `alert_dialog` | Present | In review | Audit: `docs/audits/shadcn-alert-dialog.md`; shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` |
| aspect-ratio | `aspect_ratio` | Present | Unreviewed |  |
| avatar | `avatar` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`avatar-demo`) |
| badge | `badge` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`badge-demo`) |
| breadcrumb | `breadcrumb` | Present | Unreviewed |  |
| button | `button` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`button-demo`) |
| button-group | `button_group` | Present | Unreviewed | Thin wrapper over `toggle_group` styling |
| calendar | `calendar` | Present | Unreviewed | Headless month grid lives in `fret-ui-kit` (`headless::calendar`); UI surface lives in `fret-ui-shadcn` |
| card | `card` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`card-with-form`) |
| carousel | `carousel` | Defer | Unreviewed | Not editor-critical |
| chart | `chart` | Defer | Unreviewed | Not editor-critical |
| checkbox | `checkbox` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`checkbox-demo`) |
| collapsible | `collapsible` | Present | Unreviewed | Headless open/close + a11y semantics |
| command | `command` | Present | In review | `CommandPalette` provides cmdk-style active-descendant navigation + filtering/scoring (value + keywords), plus group/separator/empty + checkmark/shortcut; audit: `docs/audits/shadcn-command.md`; shadcn-web gates: `web_vs_fret_command_dialog_*` + `web_vs_fret_combobox_demo_*` |
| context-menu | `context_menu` | Present | In review | Right click + (macOS) ctrl-click + Shift+F10; anchors to click position for web/Radix parity; audit: `docs/audits/shadcn-context-menu.md`; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`, `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` |
| dialog | `dialog` | Present | In review | Audit: `docs/audits/shadcn-dialog.md`; shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` |
| drawer | `drawer` | Present | Unreviewed | `sheet` facade (defaults to bottom); overlay policy |
| dropdown-menu | `dropdown_menu` | Present | In review | Menu navigation + typeahead + dismissible popover infra (ADR 0074); now includes `Label`/`Group`/`Shortcut` + destructive items; audit: `docs/audits/shadcn-dropdown-menu.md`; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` |
| empty | `empty` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`empty-avatar`, `empty-avatar-group`) |
| field | `field` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`field-*`) |
| form | `form` | Present | Unreviewed |  |
| hover-card | `hover_card` | Present | In review | Hover intent + anchored placement; supports custom anchor via `HoverCard::anchor_element(...)`; audit: `docs/audits/shadcn-hover-card.md` |
| input | `input` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`input-demo`) |
| input-group | `input_group` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`input-group-dropdown`) |
| input-otp | `input_otp` | Present | Unreviewed | Slots rendered over transparent `TextInput`; digits-only clamping; a11y TBD |
| item | `item` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`item-avatar`) |
| kbd | `kbd` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`kbd-demo`) |
| label | `label` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`label-demo`) |
| menubar | `menubar` | Present | In review | Click-to-open; hover switching; audit: `docs/audits/shadcn-menubar.md`; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` |
| native-select | `native_select` | Defer | Unreviewed | Can map to `select` + platform-native later |
| navigation-menu | `navigation_menu` | Present | In review | `viewport=true` mobile geometry (root-anchored viewport) + `viewport=false` popover placement; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (navigation-menu-demo.* open variants, incl. `components-mobile`) |
| pagination | `pagination` | Present | Unreviewed |  |
| popover | `popover` | Present | Pass | Anchored placement + click-through outside press dismissal (ADR 0069); non-modal (no focus trap); audit: `docs/audits/shadcn-popover.md` |
| progress | `progress` | Present | In review | shadcn-web gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` + `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`progress-demo`) |
| radio-group | `radio_group` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`radio-group-demo`) |
| resizable | `resizable` | Present | Unreviewed | Runtime-owned drag + layout; multi-panel group; a11y TBD |
| scroll-area | `scroll_area` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`scroll-area-*`) |
| select | `select` | Present | In review | Item-aligned + popper placement (side/align/offsets); scroll buttons; item-aligned scroll-button reposition + post-position “scroll selected into view” behavior for Radix parity; shadcn-web gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_select_demo_overlay_placement_matches`, `web_vs_fret_select_demo_open_option_metrics_match`, `web_vs_fret_select_scrollable_listbox_width_matches`, `web_vs_fret_select_scrollable_listbox_option_insets_match`, `web_vs_fret_select_scrollable_listbox_option_height_matches`, `web_vs_fret_select_scrollable_scroll_button_height_matches`, `web_vs_fret_select_scrollable_viewport_insets_match`, plus small/tiny viewport variants); audit: `docs/audits/shadcn-select.md` |
| separator | `separator` | Present | In review | shadcn-web gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`separator-demo`) |
| sheet | `sheet` | Present | In review | Audit: `docs/audits/shadcn-sheet.md`; shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` |
| sidebar | `sidebar` | Present | Unreviewed |  |
| skeleton | `skeleton` | Present | Unreviewed |  |
| slider | `slider` | Present | In review | Declarative composition over primitives (PointerRegion hooks + bounds); a11y TBD; shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`slider-demo`) |
| sonner | `sonner` | Present | In review | Toast store + overlay layer + timers; upsert-by-id; swipe-to-dismiss; hover pause/resume; max-toasts eviction; action/cancel; manual promise handle; audit: `docs/audits/shadcn-sonner.md` |
| spinner | `spinner` | Present | Unreviewed |  |
| switch | `switch` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`switch-demo`) |
| table | `table` | Present | Unreviewed |  |
| tabs | `tabs` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`tabs-demo`) |
| textarea | `textarea` | Present | In review | Wrapper over declarative `TextArea` (runtime `TextArea` engine); a11y TBD; shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`textarea-demo`) |
| toggle | `toggle` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`toggle-demo`) |
| toggle-group | `toggle_group` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`toggle-group-demo`) |
| tooltip | `tooltip` | Present | In review | Hover intent + placement; rendered via overlay root (not clipped); audit: `docs/audits/shadcn-tooltip.md` |

## Non-registry surfaces

These are shadcn-style surfaces referenced by docs/demos but not part of the `registry:ui` baseline:

| Surface | Rust module | Status | Audit | Notes |
| --- | --- | --- | --- | --- |
| combobox | `combobox` | Present | In review | Implemented as `Popover` + `CommandPalette` recipe; audit: `docs/audits/shadcn-combobox.md`; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (web listbox height, open-state goldens: `combobox-demo.open.json`, `combobox-demo.vp375x320.open.json`) |
| date picker | `date_picker` | Present | Unreviewed | `Popover` + `Calendar` recipe; v1 is single-date selection |
| data table / datagrid | `data_table` | Present | Unreviewed | Extension surface (not a `registry:ui` item upstream); `DataGrid` defaults to canvas; use `experimental::DataGridElement` for rich per-cell UI |
| toast | `toast` | Skip | Unreviewed | Upstream `toast` is deprecated in favor of `sonner`; this repo ships `sonner` |
| typography | `typography` | Skip | Unreviewed | Upstream typography page is docs-only and not shipped as a component |

Notes:
- "Present" means a declarative module exists and compiles; it may still be below the "Definition of Done" parity bar (keyboard/APG, a11y checklist, tests).
- Most "Missing" entries were previously implemented as retained widgets and intentionally deleted under the declarative-only boundary. They should come back as declarative components backed by `fret-ui-kit` infra + `fret-ui` mechanisms.
- `data_table` is not a `registry:ui` item upstream; treat it as an extension (API may evolve as the TanStack-aligned headless engine is integrated).

## Recommended Order (Near-term)

1. `fret-ui-kit`: declarative primitives and headless helpers used by everything (pressable, list/tree, focus).
2. `fret-ui-shadcn`: primitives first (`Button` -> `Input/Textarea` -> `Checkbox/Switch/RadioGroup` -> `Tabs/Accordion`).
3. Overlays (policy lives in components, mechanism lives in runtime): `Popover` -> `Dialog` -> `Tooltip/HoverCard` -> menus -> `Toast`.
4. Complex components: calendar/date picker, navigation menu, data table (virtualization + selection).

## Definition of Done (Per Component)

- API: shadcn-style public names (`UpperCamelCase` types like `HoverCardTrigger`), declarative-only authoring.
- Behavior: keyboard + focus outcomes match APG/Radix targets (see `docs/reference-stack-ui-behavior.md`).
- A11y: correct semantics roles/flags (ADR 0033), and passes the manual checks in `docs/a11y-acceptance-checklist.md`.
- Tests: add nextest unit/contract tests in the owning crate; keep `cargo nextest run --workspace` green.

## How to Reference the Previous Implementation

- Inspect a file from before deletion: `git show <rev>:<path>`
- Compare old/new behavior: `git diff <rev1>..<rev2> -- <path>`
- Trace changes: `git log -- <path>`

## Infrastructure Backlog (components-ui / runtime)

The goal is to keep `fret-ui-shadcn` mostly "composition + styling", and put reusable mechanisms/state in `fret-ui` / `fret-ui-kit`.

**Overlay stack (highest leverage)**
- `fret-ui` (mechanism): multi-root rendering per window, overlay layer install/uninstall, outside-press observers, modal barrier semantics, focus restore primitives.
- `fret-ui-kit` (policy): `WindowOverlays`-style request queues and rendering for popovers/menus/dialogs/toasts; consistent focus-restore rules (ADR 0069).

**Headless state machines**
- Hover intent (tooltip/hover-card delays), menu navigation (typeahead + roving), focus trapping for dialogs/sheets, and richer typeahead buffer (prefix match with timeout).

**Declarative primitives (Tailwind-ish building blocks)**
- `separator`, `scroll_area`, `textarea` (wrapper over runtime `TextArea`), `slider`, `resizable` panels/splitters.
- Input "slots" patterns: `input_group` (leading/trailing icons, clear buttons), `input_otp` helpers.

**Notifications**
- `sonner`/toast: global service API + per-window overlay root + timers + action dispatch.

**Command palette (`command` / cmdk-style)**
- Component surface belongs in `fret-ui-shadcn` (shadcn taxonomy), but the heavy lifting should live in `fret-ui-kit`:
  - headless filtering/scoring + match highlighting ranges
  - keyboard navigation (up/down/home/end, typeahead, disabled skipping)
  - optional virtualization integration
- Potential runtime/a11y gaps to track:
  - Listbox semantics are supported (`SemanticsRole::{ListBox, ListBoxOption}`) and mapped to AccessKit roles; prefer this for cmdk/select-style surfaces.
  - Active-descendant semantics are supported (`SemanticsNode.active_descendant`) and used by `CommandPalette` to keep focus in the input while announcing the active result.
  - Virtualized a11y contract is still evolving; avoid virtualization for v1 unless necessary, or define an AT-facing mirror strategy.

## Planned Infra Modules (Concrete)

Intended new building blocks (names tentative):

- `ecosystem/fret-ui-kit/src/headless/hover_intent.rs` (tooltip/hover-card delays + cancellation)
- `ecosystem/fret-ui-kit/src/headless/menu_nav.rs` (arrow key navigation + typeahead buffer + disabled skipping)
- `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs` (dialog/sheet focus trap helpers + restore hooks)
- `ecosystem/fret-ui-shadcn/src/separator.rs` (simple visual + semantics)
- `ecosystem/fret-ui-shadcn/src/scroll_area.rs` (Scroll + scrollbar styling wrapper)
- `ecosystem/fret-ui-shadcn/src/textarea.rs` (runtime `TextArea` chrome wrapper)
- `ecosystem/fret-ui-shadcn/src/slider.rs` (pointer/keyboard input; a11y TBD; uses `fret-ui-kit` headless slider)
- Extend `ecosystem/fret-ui-kit/src/window_overlays/*` with: tooltip layer, menu layer, dialog/sheet layer, toast layer

Cross-cutting a11y constraint to keep in mind:

- Roving-focus "items" often should be *not* in Tab traversal, but still AT-focusable/activatable; ensure semantics focusability is not accidentally tied to Tab-stop (see `Pressable` semantics behavior).

## Reference: gpui-component Layering (Upstream Inspiration)

`repo-ref/gpui-component` is a useful comparison point:

- GPUI provides mechanisms like `DismissEvent`, `anchored(...)` placement, focus handles, and deferred overlays.
- gpui-component implements policy and styling at the component layer (`Popover::overlay_closable`, tooltip styling, input popovers, etc).

This matches Fret's intended split: `fret-ui` as mechanism; `fret-ui-kit`/`fret-ui-shadcn` as policy + composition.

## Tracking Table (Update as work proceeds)

| Area | Component | Status | Owner crate | A11y | Tests | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Boundary | Retained API hidden | Done | `fret-ui` | - | - | `widget` + `create_node` are crate-private |
| Infra | Declarative tree | Done | `fret-ui-kit` | Partial | Partial | Expand with roving focus + typeahead helpers |
| Primitives | Button | Present | `fret-ui-shadcn` | Partial | Not started | Style parity + a11y checklist still pending |
| Primitives | Input | Present | `fret-ui-shadcn` | Partial | Not started | Uses runtime `TextInput` semantics + theming |
| Overlays | Select | Present | `fret-ui-shadcn` | Partial | Partial | Uses `fret-ui-kit/window_overlays` dismissible popover |
