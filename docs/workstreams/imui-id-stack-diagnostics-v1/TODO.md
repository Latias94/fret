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

## M3 - Diagnostics Query Surface

- [x] Add `fretboard diag query identity-warnings` as the bounded triage entry point for
  `debug.element_runtime.identity_warnings`.
- [x] Support focused filters for warning kind, window, element, list id, element path, and source
  file.
- [x] Keep the query read-only over existing bundle snapshots; do not expand script capabilities or
  public IMUI identity APIs.
- [x] Add focused `fret-diag` gates for handler behavior, clap contract parsing, and cutover
  command conversion.

## M4 - IMUI Duplicate-Key Authoring Proof

- [x] Refactor `ImUi::for_each_keyed` to delegate to `ElementContext::for_each_keyed` instead of
  manually looping through `ui.id(...)`.
- [x] Align `ImUi::for_each_keyed` with the runtime keyed-list shape:
  `items: &[T]`, `key(&T) -> K`, `f(ui, index, &T)`.
- [x] Prove duplicate keyed-list hashes emit structured diagnostics from the IMUI authoring callsite.
- [x] Keep `ui.id(...)` / `ui.push_id(...)` available for explicit subtree identity outside list
  iteration.

## M5 - Closeout Readiness

- [ ] Record final gates and evidence.
- [ ] Decide whether a full ID-stack browser, label-to-`test_id` inference, or table column
  identity deserves separate follow-ons.
- [ ] Close or downgrade the lane to maintenance once the structured diagnostics slice is stable.
