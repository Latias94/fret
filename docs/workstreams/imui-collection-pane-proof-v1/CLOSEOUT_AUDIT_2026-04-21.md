# Closeout Audit — 2026-04-21

This audit records the final closeout read for the ImUi collection + pane proof v1 lane.

Goal:

- verify whether the shipped M2 collection-first proof and M3 pane-first proof still leave an
  active helper-surface design problem,
- separate the landed first-party proof pair from broader follow-on topics that do not belong in
  this folder,
- and decide whether the lane should remain active or become a closed closeout record.

## Audited evidence

Core lane docs:

- `docs/workstreams/imui-collection-pane-proof-v1/DESIGN.md`
- `docs/workstreams/imui-collection-pane-proof-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/M3_PANE_PROOF_CLOSURE_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/TODO.md`
- `docs/workstreams/imui-collection-pane-proof-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-pane-proof-v1/EVIDENCE_AND_GATES.md`

Umbrella and reference docs:

- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`

Implementation / gate anchors:

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `tools/gate_imui_workstream_source.py`
- `tools/gate_imui_facade_teaching_source.py`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`

Validation run used for closeout:

- `python tools/gate_imui_workstream_source.py`
- `python tools/gate_imui_facade_teaching_source.py`
- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test workspace_shell_pane_proof_surface --test editor_notes_editor_rail_surface --no-fail-fast`
- `cargo nextest run -p fret-imui collection_drag_payload_preserves_selected_keys_across_order_flip --no-fail-fast`
- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 -m json.tool docs/workstreams/imui-collection-pane-proof-v1/WORKSTREAM.json > /dev/null`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

## Findings

### 1. The collection-first proof now closes the missing asset-browser breadth without a new helper lane

The shipped M2 slice already proves the important collection-first outcome on a first-party surface:

- `apps/fret-examples/src/imui_editor_proof_demo.rs` now carries a collection-first asset browser
  proof with stable keyed identity, visible-order flipping, selected-set drag payloads, and an
  app-owned import target,
- `ecosystem/fret-imui/src/tests/interaction.rs` keeps keyed selection persistence under order
  flips executable through
  `collection_drag_payload_preserves_selected_keys_across_order_flip`,
- and the lane-local source-policy gate in `tools/gate_imui_workstream_source.py` freezes that
  this proof lives in the current demo instead of forcing a new dedicated asset-grid/file-browser
  demo.

Conclusion:

- the lane no longer has an open collection-first proof gap that justifies keeping execution active.

### 2. The pane-first proof now closes the missing nested pane composition breadth without widening `child_region`

The shipped M3 slice already proves the important pane-first outcome on the current shell-mounted
surface:

- `apps/fret-examples/src/workspace_shell_demo.rs` now mounts nested toolbar / tabs / inspector /
  status composition inside pane content using the current `child_region` story,
- `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs` locks the proof surface against
  source drift,
- and the lane reuses the existing workspace shell diagnostics floor instead of inventing a
  narrower pane-only diagnostics path.

Conclusion:

- the lane no longer has an open pane-first proof gap that justifies helper widening or a new
  dedicated pane demo.

### 3. The current helper surface is sufficient; M4 does not justify widening

After re-reading the shipped proof pair, the remaining evidence supports a no-widening verdict:

- `ecosystem/fret-ui-kit/src/imui/multi_select.rs` was already sufficient for the shipped
  collection proof once the demo assembled a real keyed asset-browser surface,
- `ecosystem/fret-ui-kit/src/imui/child_region.rs` was already sufficient for the shipped
  nested-pane proof once the workspace shell proof used the existing explicit container story,
- and no M2/M3 evidence shows that a broader shared helper, a pane-only helper facade, or a
  narrower diagnostics path is required to keep the current proof pair reviewable.

Conclusion:

- M4 closes on a no-helper-widening verdict.

### 4. The remaining pressure belongs to different owners and should not reopen this folder

What still remains after closeout is real, but it is not this lane's unfinished work:

1. Key ownership
   - still needs its own narrow owner/proof lane if first-party evidence grows beyond the current
     separation.
2. Promoted shell helpers
   - already belongs with the closed P1 shell closeout verdict and any future narrower shell
     follow-on, not this proof lane.
3. Broader menu/tab policy
   - already belongs with the separate trigger-response / policy closeout chain.
4. Runner/backend multi-window parity
   - remains active in `docs/workstreams/docking-multiwindow-imgui-parity/`.

Conclusion:

- this folder should stay closed and act as the closeout record for the shipped proof pair rather
  than a generic backlog for future ImGui-shaped pressure.

## Decision from this audit

Treat `imui-collection-pane-proof-v1` as:

- closed for the collection-first and pane-first proof-breadth goal,
- a closeout record for the shipped M2/M3 proof pair,
- and not the place to continue helper widening, pane-only demo splitting, or pane-only diagnostics
  growth by default.

## Immediate execution consequence

From this point forward:

1. keep `apps/fret-examples/src/imui_editor_proof_demo.rs` as the shipped collection-first proof
   surface unless a future narrower lane proves it is insufficient,
2. keep `apps/fret-examples/src/workspace_shell_demo.rs` as the shipped pane-first proof surface
   and `apps/fret-examples/src/editor_notes_demo.rs` as supporting pane-rail evidence,
3. keep `ecosystem/fret-ui-kit/src/imui/child_region.rs` unchanged for this lane,
4. do not add helper widening, a narrower pane-only demo, or a narrower pane-only diagnostics path
   in this folder without fresh cross-surface evidence,
5. start or resume a different narrow lane if future pressure shifts to key ownership, promoted
   shell helpers, broader menu/tab policy, or runner/backend multi-window parity.
