# Shadcn Declarative Implementation Progress


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- gpui-component: https://github.com/longbridge/gpui-component
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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

- Keys referenced by tests: `578/578` (`100%`, tracked-only, normalized `.open`) as of 2026-02-03
  - Note: this is **breadth coverage** (every golden key is gated somewhere), not full 1:1 parity across
  viewports, DPIs, fonts, and interaction state machines.

## Near-Term Roadmap (shadcn-web v4/new-york-v4)

Strategy: fill **breadth first** (one canonical viewport per page), then add a small set of targeted
viewport/DPI stress variants for the highest-risk families (menus, listboxes, calendars, typography).

Workstream notes (implementation-oriented; not contracts):

- `docs/workstreams/shadcn-web-goldens-v4/shadcn-web-goldens-v4.md`
- `docs/workstreams/shadcn-web-goldens-v4/shadcn-web-goldens-v4-todo.md`

Decision note (scope + sequencing):

- We do **not** wait for 100% component breadth before adding resolution/viewport stress variants.
  For overlay families (menus/listboxes/popovers/dialogs) and typography, the “constrained viewport” gates
  are part of the *first* meaningful parity check because they validate max-height clamping, scroll buttons,
  truncation/wrap behavior, and “menu height” as a styling outcome.
- We *do* keep the **DPI/font-metrics** dimension small until breadth is higher, because it tends to be
  more sensitive and is best added once core geometry is stable.

P0 (next):

- Consolidate a **depth checklist** for interaction states (hover/focus/active/open) and constrained
  viewports so “what is gated” is explicit and auditable.
- Add scripted interaction variants (hover/focus) for high-risk overlay families where geometry alone
  does not catch regressions (e.g. hovered item chrome, focus ring, active/pressed states).
  - Current: `highlight-first` + `focus-first` variants are gated for Menubar/DropdownMenu and Select listboxes.
  - Next: extend scripted variants to pressed/disabled and more overlay families (NavigationMenu, Popover, Tooltip).
- Keep expanding constrained viewport variants only when they materially increase signal (max-height clamping,
  scroll buttons, truncation/wrap).

P2:

- Add a small, targeted DPI/font-metrics matrix once the interaction-state gates are stable (typography +
  menus/listboxes first).

## Scope

- `ecosystem/fret-ui-shadcn`: shadcn/ui v4 naming + taxonomy surface (recipes).
- `ecosystem/fret-ui-kit`: reusable infra (tokens/recipes/headless helpers).
- `crates/fret-ui`: runtime substrate (contracts/mechanisms only).

Related (out-of-scope for v4 parity tracking):

- Shadcn-styled “blocks/recipes” that are not part of the v4 taxonomy live under
  `fret-ui-shadcn::extras` and are tracked separately in `docs/workstreams/shadcn-extras/shadcn-extras.md`.

## Layering & Ownership

This repo intentionally splits responsibilities across three layers (similar to Tailwind + headless + Radix/RSC composition, but in Rust):

- `fret-ui` (**mechanisms/contracts**): element tree, hit-test, focus, semantics/a11y, overlay roots/layers, outside-press observers, layout, paint.
- `fret-ui-kit` (**design-system + infra**, Tailwind-ish): token-driven styling (`Theme` keys + refinements), reusable declarative helpers (`scroll`, `text_field`, etc), and headless state machines (`roving_focus`, hover intent, menu navigation).
- `fret-ui-shadcn` (**taxonomy + recipes**): shadcn/ui v4 naming surface and component composition; no retained widgets, no renderer/platform deps.

App/editor-specific composition belongs in `fret-editor` and ecosystem app layers (e.g. `fret-bootstrap`) (app toolbars, menu bars, command palette wiring).

### Interaction Policy (Action Hooks)

Cross-cutting interaction policies (toggle models, close overlays, selection writes, "dismiss on escape/outside press", etc.) are *component-owned*:

- `fret-ui` provides hook plumbing (`on_activate`, `on_dismiss_request`) as a mechanism-only substrate (ADR 0074).
- The `fret` app-facing facade now keeps the lower-level activation glue under `cx.actions()`
  (`action`, `action_payload`, `listen`) and teaches the default
  app lane through `widget.action(act::Save)`, `widget.action_payload(act::Remove, payload)`, and
  `widget.listen(|host, acx| { ... })` for activation-only surfaces via an explicit
  `use fret::app::AppActivateExt as _;` import on
  `fret::app::AppActivateSurface` / `AppActivateExt`.
  Shadcn widgets that already expose native `.action(...)` / `.action_payload(...)` slots or a
  widget-owned `.on_activate(...)` hook (`Button`, `SidebarMenuButton`, `Badge`,
  `extras::{BannerAction, BannerClose, Ticker}`) now stay on that native surface and are no
  longer part of the `AppActivateSurface` bridge list.
  The first-party `badge/link.rs` example now overrides link launching through
  `Badge::on_activate(...)` instead of reopening the activation bridge just to keep diagnostics
  runs side-effect free.
  Extracted `UiCx` helper functions now get the same grouped action/data surface through
  `UiCxActionsExt` / `UiCxDataExt`, and AI widgets such as `WorkflowControlsButton`,
  `MessageAction`, `ArtifactAction`, `ArtifactClose`, `CheckpointTrigger`,
  `ConversationDownload`, `PromptInputButton`, `WebPreviewNavigationButton`, and
  `ConfirmationAction` now stay on their native `.action(...)` slots or widget-owned
  `.on_activate(...)` hooks instead of extending the bridge table. First-party UI Gallery button
  and sidebar snippets now also use `UiCxActionsExt` plus widget-owned `.on_activate(...)` for
  local listeners instead of importing `AppActivateExt`. As of 2026-03-16, the first-party
  default widget bridge table is intentionally empty.
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
- ADR: `docs/adr/0160-unified-authoring-builder-surface-v1.md`
- Coverage audit note: `docs/workstreams/authoring-ergonomics-fluent-builder/authoring-ergonomics-fluent-builder.md`
- TODO tracker: `docs/workstreams/authoring-ergonomics-fluent-builder/authoring-ergonomics-fluent-builder-todo.md`

### Status

- `fret-ui-kit`: `ui()` is available for any type that implements `UiPatchTarget`.
- `fret-ui-shadcn`: coverage is incremental via `ui_ext/*` (no component internals required unless a
  component does not yet support chrome/layout refinements).
- Dev note (Windows worktrees): if incremental builds pick up stale artifacts from another worktree,
  run `cargo clean -p fret-ui-kit -p fret-ui-shadcn` (or set a per-worktree `CARGO_TARGET_DIR`).

### Authoring Golden Path

Recommended imports for app code:

```rust
use fret_ui_shadcn::{facade as shadcn, prelude::*};
```

### Current authoring-surface closeout status (2026-03-16)

The discovery-lane closure is now structurally landed. The main remaining shadcn work is
**docs/gate hygiene on top of that closure**, not another public-surface redesign.

- ordinary component/app authors should make one first-contact choice for component families:
  `use fret_ui_shadcn::{facade as shadcn, prelude::*};`
- `raw::*` remains the only explicit component-family escape hatch
- `app::*` and `themes::*` are setup lanes, not peer discovery lanes
- `advanced::*` is an implementation/debug/source-alignment lane, not a competing default
- component-family root modules are now crate-private instead of public/doc-hidden residue

Release-blocking closeout consequences:

- stale first-party docs/examples that still make any non-facade lane feel like a peer path
  should be treated as authoring-surface debt even if the underlying component implementation is
  correct
- source-policy tests are guardrails, not the desired end-state; the goal is to remove the need
  for humans to mentally sort multiple peer discovery lanes in the first place
- broader ecosystem trait/sugar work should read from this settled discovery story rather than
  compete with it

### Extension-contract sequencing (2026-03-16)

The next ecosystem-trait pass should be **sequenced after**, not during, the current authoring
surface closeout.

- start gate: do not budget new public integration traits while the canonical todo/default-path
  docs and templates still need wording churn for the default lane
- every new ecosystem contract should declare its tier up front:
  - app-level install/setup/integration
  - component-level composition/adaptor
  - explicit advanced/raw hook
- non-goal: do not widen `fret::app::prelude::*`, `fret::component::prelude::*`, or the flat
  `fret_ui_shadcn` discovery surface in the name of extensibility
- first-party UI Gallery snippets/pages remain the teaching-source checkpoint for default shadcn
  authoring; if a proposed trait or helper does not make that surface clearer, it likely does not
  belong on the default lane
- when a family intentionally needs both a compact default lane and a raw/parts lane, ship both
  exemplars and lock them with a source-policy gate instead of relying on docs prose alone

Guidelines:

- Prefer `ui()` for all authoring (chrome + layout + debug helpers).
- Prefer composing shadcn components over introducing new wrapper nodes.
- First-party app surfaces should prefer `use fret_ui_shadcn::{facade as shadcn, prelude::*};`;
  `apps/fret-examples` and the curated `apps/fret-ui-gallery` snippet batches are now gated this way.
- First-party non-demo ecosystem crates should avoid the flat `fret_ui_shadcn::{Button, ...}` root
  lane as well; `ecosystem/fret-ui-ai/src/elements/**` now stays on explicit `facade::*` /
  documented `raw::*` imports, with both a crate-local source test and the repo-level
  `tools/gate_fret_ui_ai_curated_shadcn_surfaces.py` check enforcing that rule.
- For widgets that already expose stable action slots, prefer `.action(...)` /
  `.action_payload(...)`; curated first-party button and action-capable UI Gallery slices are now
  policy-gated away from legacy `.on_click(...)`.
- For activation-only surfaces rendered inside a `fret` app shell, prefer
  `use fret::app::AppActivateExt as _;` plus `.action(act::Save)` /
  `.action_payload(act::Remove, payload)` / `.listen(...)`. Do not reopen raw `.on_activate(...)`
  on first-party snippet surfaces unless the example is intentionally documenting a raw/advanced
  seam.
- Non-curated seams should stay explicit in app code: use `fret_ui_shadcn::advanced::*` for
  environment / `UiServices` hooks, and use `shadcn::raw::*` only for the documented escape-hatch
  lanes (`typography` prose helpers, `extras`, breadcrumb primitives, the experimental
  `DataGridElement` family, low-level icon helpers, and module-local advanced enums/styles such as
  `raw::{button, calendar, context_menu, dropdown_menu, kbd, menubar, select, switch, tabs,
  toggle_group}::*`) instead of importing `fret_ui_shadcn::*` directly.
- Theme presets are no longer a parallel root lane either: first-party code should use
  `shadcn::themes::*` (or `fret_ui_shadcn::facade::themes::*` in non-aliased contexts), not the
  historical `fret_ui_shadcn::shadcn_themes::*` path.
- Treat the flat `fret_ui_shadcn::*` crate root as non-authoring glue only; component-family
  exports now live on `facade as shadcn`, with `raw::*` as the explicit escape hatch.
- The curated `prelude` and crate-internal recipe/helper glue now also source their
  authoring-critical names from explicit module/facade paths instead of hidden flat root
  reexports; component families, direction utilities, themes, and authoring glue now all route
  through explicit facade/prelude/raw lanes, and internal recipe code now imports icon helpers
  directly from `fret_ui_kit::declarative::icon` rather than relying on a flat-root shim.
- The current first-party source-policy tests that ban root-style imports are evidence of remaining
  public-surface duplication, not proof that the current lane budget is ideal forever.
  The next cleanup step is to keep deleting stale teaching copy and to keep the curated
  facade/raw/app/themes/advanced budget stable, not to normalize any new peer discovery lane.
- `StyledExt` exists in `fret-ui-kit` but is intentionally not part of the shadcn prelude to avoid splitting the
  ecosystem into competing patterns.

### Authoring surface alignment rules

This tracker follows the repo-wide authoring reset and the focused conversion-surface follow-up:

- app-facing starter docs, cookbook snippets, and UI Gallery teaching samples should prefer `Ui`,
  `UiChild`, and `UiCx` on the app lane rather than raw `AnyElement` or legacy split conversion
  trait names,
- reusable generic helpers in `fret-ui-shadcn` / `fret-ui-kit` should converge on the unified
  component conversion trait tracked in
  `docs/workstreams/into-element-surface-fearless-refactor-v1/DESIGN.md`,
- advanced/manual-assembly reusable helpers should prefer `IntoUiElement<H>` directly, including
  heterogeneous child-collection and single-child builder seams,
- first-party UI Gallery page/document scaffolds are part of the shadcn teaching surface too, so
  they should prefer typed wrapper entry points such as `DocSection::build(...)` instead of
  eagerly landing previews into `AnyElement` before the scaffold layer,
- shadcn opt-in authoring glue in `ecosystem/fret-ui-shadcn/src/ui_ext/` now also lands through
  `IntoUiElement<H>` directly, so adapter macros do not re-teach `UiIntoElement`,
- shadcn `ui_builder_ext/*` helper closures now also accept values that implement
  `IntoUiElement<H>`, so trigger/content/cell builders do not have to pre-land into `AnyElement`,
- `UiHostBoundIntoElement`, `UiBuilderHostBoundIntoElementExt`, and `UiChildIntoElement` are
  already deleted from code and should not be taught on first-party shadcn surfaces,
- authoring-critical first-party family lanes should stay reachable from the hidden compatibility
  root plus `fret_ui_shadcn::facade` as an implementation constraint, but the curated `facade`
  remains the only default teaching/discovery lane; source-policy tests now guard the selected
  `Select` / `Combobox` / `ComboboxChips` / `Command` / `NavigationMenu` / `Pagination` exports so
  compact/default examples do not depend on root-only names,
- keep `AnyElement` explicit only for justified raw seams such as diagnostics, overlay/controller
  internals, or low-level helper plumbing.
- Current inventoried raw/bridge helpers on the shadcn lane are intentionally small:
  `kbd.rs::kbd_icon(...)`,
  `text_edit_context_menu.rs::{text_edit_context_menu,text_selection_context_menu,text_edit_context_menu_controllable,text_selection_context_menu_controllable}`.
- Those text-edit context-menu helpers are a deliberate final wrapper seam, not a pending typed
  promotion: they accept typed triggers, but the helper itself owns the root `ContextMenu::build`
  landing call plus the fixed command entry set.
- Combobox anchor overrides now reuse the generic overlay builder path
  `PopoverAnchor::build(...).into_anchor(cx)` instead of keeping a combobox-specific raw alias.
- `tooltip.rs::TooltipContent::{build,text}(...)` now stay on the typed lane: `build(...)`
  returns `TooltipContent`, `text(...)` returns `impl IntoUiElement<H>`, and first-party tooltip
  snippets/gallery previews now teach that builder path instead of eager `AnyElement` helpers.
- `state.rs::{use_selector_badge,query_status_badge}` now return typed `Badge` values again, and
  `query_error_alert(...)` now returns `Option<Alert>` so the conditional alert helper no longer
  forces a root-level landed value either.
- Remaining legacy module-local helpers that still return `AnyElement`
  are now cleared on the shadcn lane; module-local root convenience helpers no longer add extra
  `-> AnyElement` surface beyond the explicitly documented raw/bridge seams above.
- Selected public `Model<_>` seams that remain intentionally visible are now tracked by
  `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/SHADCN_RAW_MODEL_ALLOWLIST_AUDIT_2026-03-19.md`
  and guarded by `surface_policy_tests.rs`; new raw-model seams on managed/source-aligned lanes
  should update that audit and gate together instead of silently widening the default authoring
  surface.
- That allowlist is for explicit controlled roots, controller/output handoff, and a small number
  of specialized source-aligned seams. It is not a license to reintroduce raw `Model<_>`
  constructors on compact/default shadcn paths that already have typed wrappers or narrow bridge
  traits.
- Follow-up shrink now also covers several previously-audited specialized constructors:
  `InputGroup::new(...)`, `SidebarInput::new(...)`, `CalendarHijri::new(...)`, and
  `extras::Rating::new(...)` now use narrow bridge traits instead of forcing raw `Model<_>`
  handles on the public constructor surface.

This matters beyond local helper signatures: when a docs/page scaffold turns a still-typed preview
into `AnyElement` too early, the exemplar surface starts teaching the wrong authoring pattern even
if the underlying component recipe is already aligned.

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
same fluent `ui()` vocabulary as components). For shrink-wrapping rows/columns, prefer `ui::h_row` / `ui::v_stack`.

Horizontal row:

```rust
let row = ui::h_flex(move |cx| {
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
let col = ui::v_flex(move |cx| {
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
let overlay = ui::stack(move |cx| {
    vec![
        // Underlay (e.g. modal barrier)
        ui::container(|_cx| Vec::new())
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
let title = ui::text("Settings")
    .text_base()
    .font_semibold()
    .truncate()
    .into_element(cx);

let field_label = ui::label("Username").into_element(cx);
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
| `calendar` | `Calendar` | Chrome+Layout |  |
| `calendar_range` | `CalendarRange` | Chrome+Layout |  |
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
- Data table controls: `DataTableGlobalFilterInput/DataTableViewOptions` now follow the same narrow bridge direction as `Input` / `Toggle`-style roots (`IntoTextValueModel` / `IntoBoolModel`) instead of requiring raw `Model<_>` handles on their compact constructors.
- Sonner: `Toaster`.
- Table: `TableHeader/TableBody/TableFooter/TableRow/TableHead/TableCaption`.
- Wrappers: `PopoverTrigger/PopoverAnchor/TooltipTrigger/TooltipAnchor/HoverCardTrigger/HoverCardAnchor/DrawerTrigger`.


## Alignment Queue (2026-03)

Use this table to sequence **which component to align next**. The registry-status tables below remain
canonical for breadth/presence/audit state; this queue adds priority, risk class, and the
"default-style ownership" lens so we do not mistake page/container constraints for recipe defaults.

| Component | Rust module | Priority | Risk class | Primary upstream truth | Likely owner layer | Default style owner | Recommended first gate | Why now |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Calendar | `calendar` | P0 | Field-family grid + selection + responsive layout | shadcn docs/examples + Base UI + APG | `fret-ui-shadcn` + `fret-ui-kit` | Mixed | Focused layout/a11y test + intrinsic-width unit test + gallery page-alignment evidence | Reviewed 2026-03: recipe defaults are correct (`bg/p` + day chrome), while `rounded-lg border` and page/demo shell width remain caller-owned |
| Date Picker | `date_picker` | P0 | Popover + field + calendar composition | shadcn docs/examples + Radix + Base UI | `fret-ui-shadcn` + `fret-ui-kit` | Mixed | Trigger-width ownership unit test + gallery page-alignment evidence | Reviewed 2026-03: trigger width is caller-owned; compact builder keeps recipe-owned trigger chrome and `PopoverContent` `w-auto p-0` |
| Navigation Menu | `navigation_menu` | P0 | Overlay viewport + responsive sizing + focus routing | Radix + shadcn docs/examples | `fret-ui-shadcn` + `fret-ui-kit` | Mixed | Overlay placement/scripted open-state gate | Already complex; easy to regress viewport-owned vs caller-owned sizing |
| Sidebar | `sidebar` | P0 | Container negotiation + responsive layout ownership | shadcn docs/examples | `fret-ui-shadcn` | Mixed | Provider-width unit tests + gallery page-alignment evidence | Reviewed 2026-03: `SidebarProvider::width/_icon/_mobile` is the primary width owner, theme tokens remain recipe fallback, and page shell flex/height constraints stay caller-owned |
| Drawer | `drawer` | P0 | Sheet-like overlay policy + responsive presentation | Radix semantics + shadcn composition | `fret-ui-shadcn` + `fret-ui-kit` | Mixed | Overlay open/close gate + geometry invariant | Reviewed 2026-03: side/height defaults already matched upstream, `DrawerClose::from_scope().build(cx, child)` covers the composable close-child gap, and `children([DrawerPart::...])` now closes the primary root authoring drift without moving policy into mechanism |
| Resizable | `resizable` | P0 | Drag routing + layout + a11y | shadcn examples + Base UI | `fret-ui` + `fret-ui-shadcn` | Mixed | Focused drag/layout test + gallery page-alignment gate | Reviewed 2026-03: `ResizablePanelGroup` owns fill sizing and handle chrome, while outer border/rounded shells stay caller-owned; no clear implementation drift found yet |
| Input OTP | `input_otp` | P0 | Text engine + slot metrics + focus choreography | shadcn docs/examples + Base UI | `fret-ui-shadcn` + runtime text surfaces | Recipe-heavy | Focus/input invariant test + control-id semantics test + gallery form evidence | Reviewed 2026-03: default gap/slot chrome ownership already matched upstream, separator semantics and `control_id` form association were aligned, and the gallery now mirrors the docs path more closely (`About` / `Usage` / `API Reference`) while teaching the upstream-shaped parts bridge directly in `Usage` and keeping `Compact Builder` as the explicit Fret shorthand follow-up |
| Form | `form` | P1 | Public-surface drift + field ownership | shadcn docs/examples | `fret-ui-shadcn` | Caller-heavy | Gallery usage anchor + focused unit test | Reviewed 2026-03: `FormControl` is slot-like for one child, while `FormField` keeps Fret-native helper ergonomics and leaves width/layout negotiation caller-owned |
| Field | `field` | P1 | Description/error ownership + width negotiation | shadcn docs/examples | `fret-ui-shadcn` + `fret-ui-kit` | Mixed | Layout invariant test | Reviewed 2026-03: `FieldDescription` remains recipe-owned `w-full`, while plain `FieldLabel`/`FieldTitle` keep intrinsic-width defaults and full-width comes from `Field` orientation or wrapped-card label composition |
| Input Group | `input_group` | P1 | Slot stretch + icon/affordance sizing | shadcn docs/examples + Base UI | `fret-ui-shadcn` | Mixed | Root-width override unit test + gallery page-alignment evidence | Reviewed 2026-03: root `w-full min-w-0` is recipe-owned because upstream source defines it, caller layout refinements still override explicit width choices, and first-party snippets now treat `InputGroup::new(model)` as the ergonomic lane while explicit parts remain the docs-parity lane |
| Table | `table` | P1 | Caller-owned width/overflow + semantics | shadcn docs/examples | `fret-ui-shadcn` | Caller-heavy | Existing width-override unit test + gallery page evidence | Reviewed 2026-03: root recipe correctly owns `w-full overflow-x-auto`; the remaining drift stayed in recipe/docs space rather than mechanism; `table_head_children(...)` / `table_caption_children(...)` now cover the missing composable `th` / `caption` lanes; checkbox-column padding plus mixed-height body-row centering now both land in `TableHead` / `TableCell`; and the gallery mirrors the upstream docs path through `API Reference` before a focused `Children (Fret)` follow-up |
| Pagination | `pagination` | P1 | Inline layout + truncation + responsive ownership | shadcn docs/examples | `fret-ui-shadcn` | Caller-heavy | Focused semantics + layout invariant tests | Reviewed 2026-03: root `w-full justify-center` ownership already matched upstream, while `PaginationContent/Item` needed explicit list semantics and navigation landmark is approximated with `Region + label` until a dedicated role exists |
| Breadcrumb | `breadcrumb` | P1 | Inline overflow + separator ownership | shadcn docs/examples | `fret-ui-shadcn` | Caller-heavy | Focused semantics + existing layout/web gates | Reviewed 2026-03: default wrap/gap/truncation ownership already matched upstream, while breadcrumb landmark/list/current-page semantics needed explicit alignment and presentation-only affordances are now hidden |

Queue policy:

- `P0`: likely to expose mechanism-vs-recipe mistakes or default-style ownership mistakes.
- `P1`: likely to expose public-surface drift or repeated layout-policy footguns.
- `Default style owner` means where we should *start* the investigation, not a hard-coded answer for every slot.
- When a component is pulled into active work, update both this queue and the breadth table below.

## shadcn/ui v4 Registry Baseline

The upstream reference in `repo-ref/ui` defines 54 `registry:ui` components (`repo-ref/ui/apps/v4/registry.json`).

Status below uses Rust module naming (hyphenated names normalized to `_`).

Audit column is a lightweight review marker for shadcn parity against `repo-ref/ui` docs/examples:
`Unreviewed` -> `In review` -> `Pass` (or `Defer`/`Skip` when applicable).

| Registry name | Rust module | Status | Audit | Notes |
| --- | --- | --- | --- | --- |
| accordion | `accordion` | Present | In review | Selection model drives open/close; measured-height motion is wired; gallery order now mirrors shadcn docs with dedicated `Basic` / `Multiple` / `Disabled` / `Borders` / `Card` / `RTL` sections, and usage can use the composable Radix-shaped surface; audit: `docs/audits/shadcn-accordion.md`; shadcn-web layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`accordion-demo`, light+dark) |
| alert | `alert` | Present | In review | shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`alert-demo`) |
| alert-dialog | `alert_dialog` | Present | In review | Default copyable root path is `AlertDialog::new_controllable(cx, None, false).children([AlertDialogPart::trigger(...), AlertDialogPart::content(...)])`; `children([...])` + `from_scope(...)` align the common shadcn call site more closely, `AlertDialogPart` is now on the curated facade so the default import lane stays copyable, `AlertDialogContent::build(...)` now covers the typed content side of that same lane, and `AlertDialogCancel::variant(...)` keeps non-default destructive cancel styling available for upstream example surfaces; `compose()`, `Parts`, `Detached Trigger`, and `Rich Content` stay as focused follow-ups for explicit ownership or advanced seams; root surface is still recipe-layer part collection rather than full JSX nesting; audit: `docs/audits/shadcn-alert-dialog.md`; first-party page copy now follows the docs-page examples (`Show Dialog`, `Share Project`, dual-example RTL); shadcn-web chrome gate currently covers `alert-dialog-demo`, while docs-only `Small` / `Media` / `Small with Media` / `Destructive` visual evidence is locked with `tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-docs-example-open-screenshots.json` until routed upstream goldens exist |
| aspect-ratio | `aspect_ratio` | Present | In review | Audit: `docs/audits/radix-aspect-ratio.md`; layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/basic.rs` (`aspect-ratio-demo`); docs page: `apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs` |
| avatar | `avatar` | Present | In review | Audit: `docs/audits/shadcn-avatar.md`; gallery order now mirrors shadcn docs with dedicated `Badge with Icon` / `Avatar Group with Icon` / `API Reference` sections, while `Fallback only` stays a Fret-only follow-up; shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`avatar-demo`) |
| badge | `badge` | Present | In review | Audit: `docs/audits/shadcn-badge.md`; gallery order now mirrors shadcn docs with a dedicated `API Reference` section, while numeric count badges stay in a Fret-only follow-up; shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`badge-demo`) |
| breadcrumb | `breadcrumb` | Present | In review | Semantics alignment landed for root/list/current-page/presentation affordances; gallery `Usage` now teaches the curated facade aliases for the upstream-shaped composition lane, while raw primitives stay explicit for advanced docs-parity examples and the compact builder remains a Fret shorthand; audit: `docs/audits/shadcn-breadcrumb.md`; gates: `ecosystem/fret-ui-shadcn/src/breadcrumb.rs` |
| button | `button` | Present | In review | Audit: `docs/audits/shadcn-button.md`; gallery order now mirrors shadcn docs with explicit `Usage` / `Cursor` / `Default` / `Outline` / `Secondary` / `Ghost` / `Destructive` / `Link` / `API Reference` sections; the second upstream Link path maps to `ButtonRender::Link` (`Link (Semantic)`), logical `data-icon` child compositions now map to `leading_child(...)` / `trailing_child(...)` (with plural helpers for multi-node landed content), the full-row override stays explicit via `child(...)` / `children(...)`, and button docs sections now expose stable page-scoped ids for page-level diag gates; cursor ownership stays runtime/pressable-owned; shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`button-demo`) |
| button-group | `button_group` | Present | In review | Audit: `docs/audits/shadcn-button-group.md`; gallery order now mirrors shadcn docs with dedicated `Usage` / `Accessibility` / `ButtonGroup vs ToggleGroup` / `API Reference` sections; embedded `InputGroup` / `Select` examples now also follow those child families' default lanes instead of reintroducing parts adapters, `ButtonGroupText` keeps an explicit `.children(...)` surface, and `Flex-1 items` stays a Fret-only caller-owned layout example |
| calendar | `calendar` | Present | In review | Audit: `docs/audits/shadcn-calendar.md`; headless month grid lives in `fret-ui-kit` (`headless::calendar`); gallery now includes upstream doc-only `About` / `Date Picker` / `Selected Date (With TimeZone)` / `API Reference` sections before Fret-only follow-ups; caller owns `rounded/border` and page width, recipe owns inner chrome, `caption_layout(...)` now stays shared across `Calendar` and single-month `CalendarRange`, `CalendarDayButton::supporting_text_by(...)` covers the copyable custom-days slot across both typed surfaces without adding a generic root children API, and the UI Gallery `Custom Cell Size` snippet now translates upstream `md:[--cell-size:*]` call-site behavior with viewport queries instead of baking responsive size policy into the recipe |
| card | `card` | Present | In review | Audit: `docs/audits/shadcn-card.md`; gallery order now mirrors the shadcn Card docs through `API Reference`, `Rich Title (Fret)` keeps `card_title_children(...)` copyable on the default app-facing surface, `CardTitle::new_children(...)` / `card_title_children(...)` close the remaining title children-surface drift, and `CardFooter` owns a fill-width + `min-w-0` inner row/column budget so footer-only text wraps against the card width instead of collapsing per word; shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`card-with-form`) |
| carousel | `carousel` | Defer | In review | Audit: `docs/audits/shadcn-carousel.md`; gallery now mirrors the upstream Carousel docs structure first (`Demo` / `About` / `Usage` / `Examples` / `Options` / `API` / `Events` / `Plugins` / `RTL`), then inserts an explicit `Fret Follow-ups` bridge before the shorthand / adapter / engine surfaces and trailing `API Reference`; the first-party split is now taught consistently (`Usage` mirrors the upstream parts-shaped docs lane, `Compact Builder` keeps the Fret shorthand visible, the docs-path examples and the dedicated `Loop` preview still stay on the compact builder lane, and only explicit parts/custom-control snippets `Parts` / `Events` / `RTL` stay on the targeted parts lane); no active mechanism/public-surface drift is identified, and status stays `Defer` because carousel is not currently editor-critical |
| chart | `chart` | Defer | In review | Audit: `docs/audits/shadcn-chart.md`; chart recipe surface, tooltip/legend contracts, and existing chart gates are already partially audited; status stays `Defer` because broader chart-engine work is not currently editor-critical |
| checkbox | `checkbox` | Present | In review | Audit: `docs/audits/shadcn-checkbox.md`; gallery now mirrors the base Checkbox docs path first with explicit `API Reference`, restores the upstream `Description` / `Group` framing, and teaches a derived mixed-state `Table` select-all example on the same action-first snapshot path while `Label Association` and `With Title` stay as focused Fret follow-ups; `Checkbox::new(...)`, `new_optional(...)`, and `new_tristate(...)` now consistently accept narrow checked-state bridge traits, `Checkbox::from_checked(...)` / `from_checked_state(...)` remain the narrow source-aligned snapshot path, no generic children API is warranted, and checkbox chrome/focus ring/indicator visuals remain recipe-owned while surrounding field layout stays caller-owned |
| collapsible | `collapsible` | Present | In review | Audit: `docs/audits/shadcn-collapsible.md`; gallery now mirrors the base Collapsible docs path first with explicit `Controlled State` / `Basic` / `Settings Panel` / `File Tree` / `RTL` / `API Reference` sections, the lead `Demo` follows the official shadcn repository-list example, `Usage` keeps the composable children lane copyable on the curated facade via `CollapsibleRoot` / `CollapsibleTriggerPart` / `CollapsibleContentPart`, `Basic` now matches the upstream muted-open background plus `Learn More` xs CTA, `Settings Panel` restores the nested-left-column content structure and `0` defaults, and `File Tree` now mirrors the explorer tabs shell, current docs data set, and default-closed folders; the compact top-level wrapper remains the Fret-first ergonomic surface, raw collapsible primitives remain an explicit escape hatch, and disclosure motion/semantics remain recipe-owned while surrounding width/gap/card layout stay caller-owned |
| command | `command` | Present | In review | Classified as a direct recipe root/bridge family: `command(...)` / `CommandPalette` remain the public recipe root story, while split `CommandInput/List/Item` composition is intentionally deferred until a shared context contract is justified; first-party `apps/fret-ui-gallery/src/ui/snippets/command/*.rs` is now source-gated so the default docs-aligned snippets cannot drift onto `CommandInput::new(...)` / `CommandList::new(...)`, while a dedicated `Composable Shell (Fret)` follow-up keeps the current lower-level manual split lane copyable and explicit; the main copyable command snippets no longer depend on gallery-local `super::...` helpers, the lead docs demo now keeps the upstream `max-w-sm` cap without a local shadow override, `Basic` now matches the upstream minimal dialog teaching lane more closely (`Open Menu`, `Suggestions`, recipe-default dialog label), and the remaining cmdk-specific mechanism demos now live in an explicit Fret-only `Behavior Demos` follow-up section instead of overloading `Shortcuts`/`Groups`/`Scrollable`; `CommandDialog` also forwards palette test-id builders so docs-aligned dialog examples stay automation-friendly, and both `CommandPalette` / `CommandDialog` now expose `list_viewport_test_id(...)` so diagnostics can target the internal scroll viewport semantics surface instead of the outer listbox wrapper; cmdk-style active-descendant navigation + filtering/scoring (value + keywords), plus group/separator/empty + checkmark/shortcut; root chrome stays recipe-owned while width caps such as `max-w-sm` stay caller-owned; gallery page now mirrors the upstream docs path after skipping `Installation` (`Demo`, `About`, `Usage`, `Basic`, `Shortcuts`, `Groups`, `Scrollable`, `RTL`, `API Reference`) before Fret-only extras; audit: `docs/audits/shadcn-command.md`; gates: `web_vs_fret_command_dialog_*`, `web_vs_fret_combobox_demo_*`, `gallery_command_core_examples_keep_upstream_aligned_targets_present`, `gallery_command_docs_demo_keeps_upstream_max_width`, `gallery_command_basic_opens_dialog_with_default_recipe_a11y_label`, `gallery_command_follow_up_sections_remain_explicit_after_docs_aligned_examples`, `ui-gallery-command-scrollable-filter-clamps-scroll` |
| context-menu | `context_menu` | Present | In review | Right click + (macOS) ctrl-click + Shift+F10; anchors to click position for web/Radix parity; default copyable root path is `ContextMenu::uncontrolled(cx).compose().trigger(...).content(...).entries(...)` while managed-open seams stay explicit on `ContextMenu::from_open(...)` / `new_controllable(...)`; trigger surface stays caller-owned while explicit panel width overrides live on `ContextMenuContent::min_width(...)`; lower-level parts bridges remain `ContextMenu::build_parts(...)` / `into_element_parts(...)`; audit: `docs/audits/shadcn-context-menu.md`; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`, `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` |
| dialog | `dialog` | Present | In review | Default copyable root path is `Dialog::new_controllable(cx, None, false).children([DialogPart::trigger(...), DialogPart::content(...)])`; `DialogPart` is now on the curated facade so the default import lane stays copyable, `DialogContent::build(...)` now covers the typed content side of that same lane, `DialogClose::from_scope().build(cx, button)` keeps footer/custom close actions aligned with upstream `<DialogClose asChild>`, and `compose()` plus `Parts` stay focused follow-ups for explicit builder-style or root-part ownership; gallery order now mirrors shadcn docs through `API Reference` before Fret-specific follow-ups; audit: `docs/audits/shadcn-dialog.md`; shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` |
| drawer | `drawer` | Present | In review | `direction(...)` now aliases the upstream Vaul/shadcn placement prop, `disable_pointer_dismissal(...)` covers the Base UI-style root dismissal alias, `modal(false)` / `modal_mode(DrawerModalMode::{NonModal,TrapFocus})` now cover the Base UI non-modal follow-up, `DrawerClose::from_scope().build(...)` covers the common `asChild` close path, and the non-modal/trap-focus follow-up now defaults initial focus to the drawer popup root instead of the first focusable descendant while still painting a click-through visual scrim through `DrawerOverlay` / `overlay_color(...)`; default copyable root path is `Drawer::new_controllable(cx, None, false).children([DrawerPart::trigger(...), DrawerPart::content_with(...)])`, while `DrawerContent::build(...)` now covers the typed content side of that same lane, `compose()` stays the builder-first alternative, and Vaul-specific `snap_points(...)` stays a focused follow-up on that same root lane; controlled snap points now exist as recipe-owned authored-index policy via `snap_point(...)`, `on_snap_point_change(...)`, and `snap_to_sequential_points(...)`; nested child open/frontmost-height state now feeds parent drag arbitration in the recipe layer so a nested child can suppress parent drag-to-dismiss, but deeper nested swipe/input routing plus Base UI-style background indentation visuals remain follow-up work in `fret-ui-kit` / recipe policy rather than `fret-ui`; gallery order now mirrors shadcn docs before the Fret-specific snap-points recipe; audit: `docs/audits/shadcn-drawer.md` |
| dropdown-menu | `dropdown_menu` | Present | In review | Menu navigation + typeahead + dismissible popover infra (ADR 0074); now includes `Label`/`Group`/`Shortcut` + destructive items; default copyable root path is `DropdownMenu::uncontrolled(cx).compose().trigger(...).content(...).entries(...)` while managed-open seams stay explicit on `DropdownMenu::from_open(...)` / `new_controllable(...)`; trigger sizing stays caller-owned while explicit panel width overrides live on `DropdownMenuContent::min_width(...)`; lower-level parts bridges remain `DropdownMenu::build_parts(...)` / `into_element_parts(...)`; audit: `docs/audits/shadcn-dropdown-menu.md`; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` |
| empty | `empty` | Present | In review | Audit: `docs/audits/shadcn-empty.md`; gallery now mirrors the base Empty docs with explicit `Usage` / `API Reference` sections, the `Usage` snippet leads with direct `Empty::new([...])` compound-children composition, and the `Demo` / `RTL` previews now follow the upstream `new-york-v4` teaching shape more closely (icon media, centered two-button action row, semantic link CTA); the current recipe intentionally stays aligned to the in-repo `new-york-v4` web geometry gates (`empty-demo`, `empty-background`, `empty-outline`) instead of re-translating base source defaults in this pass; caller still owns preview height, background paint, inline content layout, embedded `InputGroup` width, and page/grid placement constraints |
| field | `field` | Present | In review | Audit: `docs/audits/shadcn-field.md`; gallery order now mirrors the base Field docs with explicit `Form` / `Accessibility` / `API Reference` sections and restores the upstream `RTL -> Responsive Layout -> Validation and Errors` ordering, while `Usage` and `Anatomy` are now snippet-backed instead of page-local Rust strings; `Field::build(...)` is the intentional field-local control-association lane for `Select`, `Input`, and `Textarea`, while `Field::new([...])` stays the plain eager-children wrapper; no new mechanism gap found, and width ownership stays explicit (`FieldDescription` recipe-owned full-width, plain `FieldLabel` / `FieldTitle` intrinsic by default); gates: `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`, `tools/diag-scripts/ui-gallery/field/ui-gallery-field-docs-smoke.json`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`field-*`) |
| form | `form` | Present | In review | `FormControl` no longer aliases `FieldContent`; audit: `docs/audits/shadcn-form.md`; focused gates live in `ecosystem/fret-ui-shadcn/src/form.rs` |
| hover-card | `hover_card` | Present | In review | `HoverCard::new(cx, trigger, content)` remains the right recipe-level bridge (no extra generic `compose()` needed); gallery order and usage now align directly with shadcn docs; supports custom anchor via `HoverCard::anchor_element(...)`; audit: `docs/audits/shadcn-hover-card.md` |
| input | `input` | Present | In review | Audit: `docs/audits/shadcn-input.md`; gallery now mirrors the base Input docs with a dedicated `Usage` snippet source plus stable page-scoped `ui-gallery-input-*` section ids, while `Label Association`, `API Reference`, and `Notes` stay as focused Fret follow-ups after the upstream path; `Input::control_id(...)` now also inherits the field-local association inside `Field::build(...)`, so builder-composed fields get label/description/focus forwarding without changing the default explicit-id teaching lane; root width/height defaults remain recipe-owned, file input stays composed `Input + Browse`, page-level gallery gates now target the real docs structure, and shadcn-web chrome gates continue to cover `input-demo` / invalid / focus states |
| input-group | `input_group` | Present | In review | Audit: `docs/audits/shadcn-input-group.md`; gallery order now mirrors the base docs with explicit `Usage` / `Align` / `API Reference` sections, the `Usage` section is now a snippet-backed compact-shorthand example instead of a page-local Rust string, while `Tooltip` / `Label Association` / `Button Group` stay as Fret follow-ups; `InputGroup::new(model)` is the first-party ergonomic lane, explicit addon/control parts remain the docs-parity lane, narrow `custom_input(...)` / `custom_textarea(...)` entry points cover the upstream `Custom Input` docs path without widening the recipe to generic root `children(...)`, root `w-full min-w-0` remains recipe-owned, and caller override is gated in `ecosystem/fret-ui-shadcn/src/input_group.rs`; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`input-group-dropdown`, `input-group-custom`, and related `input-group-*` cases), `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`, `tools/diag-scripts/ui-gallery/input/ui-gallery-input-group-docs-smoke.json` |
| input-otp | `input_otp` | Present | In review | Classified as a direct recipe root/bridge family: `Usage` now teaches the upstream-shaped `InputOTPGroup` / `InputOTPSlot` / `InputOTPSeparator` bridge directly, `Compact Builder` keeps `InputOTP::new(model)` plus `length(...)` / optional `group_size(...)` visible as the Fret shorthand follow-up, and `InputOtpPart` now accepts direct `From` conversions from the group/slot/separator bridge values; default slot chrome ownership aligns with upstream and separator semantics + `control_id` form association are landed; audit: `docs/audits/shadcn-input-otp.md`; gates: `ecosystem/fret-ui-shadcn/src/input_otp.rs`, `apps/fret-ui-gallery/src/ui/pages/input_otp.rs`, `tools/diag-scripts/ui-gallery/input/ui-gallery-input-otp-docs-smoke.json` |
| item | `item` | Present | In review | Audit: `docs/audits/shadcn-item.md`; gallery now mirrors the base Item docs path first with explicit `Usage` / `Item vs Field` / `API Reference` sections, while `Gallery` and `Link (render)` stay as focused Fret follow-ups; intrinsic item chrome, slot spacing, and size presets remain recipe-owned, and existing item web layout gates continue to cover `item-demo` / `item-size` / `item-avatar` / `item-link` |
| kbd | `kbd` | Present | In review | Audit: `docs/audits/shadcn-kbd.md`; gallery now mirrors the base Kbd docs with a snippet-backed single-key `Usage` section, docs-aligned textual/glyph examples (`⌘`, `⇧`, `⌥`, `⌃`, `⏎`) on the primary teaching lane, and stable section-scoped `ui-gallery-kbd-*` ids for docs diagnostics; `Kbd::from_children([...])` remains the explicit Fret escape hatch for icon-only caps rather than a broader generic children API; fixed-height keycap chrome stays recipe-owned and button / tooltip / input-group placement remains caller-owned composition; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/kbd.rs`, `ecosystem/fret-ui-shadcn/tests/fret_kbd_tooltip_slot.rs`, `tools/diag-scripts/ui-gallery/kbd/ui-gallery-kbd-docs-smoke.json` |
| label | `label` | Present | In review | Audit: `docs/audits/shadcn-label.md`; gallery now mirrors the base Label docs path first with explicit `API Reference`, while `Field` / `FieldLabel` remain the form-owned follow-up path; label text sizing stays recipe-owned, surrounding layout stays caller-owned, and existing label web gates continue to cover `label-demo` geometry and marker parity |
| menubar | `menubar` | Present | In review | Click-to-open; hover switching; Escape close focus is gated against radix timelines; compact Fret-first root lane is `Menubar::new([MenubarMenu::new(...).entries([...])])`, while `MenubarTrigger::into_menu().entries_parts(...)` remains the upstream-shaped copyable lane and now drives the first-party docs-path snippets (`Usage`, `Checkbox`, `Radio`, `Submenu`, `With Icons`, `RTL`) in UI Gallery; `MenubarContent::{min_width, submenu_min_width}` cover shadcn-style content width overrides while root width stays caller-owned; `API Reference` now records the owner split and why no extra generic children API is needed; audit: `docs/audits/shadcn-menubar.md`; gates: `ecosystem/fret-ui-shadcn/tests/{radix_web_primitives_state,web_vs_fret_overlay_placement}.rs` |
| native-select | `native_select` | Defer | In review | Audit: `docs/audits/shadcn-native-select.md`; gallery now mirrors the docs path first with explicit `Groups` / `Disabled` / `Invalid` / `Native Select vs Select` / `RTL` / `API Reference` sections, while `Label Association` stays a focused Fret follow-up; trigger chrome, chevron, and default `default` / `sm` heights remain recipe-owned, surrounding width negotiation stays caller-owned, and no generic children API is needed beyond `options(...)` / `optgroups(...)` while backend-native parity stays deferred |
| navigation-menu | `navigation_menu` | Present | In review | Classified as a dual-lane family: `navigation_menu(cx, model, |cx| ..)` is the compact typed lane, while `NavigationMenuRoot/List/Item/Trigger/Content/Link/Viewport/Indicator` remain the upstream-shaped lane; `NavigationMenuLink::{new,child}` now follow the same narrow single-selection bridge as the compact root lane instead of requiring raw `Model<Option<Arc<str>>>` handles; mobile `viewport` mode + anchored placement; top-level link parity now uses contentless `NavigationMenuItem` `href/target/rel`, and custom trigger composition stays on `trigger` / `trigger_child` / `trigger_children` rather than a separate DOM-style children API; trigger chrome stays recipe-owned; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (navigation-menu-demo.* open variants), `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` |
| pagination | `pagination` | Present | In review | Classified as a dual-lane family: `Usage` now teaches the upstream-shaped `Pagination` / `PaginationContent` / `PaginationItem` / `PaginationLink` parts lane directly, while `Compact Builder` keeps `pagination(...)` / `pagination_content(...)` / `pagination_item(...)` / `pagination_link(...)` visible as the Fret shorthand follow-up; no extra generic `compose()` is needed; gallery order now mirrors the shadcn docs path with explicit `Routing` and `Custom Text` sections; audit: `docs/audits/shadcn-pagination.md`; gates: `ecosystem/fret-ui-shadcn/src/pagination.rs`, `tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-docs-smoke.json` |
| popover | `popover` | Present | Pass | Default recipe path now uses `Popover::new(cx, trigger, content)`; `PopoverTrigger::build(...)` plus `PopoverContent::build(cx, ...)` now cover the typed compound-parts lane, while `from_open(...).into_element_with(...)` / `into_element_with_anchor(...)` stay as explicit advanced seams for managed-open and anchor-aware sizing flows; no extra generic `children([...])` / `compose()` root lane is currently warranted; `PopoverContent` now keeps fill-width wrapping without stretching inline-sized children by default; anchored placement + click-through outside press dismissal (ADR 0069); usage/gallery now align directly with shadcn docs and keep a Fret-only intrinsic-width regression section afterwards; audit: `docs/audits/shadcn-popover.md`; diag: `tools/diag-scripts/ui-gallery/popover/ui-gallery-popover-inline-children-button-not-stretched.json` |
| progress | `progress` | Present | In review | shadcn-web gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` + `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`progress-demo`) |
| radio-group | `radio_group` | Present | In review | Audit: `docs/audits/shadcn-radio-group.md`; gallery now mirrors the base Radio Group docs path first with explicit `Description` / `Choice Card` / `Fieldset` / `Disabled` / `Invalid` / `RTL` / `API Reference` sections, while `Label Association` stays as a focused Fret follow-up; radio-group selection semantics, icon chrome, and focus ring remain recipe-owned, surrounding fieldset/card layout stays caller-owned, and existing layout/focus gates continue to cover `radio-group-demo` parity |
| resizable | `resizable` | Present | In review | Runtime-owned drag + layout; gallery core-example gate added; audit: `docs/audits/shadcn-resizable.md`; gates: `ecosystem/fret-ui-shadcn/tests/resizable_panel_group_layout.rs`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/resizable.rs`, `apps/fret-ui-gallery/src/driver/render_flow.rs` |
| scroll-area | `scroll_area` | Present | In review | shadcn-web layout gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`scroll-area-*`) |
| select | `select` | Present | In review | Classified as a direct recipe root/bridge family: `Select::new(...)` / `new_controllable(...)` plus the compact direct chain `.trigger(...).value(...).content(...).entries(...)` are now the default copyable root story, `apps/fret-ui-gallery/src/ui/snippets/select/*.rs` is now fully off `into_element_parts(...)`, and `into_element_parts(...)` remains the upstream-shaped adapter on that same lane rather than a generic `compose()` story; gallery now mirrors the upstream docs path first (`Demo`, `Usage`, `Align Item With Trigger`, `Groups`, `Scrollable`, `Disabled`, `Invalid`, `RTL`, `API Reference`), while `Label Association`, `Field Builder Association`, and `Diag Surface` stay as focused Fret follow-ups; `Field::build(...) + FieldLabel + Select + FieldDescription` now lands the field-local label/description association path without explicit ids, and the gallery teaches that builder lane separately from the standalone `control_id(...)` example; composable children stay intentionally narrow through `Select::into_element_parts(...)` + `SelectContent::with_entries(...)` instead of widening the default lane to arbitrary children; item-aligned + popper placement (side/align/offsets); scroll buttons; item-aligned scroll-button reposition + post-position “scroll selected into view” behavior for Radix parity; shadcn-web gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (`web_vs_fret_select_demo_overlay_placement_matches`, `web_vs_fret_select_demo_open_option_metrics_match`, `web_vs_fret_select_scrollable_listbox_width_matches`, `web_vs_fret_select_scrollable_listbox_option_insets_match`, `web_vs_fret_select_scrollable_listbox_option_height_matches`, `web_vs_fret_select_scrollable_scroll_button_height_matches`, `web_vs_fret_select_scrollable_viewport_insets_match`, plus small/tiny viewport variants); audit: `docs/audits/shadcn-select.md` |
| separator | `separator` | Present | In review | Audit: `docs/audits/shadcn-separator.md`; gallery order now mirrors shadcn docs with dedicated `Vertical` / `Menu` / `List` / `RTL` / `API Reference` sections; shadcn-web gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`separator-demo`) |
| sheet | `sheet` | Present | In review | Default copyable root path is `Sheet::new_controllable(cx, None, false).children([SheetPart::trigger(...), SheetPart::content(...)])`; `SheetPart` is now on the curated facade so the default import lane stays copyable, `SheetContent::build(...)` covers the typed content side of that same lane, default close affordance lives in `SheetContent` and aligns with upstream `showCloseButton` / `show_close_button(false)`, while `compose()` and `Parts` stay focused follow-ups for explicit builder-style assembly or root-part ownership; gallery order now mirrors shadcn docs before Fret-specific parts docs; audit: `docs/audits/shadcn-sheet.md`; shadcn-web chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs` |
| sidebar | `sidebar` | Present | In review | Provider-owned width API (`width/_icon/_mobile`) + docs-first gallery page; audit: `docs/audits/shadcn-sidebar.md`; gates: `ecosystem/fret-ui-shadcn/src/sidebar.rs`, `apps/fret-ui-gallery/src/driver/render_flow.rs` |
| skeleton | `skeleton` | Present | In review | Audit: `docs/audits/shadcn-skeleton.md`; gallery now mirrors the base Skeleton docs path first with explicit `Demo` / `Usage` / `Avatar` / `Card` / `Text` / `Form` / `Table` / `RTL` / `API Reference` sections; `Skeleton::new()` stays the source-aligned size-free leaf primitive, `Skeleton::block()` remains a focused Fret convenience, default chrome/pulse stay recipe-owned, and explicit placeholder sizing/shape remain caller-owned |
| slider | `slider` | Present | In review | Audit: `docs/audits/shadcn-slider.md`; gallery now mirrors the shadcn/Base UI docs path first with explicit `Demo` / `Usage` / `Range` / `Multiple Thumbs` / `Vertical` / `Controlled` / `Disabled` / `RTL` / `API Reference` sections, while `Label Association`, `Extras`, and `Notes` stay as focused Fret follow-ups; slider semantics now live on thumbs rather than the root for better Radix/Base UI alignment, the existing `slider(model)` / `new_controllable(...)` surface is sufficient, and no generic children API is warranted on the shadcn lane while Base UI's compound parts stay a headless reference for future `fret-ui-kit` work; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`slider-demo`), `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-set-value.json`, `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-range-drag-stability.json`, `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-label-click-focus.json` |
| sonner | `sonner` | Present | In review | Toast store + overlay layer + timers; upsert-by-id; swipe-to-dismiss; hover pause/resume; max-toasts eviction; action/cancel; manual promise handle; audit: `docs/audits/shadcn-sonner.md` |
| spinner | `spinner` | Present | In review | Audit: `docs/audits/shadcn-spinner.md`; gallery now mirrors the radix Spinner docs path first with explicit `Demo` / `Usage` / `Customization` / `Size` / `Button` / `Badge` / `Input Group` / `Empty` / `RTL` sections, while `Extras` and `API Reference` stay as focused Fret follow-ups; default icon/size/spin/current-color behavior remain recipe-owned, explicit icon/color/size refinements remain caller-owned, spinner semantics now align to upstream status/live-region expectations instead of a numeric progress role, and Button/Badge/InputGroup slot surfaces remain sufficient so no generic children API is needed on the spinner leaf; existing unit/layout gates continue to cover spinner timing plus web geometry in `ecosystem/fret-ui-shadcn/src/spinner.rs` and `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/spinner.rs` |
| switch | `switch` | Present | In review | Audit: `docs/audits/shadcn-switch.md`; gallery now mirrors the base Switch docs path first with explicit `Demo` / `Usage` / `Description` / `Choice Card` / `Disabled` / `Invalid` / `Size` / `RTL` / `API Reference` sections, docs-path `Demo` / `Size` / `RTL` rows now keep source-aligned label/control binding, and `Size` now matches the upstream vertical `FieldGroup` composition before `Label Association` and `Style Override` continue as focused Fret follow-ups; intrinsic switch chrome and sizes remain recipe-owned, `Switch` now exposes `aria_invalid(...)` directly for source parity, and existing web chrome/layout gates continue to cover `switch-demo` and focus states |
| table | `table` | Present | In review | Width/overflow ownership aligns with the upstream wrapper; `table_head_children(...)` / `table_caption_children(...)` now cover the missing composable `th` / `caption` lanes; checkbox-column padding plus mixed-height body-row centering stay recipe-owned in `TableHead` / `TableCell`; and the gallery mirrors the docs path through `Data Table` / `RTL` / `API Reference` before a focused `Children (Fret)` follow-up; audit: `docs/audits/shadcn-table.md`; gates: `ecosystem/fret-ui-shadcn/src/table.rs`, `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/table.rs`, `tools/diag-scripts/ui-gallery/table/ui-gallery-table-docs-smoke.json` |
| tabs | `tabs` | Present | In review | Audit: `docs/audits/shadcn-tabs.md`; shadcn-web layout gates continue to cover `tabs-demo`, root width constraints stay caller-owned while list/trigger chrome remains recipe-owned, `TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent` already cover the compound-parts lane so no extra root `children([...])` API is warranted, and the gallery now mirrors the docs path through `API Reference` before `Composable Parts (Fret)`, `List`, `Vertical (Line)`, `Extras`, and `Notes` continue as explicit follow-ups; docs gate: `tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-docs-smoke.json` |
| textarea | `textarea` | Present | In review | Audit: `docs/audits/shadcn-textarea.md`; gallery now mirrors the base Textarea docs with explicit `API Reference`, while `With Text` and `Label Association` stay as focused Fret follow-ups; `Textarea::control_id(...)` now also inherits the field-local association inside `Field::build(...)`, so builder-composed fields get label/description/focus forwarding without changing the default explicit-id teaching lane; root `w-full min-w-0`, control chrome, min height, and resize behavior remain recipe-owned, and shadcn-web chrome gates continue to cover `textarea-demo` / invalid / focus states |
| toggle | `toggle` | Present | In review | Audit: `docs/audits/shadcn-toggle.md`; gallery now mirrors the base Toggle docs path first with explicit `Demo` / `Usage` / `Outline` / `With Text` / `Size` / `Disabled` / `RTL` sections, then keeps `Children (Fret)`, `Label Association`, and `API Reference` as focused Fret follow-ups; `children([...])` remains the source-aligned composable content surface, toggle chrome/size presets/pressed-state colors remain recipe-owned, and existing shadcn-web chrome gates continue to cover `toggle-demo` and focus states |
| toggle-group | `toggle_group` | Present | In review | Audit: `docs/audits/shadcn-toggle-group.md`; gallery now mirrors the base Toggle Group docs path first with source-aligned `Demo` / `Usage` / `Outline` / `Size` / `Spacing` / `Vertical` / `Disabled` / `Custom` / `RTL` / `API Reference` snippets, while `Children (Fret)`, `Single`, `Small`, `Large`, `Label Association`, and layout regression slices stay as focused Fret follow-ups; the builder-preserving `toggle_group_*` helper family is now taught as the composable-children lane instead of widening the default root surface, `ToggleGroupItem::refine_layout(...)` and `refine_style(...)` continue to cover upstream custom item-root sizing/rounding, selection semantics and segmented chrome remain recipe-owned, and existing web chrome/layout gates continue to cover `toggle-group-demo` and size/spacing variants |
| tooltip | `tooltip` | Present | In review | `Tooltip::new(cx, trigger, content)` remains the right recipe-level bridge; no extra generic `children([...])` / `compose()` root lane is currently warranted because the root only needs trigger/content; `TooltipTrigger::build(...)` plus `TooltipContent::build(cx, ...)` define the typed compound-parts lane on first-party snippets; Gallery now mirrors the shadcn/base docs path through `API Reference`, keeps the `Usage` code tab standalone via a local `TooltipProvider` wrapper, then appends Fret-specific `Long Content` / `Keyboard Focus` parity sections; rendered via overlay root (not clipped); audit: `docs/audits/shadcn-tooltip.md` |

## Non-registry surfaces

These are shadcn-style surfaces referenced by docs/demos but not part of the `registry:ui` baseline:

| Surface | Rust module | Status | Audit | Notes |
| --- | --- | --- | --- | --- |
| combobox | `combobox` | Present | In review | Classified as a direct recipe root/bridge family: `Combobox::new(value, open)` plus the compact direct chain `.trigger(...).input(...).clear(...).content(...)` are now the default recipe root story, `ComboboxChips` now also supports a matching compact chain, the first-party `apps/fret-ui-gallery/src/ui/snippets/combobox/*.rs` surface is now off `into_element_parts(...)`, and `into_element_parts(...)` remains the focused upstream-shaped patch seam for trigger/input/content customization when callers explicitly want that shape; Fret intentionally does not add another generic `compose()` builder on top of the `Popover + CommandPalette` recipe; audit: `docs/audits/shadcn-combobox.md`; gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` (web listbox height, open-state goldens: `combobox-demo.open.json`, `combobox-demo.vp375x320.open.json`) |
| date picker | `date_picker` | Present | In review | `Popover` + `Calendar` recipe; trigger width is caller-owned; audit: `docs/audits/shadcn-date-picker.md`; gates: `ecosystem/fret-ui-shadcn/src/date_picker.rs`, `apps/fret-ui-gallery/src/driver/render_flow.rs` |
| data table / datagrid | `data_table` | Present | In review | Audit: `docs/audits/shadcn-data-table.md`; this stays an extension surface rather than a tiny `registry:ui` leaf, and the gallery now makes that explicit via `Default Recipe (Fret)` plus the guide-aligned `Basic Table` / `Guide Demo` / `RTL` / `Guide Outline` / `API Reference` sections; both the selection-column examples and the row-action dropdown menus now use typed `.action(...)` / `.action_payload(...)` plus grouped `cx.actions().models::<A>(...)` / `payload_models::<A>(...)` instead of root command routing or per-row `CommandId::new(...)` strings, while `DataTable` + companion recipes remain recipe-owned and app-specific columns/data/filters and page negotiation stay caller-owned; reviewed 2026-03: no extra generic children API is justified, the non-retained nested-action hit bug traced to a `fret-ui-kit` content-shell sizing issue, the retained/shadcn-default row-action hit bug traced to the retained row wrapper when `pointer_row_selection=false`, the missing Guide Demo header labels/icons traced to stacked header clipping between `fret-ui-kit` and the shadcn header recipe rather than the renderer, and one flaky gallery gate was script drift after page reflow rather than a widget regression; existing web layout gates continue to cover `data-table-demo` geometry in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/table.rs` |
| toast | `toast` | Skip | In review | Audit: `docs/audits/shadcn-toast.md`; upstream `toast` is deprecated in favor of `sonner`, so this repo keeps `toast` only as a compatibility alias while `sonner` remains the maintained parity target |
| typography | `typography` | Skip | In review | Audit: `docs/audits/shadcn-typography.md`; upstream typography is a docs-only page, so Fret keeps helper functions and gallery examples without treating it as a registry component contract |

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
