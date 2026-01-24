# Component Authoring Contracts (fret-ui + fret-ui-kit)

This document is a **component-authoring-focused** checklist of the public APIs and structural
contracts that upper-layer code is expected to use.

It complements (but does not replace) the ADRs:

- Runtime contract surface: `docs/runtime-contract-matrix.md`
- Closure index: `docs/ui-closure-map.md`
- Action hooks (policy split): `docs/action-hooks.md`

## Layering (what goes where)

- `crates/fret-ui` (**mechanisms/contracts only**): element tree, layout, hit-test, focus,
  semantics/a11y snapshot, overlay roots/layers, placement solver, scroll/virtualization
  mechanisms, text input/area engines.
- `ecosystem/fret-ui-kit` (**policy helpers + infra**): token-driven styling (`ChromeRefinement`
  / `LayoutRefinement`), headless state machines (roving/typeahead/menu nav), overlay orchestration
  policy, and ergonomic declarative helpers.
- `ecosystem/fret-ui-shadcn` (**recipes/taxonomy**): shadcn/ui-aligned component naming and
  composition; should not introduce new runtime contracts.

If a new surface “needs runtime changes”, treat it as a decision gate: write/update an ADR and add
at least one regression test before expanding usage.

## Public authoring entry points (what upper components should use)

### Declarative authoring context

- `fret_ui::ElementContext` (the primary API surface for building element subtrees)
  - Identity: `ElementContext::scope(...)`, `ElementContext::keyed(...)`
  - Local state: `ElementContext::{with_state, with_state_for}`
  - Model reads + observation: `ElementContext::{observe_model, read_model_ref, get_model_*}`
  - Focus reads: `ElementContext::{focused_element, is_focused_element}`
  - Cross-frame geometry queries: `ElementContext::{last_bounds_for_element, last_visual_bounds_for_element}`
- Element props (mechanism vocabulary): `fret_ui::element::*Props` and `fret_ui::element::LayoutStyle`

### Component-layer ergonomics (recommended defaults)

- Action hook helpers: `fret_ui_kit::declarative::action_hooks::ActionHooksExt`
- Model read+observe helpers: `fret_ui_kit::declarative::ModelWatchExt`
- Collection semantics helpers: `fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt`
- Overlay request facade: `fret_ui_kit::{OverlayController, OverlayRequest, OverlayPresence}`
- Anchoring helpers for overlays: `fret_ui_kit::overlay::*`
- Control chrome wrapper (focus ring + clipping split): `fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props`
- Styling refinements: `fret_ui_kit::{ChromeRefinement, LayoutRefinement, StyledExt, Space, Radius, ColorRef, MetricRef}`
- Unified authoring builder surface (ADR 0175): `fret_ui_kit::{UiExt, UiPatchTarget, UiIntoElement, UiBuilder}`

## Unified authoring builder surface (ADR 0175)

Goal: allow ecosystem and third-party crates to compose components with a single default “dialect”:

- `component.ui().px_3().py_2().w_full().into_element(cx)`

This is an ecosystem-only surface (lives in `fret-ui-kit`) and is expected to evolve faster than
kernel contracts.

### Minimal third-party integration contract

If you want other crates to style/layout your component in a uniform way, implement these traits on
your public component types:

- `UiPatchTarget`: apply aggregated `{ chrome, layout }` patches.
- `UiIntoElement`: render into `AnyElement` (so `UiBuilder::into_element(cx)` works).
- `UiSupportsChrome` / `UiSupportsLayout` (optional): enable the full fluent method set.

Practical pattern (recommended):

1. Keep your type as a plain builder struct that stores fields (`chrome`, `layout`, `children`, etc.).
2. Implement your rendering as an inherent `into_element(self, cx)` method (or `RenderOnce` if needed).
3. Implement `UiPatchTarget` by delegating to your existing `refine_style/refine_layout` methods.

### Children collection rule (ecosystem convention)

Public constructors/setters that accept children should use:

- `children: impl IntoIterator<Item = AnyElement>` (call-site flexible),
- store internally as `Vec<AnyElement>` (stable storage),
- avoid forcing `vec![...]` at the call site.

### What not to do

- Do not install global policy as a side effect of constructing a component (e.g. overlay controllers,
  keymaps, theme registries). Use explicit `install(...)` entrypoints in app-facing crates instead.
- Do not leak kernel-level props types in your public API unless you are intentionally exposing a
  mechanism surface.

## Identity and element-local state (hard-to-change contract)

- `ElementContext::scope(...)` derives stable element identity from the callsite.
- `ElementContext::keyed(key, ...)` derives stable identity from `(callsite, key)` and should be used
  for list-like rendering (virtual list items, menus, etc.).
- `ElementContext::{with_state, with_state_for}` store **element-local, cross-frame** state in the
  runtime store keyed by `(GlobalElementId, TypeId)`.

Practical guidance:

- Use `keyed` whenever the child set can reorder or be filtered.
- Avoid capturing element IDs in long-lived app state unless you also control their lifetime and
  re-derivation strategy (IDs are stable but not global identifiers).

## Models and invalidation (opt-in observation)

Model invalidation is explicit: if you read a model during rendering but do not register
observation, the runtime may not invalidate layout/paint when the model changes.

Recommended patterns:

- Use `fret_ui_kit::declarative::ModelWatchExt`:
  - `cx.watch_model(&model).layout().cloned()`
  - `cx.watch_model(&model).paint().copied()`
- Or use `ElementContext::{read_model_ref, get_model_*}` and pass the correct `Invalidation`.

## Interaction policy via action hooks (ADR 0074)

The runtime provides trigger points; components provide policy.

Runtime hook surfaces (registered during rendering via `ElementContext`):

- `pressable_on_activate` (and `*_add_*`/`*_clear_*`)
- `dismissible_on_dismiss_request`
- `pointer_region_on_pointer_{down,move,up}`
- `roving_on_{active_change,typeahead,navigate}`
- `key_on_key_down_for` (per-element key handling)

Component-layer convenience:

- Prefer `fret_ui_kit::declarative::action_hooks::ActionHooksExt` for common policies:
  - toggle/set models, dispatch commands, close overlays, roving selection writes, typeahead rules.

## Semantics / a11y (what you can set today)

Mechanism vocabulary:

- `fret_ui::element::SemanticsProps` (transparent wrapper for role/label/value/flags)
- `fret_ui::element::PressableA11y` (role + label + selected/expanded/checked + collection position)
- `fret_core::SemanticsRole` (current public role set)
- `SemanticsNode::{active_descendant,pos_in_set,set_size}` (virtualized collection + composite widgets)

Important constraints:

- `PressableProps.focusable` controls Tab traversal stops; roving focus commonly sets `focusable=false`
  while still allowing programmatic focus.
- For large/virtualized collections, stamp collection position via
  `CollectionSemanticsExt::with_collection_position(...)` to preserve AT context.

Known limitations (current role vocabulary):

- There is no dedicated `ListBox`/`Option` role pair in `SemanticsRole` today; list-like widgets
  typically use `List` / `ListItem` (or `Menu` / `MenuItem`) until/if the role set expands.

## Focus and traversal

Mechanism primitives:

- `fret_ui::element::FocusScopeProps` (trap focus traversal within a subtree)
- `fret_ui::element::{RovingFlexProps, RovingFocusProps}` (roving focus mechanism)
- Modal barrier behavior is implemented in the runtime’s multi-root overlay substrate; overlay
  policy should be in components.

Focus state for styling:

- `ElementContext::focused_element()` exposes the currently focused element for the window (as a
  `GlobalElementId`) when it can be mapped from the focused `NodeId`.
- `ElementContext::is_focused_element(id)` is the recommended way to compute “focused” affordances
  (menu item active background, tooltip open-on-focus, focus chrome variants) without introducing
  component-local focus mirrors.
- `fret_ui::element::PressableState` includes `focused: bool` so pressable-based components can
  style against `hovered/pressed/focused` consistently.

Practical guidance:

- Use focus scopes for dialogs/sheets/tab-traps; do not bake “modal policy” into the runtime.
- Keep focus restore rules in component overlay policy (see `window_overlays` and ADR 0069).

## Overflow and clipping (ADR 0088)

`LayoutStyle.overflow` is a **paint + hit-test contract** (ADR 0057, ADR 0063), not a styling knob.
Components should converge on predictable conventions instead of sprinkling ad-hoc `Overflow::Clip`:

- Default primitives remain `Overflow::Visible` to preserve composability (badges, shadows, focus rings).
- Introduce `Overflow::Clip` at the node that owns the rounded chrome (background + border + corner radius).
- Keep focus rings outside the clipped chrome: prefer `Pressable (visible) -> chrome container (clip) -> content`.
- Do not rely on `Overflow::Visible` to "escape" for overlays; install overlays via `OverlayController`/portals.

Recommended helper:

- `fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props`

## Overlays (policy vs mechanism)

- Mechanism: multi-root overlay/layer substrate and modal barrier gating live in `fret-ui`.
- Policy: request queues, dismissal rules, and focus restore live in `fret-ui-kit`.

Recommended entry point:

- Use `OverlayController::{begin_frame, request, render}` to coordinate overlay roots per window.
- Use `fret_ui_kit::overlay::anchor_bounds_for_element(...)` and
  `ElementContext::last_visual_bounds_for_element(...)` for render-transform-aware anchoring.
- If placement anchoring should differ from the interactive trigger, model them explicitly:
  - keep the trigger element for hover/focus/dismissal/restore policies, and
  - accept an optional "anchor element" (`GlobalElementId`) used only for placement bounds.
  (See the shadcn recipe layer: `Popover::anchor_element(...)`, `Tooltip::anchor_element(...)`,
  `HoverCard::anchor_element(...)`.)
- Prefer a layout-only `*Anchor` wrapper type in the recipe layer when upstream exposes an anchor
  surface (e.g. Radix `PopoverAnchor`): it should just wrap a child and expose `element_id()`.

Avoid:

- Depending directly on `fret_ui_kit::window_overlays::*` internals unless you explicitly
  enable `fret-ui-kit/unstable-internals`.

## Scrolling and virtualization

Mechanism primitives:

- `fret_ui::element::ScrollProps` + `fret_ui::element::ScrollAxis`
- `fret_ui::scroll::ScrollHandle` (imperative handle with clamped offset + sizes)
- `fret_ui::element::ScrollbarProps` (mechanism-only scrollbar primitive)
- `fret_ui::virtual_list` + `fret_ui::element::VirtualList*` props/state

Current constraints to account for:

- `ScrollbarProps` supports both vertical and horizontal scrollbars via `ScrollbarAxis`. Dual-axis
  scrolling is expressed as two independent scrollbars.
- `VirtualListState` stores either Y or X scroll state (`offset_y/viewport_h` for vertical lists,
  `offset_x/viewport_w` for horizontal lists).

Component-layer helpers:

- Use `fret_ui_kit::declarative::scroll::*` wrappers for common “overflow scroll + optional
  scrollbar” ergonomics.

## Text input / text area (IME, caret, selection)

Component-facing surfaces:

- `fret_ui::element::{TextInputProps, TextAreaProps}`
- `fret_ui::{TextInputStyle, TextAreaStyle}`

Notes:

- Text engines live in `fret-ui` and are intentionally not exposed as retained widgets.
- IME and accessibility semantics are part of the runtime contract (see ADR 0071 and the acceptance
  checklist in `docs/ime-acceptance-checklist.md`).
- `TextInputProps` supports `placeholder` (render-only; not reflected as the editable value).

## Styling and tokens (component-layer, token-driven)

Recommended building blocks:

- `ChromeRefinement` / `LayoutRefinement` and `StyledExt` for "Tailwind-ish" composition.
- Theme reads should be token-driven (via `Theme` keys), not hard-coded constants.

Feature flags to be aware of:

- `fret-ui-kit/icons`: integrates shared icons (`fret-icons`)
- `fret-ui-kit/recipes`: opinionated higher-level helpers (implies `icons`)
- `fret-ui-kit/unstable-internals`: exposes overlay orchestration internals directly

## Checklist for adding a new component surface

1. **Layering:** ensure it can be expressed via existing `fret-ui` mechanisms + component policy.
2. **Identity:** use `scope`/`keyed` correctly; avoid unstable identity in lists.
3. **State:** store UI-local state via `with_state` or app models; avoid global singletons in
   runtime.
4. **Invalidation:** register observation for every model read (layout vs paint vs hit-test).
5. **Hooks:** implement interaction policy via action hooks + headless helpers.
6. **Semantics:** pick a role, set label/value/flags, and stamp collection metadata where needed.
7. **Overlays:** request through `OverlayController` and anchor via visual bounds when transforms
   matter.
8. **Tests:** add nextest unit/contract tests in the owning crate for the hard behaviors.
