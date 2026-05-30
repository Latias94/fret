# Material 3 Navigation Drawer Overlay Packet v1

Status: Closed
Last updated: 2026-05-27

## Why This Lane Exists

The closed Material 3 component alignment sweep split NavigationDrawer and ModalNavigationDrawer
into a narrow follow-on. The sweep proved selector seed coverage and found no mechanism gap, but the
broader `material3_headless_navigation_suite_goldens_v1` still fails on stale geometry drift across
NavigationBar, NavigationRail, NavigationDrawer, and ModalNavigationDrawer cases.

This lane classifies that drift and turns the drawer/modal-drawer follow-on into a packet with
explicit owner boundaries, refreshed or repaired visual evidence, and focused overlay/focus/motion
gates.

## Target State

- The navigation headless golden drift is classified as real recipe behavior, stale expectation, or
  unstable harness setup.
- NavigationDrawer and ModalNavigationDrawer have a parity packet covering geometry, selected item
  chrome, scrim/panel layering, focus containment/restore, and stable part IDs.
- Any shared navigation geometry drift is evaluated for Material foundation only when more than one
  component proves the same owner problem.
- Overlay/focus policy remains in `fret-ui-kit` unless a packet proves a reusable policy defect.
- `material3_headless_navigation_suite_goldens_v1` either passes or has a documented replacement
  gate for the unresolved portion.

## Source Precedence

- Material Design 3: visual intent for navigation drawer shape, active indicator, scrim, and modal
  layering.
- Compose Material3: non-DOM drawer state, selected item semantics, modal drawer behavior, and focus
  expectations.
- MUI Material UI: web-facing drawer/modal defaults and browser overlay behavior.
- Base UI and in-tree shadcn/Radix work: headless focus/dismiss patterns and Fret-side selector
  conventions only.

## Layer Ownership

- `ecosystem/fret-ui-material3/src/navigation_drawer.rs`: drawer item recipe, selected pill
  geometry, root/item part IDs, and NavigationDrawer semantics.
- `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs`: modal drawer root/scrim/panel
  composition, motion wiring, and stable overlay part IDs.
- `ecosystem/fret-ui-material3/src/foundation`: shared navigation geometry or motion helpers only
  if the packet proves repeated recipe duplication.
- `ecosystem/fret-ui-kit`: modal overlay request, scrim dismissal, focus trap, focus restore, and
  overlay controller policy.
- `crates/*`: out of scope unless the packet proves a hard mechanism contract gap.

## In Scope

- Golden drift classification for the navigation headless suite.
- NavigationDrawer and ModalNavigationDrawer packet artifact.
- Selector, semantics, overlay, and headless golden gates for drawer/modal drawer.
- Recipe or Material foundation refactors proven by the packet.

## Out Of Scope

- Reworking Tabs/NavigationBar/NavigationRail active-indicator foundation unless this packet proves
  the current golden drift is a shared foundation problem.
- Moving overlay/focus policy into Material recipes.
- Broad gallery redesign or 1:1 upstream source compatibility.

## Closeout Condition

This lane can close when:

- the current navigation golden mismatch is classified,
- drawer/modal-drawer owner boundaries are recorded,
- touched recipe/foundation code has targeted gates,
- navigation goldens pass or the remaining mismatch has a documented replacement/follow-on gate,
- and remaining work is split into a narrower lane instead of reopening the broad component sweep.
