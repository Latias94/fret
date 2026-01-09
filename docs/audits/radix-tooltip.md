# Radix Primitives Audit — Tooltip

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

## Follow-ups (recommended)

- Consider tightening parity with the upstream trigger event model (open-on-pointermove gating,
  pointer-in-transit suppression, close-on-pointerdown/click) if strict behavioral matching becomes
  a goal outside of the current shadcn recipes.
