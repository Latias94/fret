# ImUi Facade Internal Modularization v1 - TODO

Status: closed closeout record
Last updated: 2026-04-21

## Lane setup

- [x] Create the lane as a narrow follow-on under the immediate-mode maintenance umbrella.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, `docs/todo-tracker.md`,
      and a minimal source-policy gate.
- [x] Freeze that this lane does not widen `crates/fret-ui`, reopen closed parity lanes, or hide
      new helper behavior inside structural refactors.

## M0 - Baseline and first-slice choice

- [x] Audit the current `fret-ui-kit::imui` hotspot files and freeze why this lane exists.
      Result: `M0_BASELINE_AUDIT_2026-04-21.md`.
- [x] Name the safest first slice and defer the more coupled owners.
      Result: M0 now freezes `options.rs` + `response.rs` as the first slice while deferring
      `interaction_runtime.rs` + `imui.rs`.

## M1 - Options/response structural split

- [x] Keep `options.rs` as a stable outward hub while moving definitions into private owner files.
      Result: `M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md` and `ecosystem/fret-ui-kit/src/imui/options/`.
- [x] Keep `response.rs` as a stable outward hub while moving definitions into private owner files.
      Result: `M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md` and `ecosystem/fret-ui-kit/src/imui/response/`.
- [x] Prove that current first-party build/test surfaces still compile against the unchanged public
      facade.
      Result: `EVIDENCE_AND_GATES.md` now freezes the current gate floor for this slice.

## M2 - Interaction-runtime owner split

- [x] Split `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs` by owner without changing
      current interaction behavior.
      Result: `M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md` and
      `ecosystem/fret-ui-kit/src/imui/interaction_runtime/`.
- [x] Keep hover-query, lifecycle, disabled-scope, and drag bookkeeping reviewable as separate
      internal concerns.
      Result: `interaction_runtime.rs` now re-exports the stable helper surface over
      `models.rs`, `disabled.rs`, `lifecycle.rs`, `hover.rs`, and `drag.rs`.

## M3 - Root facade hub split

- [x] Reduce `ecosystem/fret-ui-kit/src/imui.rs` so it stops acting as one flat hub for
      re-exports, facade glue, and helper-local utilities.
      Result: `M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`,
      `ecosystem/fret-ui-kit/src/imui/facade_support.rs`, and
      `ecosystem/fret-ui-kit/src/imui/floating_options.rs`.
- [x] Keep all current public re-exports stable while making future narrow slices cheaper to
      review.
      Result: the root hub now re-imports the same outward names while sibling modules still read
      the existing `super::...` helper paths.

## M4 - Closeout or split again

- [x] Move the remaining `ImUiFacade` / `UiWriterImUiFacadeExt` writer glue out of the root hub.
      Result: `M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md` and
      `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`.
- [x] Close the lane once the hotspot files are decomposed enough that new narrow follow-ons no
      longer require monolithic edits.
      Result: `CLOSEOUT_AUDIT_2026-04-21.md`.
- [x] Start a different narrow lane instead of widening this folder if future pressure becomes
      primarily about new helper semantics rather than internal structure.
      Result: the closeout rule now freezes this lane as closed and follow-on only.
