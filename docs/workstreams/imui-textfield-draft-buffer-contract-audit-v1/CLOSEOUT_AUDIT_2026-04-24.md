# ImUi TextField Draft Buffer Contract Audit v1 - Closeout Audit - 2026-04-24

Status: closed narrow P1 audit lane

## Verdict

Treat `imui-textfield-draft-buffer-contract-audit-v1` as a closed no-public-API verdict.

Do not expose a `TextField` preserved draft-buffer contract yet. The existing internal draft model,
buffered session state, pending blur handling, and commit/cancel helpers are too coupled to focus and
timer semantics to safely publish without a narrower API-proof lane.

## What Remains Allowed

- App-owned status/action markers like `editor_notes_demo.rs` already uses.
- Internal `fret-ui-editor` tests around buffered focus transitions and `PreserveDraft` behavior.
- A future narrow API-proof lane if a real proof surface needs external preserved-draft
  commit/discard.

## What Stays Closed

- Public draft model handles from `TextFieldOptions`.
- Generic app-facing commit/discard buttons wired into hidden `TextField` draft state.
- `fret-ui-kit::imui`, `fret-imui`, `fret-authoring`, or `crates/fret-ui` API widening.
- Persistence, dirty-close prompts, command bus, clipboard, or menu integration.

## Gates

- `cargo fmt -p fret-examples --check`
- `python tools/gate_imui_workstream_source.py`
- `python3 -m json.tool docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`
