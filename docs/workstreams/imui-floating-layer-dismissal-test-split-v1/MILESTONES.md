# ImUi Floating Layer Dismissal Test Split v1 Milestones

Status: Closed
Last updated: 2026-06-06

## M0 - Split Target

- [x] Confirm `layer_dismissal.rs` contained separate menu and popover outside-press proofs.
- [x] Preserve behavior coverage while removing the aggregate test bodies from the hub.

## M1 - Landed Split

- [x] Keep `layer_dismissal.rs` as a module-only hub.
- [x] Move menu outside-press dismissal coverage into `layer_dismissal/menu.rs`.
- [x] Move popover click-through dismissal coverage into `layer_dismissal/popover.rs`.
- [x] Keep the focused `floating::layer_dismissal` filter as the first repro surface.

## M2 - Closeout

- [x] Record the closed lane and evidence gates.
- [x] Add source-gate anchors for the proof owner boundary.
- [x] Update the workstream catalog.
