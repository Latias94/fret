# ImUi Next Gap Audit v1 - Closeout Audit - 2026-04-24

Status: closed narrow P1 audit lane

## Verdict

Treat `imui-next-gap-audit-v1` as a closed decision record. The next non-multi-window IMUI
implementation should start `imui-editor-notes-draft-actions-v1` and remain app-owned inside
`editor_notes_demo.rs` until stronger evidence proves a reusable contract.

## Closed Decisions

- Do not reopen `imui-editor-grade-product-closure-v1` for implementation-heavy work.
- Do not reopen collection helper readiness without one exact shared helper candidate.
- Do not widen `fret-ui-kit::imui`, `fret-imui`, `fret-authoring`, or `crates/fret-ui` from this
  audit.
- Keep macOS/multi-window/tear-off work parked in runner/backend-owned lanes until acceptance can be
  captured.

## Recommended Follow-On

Start `imui-editor-notes-draft-actions-v1` with this minimum scope:

- Add app-owned draft action affordances to `editor_notes_demo.rs`.
- Reuse existing local notes models.
- Avoid persistence, workspace dirty-close prompts, command bus, clipboard, and generic APIs.
- Gate with `editor_notes_editor_rail_surface` plus a source-policy test.

## Gates

- `cargo fmt -p fret-examples --check`
- `python tools/gate_imui_workstream_source.py`
- `python3 -m json.tool docs/workstreams/imui-next-gap-audit-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`
