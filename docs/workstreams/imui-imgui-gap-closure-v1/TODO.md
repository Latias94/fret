# ImUi Dear ImGui Gap Closure v1 - TODO

Status: Active
Last updated: 2026-05-06

## P0 - Source Baseline

- [x] Create the dedicated `imui-imgui-gap-closure-v1` workstream.
- [x] Refresh the current-source audit from Fret source and `repo-ref/imgui`.
- [x] Mark `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md` as partially superseded for
      current gap reads.
- [x] Wire the new lane into `docs/workstreams/README.md`, `docs/roadmap.md`, and
      `docs/todo-tracker.md`.
- [x] Run the P0 doc/source gates listed in `EVIDENCE_AND_GATES.md`.
      Result: `json shape`, `workstream catalog`, `imui facade teaching source`,
      `imui workstream source`, `git diff --check`, and `cargo check -p fret-examples-imui`
      all pass.

## P1 - Fearless Cleanup / Deletion Candidates

- [ ] Audit public teaching imports for stale direct `fret_imui::` or `fret_ui_kit::imui::`
      default-path examples.
      First slice landed: `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs` now routes
      `TableSortDirection` through the app-facing `fret::imui::kit` facade, and
      `tools/gate_imui_facade_teaching_source.py` forbids the stale direct kit import from
      returning there.
      Second slice landed: `apps/fret-examples/src/workspace_shell_demo.rs` now routes pane-proof
      IMUI option types through `fret::imui::kit`, and both IMUI source gates forbid direct
      `fret_ui_kit::imui` imports from returning to that default pane-first proof.
- [ ] Identify duplicate helper aliases that can be deleted behind a source-policy gate.
- [ ] Check whether `fret-ui-editor::imui` remains a pure adapter over declarative editor controls.
- [ ] Check large `fret-ui-kit::imui` implementation files for owner splits that can be performed
      without public API changes.
      Candidate surfaced by the first pass: keep `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`
      aligned with the current teaching-source marker set and then use the same gate to audit the
      remaining IMUI teaching surfaces.

## P2 - User-Usable Golden Path

- [ ] Pick the smallest runnable proof that should teach a complete editor panel.
- [ ] Verify the proof includes state, command/action dispatch, editor controls, menu/popup, and
      diagnostic-friendly `test_id`s.
- [ ] Promote missing cookbook/docs references only after the proof runs and source gates pass.

## P3 - Dear ImGui-Class Follow-On Candidates

- [ ] Porting sugar readiness: `SameLine`/item-width/label-ID helpers only if two proof surfaces pay
      the same tax.
- [ ] Diagnostics/devtools readiness: define a Fret equivalent of Demo/Metrics/Debug discoverability.
- [ ] Collection helper readiness: keep app-owned until both proof surfaces require one helper.
- [ ] Child-region depth: reopen only with a concrete `BeginChild()`-style behavior target.
- [ ] Multi-window parity: continue in `docking-multiwindow-imgui-parity`.

## Closeout

- [ ] Add a closeout audit once the first cleanup/refactor slice lands and gates pass.
