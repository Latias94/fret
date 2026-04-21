# ImUi Collection + Pane Proof v1 - Milestones

Status: closed closeout record
Last updated: 2026-04-21

Status note (2026-04-21): this file now records the closed proof-pair verdict only. Active
implementation should move to a different narrow lane if fresh evidence exceeds this closeout.

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why this is a new narrow follow-on instead of a reopened umbrella
  backlog,
- the lane names one current proof pair plus one current gate floor,
- and the owner split is explicit enough to avoid drifting into shell-helper or runtime work.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Current status:

- Closed on 2026-04-21 via `M0_BASELINE_AUDIT_2026-04-21.md`.

## M1 - Proof roster freeze

Exit criteria:

- one collection-first proof surface is named,
- one pane-first proof surface is named,
- and the lane explicitly records what is still deferred.

Primary evidence:

- `DESIGN.md`
- `M1_PROOF_ROSTER_FREEZE_2026-04-21.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`

Current status:

- Closed on 2026-04-21.
- `M1_PROOF_ROSTER_FREEZE_2026-04-21.md` now freezes:
  `imui_editor_proof_demo` as the current collection-first proof surface,
  `workspace_shell_demo` as the current pane-first proof surface,
  and `editor_notes_demo` as the supporting minimal pane rail proof.
- The lane now explicitly rejects creating:
  a dedicated asset-grid/file-browser proof demo or a narrower child-region-only proof demo by
  default at M1.
- The defer list is now explicit and frozen for M1:
  key ownership, shell-helper promotion, runner/backend multi-window parity, and broader menu/tab
  policy all stay out of scope unless another narrow lane proves otherwise.

## M2 - Collection-first proof closure

Exit criteria:

- the repo has one first-party collection proof that goes beyond row-list multi-select,
- the box-select / marquee question is answered explicitly,
- and focused interaction proof keeps the collection story executable.

Primary evidence:

- `M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- lane-local source-policy and interaction gates

Current status:

- Closed on 2026-04-21.
- `M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md` now keeps
  `apps/fret-examples/src/imui_editor_proof_demo.rs` as the collection-first M2 proof surface.
- The collection-first proof now closes with an in-demo asset-browser/file-browser surface instead
  of a new dedicated demo.
- Marquee / box-select is now explicitly deferred for M2 until another narrower proof shows the
  current click / range / toggle stack is insufficient.
- `ecosystem/fret-imui/src/tests/interaction.rs` now keeps selected collection drag payloads and
  keyed persistence executable under a focused interaction gate.

## M3 - Pane-first proof closure

Exit criteria:

- the repo has one first-party pane composition proof that exercises the current `child_region`
  posture or an accepted successor,
- the proof mounts real editor chrome instead of isolated helper snippets,
- and the lane decides whether helper widening is actually needed.

Primary evidence:

- `M3_PANE_PROOF_CLOSURE_2026-04-21.md`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- pane-focused source-policy and diagnostics gates

Current status:

- Closed on 2026-04-21.
- `M3_PANE_PROOF_CLOSURE_2026-04-21.md` now keeps
  `apps/fret-examples/src/workspace_shell_demo.rs` as the pane-first M3 proof surface.
- The pane-first proof now closes with a shell-mounted immediate pane composition inside the
  existing workspace shell demo instead of a narrower dedicated pane demo.
- `ecosystem/fret-ui-kit/src/imui/child_region.rs` now remains unchanged for M3 because the
  current seam proved sufficient for nested toolbar / tabs / inspector / status composition.
- The lane now reuses the existing workspace shell diagnostics floor for pane proof launch
  evidence instead of creating a narrower pane-only diagnostics path.

## M4 - Helper decision or closeout

Exit criteria:

- the lane either closes with the proof pair and explicit owner split,
- or lands a bounded helper widening justified by that proof pair,
- or splits again because the remaining pressure belongs to another narrower owner/problem.

Primary evidence:

- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `CLOSEOUT_AUDIT_2026-04-21.md`
- future follow-on lane docs when fresh evidence exceeds this closeout

Current status:

- Closed on 2026-04-21 via `CLOSEOUT_AUDIT_2026-04-21.md`.
- The lane now keeps the shipped M2 collection-first proof in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`.
- The lane now keeps the shipped M3 pane-first proof in
  `apps/fret-examples/src/workspace_shell_demo.rs`, with
  `apps/fret-examples/src/editor_notes_demo.rs` retained as supporting pane-rail evidence.
- `ecosystem/fret-ui-kit/src/imui/child_region.rs` remains unchanged because the current seam
  proved sufficient once the pane proof was assembled on the existing shell-mounted surface.
- M4 closes on a no-helper-widening verdict:
  no narrower collection/pane demo split and no narrower pane-only diagnostics path are required.
