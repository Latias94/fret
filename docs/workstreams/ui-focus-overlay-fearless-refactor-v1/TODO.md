# TODO

## Phase A (Reachability hardening)

- [x] Replace outside-press “inside layer” detection with child-edge reachability.
- [x] Replace outside-press branch containment (`is_descendant`) with child-edge reachability.
- [x] Add a retained/view-cache style test where parent pointers are intentionally stale but
      children edges are correct; outside-press behavior must be stable.

## Phase B (Prevent default suppresses focus clearing)

- [x] Record per-dismissible-root “last dismiss request outcome” for the current tick.
- [x] Gate outside-press default focus clearing on `default_prevented` for that same tick.
- [x] Add/extend tests:
  - [x] `prevent_default` keeps focus stable on outside press.
  - [x] non-prevented outside press still clears focus (baseline behavior).

## Phase C (Dispatch snapshot)

- [x] Write `MILESTONES.md`-driven detailed design + data model (`M2_DISPATCH_SNAPSHOT_DESIGN.md`).
- [ ] Introduce a snapshot struct (per window, per frame) capturing:
  - [ ] focus containment relationships
  - [ ] hit-test primitives + transforms
  - [ ] tab stops and traversal order
  - [ ] input handler bindings (IME/text)
- [ ] Add a “snapshot parity” debug view/diagnostic:
  - [ ] compare snapshot containment vs Phase A reachability
  - [ ] report divergences with evidence anchors

