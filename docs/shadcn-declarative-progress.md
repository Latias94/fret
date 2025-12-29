# Shadcn Declarative Implementation Progress

Tracks the ongoing work to rebuild Fret's shadcn-aligned component surface as a declarative-only API.

## Source of Truth

This file is the canonical tracker for shadcn/ui v4 parity and the declarative-only migration.

Historical documents under `docs/archive/` are kept for context only and may be stale:

- `docs/archive/backlog/shadcn-v4-component-parity-todo.md` (archived)

## Scope

- `crates/fret-components-shadcn`: shadcn/ui v4 naming + taxonomy surface (recipes).
- `crates/fret-components-ui`: reusable infra (tokens/recipes/headless helpers).
- `crates/fret-ui`: runtime substrate (contracts/mechanisms only).

## Layering & Ownership

This repo intentionally splits responsibilities across three layers (similar to Tailwind + headless + Radix/RSC composition, but in Rust):

- `fret-ui` (**mechanisms/contracts**): element tree, hit-test, focus, semantics/a11y, overlay roots/layers, outside-press observers, layout, paint.
- `fret-components-ui` (**design-system + infra**, Tailwind-ish): token-driven styling (`Theme` keys + refinements), reusable declarative helpers (`scroll`, `text_field`, etc), and headless state machines (`roving_focus`, hover intent, menu navigation).
- `fret-components-shadcn` (**taxonomy + recipes**): shadcn/ui v4 naming surface and component composition; no retained widgets, no renderer/platform deps.

App/editor-specific composition belongs in `fret-components-app` / `fret-editor` (e.g. app toolbars, menu bars, command palette wiring).

### Interaction Policy (Action Hooks)

Cross-cutting interaction policies (toggle models, close overlays, selection writes, “dismiss on escape/outside press”, etc.) are *component-owned*:

- `fret-ui` provides hook plumbing (`on_activate`, `on_dismiss_request`) as a mechanism-only substrate (ADR 0074).
- `fret-components-ui` and `fret-components-shadcn` register handlers to implement policies for each component.
- Legacy runtime shortcuts on `PressableProps` / dismissible roots have been removed from `crates/fret-ui`.
  Use component-owned action hooks (`fret-components-ui::declarative::action_hooks::ActionHooksExt`) instead.

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

| Registry name | Rust module | Status | Notes |
| --- | --- | --- | --- |
| accordion | `accordion` | Present | Selection model drives open/close; no animation yet |
| alert | `alert` | Present |  |
| alert-dialog | `alert_dialog` | Present | Modal overlay policy; Tab traversal wraps within modal barrier (ADR 0068) |
| aspect-ratio | `aspect_ratio` | Present |  |
| avatar | `avatar` | Present |  |
| badge | `badge` | Present |  |
| breadcrumb | `breadcrumb` | Present |  |
| button | `button` | Present |  |
| button-group | `button_group` | Present | Thin wrapper over `toggle_group` styling |
| calendar | `calendar` | Missing | Large surface; likely deferred unless needed |
| card | `card` | Present |  |
| carousel | `carousel` | Defer | Not editor-critical |
| chart | `chart` | Defer | Not editor-critical |
| checkbox | `checkbox` | Present |  |
| collapsible | `collapsible` | Present | Headless open/close + a11y semantics |
| command | `command` | Present | First pass: visual shell + roving list navigation; filtering is app-owned |
| context-menu | `context_menu` | Present | Right click + (macOS) ctrl-click + Shift+F10 |
| dialog | `dialog` | Present | Modal barrier + Escape + overlay dismissal; Tab traversal wraps within modal barrier (ADR 0068) |
| drawer | `drawer` | Present | `sheet` facade (defaults to bottom); overlay policy |
| dropdown-menu | `dropdown_menu` | Present | Menu navigation + typeahead + dismissible popover infra (ADR 0074) |
| empty | `empty` | Present |  |
| field | `field` | Present | Repo-specific “form field” primitives |
| form | `form` | Present |  |
| hover-card | `hover_card` | Present | Needs tooltip/popover-grade overlay infra for parity |
| input | `input` | Present |  |
| input-group | `input_group` | Present | Composition over `input` + slots/icons |
| input-otp | `input_otp` | Present | Slots rendered over transparent `TextInput`; digits-only clamping; a11y TBD |
| item | `item` | Present | Repo-specific list/item recipes aligned with shadcn style |
| kbd | `kbd` | Present |  |
| label | `label` | Present |  |
| menubar | `menubar` | Present | Click-to-open; hover switching + richer keyboard policy TBD |
| native-select | `native_select` | Defer | Can map to `select` + platform-native later |
| navigation-menu | `navigation_menu` | Defer | Complex; not editor-critical |
| pagination | `pagination` | Present |  |
| popover | `popover` | Present | Anchored placement + click-through outside press dismissal (ADR 0069); non-modal (no focus trap) |
| progress | `progress` | Present |  |
| radio-group | `radio_group` | Present |  |
| resizable | `resizable` | Present | Runtime-owned drag + layout; multi-panel group; a11y TBD |
| scroll-area | `scroll_area` | Present | Declarative wrapper over `Scroll` + styling |
| select | `select` | Present | Uses `window_overlays` dismissible popover infra |
| separator | `separator` | Present | Simple primitive; declarative-only |
| sheet | `sheet` | Present | Modal barrier + Escape + overlay dismissal; Tab traversal wraps within modal barrier (ADR 0068) |
| sidebar | `sidebar` | Present |  |
| skeleton | `skeleton` | Present |  |
| slider | `slider` | Present | Runtime `Slider` engine + shadcn wrapper; a11y TBD |
| sonner | `sonner` | Present | Toast store + overlay layer + timers (no a11y yet) |
| spinner | `spinner` | Present |  |
| switch | `switch` | Present |  |
| table | `table` | Present |  |
| tabs | `tabs` | Present |  |
| textarea | `textarea` | Present | Wrapper over declarative `TextArea` (runtime `TextArea` engine); a11y TBD |
| toggle | `toggle` | Present |  |
| toggle-group | `toggle_group` | Present |  |
| tooltip | `tooltip` | Present | Hover intent + placement; rendered via overlay root (not clipped) |

## Non-registry surfaces

These are shadcn-style surfaces referenced by docs/demos but not part of the `registry:ui` baseline:

| Surface | Rust module | Status | Notes |
| --- | --- | --- | --- |
| combobox | `combobox` | Present | Search input + popover list; active-descendant semantics are still pending (ADR 0073) |

Notes:
- “Present” means a declarative module exists and compiles; it may still be below the “Definition of Done” parity bar (keyboard/APG, a11y checklist, tests).
- Most “Missing” entries were previously implemented as retained widgets and intentionally deleted under the declarative-only boundary. They should come back as declarative components backed by `fret-components-ui` infra + `fret-ui` mechanisms.
- `data_table` exists in `fret-components-shadcn` but is not a `registry:ui` item upstream; treat as an extension.

## Recommended Order (Near-term)

1. `fret-components-ui`: declarative primitives and headless helpers used by everything (pressable, list/tree, focus).
2. `fret-components-shadcn`: primitives first (`Button` -> `Input/Textarea` -> `Checkbox/Switch/RadioGroup` -> `Tabs/Accordion`).
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

The goal is to keep `fret-components-shadcn` mostly “composition + styling”, and put reusable mechanisms/state in `fret-ui` / `fret-components-ui`.

**Overlay stack (highest leverage)**
- `fret-ui` (mechanism): multi-root rendering per window, overlay layer install/uninstall, outside-press observers, modal barrier semantics, focus restore primitives.
- `fret-components-ui` (policy): `WindowOverlays`-style request queues and rendering for popovers/menus/dialogs/toasts; consistent focus-restore rules (ADR 0069).

**Headless state machines**
- Hover intent (tooltip/hover-card delays), menu navigation (typeahead + roving), focus trapping for dialogs/sheets, and richer typeahead buffer (prefix match with timeout).

**Declarative primitives (Tailwind-ish building blocks)**
- `separator`, `scroll_area`, `textarea` (wrapper over runtime `TextArea`), `slider`, `resizable` panels/splitters.
- Input “slots” patterns: `input_group` (leading/trailing icons, clear buttons), `input_otp` helpers.

**Notifications**
- `sonner`/toast: global service API + per-window overlay root + timers + action dispatch.

**Command palette (`command` / cmdk-style)**
- Component surface belongs in `fret-components-shadcn` (shadcn taxonomy), but the heavy lifting should live in `fret-components-ui`:
  - headless filtering/scoring + match highlighting ranges
  - keyboard navigation (up/down/home/end, typeahead, disabled skipping)
  - optional virtualization integration
- Potential runtime/a11y gaps to track:
  - We currently lack a ListBox/Option-style role pair; `List`/`ListItem` works but may not map ideally to OS AT.
  - We do not have an "active descendant" semantics link (to announce the active result while keeping focus in the text field, as cmdk does). If we want true cmdk parity, consider adding a semantics association rather than moving focus away from the input.
  - Virtualized a11y contract is still evolving; avoid virtualization for v1 unless necessary, or define an AT-facing mirror strategy.

## Planned Infra Modules (Concrete)

Intended new building blocks (names tentative):

- `crates/fret-components-ui/src/headless/hover_intent.rs` (tooltip/hover-card delays + cancellation)
- `crates/fret-components-ui/src/headless/menu_nav.rs` (arrow key navigation + typeahead buffer + disabled skipping)
- `crates/fret-components-ui/src/headless/focus_trap.rs` (dialog/sheet focus trap + restore hooks)
- `crates/fret-components-ui/src/declarative/separator.rs` (simple visual + semantics)
- `crates/fret-components-ui/src/declarative/scroll_area.rs` (Scroll + scrollbar styling wrapper)
- `crates/fret-components-ui/src/declarative/textarea.rs` (runtime `TextArea` chrome wrapper)
- `crates/fret-ui/src/slider.rs` (pointer/keyboard input; a11y TBD)
- Extend `crates/fret-components-ui/src/window_overlays.rs` with: tooltip layer, menu layer, dialog/sheet layer, toast layer

Cross-cutting a11y constraint to keep in mind:

- Roving-focus “items” often should be *not* in Tab traversal, but still AT-focusable/activatable; ensure semantics focusability is not accidentally tied to Tab-stop (see `Pressable` semantics behavior).

## Reference: gpui-component Layering (Upstream Inspiration)

`repo-ref/gpui-component` is a useful comparison point:

- GPUI provides mechanisms like `DismissEvent`, `anchored(...)` placement, focus handles, and deferred overlays.
- gpui-component implements policy and styling at the component layer (`Popover::overlay_closable`, tooltip styling, input popovers, etc).

This matches Fret’s intended split: `fret-ui` as mechanism; `fret-components-ui`/`fret-components-shadcn` as policy + composition.

## Tracking Table (Update as work proceeds)

| Area | Component | Status | Owner crate | A11y | Tests | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Boundary | Retained API hidden | Done | `fret-ui` | - | - | `widget` + `create_node` are crate-private |
| Infra | Declarative tree | Done | `fret-components-ui` | Partial | Partial | Expand with roving focus + typeahead helpers |
| Primitives | Button | Present | `fret-components-shadcn` | Partial | Not started | Style parity + a11y checklist still pending |
| Primitives | Input | Present | `fret-components-shadcn` | Partial | Not started | Uses runtime `TextInput` semantics + theming |
| Overlays | Select | Present | `fret-components-shadcn` | Partial | Partial | Uses `fret-components-ui/window_overlays` dismissible popover |
