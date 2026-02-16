# Radix Primitives Audit — Tooltip


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned tooltip substrate against the upstream Radix
`@radix-ui/react-tooltip` implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/tooltip/src/tooltip.tsx`
- Public exports: `repo-ref/primitives/packages/react/tooltip/src/index.ts`

Key upstream concepts:

- `Tooltip` root supports controlled/uncontrolled open state (`open` / `defaultOpen`) and
  `onOpenChange`.
- Provider-scoped delay group behavior (`TooltipProvider`) coordinates open delays across tooltips.
- Tooltip content is rendered in a portal and anchored via Popper.

## Fret mapping

Fret models Radix tooltip outcomes by composing:

- Delay group state machine: `ecosystem/fret-ui-kit/src/headless/tooltip_delay_group.rs`
- Provider scoping: `ecosystem/fret-ui-kit/src/tooltip_provider.rs`
- Hover intent state machine: `ecosystem/fret-ui-kit/src/headless/hover_intent.rs`
- Pointer “safe hover” corridor math: `ecosystem/fret-ui-kit/src/headless/safe_hover.rs`
- Placement: `ecosystem/fret-ui-kit/src/primitives/popper.rs` (+ `popper_content.rs`)
- Portal rendering: per-window overlay roots via `ecosystem/fret-ui-kit/src/window_overlays/*`
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`
- shadcn recipe wiring (reference implementation): `ecosystem/fret-ui-shadcn/src/tooltip.rs`

## Current parity notes

- Pass: Provider-scoped delay group behavior is available via `primitives::tooltip` re-exports.
- Pass: Tooltip overlays can be requested with a stable root name via
  `primitives::tooltip::tooltip_request(...)` and `request_tooltip(...)`.
- Pass: Controlled/uncontrolled open modeling is available via
  `primitives::tooltip::tooltip_use_open_model(...)` (backed by the shared controllable-state
  substrate).
- Pass: Hover open/close and blur-close behavior are modeled via a reusable primitives helper:
  `primitives::tooltip::tooltip_update_interaction(...)` (built on `HoverIntentState` +
  provider delay-group policy).
- Pass: Hoverable-content grace behavior (`disableHoverableContent` outcome) is modeled via a
  deterministic safe-hover corridor between trigger and content, wired via:
  - `primitives::tooltip::tooltip_last_pointer_model(...)`
  - `primitives::tooltip::tooltip_install_pointer_move_tracker(...)`
  - `primitives::tooltip::tooltip_update_interaction(...)`
  (shadcn tooltip uses this wiring as a reference recipe).
- Pass: Opening a tooltip closes other tooltips in the same provider scope (Radix `TOOLTIP_OPEN`
  broadcast outcome), implemented via `tooltip_provider::note_opened_tooltip(...)` and
  `primitives::tooltip::tooltip_update_interaction(...)`.
- Pass: Trigger pointer gating and close-on-pointerdown/click outcomes are modeled in the shadcn
  tooltip recipe (`ecosystem/fret-ui-shadcn/src/tooltip.rs`):
  - Hover-open is gated behind the first non-touch `pointermove` (Radix `hasPointerMoveOpenedRef`),
    tracked via a lightweight `PointerRegion` wrapper (pointer moves do not stop propagation for
    Pressables).
  - `pointerdown` requests a close and suppresses focus-driven re-open (Radix `isPointerDownRef` +
    `onPointerDown` close behavior), wired via `ElementContext::pressable_add_on_pointer_down_for`.
  - Click/keyboard activation requests a close and suppresses focus-driven re-open (Radix `onClick`
    close outcome), wired via `ElementContext::pressable_add_on_activate_for`.
- Pass: Outside-press dismissal is supported via `DismissibleLayer` observer routing (Radix
  `onPointerDownOutside` + `onDismiss` outcomes). The shadcn tooltip recipe installs a dismiss
  handler on its overlay request (`OverlayRequest.dismissible_on_dismiss_request`) so an outside
  press requests close without blocking underlay input.
- Pass: Escape-to-dismiss routes to the topmost overlay root, matching Radix's "only the highest
  layer handles it" outcome (global arbitration in `crates/fret-ui/src/tree/dispatch.rs`).
- Pass: Scroll-to-dismiss is supported for tooltip overlays by wiring the tooltip trigger node as a
  scroll-dismiss descendant on the overlay layer. When a wheel event scrolls an ancestor of the
  trigger, the tooltip requests dismissal (Radix scroll listener closes when
  `event.target.contains(trigger)`).
- Pass: Provider-scoped pointer-in-transit suppression is modeled via a provider model:
  - The currently open tooltip publishes a transit corridor geometry via
    `tooltip_provider::set_pointer_transit_geometry(...)`.
  - Other tooltip triggers consult `tooltip_provider::pointer_transit_geometry_model(...)` to
    avoid setting the "pointermove opened" gate while the pointer lies inside that corridor
    (Radix `isPointerInTransitRef`).

## Follow-ups (recommended)

- Consider auditing remaining tooltip content dismissal/focus edge cases if strict behavioral
  matching becomes a goal outside of the current shadcn recipes.

## Conformance gates

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` validates tooltip placement (popper
  gap + cross-axis delta) against the Radix Vega web golden
  (`goldens/radix-web/v4/radix-vega/tooltip-example.tooltip.hover-show-hide.light.json`).
- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` validates hover open/close and
  Escape dismissal outcomes against the Radix Vega web goldens.
- Run: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
