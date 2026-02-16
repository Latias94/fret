# Radix Primitives Coverage & Downshift Plan (Fret)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This document tracks how `fret-ui-kit::primitives` maps to upstream Radix UI Primitives
(`repo-ref/primitives/packages/react/*`), and proposes a downshift plan for achieving a
"one-module-per-Radix-package" facade surface that others can reuse outside the shadcn layer.

## Goal

- Provide a **Radix-named** primitives facade in `ecosystem/fret-ui-kit/src/primitives/*` that is
  easy to discover and reuse.
- Keep primitives **thin** (stable naming + small adapters), pushing logic into:
  - `ecosystem/fret-ui-kit/src/headless/*` (pure logic/state machines), and
  - `ecosystem/fret-ui-kit/src/declarative/*` (ElementContext wiring).
- Keep `ecosystem/fret-ui-shadcn` as the **skin/recipe** layer: tokens, spacing, layout, icon slots,
  and shadcn-friendly ergonomics.

## Current mapping (UI-ish packages)

Radix packages listed here are the "UI-ish" subset (excluding low-level React hooks like `use-*`,
and infrastructure helpers like `primitive`, `portal`, `slot`, etc.).

Note: Even though low-level `use-*` packages are excluded from the main mapping list, Fret provides
a small `controllable_state` helper because it is a common contract shared by many primitives.

### Implemented (direct)

- `accordion` -> `ecosystem/fret-ui-kit/src/primitives/accordion.rs`
- `avatar` -> `ecosystem/fret-ui-kit/src/primitives/avatar.rs`
- `aspect-ratio` -> `ecosystem/fret-ui-kit/src/primitives/aspect_ratio.rs`
- `alert-dialog` -> `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs`
- `arrow` -> `ecosystem/fret-ui-kit/src/primitives/arrow.rs` (facade over `popper_arrow`)
- `checkbox` -> `ecosystem/fret-ui-kit/src/primitives/checkbox.rs`
- `collapsible` -> `ecosystem/fret-ui-kit/src/primitives/collapsible.rs`
- `collection` -> `ecosystem/fret-ui-kit/src/primitives/collection.rs`
- `context-menu` -> `ecosystem/fret-ui-kit/src/primitives/context_menu.rs` (facade over `menu`)
- `dialog` -> `ecosystem/fret-ui-kit/src/primitives/dialog.rs`
- `dismissable-layer` -> `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs`
  - Audit: `docs/audits/radix-dismissable-layer.md`
- `direction` -> `ecosystem/fret-ui-kit/src/primitives/direction.rs`
- `dropdown-menu` -> `ecosystem/fret-ui-kit/src/primitives/dropdown_menu.rs` (facade over `menu`)
- `focus-scope` -> `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs`
- `hover-card` -> `ecosystem/fret-ui-kit/src/primitives/hover_card.rs`
- `label` -> `ecosystem/fret-ui-kit/src/primitives/label.rs`
- `menu` -> `ecosystem/fret-ui-kit/src/primitives/menu/*`
  - Audit: `docs/audits/radix-menu.md`
- `menubar` -> `ecosystem/fret-ui-kit/src/primitives/menubar.rs` (facade over `menu`, plus trigger-row + ArrowLeft/Right switching policy in `primitives/menubar/trigger_row.rs`)
- `navigation-menu` -> `ecosystem/fret-ui-kit/src/primitives/navigation_menu.rs`
- `popover` -> `ecosystem/fret-ui-kit/src/primitives/popover.rs` (facade over window overlays + popper)
- `popper` -> `ecosystem/fret-ui-kit/src/primitives/popper.rs` (+ `popper_content.rs`)
- `presence` -> `ecosystem/fret-ui-kit/src/primitives/presence.rs`
- `progress` -> `ecosystem/fret-ui-kit/src/primitives/progress.rs`
- `portal` -> `ecosystem/fret-ui-kit/src/primitives/portal.rs` (facade over overlay root naming/scoping)
- `radio-group` -> `ecosystem/fret-ui-kit/src/primitives/radio_group.rs`
- `roving-focus` -> `ecosystem/fret-ui-kit/src/primitives/roving_focus_group.rs`
- `scroll-area` -> `ecosystem/fret-ui-kit/src/primitives/scroll_area.rs`
- `separator` -> `ecosystem/fret-ui-kit/src/primitives/separator.rs`
- `slider` -> `ecosystem/fret-ui-kit/src/primitives/slider.rs`
- `select` -> `ecosystem/fret-ui-kit/src/primitives/select.rs`
- `switch` -> `ecosystem/fret-ui-kit/src/primitives/switch.rs`
- `tabs` -> `ecosystem/fret-ui-kit/src/primitives/tabs.rs`
- `toast` -> `ecosystem/fret-ui-kit/src/primitives/toast.rs` (facade over `window_overlays::toast`)
- `toolbar` -> `ecosystem/fret-ui-kit/src/primitives/toolbar.rs` (facade over roving focus + toggle group + separator)
- `toggle` -> `ecosystem/fret-ui-kit/src/primitives/toggle.rs`
- `toggle-group` -> `ecosystem/fret-ui-kit/src/primitives/toggle_group.rs`
- `tooltip` -> `ecosystem/fret-ui-kit/src/primitives/tooltip.rs` (facade over provider + delay-group + popper)
- `visually-hidden` -> `ecosystem/fret-ui-kit/src/primitives/visually_hidden.rs`

### Partially implemented (split / naming drift)

None currently tracked.

### Implemented in shadcn layer (should downshift)

These shadcn components exist today, but their reusable behavior/policy should be moved into
Radix-named primitives facades so non-shadcn consumers can reuse them:

None currently tracked.

### Intentional differences

Radix exposes separate packages for `dropdown-menu`, `context-menu`, and `menubar`. In Fret we
currently share behavior via a single `menu` primitives family and expose three shadcn wrappers:

- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/menubar.rs`

This is good for deduplication, but if we want "one module per Radix package" for discoverability,
we keep thin facades that delegate to `primitives::menu`:

- `ecosystem/fret-ui-kit/src/primitives/dropdown_menu.rs`
- `ecosystem/fret-ui-kit/src/primitives/context_menu.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar.rs`

These facades are intentionally small and mostly re-export shared menu helpers under
Radix package names (for discoverability), for example:

- `*_root_name` (overlay root naming convention)
- `*_sync_root_open_and_ensure_submenu` (submenu model + timer wiring)
- `*_dismissible_request` (menu overlay request policy)
- `wire_*_open_on_*` (trigger keyboard / pointer open policies)

## Downshift proposal (how to match Radix interfaces)

Radix interfaces are authored as composable components (`Root` / `Trigger` / `Content` / ...). In
Fret we can mirror the same **conceptual split** without copying React/DOM mechanics:

1. **Primitives module exposes Radix-named concepts**
   - Options/enums: `Side`, `Align`, `Orientation`, `ActivationMode`, etc.
   - A11y stamping helpers (roles/outcomes): `*_a11y(...)` and `*_semantics(...)`.
   - Policy hooks: close on escape/outside, focus trapping, focus restore, typeahead, etc.
2. **Headless logic stays headless**
   - Presence state machines, safe-hover intent, roving focus math, scroll hide delay, etc.
3. **Declarative wiring stays declarative**
   - `ElementContext` hooks that connect headless logic to runtime events and timers.
4. **shadcn wrappers become pure skin**
   - Spacing, border/radius/shadow tokens, icon slots, typography, shadcn ergonomics.

### Concrete module shapes (example)

For overlay-ish primitives (`dialog`, `popover`, `tooltip`), a practical shape is:

- `pub struct <Xxx>Options { ... }` (Radix prop equivalents; Rust-native naming ok)
- `pub fn <xxx>_placement(...)` (delegates to `popper` / anchored placement)
- `pub fn <xxx>_dismiss_policy(...)` (delegates to `dismissable_layer`)
- `pub fn <xxx>_focus_policy(...)` (delegates to `focus_scope`)
- `pub fn render_<xxx>_root_with_hooks(...)` (delegates to overlay roots; caller supplies content)

This keeps primitives reusable without forcing shadcn's visual layout on consumers.

## Suggested priority

If we want the biggest reuse win with minimal risk:

1. Add thin facades for `dropdown-menu`, `context-menu`, `menubar` (delegating to `menu`).
2. Add `primitives/tooltip.rs` facade (compose provider + delay group + popper; keep skin out).
3. Downshift form controls (`checkbox`, `switch`, `slider`, `select`) as a11y + interaction policy.
