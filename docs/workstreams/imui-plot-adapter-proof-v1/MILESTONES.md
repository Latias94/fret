# IMUI Plot Adapter Proof v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Boundary Locked

Exit criteria:

- The lane exists as a narrow follow-on from the editor workbench golden path.
- The design names `fret-plot` as the owner and forbids `fret-imui` / `fret-ui-kit` plot widening.

## M1 - Optional Adapter Compiles

Exit criteria:

- `fret-plot/imui` exists as an opt-in feature.
- `fret_plot::imui` delegates to declarative plot panels through `UiWriter`.
- `cargo check -p fret-plot --features imui` passes.

## M2 - Policy Gate Freezes The Shape

Exit criteria:

- The focused `fret-plot` source-policy test proves the adapter is opt-in and declarative-only.
- `tools/gate_imui_workstream_source.py` prevents accidental plot dependency growth in
  `fret-imui` and `fret-ui-kit`.

## M3 - Product Adoption Decision

Exit criteria:

- A future cookbook or canonical workbench use is either added with its own proof surface, or
  explicitly deferred because the thin crate-local adapter is enough.
