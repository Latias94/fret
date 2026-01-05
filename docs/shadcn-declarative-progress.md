# Shadcn Declarative Implementation Progress

Tracks the ongoing work to rebuild Fret's shadcn-aligned component surface as a declarative-only API.

## Source of Truth

This file is the canonical tracker for shadcn/ui v4 parity and the declarative-only migration.

Historical documents under `docs/archive/` are kept for context only and may be stale:

- `docs/archive/backlog/shadcn-v4-component-parity-todo.md` (archived)

## Scope

- `ecosystem/fret-ui-shadcn`: shadcn/ui v4 naming + taxonomy surface (recipes).
- `ecosystem/fret-ui-kit`: reusable infra (tokens/recipes/headless helpers).
- `crates/fret-ui`: runtime substrate (contracts/mechanisms only).

## Layering & Ownership

This repo intentionally splits responsibilities across three layers (similar to Tailwind + headless + Radix/RSC composition, but in Rust):

- `fret-ui` (**mechanisms/contracts**): element tree, hit-test, focus, semantics/a11y, overlay roots/layers, outside-press observers, layout, paint.
- `fret-ui-kit` (**design-system + infra**, Tailwind-ish): token-driven styling (`Theme` keys + refinements), reusable declarative helpers (`scroll`, `text_field`, etc), and headless state machines (`roving_focus`, hover intent, menu navigation).
- `fret-ui-shadcn` (**taxonomy + recipes**): shadcn/ui v4 naming surface and component composition; no retained widgets, no renderer/platform deps.

App/editor-specific composition belongs in `fret-app-kit` / `fret-editor` (e.g. app toolbars, menu bars, command palette wiring).

### Interaction Policy (Action Hooks)

Cross-cutting interaction policies (toggle models, close overlays, selection writes, “dismiss on escape/outside press”, etc.) are *component-owned*:

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

## shadcn/ui v4 Registry Baseline

The upstream reference in `repo-ref/ui` defines 54 `registry:ui` components (`repo-ref/ui/apps/v4/registry.json`).

Status below uses Rust module naming (hyphenated names normalized to `_`).

Audit column is a lightweight review marker for shadcn parity against `repo-ref/ui` docs/examples:
`Unreviewed` → `In review` → `Pass` (or `Defer`/`Skip` when applicable).

| Registry name | Rust module | Status | Audit | Notes |
| --- | --- | --- | --- | --- |
| accordion | `accordion` | Present | Unreviewed | Selection model drives open/close; no animation yet |
| alert | `alert` | Present | Unreviewed |  |
| alert-dialog | `alert_dialog` | Present | Unreviewed | Modal overlay policy; Tab traversal wraps within modal barrier (ADR 0068) |
| aspect-ratio | `aspect_ratio` | Present | Unreviewed |  |
| avatar | `avatar` | Present | Unreviewed |  |
| badge | `badge` | Present | Unreviewed |  |
| breadcrumb | `breadcrumb` | Present | Unreviewed |  |
| button | `button` | Present | Unreviewed |  |
| button-group | `button_group` | Present | Unreviewed | Thin wrapper over `toggle_group` styling |
| calendar | `calendar` | Missing | Unreviewed | Large surface; likely deferred unless needed |
| card | `card` | Present | Unreviewed |  |
| carousel | `carousel` | Defer | Unreviewed | Not editor-critical |
| chart | `chart` | Defer | Unreviewed | Not editor-critical |
| checkbox | `checkbox` | Present | Unreviewed |  |
| collapsible | `collapsible` | Present | Unreviewed | Headless open/close + a11y semantics |
| command | `command` | Present | In review | `CommandPalette` provides cmdk-style active-descendant navigation + filtering/scoring (value + keywords), plus group/separator/empty + checkmark/shortcut |
| context-menu | `context_menu` | Present | In review | Right click + (macOS) ctrl-click + Shift+F10; audit: `docs/audits/shadcn-context-menu.md` |
| dialog | `dialog` | Present | Unreviewed | Modal barrier + Escape + overlay dismissal; Tab traversal wraps within modal barrier (ADR 0068) |
| drawer | `drawer` | Present | Unreviewed | `sheet` facade (defaults to bottom); overlay policy |
| dropdown-menu | `dropdown_menu` | Present | In review | Menu navigation + typeahead + dismissible popover infra (ADR 0074); now includes `Label`/`Group`/`Shortcut` + destructive items; audit: `docs/audits/shadcn-dropdown-menu.md` |
| empty | `empty` | Present | Unreviewed |  |
| field | `field` | Present | Unreviewed | Repo-specific “form field” primitives |
| form | `form` | Present | Unreviewed |  |
| hover-card | `hover_card` | Present | In review | Hover intent + anchored placement; supports custom anchor via `HoverCard::anchor_element(...)`; audit: `docs/audits/shadcn-hover-card.md` |
| input | `input` | Present | Unreviewed |  |
| input-group | `input_group` | Present | Unreviewed | Composition over `input` + slots/icons |
| input-otp | `input_otp` | Present | Unreviewed | Slots rendered over transparent `TextInput`; digits-only clamping; a11y TBD |
| item | `item` | Present | Unreviewed | Repo-specific list/item recipes aligned with shadcn style |
| kbd | `kbd` | Present | Unreviewed |  |
| label | `label` | Present | Unreviewed |  |
| menubar | `menubar` | Present | In review | Click-to-open; hover switching; audit: `docs/audits/shadcn-menubar.md` |
| native-select | `native_select` | Defer | Unreviewed | Can map to `select` + platform-native later |
| navigation-menu | `navigation_menu` | Defer | Unreviewed | Complex; not editor-critical |
| pagination | `pagination` | Present | Unreviewed |  |
| popover | `popover` | Present | Pass | Anchored placement + click-through outside press dismissal (ADR 0069); non-modal (no focus trap); audit: `docs/audits/shadcn-popover.md` |
| progress | `progress` | Present | Unreviewed |  |
| radio-group | `radio_group` | Present | Unreviewed |  |
| resizable | `resizable` | Present | Unreviewed | Runtime-owned drag + layout; multi-panel group; a11y TBD |
| scroll-area | `scroll_area` | Present | Unreviewed | Declarative wrapper over `Scroll` + styling |
| select | `select` | Present | In review | Anchored placement supports `side`/`align` + offsets; roving navigation supports `loop` default; audit: `docs/audits/shadcn-select.md` |
| separator | `separator` | Present | Unreviewed | Simple primitive; declarative-only |
| sheet | `sheet` | Present | Unreviewed | Modal barrier + Escape + overlay dismissal; Tab traversal wraps within modal barrier (ADR 0068) |
| sidebar | `sidebar` | Present | Unreviewed |  |
| skeleton | `skeleton` | Present | Unreviewed |  |
| slider | `slider` | Present | Unreviewed | Declarative composition over primitives (PointerRegion hooks + bounds); a11y TBD |
| sonner | `sonner` | Present | Unreviewed | Toast store + overlay layer + timers; upsert-by-id; swipe-to-dismiss; hover pause/resume; max-toasts eviction; manual promise handle (no a11y yet) |
| spinner | `spinner` | Present | Unreviewed |  |
| switch | `switch` | Present | Unreviewed |  |
| table | `table` | Present | Unreviewed |  |
| tabs | `tabs` | Present | Unreviewed |  |
| textarea | `textarea` | Present | Unreviewed | Wrapper over declarative `TextArea` (runtime `TextArea` engine); a11y TBD |
| toggle | `toggle` | Present | Unreviewed |  |
| toggle-group | `toggle_group` | Present | Unreviewed |  |
| tooltip | `tooltip` | Present | In review | Hover intent + placement; rendered via overlay root (not clipped); audit: `docs/audits/shadcn-tooltip.md` |

## Non-registry surfaces

These are shadcn-style surfaces referenced by docs/demos but not part of the `registry:ui` baseline:

| Surface | Rust module | Status | Audit | Notes |
| --- | --- | --- | --- | --- |
| combobox | `combobox` | Present | In review | Now implemented as `Popover` + `CommandPalette` recipe; audit: `docs/audits/shadcn-combobox.md` |
| date picker | `date_picker` | Missing | Unreviewed | Upstream is a `Popover` + `Calendar` recipe; consider shipping as a recipe/example first |
| data table / datagrid | `data_table` | Feature-gated | Unreviewed | Enabled by `fret-ui-shadcn/datagrid`; not a `registry:ui` item upstream |
| toast | `toast` | Skip | Unreviewed | Upstream `toast` is deprecated in favor of `sonner`; this repo ships `sonner` |
| typography | `typography` | Skip | Unreviewed | Upstream typography page is docs-only and not shipped as a component |

Notes:
- “Present” means a declarative module exists and compiles; it may still be below the “Definition of Done” parity bar (keyboard/APG, a11y checklist, tests).
- Most “Missing” entries were previously implemented as retained widgets and intentionally deleted under the declarative-only boundary. They should come back as declarative components backed by `fret-ui-kit` infra + `fret-ui` mechanisms.
- `data_table` is not a `registry:ui` item upstream; treat it as an extension and keep it feature-gated.

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

The goal is to keep `fret-ui-shadcn` mostly “composition + styling”, and put reusable mechanisms/state in `fret-ui` / `fret-ui-kit`.

**Overlay stack (highest leverage)**
- `fret-ui` (mechanism): multi-root rendering per window, overlay layer install/uninstall, outside-press observers, modal barrier semantics, focus restore primitives.
- `fret-ui-kit` (policy): `WindowOverlays`-style request queues and rendering for popovers/menus/dialogs/toasts; consistent focus-restore rules (ADR 0069).

**Headless state machines**
- Hover intent (tooltip/hover-card delays), menu navigation (typeahead + roving), focus trapping for dialogs/sheets, and richer typeahead buffer (prefix match with timeout).

**Declarative primitives (Tailwind-ish building blocks)**
- `separator`, `scroll_area`, `textarea` (wrapper over runtime `TextArea`), `slider`, `resizable` panels/splitters.
- Input “slots” patterns: `input_group` (leading/trailing icons, clear buttons), `input_otp` helpers.

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
- `ecosystem/fret-ui-kit/src/headless/focus_trap.rs` (dialog/sheet focus trap + restore hooks)
- `ecosystem/fret-ui-kit/src/declarative/separator.rs` (simple visual + semantics)
- `ecosystem/fret-ui-kit/src/declarative/scroll_area.rs` (Scroll + scrollbar styling wrapper)
- `ecosystem/fret-ui-kit/src/declarative/textarea.rs` (runtime `TextArea` chrome wrapper)
- `crates/fret-ui/src/slider.rs` (pointer/keyboard input; a11y TBD)
- Extend `ecosystem/fret-ui-kit/src/window_overlays.rs` with: tooltip layer, menu layer, dialog/sheet layer, toast layer

Cross-cutting a11y constraint to keep in mind:

- Roving-focus “items” often should be *not* in Tab traversal, but still AT-focusable/activatable; ensure semantics focusability is not accidentally tied to Tab-stop (see `Pressable` semantics behavior).

## Reference: gpui-component Layering (Upstream Inspiration)

`repo-ref/gpui-component` is a useful comparison point:

- GPUI provides mechanisms like `DismissEvent`, `anchored(...)` placement, focus handles, and deferred overlays.
- gpui-component implements policy and styling at the component layer (`Popover::overlay_closable`, tooltip styling, input popovers, etc).

This matches Fret’s intended split: `fret-ui` as mechanism; `fret-ui-kit`/`fret-ui-shadcn` as policy + composition.

## Tracking Table (Update as work proceeds)

| Area | Component | Status | Owner crate | A11y | Tests | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Boundary | Retained API hidden | Done | `fret-ui` | - | - | `widget` + `create_node` are crate-private |
| Infra | Declarative tree | Done | `fret-ui-kit` | Partial | Partial | Expand with roving focus + typeahead helpers |
| Primitives | Button | Present | `fret-ui-shadcn` | Partial | Not started | Style parity + a11y checklist still pending |
| Primitives | Input | Present | `fret-ui-shadcn` | Partial | Not started | Uses runtime `TextInput` semantics + theming |
| Overlays | Select | Present | `fret-ui-shadcn` | Partial | Partial | Uses `fret-ui-kit/window_overlays` dismissible popover |
