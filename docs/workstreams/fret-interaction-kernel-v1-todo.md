# Fret Interaction Kernel (v1) — TODO

This is a working checklist for the `fret-interaction` kernel workstream.

## M0 — Contract and scaffolding

- [ ] Create `ecosystem/fret-interaction` crate with initial module boundaries.
- [ ] Document mapping conventions (pan/zoom; screen <-> world) and name them in the API.
- [ ] Decide what belongs in `fret-interaction` vs `fret-ui-kit` vs `fret-node`.
- [ ] Add a minimal unit test suite for math helpers (round-trips, sanitization).

## M1 — `imui` floating windows migration

- [ ] Move drag threshold math from `ecosystem/fret-ui-kit/src/imui.rs` into `fret-interaction`.
- [ ] Move device-pixel snapping helpers (if still needed) into a shared `dpi` module.
- [ ] Migrate title bar drag and resize handle gesture code to shared state machines.
- [ ] Add/extend `fretboard diag` gates:
  - [ ] title bar drag screenshots + stale paint check
  - [ ] fractional DPI wrapping + no overlap

## M2 — `fret-node` viewport / interaction migration

- [ ] Replace duplicated viewport math with `fret-interaction::viewport`.
- [ ] Ensure existing conformance tests still pass:
  - `viewport_helper_conformance`
  - `viewport_animation_conformance`
- [ ] Keep XyFlow parity knobs in `fret-node` (kernel only supplies primitives).

## M3 — Docking/multi-window integration

- [ ] Identify which parts belong in the kernel vs runner/docking policy.
- [ ] Add at least one diag repro for multi-window hover arbitration while dragging.

