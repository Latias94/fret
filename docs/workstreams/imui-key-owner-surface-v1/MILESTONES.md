# ImUi Key Owner Surface v1 - Milestones

Status: closed closeout record
Last updated: 2026-04-21

Status note (2026-04-21): this file now records the closed key-owner verdict only. Active
implementation should move to a different narrow lane if fresh evidence exceeds this closeout.

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why key-owner work is a new narrow follow-on instead of reopened
  umbrella backlog,
- the owner split is explicit enough to avoid runtime drift,
- and the lane names one current repro/gate/evidence package.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Current status:

- Closed on 2026-04-21 via `M0_BASELINE_AUDIT_2026-04-21.md`.

## M1 - Proof roster and scope freeze

Exit criteria:

- one current first-open proof/contract surface is named,
- one bounded executable shortcut floor is named,
- and the lane explicitly records what is still deferred.

Primary evidence:

- `DESIGN.md`
- `M1_PROOF_ROSTER_FREEZE_2026-04-21.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`

Current status:

- Closed on 2026-04-21 via `M1_PROOF_ROSTER_FREEZE_2026-04-21.md`.
- `apps/fret-examples/src/imui_response_signals_demo.rs` now stays frozen as the current
  proof/contract surface for this lane.
- The current executable shortcut floor is now frozen as a bounded 11-test targeted `fret-imui`
  package instead of a vague full-suite dependency.
- M1 explicitly rejects promoting:
  a new dedicated key-owner proof demo,
  `imui_editor_proof_demo` as a shortcut/key-owner proof surface,
  or `workspace_shell_demo` as a shortcut/key-owner proof surface.
- The defer list is now explicit and frozen for M1:
  `ResponseExt` lifecycle vocabulary, collection/pane proof breadth, broader menu/tab policy,
  runtime keymap / IME arbitration, and runner/backend multi-window parity all stay out of scope
  unless another narrower lane proves otherwise.

## M2 - First key-owner slice or no-new-surface verdict

Exit criteria:

- the lane either lands one bounded additive key-owner surface or closes on a no-new-surface
  verdict,
- any helper growth is directly tied to first-party proof,
- and the resulting gate package stays reviewable.

Primary evidence:

- `M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md`
- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `CLOSEOUT_AUDIT_2026-04-21.md`

Current status:

- Closed on 2026-04-21 via `M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md`.
- The current helper-local `activate_shortcut` + `shortcut_repeat` + `button_command` /
  `menu_item_command` seams now remain the shipped immediate key-owner answer for this cycle.
- M2 explicitly rejects a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade and
  a broader item-local shortcut registration seam.
- The repo still does not have stronger first-party consumer pressure beyond the current
  `imui_response_signals_demo` + targeted 11-test floor.

## M3 - Closeout or split again

Exit criteria:

- the lane either closes with an explicit owner split and first verdict,
- or splits again because the remaining pressure belongs to another narrower owner/problem.

Primary evidence:

- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `CLOSEOUT_AUDIT_2026-04-21.md`
- future follow-on lane docs when stronger first-party proof exceeds this closeout

Current status:

- Closed on 2026-04-21 via `CLOSEOUT_AUDIT_2026-04-21.md`.
- This folder is now a closeout record for the M2 no-new-surface verdict.
- Future immediate key-owner pressure should start a different narrow lane with stronger
  first-party proof instead of reopening this folder by default.
