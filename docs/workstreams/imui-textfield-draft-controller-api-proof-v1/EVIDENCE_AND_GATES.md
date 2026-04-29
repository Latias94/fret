# ImUi TextField Draft Controller API Proof v1 - Evidence and Gates

Goal: prove a small external commit/discard operation handle for preserved `TextField` drafts
without exposing internal draft models or widening runtime/generic IMUI APIs.

## Evidence Anchors

- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/DESIGN.md`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/TODO.md`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/MILESTONES.md`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/WORKSTREAM.json`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/CLOSEOUT_AUDIT_2026-04-29.md`
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json`
- `ecosystem/fret-ui-editor/src/controls/text_field.rs`
- `ecosystem/fret-ui-editor/tests/text_field_api_smoke.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/src/lib.rs`

## Gates

- Gate name: `editor-notes-draft-controller-script-schema`
- `cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json`
- Gate name: `editor-notes-draft-controller-launched-diag`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json --dir target/fret-diag/editor-notes-draft-controller-proof --session-auto --timeout-ms 600000 --pack --ai-packet --launch -- cargo run -p fret-demo --bin editor_notes_demo`
- Gate name: `textfield-draft-controller-source-policy`
- `cargo nextest run -p fret-ui-editor --test text_field_api_smoke --no-fail-fast`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_textfield_draft_controller_api_proof --no-fail-fast`
- `python3 -m json.tool docs/workstreams/imui-textfield-draft-controller-api-proof-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

Latest local evidence (2026-04-28):

- `cargo nextest run -p fret-ui-editor -E 'test(draft_controller) or test(text_field_option_defaults_match_buffered_plain_text_baseline)' --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast` passed after the initial cold compile timeout was rerun with warm cache.
- The predecessor source-policy gate passed before closeout; it was renamed to
  `immediate_mode_workstream_closes_the_p1_textfield_draft_controller_api_proof` when the lane
  closed.

Latest local evidence (2026-04-29):

- `cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json` passed.
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json --dir target/fret-diag/editor-notes-draft-controller-proof --session-auto --timeout-ms 600000 --pack --ai-packet --launch -- cargo run -p fret-demo --bin editor_notes_demo` passed with `run_id=1777427121868`.
- Pack: `target/fret-diag/editor-notes-draft-controller-proof/sessions/1777426816640-22024/share/1777427121868.zip`.
- AI packet: `target/fret-diag/editor-notes-draft-controller-proof/sessions/1777426816640-22024/1777427121868/ai.packet`.
- Bundle meta check reported one window, `36` unique test ids, and `0` duplicate test ids for
  `target/fret-diag/editor-notes-draft-controller-proof/sessions/1777426816640-22024/1777427125232-editor-notes-demo-draft-controller-proof/bundle.schema2.json`.
