# Fret Node Low-Level Adapter v1 - TODO

Status: Active
Last updated: 2026-05-25

## NLA-M0 - Adapter Target Audit

- [ ] NLA-010 [owner=planner] [deps=none] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Catalog remaining direct retained context families and pick the first adapter seam.
  Validation: `rg -n "EventCx|LayoutCx|PaintCx|PrepaintCx|SemanticsCx|CommandCx|Widget" ecosystem/fret-node/src/ui/canvas/widget`
  Evidence: `docs/workstreams/fret-node-low-level-adapter-v1/HANDOFF.md`
  Handoff: Choose one behavior family, not the whole canvas.

## NLA-M1 - First Adapter Proof

- [ ] NLA-020 [owner=unassigned] [deps=NLA-010] [scope=ecosystem/fret-node/src/ui/canvas]
  Goal: Move one retained context family behind a named node adapter seam.
  Validation: `cargo check -p fret-node --features compat-retained-canvas`
  Evidence: changed adapter files and focused tests.
  Handoff: Keep default `fret-node` off retained authoring.

## NLA-M2 - Delete Or Quarantine One Retained Edge

- [ ] NLA-030 [owner=unassigned] [deps=NLA-020] [scope=ecosystem/fret-node/src/ui/canvas/widget]
  Goal: Delete or quarantine the old retained edge replaced by the first adapter proof.
  Validation: `cargo check -p fret-node`; `cargo check -p fret-node --features compat-retained-canvas`
  Evidence: source-policy test in `ecosystem/fret-node/src/lib.rs`
  Handoff: Split follow-ons per behavior family.
