# Radix Primitives Audit — DismissableLayer

This audit compares Fret's DismissableLayer-aligned substrate against the upstream Radix
`@radix-ui/react-dismissable-layer` implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/dismissable-layer/src/dismissable-layer.tsx`
- Public exports: `repo-ref/primitives/packages/react/dismissable-layer/src/index.ts`

Key upstream concepts:

- `DismissableLayer` listens for:
  - Escape key (only the highest layer handles it),
  - `pointerdown` outside,
  - focus moving outside.
- Outside interactions can be intercepted by calling `event.preventDefault()` in:
  - `onEscapeKeyDown`,
  - `onPointerDownOutside`,
  - `onFocusOutside`,
  - `onInteractOutside`.
  If not prevented, Radix calls `onDismiss()`.
- `DismissableLayerBranch` marks additional subtrees as "inside" for outside-press and
  focus-outside checks (e.g. a trigger element outside the portal subtree).
- `disableOutsidePointerEvents` makes underlay widgets inert while a layer is present by forcing
  `pointer-events: none` below the highest such layer. This yields the "click twice" behavior:
  the first click closes the layer; the second click activates the underlay widget.

## Fret mapping

Fret does not have DOM capture/bubble, but it models the same outcomes by composing:

- Outside-press observation pass (ADR 0069) + per-layer configuration:
  - `consume_outside_pointer_events` (whether outside presses are swallowed vs click-through),
  - `disable_outside_pointer_events` (whether underlay input is blocked while open).
- Escape routing and dismiss semantics:
  - `ElementContext::dismissible_on_dismiss_request(...)` (policy hook),
  - `OnDismissRequest(host, ActionCx, DismissReason)` (portable "preventDefault" analogue).
  - Global Escape arbitration: the runtime routes Escape to the topmost overlay root (matching the
    Radix "only the highest layer handles it" outcome): `crates/fret-ui/src/tree/dispatch.rs`.
- Branches:
  - `dismissable_branches: Vec<GlobalElementId>` on overlay requests, resolved to `NodeId`s for
    the outside-press observer pass.

Primary code entry points:

- Primitives facade: `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs`
- Overlay orchestration: `ecosystem/fret-ui-kit/src/window_overlays/*`
  - Requests: `ecosystem/fret-ui-kit/src/window_overlays/requests.rs`
  - Render/policy: `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- Branch resolution helper: `resolve_branch_nodes_for_trigger_and_elements(...)` in
  `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs`

## Parity notes (outcome-level)

### Dismiss requests and "preventDefault"

Radix outcome:

- Outside interaction produces a *dismiss request*; user handlers can `preventDefault()` to block
  the dismissal.

Fret mapping:

- Overlays install an optional `OnDismissRequest` handler on the layer root (via
  `dismissible_on_dismiss_request`).
- If a handler is present, Fret does not close models automatically. The handler decides whether
  to close the `open` model, implementing the same outcome as "preventDefault".
  - Focus-outside dismissal routes through the same handler with `DismissReason::FocusOutside`.

This is intentionally the only stable contract; it avoids encoding the handler inside option
structs that want `Copy/Eq` semantics.

### Outside press: click-through vs consume

Radix outcome:

- `onPointerDownOutside` fires on `pointerdown` outside, and if not prevented, `onDismiss()` runs.
- The underlying DOM click usually does not "activate" an underlay element when
  `disableOutsidePointerEvents=true`.

Fret mapping:

- Use `consume_outside_pointer_events=true` for menu-like overlays (close without activating
  underlay widgets).
- Use `consume_outside_pointer_events=false` for popover/tooltip-like overlays (close and allow
  the underlay widget to receive the event).

### disableOutsidePointerEvents and modality

Radix outcome:

- When `disableOutsidePointerEvents=true`, pointer events outside the layer subtree are disabled
  until the layer is dismissed, matching "modal-like" behavior.

Fret mapping:

- `disable_outside_pointer_events=true` enables pointer occlusion for the overlay layer:
  `PointerOcclusion::BlockMouseExceptScroll`.
  - Mouse hover/move/down/up are prevented from reaching underlay widgets while the overlay is
    open.
  - Wheel events are still allowed to route to the underlay scroll target (editor ergonomics, GPUI
    alignment).
- Fully modal overlays still use `blocks_underlay_input=true` (barrier-backed layers). Modal barrier
  dismissal is routed through the same `OnDismissRequest` contract to preserve preventDefault
  semantics (see `primitives::dialog` / `primitives::select`).

### Branches (DismissableLayerBranch)

Radix outcome:

- Branch subtrees count as "inside" for pointerdown-outside and focus-outside checks, even if they
  live outside the portal subtree.

Fret mapping:

- Overlay requests accept `dismissable_branches: Vec<GlobalElementId>`.
- The overlay controller resolves them to `NodeId`s (ignoring missing nodes), dedupes while
  preserving order, and registers them with the outside-press observer pass.

## Recommended usage (Fret)

- Prefer using the Radix-named primitives facades (`ecosystem/fret-ui-kit/src/primitives/*`) as the
  boundary when authoring shadcn recipes.
- If a component needs "preventDefault" (e.g. to keep an overlay open while showing a validation
  error), pass an `OnDismissRequest` down to the overlay request and any modal barrier element so
  all dismissal paths are routed through the same contract.
- Use `consume_outside_pointer_events` intentionally:
  - `false` for tooltip/popover-style click-through overlays,
  - `true` for menu/select-style overlays where outside clicks should not activate the underlay.

## Gaps / intentional differences

- Fret models `disableOutsidePointerEvents` via per-overlay pointer occlusion (not a global
  `body { pointer-events: none }` equivalent).
- DOM-specific details (capture/bubble ordering, touch delayed click) are not mirrored directly;
  only the observable outcomes (dismiss request routing + underlay blocking/consumption) are
  treated as alignment targets.
