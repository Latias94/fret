# Material 3 SearchView State Packet v1 - Milestones

Status: Closed
Last updated: 2026-05-28

## M0 - Source Packet

Exit criteria:

- Compose SearchBarState, docked presentation, full-screen presentation, and back handling are
  summarized with local source anchors.
- Fret ownership is classified by mechanism, kit policy, Material foundation, recipe, and
  diagnostics.

## M1 - Full-Screen Presentation

Exit criteria:

- Docked remains the default.
- Full-screen presentation is explicit and controlled by the same `open` model.
- Escape closes the full-screen overlay through existing overlay policy.

## M2 - Focus And Semantics

Exit criteria:

- Full-screen expanded input receives focus.
- Focus traversal stays in the modal overlay while open.
- Stable selectors avoid duplicate root ids.

## M3 - Gallery And Golden Guard

Exit criteria:

- The existing SearchView gallery surface still works.
- Full-screen presentation has either a focused diagnostic script or a headless/golden guard.

## M4 - Closeout

Exit criteria:

- Targeted Rust gates pass.
- Workstream JSON/catalog checks pass.
- Predictive back and platform back are split only if still needed.

Closeout note: completed on 2026-05-28. Predictive back gesture progress and a generic platform
back event remain out of scope; the shipped slice is explicit full-screen presentation with Escape
collapse and overlay-local focus.
