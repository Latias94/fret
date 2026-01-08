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
- Placement: `ecosystem/fret-ui-kit/src/primitives/popper.rs` (+ `popper_content.rs`)
- Portal rendering: per-window overlay roots via `ecosystem/fret-ui-kit/src/window_overlays/*`
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`

## Current parity notes

- Pass: Provider-scoped delay group behavior is available via `primitives::tooltip` re-exports.
- Pass: Tooltip overlays can be requested with a stable root name via
  `primitives::tooltip::tooltip_request(...)` and `request_tooltip(...)`.
- Pass: Controlled/uncontrolled open modeling is available via
  `primitives::tooltip::tooltip_use_open_model(...)` (backed by the shared controllable-state
  substrate).

