# ImUi TextField Draft Controller API Proof v1 - Closeout Audit

Status: closed
Date: 2026-04-29

## Decision

Close this lane. The admitted v1 surface is the opaque `TextFieldDraftController` operation handle in
`ecosystem/fret-ui-editor`; it is sufficient for app-authored commit/discard affordances on preserved
`TextField` drafts.

The closeout is intentionally narrow:

1. `TextFieldDraftController` commits or discards through the same internal buffered TextField
   session semantics as built-in commit/cancel behavior.
2. The controller does not expose the internal draft `Model<String>`.
3. `editor_notes_demo.rs` proves the external operation surface with app-owned `Commit draft` and
   `Discard draft` buttons.
4. The launched diagnostics proof clicks those buttons through stable `test_id` selectors and checks
   committed line count, last action, draft status, and the app-owned status row.
5. Runtime contracts, generic IMUI helpers, command-bus integration, persistence, dirty-close, and
   document-state policy remain out of scope.

## Evidence

- `ecosystem/fret-ui-editor/src/controls/text_field.rs`
- `ecosystem/fret-ui-editor/tests/text_field_api_smoke.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/src/lib.rs`
- `tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/WORKSTREAM.json`
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/EVIDENCE_AND_GATES.md`

## Diagnostics Proof

Successful local launched run:

```text
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json --dir target/fret-diag/editor-notes-draft-controller-proof --session-auto --timeout-ms 600000 --pack --ai-packet --launch -- cargo run -p fret-demo --bin editor_notes_demo
PASS (run_id=1777427121868)
```

Artifacts:

- Pack: `target/fret-diag/editor-notes-draft-controller-proof/sessions/1777426816640-22024/share/1777427121868.zip`
- AI packet: `target/fret-diag/editor-notes-draft-controller-proof/sessions/1777426816640-22024/1777427121868/ai.packet`
- Final bundle: `target/fret-diag/editor-notes-draft-controller-proof/sessions/1777426816640-22024/1777427125232-editor-notes-demo-draft-controller-proof/bundle.schema2.json`

Bounded meta check for the final bundle reported one window, `36` unique test ids, `0` duplicate
test ids, and `32` snapshots.

## Gates

- `cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-draft-controller-proof.json --dir target/fret-diag/editor-notes-draft-controller-proof --session-auto --timeout-ms 600000 --pack --ai-packet --launch -- cargo run -p fret-demo --bin editor_notes_demo`
- `cargo nextest run -p fret-ui-editor --test text_field_api_smoke --no-fail-fast`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_textfield_draft_controller_api_proof --no-fail-fast`
- `python3 -m json.tool docs/workstreams/imui-textfield-draft-controller-api-proof-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

## Follow-On Rule

Do not reopen this lane for broader editor document behavior. Start a narrower follow-on for any of:

- persistence,
- workspace dirty-close prompts,
- command palette/menu/keymap integration,
- clipboard behavior,
- generic document state,
- public `fret-ui`, `fret-ui-kit::imui`, or `fret-imui` helper widening.
