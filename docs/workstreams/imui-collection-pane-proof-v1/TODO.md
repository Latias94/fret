# ImUi Collection + Pane Proof v1 - TODO

Status: closed closeout record
Last updated: 2026-04-21

Status note (2026-04-21): this lane closes after the shipped M2/M3 proof pair showed that the
current helper surface is sufficient. Do not continue helper widening or narrower pane-only
proof/diag growth in this folder by default.

## Lane setup

- [x] Create the lane as a narrow follow-on under the immediate-mode product-closure umbrella.
- [x] Wire the lane into `docs/roadmap.md`, `docs/workstreams/README.md`, `docs/todo-tracker.md`,
      and the umbrella status docs.
- [x] Freeze that this lane does not widen `crates/fret-ui`, reopen shell-helper promotion, or own
      key-owner semantics.

## M0 - Baseline and owner freeze

- [x] Write one baseline audit that re-reads:
      - the umbrella P0 parity status,
      - the Dear ImGui parity audit,
      - the current `multi_select` / `child_region` seams,
      - the current shell closeout,
      - and the current proof demos.
      Result: `M0_BASELINE_AUDIT_2026-04-21.md`.
- [x] Freeze the default owner split for this lane.
      Result: `DESIGN.md` now keeps `fret-ui-kit::imui` as the helper owner,
      `fret-imui` as the interaction-floor owner,
      `fret-ui-editor` as the editor-composite owner,
      `apps/fret-examples` as the proof-demo owner,
      and `fret-workspace` as the shell mounting owner without reopening helper promotion.
- [x] Name the smallest current repro/gate/evidence package instead of leaving the lane open-ended.
      Result: `EVIDENCE_AND_GATES.md` now freezes the current baseline pair
      (`imui_editor_proof_demo` + `workspace_shell_demo`) and the existing gate floor.

## M1 - Proof roster and scope freeze

- [x] Freeze one collection-first proof surface for this lane.
      Result: `M1_PROOF_ROSTER_FREEZE_2026-04-21.md` now keeps
      `apps/fret-examples/src/imui_editor_proof_demo.rs` as the current collection-first proof
      surface and explicitly rejects creating a dedicated asset-grid/file-browser proof demo yet.
- [x] Freeze one pane-first proof surface for this lane.
      Result: `M1_PROOF_ROSTER_FREEZE_2026-04-21.md` now keeps
      `apps/fret-examples/src/workspace_shell_demo.rs` as the current pane-first proof surface and
      `apps/fret-examples/src/editor_notes_demo.rs` as the supporting minimal pane rail proof,
      without introducing a narrower child-region-only proof demo yet.
- [x] Freeze the explicit defer list for this lane:
      - key ownership,
      - promoted shell helpers,
      - runner/backend multi-window parity,
      - and broader menu/tab policy.
      Result: `M1_PROOF_ROSTER_FREEZE_2026-04-21.md` now keeps those owner splits explicit and the
      lane-local source-policy gate locks them against drift.

## M2 - Collection-first proof closure

- [x] Build a first-party asset-grid / file-browser proof over the current immediate stack.
      Result: `apps/fret-examples/src/imui_editor_proof_demo.rs` now carries an in-demo
      collection-first asset browser proof with stable keyed tiles, visible-order flipping,
      selected-set drag/drop, and an app-owned import target.
- [x] Decide whether marquee / box-select bridging is required and, if so, whether it belongs in:
      - `fret-ui-kit::imui`,
      - `fret-imui` proof/runtime glue,
      - or example/app composition.
      Result: `M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md` now keeps marquee / box-select deferred
      for M2 instead of widening helper/runtime scope prematurely.
- [x] Add focused interaction proof for:
      - multi-select breadth beyond row lists,
      - drag/drop on selected collections,
      - and collection selection persistence under explicit keyed identity.
      Result: `ecosystem/fret-imui/src/tests/interaction.rs` now exposes
      `collection_drag_payload_preserves_selected_keys_across_order_flip`, and
      `apps/fret-examples/src/lib.rs` now locks the new source-policy markers.
- [x] Add one lane-local source-policy gate once the collection-first proof surface is frozen.
      Result: `apps/fret-examples/src/lib.rs` now exposes
      `immediate_mode_workstream_freezes_the_p0_p1_collection_pane_proof_follow_on`.

## M3 - Pane-first proof closure

- [x] Build a pane composition proof that exercises:
      - nested `child_region` usage or an accepted successor,
      - toolbar/status/tab/inspector style composition,
      - and a shell-mounted proof without reopening helper promotion.
      Result: `apps/fret-examples/src/workspace_shell_demo.rs` now mounts a shell-owned immediate
      pane proof with nested toolbar / tabs / inspector / status regions inside pane content.
- [x] Decide whether `child_region` helper widening is actually required or whether the current
      explicit container story is sufficient after better proof assembly.
      Result: `M3_PANE_PROOF_CLOSURE_2026-04-21.md` now keeps
      `ecosystem/fret-ui-kit/src/imui/child_region.rs` unchanged for M3 because the current seam
      is sufficient for the pane proof.
- [x] Add one pane-focused gate package and, if needed, one launched diagnostics smoke path.
      Result: `apps/fret-examples/src/lib.rs` now exposes
      `immediate_mode_collection_pane_proof_m3_pane_first_workspace_shell_slice_is_explicit`,
      `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs` now locks the source
      markers, and the lane reuses the existing workspace shell diag floor instead of adding a
      narrower pane-only diagnostics path.

## M4 - Helper decision or closeout

- [x] Close the lane if the proof pair shows the current helper surface is sufficient after better
      first-party assembly and documentation.
      Result: `CLOSEOUT_AUDIT_2026-04-21.md` now closes the lane on the shipped M2/M3 proof pair
      and reclassifies this folder as a closeout record.
- [x] Land only the helper widening that is directly justified by the proof pair.
      Result: `CLOSEOUT_AUDIT_2026-04-21.md` now records the M4 no-widening verdict:
      `ecosystem/fret-ui-kit/src/imui/child_region.rs` stays unchanged, and the lane does not add
      a narrower pane-only demo or diagnostics path.
- [x] Split a different narrow follow-on instead of widening this lane if the remaining pressure
      becomes mostly:
      - key ownership,
      - promoted shell helper extraction,
      - or richer menu/tab policy.
      Result: `CLOSEOUT_AUDIT_2026-04-21.md` now keeps those topics assigned to separate narrow
      lanes instead of reopening this folder.
