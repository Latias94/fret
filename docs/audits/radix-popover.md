# Radix Primitives Audit — Popover

This audit compares Fret's Radix-aligned popover substrate against the upstream Radix
`@radix-ui/react-popover` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/popover/src/popover.tsx`
- Public exports: `repo-ref/primitives/packages/react/popover/src/index.ts`

Key upstream concepts:

- `Popover` root owns shared state: `open`, `onOpenChange`, `modal`, and a `contentId`.
- `PopoverTrigger` toggles open and stamps `aria-expanded` + `aria-controls`.
- `PopoverAnchor` optionally overrides the anchor rect for placement.
- `PopoverPortal` + `Presence` implement conditional mounting / `forceMount`.
- `PopoverContent` composes:
  - `Popper.Content` placement,
  - `FocusScope` (optional trap + auto focus hooks),
  - `DismissableLayer` (escape/outside/focus-outside dismissal).

## Fret mapping

Fret does not use React context. Instead, popover behavior is composed via:

- Runtime mechanisms: `crates/fret-ui` (focus traversal, hit-testing, semantics snapshot).
- Overlay portal + dismissal + focus restore: `ecosystem/fret-ui-kit/src/window_overlays/*` via
  `OverlayController` (ADR 0067, ADR 0069).
- Placement: `ecosystem/fret-ui-kit/src/primitives/popper.rs` (+ `popper_content.rs`).
- Presence / `forceMount`-style behavior: `OverlayPresence` + `InteractivityGate` (where needed).
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/popover.rs`.

## Current parity notes

- Pass: Controlled/uncontrolled open modeling is available via `PopoverRoot` (recommended) or
  `popover_use_open_model(...)` (thin helper), backed by the shared controllable-state substrate.
- Pass: `aria-expanded` + `aria-controls` style semantics can be stamped on the trigger via
  `apply_popover_trigger_a11y(...)`.
- Pass: Content uses a dialog-like semantics role (`SemanticsRole::Dialog`) via
  `popover_dialog_wrapper(...)`.
- Pass: Conditional mounting is modeled via `OverlayPresence` (and `forceMount` patterns can be
  expressed by keeping the subtree mounted while gating presence/interactivity).
- Pass: Dismissal (escape/outside/focus-outside) is handled by the shared window overlay policy.
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `popover_request_with_dismiss_handler(...)` and `popover_request_with_anchor_and_dismiss_handler(...)`.
  For the modal variant, `popover_modal_layer_elements_with_dismiss_handler(...)` routes modal
  barrier presses through the same `OnDismissRequest` contract.
- Pass: Custom anchor is supported by treating the anchor element as a dismissable branch and
  using its bounds for placement.
- Pass: The Radix `modal` variant is exposed via `PopoverOptions` (`variant=Modal`) and is wired
  through to the shared modal overlay request mechanism.

## Follow-ups (recommended)

- Pass: A Radix-named `PopoverContent` wiring helper exists for non-shadcn users:
  `popover_request_with_anchor(...)` (or `popover_request_with_anchor_and_dismiss_handler(...)`) for
  `DismissableLayerBranch` alignment and `popover_modal_layer_elements(...)` (or
  `popover_modal_layer_elements_with_dismiss_handler(...)`) for the modal barrier outcome.

## Conformance gates

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` validates popover placement against
  the Radix Vega web golden (`goldens/radix-web/v4/radix-vega/popover-example.popover.open-close.light.json`).
- Run: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
