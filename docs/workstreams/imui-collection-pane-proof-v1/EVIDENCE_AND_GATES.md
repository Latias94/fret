# ImUi Collection + Pane Proof v1 - Evidence & Gates

Status: closed closeout record
Last updated: 2026-05-01

Status note (2026-04-21): the shipped M2/M3 proof pair satisfied the lane closeout condition.
Keep this gate set as the regression floor for the closed proof pair rather than an invitation to
keep widening the helper surface here.

Goal: keep the closed collection/pane proof lane tied to one shipped proof pair, one explicit gate
floor, and one bounded evidence set instead of turning into another vague `imui` backlog.

## Evidence anchors (current)

- `docs/workstreams/imui-collection-pane-proof-v1/DESIGN.md`
- `docs/workstreams/imui-collection-pane-proof-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/M3_PANE_PROOF_CLOSURE_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/TODO.md`
- `docs/workstreams/imui-collection-pane-proof-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-pane-proof-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `tools/gate_imui_workstream_source.py`
- `tools/gate_imui_facade_teaching_source.py`

## First-open closeout / reopen surfaces

Use these if future evidence claims that the closed proof pair is insufficient:

1. Current collection-first baseline
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. Current pane-first baseline
   - `cargo run -p fret-demo --bin workspace_shell_demo`
3. Supporting minimal pane rail proof
   - `cargo run -p fret-demo --bin editor_notes_demo`

These are the shipped surfaces to keep stable unless a future narrower lane proves they are
insufficient.

Current frozen M1 roster:

- collection-first: `imui_editor_proof_demo`
- pane-first: `workspace_shell_demo`
- supporting minimal pane rail proof: `editor_notes_demo`

## Current focused gates

### Lane-local source-policy gate

- `python tools/gate_imui_workstream_source.py`

This gate currently proves:

- the lane still points at the frozen M1 proof roster,
- the lane now keeps the in-demo M2 asset-browser proof explicit,
- the lane now keeps the shell-mounted M3 pane proof explicit,
- the lane still rejects creating a dedicated asset-grid/file-browser proof demo by default,
- the lane still rejects creating a narrower child-region-only proof demo by default,
- and the defer list remains explicit.

### Collection focused interaction gate

- `cargo nextest run -p fret-imui collection_drag_payload_preserves_selected_keys_across_order_flip --no-fail-fast`

This gate currently proves:

- selected collection drag payloads stay app-defined,
- multi-select survives visible-order flips because selection stays keyed,
- and the M2 collection-first proof remains executable instead of purely documentary.

### Collection baseline floor

- `python tools/gate_imui_facade_teaching_source.py`

This floor currently proves:

- the editor proof still uses the official immediate/editor adapter seams,
- the proof keeps app-owned sortable and docking helpers explicit instead of quietly hardening a new
  shared shell/helper story,
- and the lane can keep using the current heavier proof while deciding whether a narrower dedicated
  collection demo is warranted.

### Pane proof floor

- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test workspace_shell_pane_proof_surface --test editor_notes_editor_rail_surface --no-fail-fast`

This floor currently proves:

- the shell-mounted editor rail surfaces remain executable,
- the current workspace shell proof now exposes a real nested pane proof,
- `workspace_shell_demo` keeps the shell-mounted immediate pane composition explicit,
- and `editor_notes_demo` remains the supporting minimal pane rail proof without forcing a narrower
  pane-only demo.

### Current immediate adapter/boundary floor

- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke`

This floor currently proves:

- the baseline immediate adapter seams still compile and stay in the intended owner layer,
- richer proof work remains facade-side,
- and the lane does not need to widen the shared response or runtime contracts just to start.

### Current interaction floor

- `cargo nextest run -p fret-imui`

This floor currently proves:

- the immediate interaction/runtime evidence remains green while the lane decides whether
  collection or pane proof breadth needs new focused tests,
- and drag/drop / floating / interaction semantics keep their current baseline behavior.

### Supporting shell diagnostics floor

- `cargo run -p fretboard-dev -- diag suite diag-hardening-smoke-workspace --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`

This floor currently proves:

- the current shell-mounted pane proof still launches with the promoted workspace smoke suite,
- and the lane can reuse that minimum without adding a narrower pane-specific diag path at M3.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-collection-pane-proof-v1/WORKSTREAM.json > /dev/null`

## Expected next gate additions

No further gate additions are planned for this folder.
`CLOSEOUT_AUDIT_2026-04-21.md` closes the lane without helper widening, without a narrower
collection/pane demo split, and without a narrower pane-only diagnostics path.
