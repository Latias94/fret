# ImUi Key Owner Surface v1 - TODO

Status: closed closeout record
Last updated: 2026-04-21

Status note (2026-04-21): this lane closes after M2 concluded that the current helper-local
shortcut seams are sufficient for the current first-party key-owner demand. Do not continue
additive key-owner surface growth in this folder by default.

## Lane setup

- [x] Create the lane as a narrow follow-on under the immediate-mode product-closure umbrella.
- [x] Wire the lane into `docs/roadmap.md`, `docs/workstreams/README.md`, `docs/todo-tracker.md`,
      and the umbrella status docs.
- [x] Freeze that this lane does not widen `crates/fret-ui`, reopen lifecycle vocabulary, reopen
      collection/pane proof work, or own richer menu/tab policy.

## M0 - Baseline and owner freeze

- [x] Write one baseline audit that re-reads:
      - the umbrella P0 parity status,
      - the Dear ImGui parity audit,
      - the current focused shortcut / command metadata seams,
      - the lifecycle closeout,
      - and the collection/pane closeout.
      Result: `M0_BASELINE_AUDIT_2026-04-21.md`.
- [x] Freeze the default owner split for this lane.
      Result: `DESIGN.md` now keeps `fret-ui-kit::imui` as the additive facade owner,
      `fret-imui` as the behavior-floor owner, `apps/fret-examples` as the proof/source-policy
      owner, and `crates/fret-app` / `crates/fret-runtime` as the fixed global keymap/command
      owner without reopening runtime growth.
- [x] Name the smallest current repro/gate/evidence package instead of leaving the lane open-ended.
      Result: `EVIDENCE_AND_GATES.md` now freezes the current first-open proof/contract demo plus
      the targeted `fret-imui` shortcut floor and lane-local source-policy gate.

## M1 - Proof roster and scope freeze

- [x] Freeze one current first-open proof/contract surface for this lane.
      Result: `M1_PROOF_ROSTER_FREEZE_2026-04-21.md` now keeps
      `apps/fret-examples/src/imui_response_signals_demo.rs` as the current proof/contract
      surface and explicitly rejects promoting a new dedicated key-owner proof demo at M1.
- [x] Freeze one current executable shortcut floor for this lane.
      Result: `M1_PROOF_ROSTER_FREEZE_2026-04-21.md` now keeps a bounded 11-test targeted
      `fret-imui` shortcut package instead of a vague full-suite dependency.
- [x] Freeze the explicit defer list for this lane:
      - `ResponseExt` lifecycle vocabulary,
      - collection/pane proof breadth,
      - broader menu/tab policy,
      - runtime keymap / IME arbitration,
      - and runner/backend multi-window parity.
      Result: `M1_PROOF_ROSTER_FREEZE_2026-04-21.md` now keeps that defer list explicit and the
      lane-local source-policy gate locks the owner split against drift.

## M2 - First key-owner slice or no-new-surface verdict

- [x] Decide whether to ship additive immediate key-owner surface beyond the current
      helper-local `activate_shortcut` and command-metadata seams.
      Result: `M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md` now closes M2 on a no-new-surface verdict:
      the current helper-local `activate_shortcut` + `shortcut_repeat` +
      `button_command` / `menu_item_command` seams stay the shipped answer for this cycle.
- [x] Land only the helper widening or demo proof that is directly justified by first-party
      evidence.
      Result: `M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md` now explicitly rejects helper widening
      because the current `imui_response_signals_demo` + targeted 11-test floor still do not
      justify a broader item-local shortcut registration seam.
- [x] Add or refine focused gates once the first bounded verdict is explicit.
      Result: `apps/fret-examples/src/lib.rs` now exposes
      `immediate_mode_key_owner_surface_m2_no_new_surface_verdict_is_explicit`, and the lane
      source-policy gate now locks the M2 verdict plus closeout state against drift.

## M3 - Closeout or split again

- [x] Close the lane if the owner split and first verdict become explicit enough.
      Result: `CLOSEOUT_AUDIT_2026-04-21.md` now closes the lane and reclassifies this folder as a
      closeout record.
- [x] Split a different narrow follow-on instead of widening this lane if the remaining pressure
      becomes mostly:
      - broader menu/tab policy,
      - runtime keymap / IME arbitration,
      - or heavier editor/shell product proof.
      Result: `CLOSEOUT_AUDIT_2026-04-21.md` now keeps lifecycle vocabulary, collection/pane proof
      breadth, broader menu/tab policy, runtime keymap / IME arbitration, and runner/backend
      parity outside this closed folder. Future immediate key-owner pressure now requires stronger
      first-party proof in a different narrow lane.
