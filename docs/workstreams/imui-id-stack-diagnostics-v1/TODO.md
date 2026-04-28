# ImUi ID Stack Diagnostics v1 - TODO

Status: active
Last updated: 2026-04-28

## M0 - Tracking

- [x] Start a narrow follow-on instead of reopening closed identity lanes.
- [x] Record assumptions-first scope and non-goals.
- [x] Add the lane to repo-level workstream indexes.

## M1 - Structured Runtime Diagnostics

- [x] Add structured identity diagnostics records for duplicate keyed-list hashes.
- [x] Add structured identity diagnostics records for unkeyed reorder.
- [x] Surface those records through the element runtime diagnostics snapshot.
- [x] Preserve existing debug tracing warnings.
- [x] Add focused `fret-ui` gates.

## M2 - IMUI Authoring Proof

- [x] Add a `fret-imui` diagnostics feature that forwards `fret-ui/diagnostics`.
- [x] Prove `ui.for_each_unkeyed` reorder emits the structured warning via runtime delegation.
- [x] Keep `ui.id` / `ui.for_each_keyed` as the recommended dynamic-list fix.

## M3 - Closeout Readiness

- [ ] Record final gates and evidence.
- [ ] Decide whether a full ID-stack browser, diagnostics query command, IMUI `for_each_keyed`
  duplicate-key proof, label-to-`test_id` inference, or table column identity deserves separate
  follow-ons.
- [ ] Close or downgrade the lane to maintenance once the structured diagnostics slice is stable.
