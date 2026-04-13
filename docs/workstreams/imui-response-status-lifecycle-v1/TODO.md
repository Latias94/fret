# ImUi Response Status Lifecycle v1 - TODO

Status: active execution lane
Last updated: 2026-04-13

## Lane setup

- [x] Create the lane as a narrow P0 follow-on under the active immediate-mode product-closure
      umbrella.
- [x] Wire the lane into the umbrella docs, `docs/workstreams/README.md`, `docs/roadmap.md`, and
      `docs/todo-tracker.md`.
- [x] Freeze that this lane does not widen `fret-authoring::Response`, `crates/fret-ui`, or the
      global key-owner model.

## M0 - Baseline and owner freeze

- [x] Write one assumptions-first baseline audit that re-reads:
      - `ResponseExt`,
      - the current transient key inventory,
      - current pressable helpers,
      - the shared response contract,
      - and the current parity audit.
      Result: `M0_BASELINE_AUDIT_2026-04-13.md`.
- [x] Freeze the default owner split for this lane.
      Result: `DESIGN.md` now keeps `fret-authoring::Response` stable, `fret-ui-kit::imui` as the
      lifecycle-vocabulary owner, `fret-imui` as the interaction-test owner, and
      `apps/fret-examples/src/imui_response_signals_demo.rs` as the first-open demo surface.
- [x] Name the smallest first vocabulary target instead of leaving the lane open-ended.
      Result: `DESIGN.md` now narrows the first lifecycle quartet to `activated`, `deactivated`,
      `edited`, and `deactivated_after_edit`.

## M1 - Vocabulary freeze

- [x] Freeze the exact meaning of:
      - `activated`,
      - `deactivated`,
      - `edited`,
      - and `deactivated_after_edit`.
      Result: the first shipped rule is now explicit in code and lane docs:
      click-only controls keep `edited = false`, value-editing controls align `edited` with
      `core.changed` where possible, and `deactivated_after_edit` remains tied to the same active
      session instead of becoming a second change signal.
- [x] Decide how click-only controls versus value-editing controls report the quartet without
      creating a second meaning beside `core.changed`.
      Result: the first landed slice now covers direct pressables, boolean controls, slider, and
      text entry with the click-only vs value-editing split kept explicit.
- [ ] Freeze the explicit defer list for this lane:
      - key-owner semantics,
      - broader collection proof breadth,
      - and menu/tab/pane policy depth.

## M2 - First implementation slice

- [x] Land facade-only `ResponseExt` fields/accessors and any required per-item/session state
      without touching `fret-authoring::Response`.
      Result: `ResponseExt` now exposes `activated`, `deactivated`, `edited`, and
      `deactivated_after_edit`, while `fret-authoring::Response` stays unchanged.
- [x] Reuse the existing transient/per-item harvesting pattern in the current pressable helpers
      instead of inventing a second response transport.
      Result: the first slice now reuses transient event keys plus per-item active-session state
      in `fret-ui-kit::imui` instead of adding a second response transport.
- [x] Extend:
      - `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`,
      - focused `ecosystem/fret-imui` interaction tests,
      - and `apps/fret-examples/src/imui_response_signals_demo.rs`
      so the first slice has one demo, one source-policy gate, and one interaction gate.
      Result: the first slice now lands on direct pressables, boolean controls, slider, and text
      entry, with response smoke plus focused button/checkbox interaction coverage and an expanded
      `imui_response_signals_demo`.

## M3 - Expansion or closeout

- [ ] Expand to menu/tab/combo family only if the first slice lands cleanly and repeated evidence
      says those surfaces need the same lifecycle vocabulary.
- [ ] Start a new narrow follow-on instead of widening this lane if the pressure shifts to key
      ownership or broader proof-depth work.
- [ ] Close this lane once the first lifecycle vocabulary and focused gates are explicit enough to
      stop using the folder as an active execution queue.
