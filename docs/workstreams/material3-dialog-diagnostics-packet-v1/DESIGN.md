# Material 3 Dialog Diagnostics Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The Material3 component matrix still left Dialog in a known-follow-on state. The implementation had
recipe-level scrim/panel/action selectors and Rust gates for focus containment, focus restore, style
overrides, and scrim dismissal, but the UI Gallery did not have a dedicated Material3 Dialog diag
script that exercised the page-level modal path.

## Truth

- Material3 Dialog recipe owns scrim/panel/action part ids, panel chrome, role wiring, and visual
  tokens.
- `fret-ui-kit` overlay policy owns modal barrier installation, focus barrier/trap behavior,
  Escape dismissal, and focus restore.
- Diagnostics should prove the dedicated Material3 Dialog gallery page wires the recipe selectors
  through the kit modal path.
- No `crates/*` mechanism change is justified by the current evidence.

## Boundaries

- Do not move modal policy into `fret-ui-material3`.
- Do not change Dialog recipe code unless diagnostics or Rust gates prove behavior drift.
- Treat this packet as a diagnostics-harness closure for the existing recipe/kit split.
