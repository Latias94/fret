# Material 3 Exposed Dropdown Diagnostics Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The component matrix still left `ExposedDropdown` in known-follow-on state even though the recipe
already inherited Autocomplete field/listbox selectors and focused Rust gates covered committed
selection, editable query, blur synchronization, and trailing icon overlay toggling. The missing
evidence was a dedicated promoted diagnostics packet that proved the gallery filtering popup path
against the stable dotted selectors.

## Truth

- `ExposedDropdown` is a Material recipe composition over `Autocomplete`, not a new mechanism.
- The recipe owns committed selection, editable query, trailing dropdown icon, and blur-time query
  synchronization to the committed label.
- Autocomplete/TextField field foundation owns the shared field chrome and active-indicator
  surfaces that `ExposedDropdown` inherits.
- Diagnostics should prove filtering keeps only matching options mounted, keeps the listbox within
  the window, and closes the popup through the trailing icon on the dedicated Material3 gallery
  surface.

## Boundaries

- Do not move committed-selection/query synchronization into `fret-ui-kit`; it is a Material
  exposed-dropdown policy until another design system needs the exact same contract.
- Do not widen `crates/*`; no runtime mechanism gap is proven by this packet.
- Keep page/container width negotiation caller-owned; this packet only promotes the filtering and
  popup choreography gate.
