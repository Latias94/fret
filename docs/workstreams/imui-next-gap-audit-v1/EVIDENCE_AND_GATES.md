# ImUi Next Gap Audit v1 - Evidence & Gates

Goal: choose the next locally testable IMUI gap after recent editor-notes and collection closeouts
without widening public APIs or requiring macOS/multi-window acceptance.

## Evidence Anchors

- `docs/workstreams/imui-next-gap-audit-v1/DESIGN.md`
- `docs/workstreams/imui-next-gap-audit-v1/TODO.md`
- `docs/workstreams/imui-next-gap-audit-v1/MILESTONES.md`
- `docs/workstreams/imui-next-gap-audit-v1/M1_NEXT_GAP_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-next-gap-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-next-gap-audit-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-notes-dirty-status-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `tools/gate_imui_workstream_source.py`

## Focused Gates

- `cargo fmt -p fret-examples --check`
- `python tools/gate_imui_workstream_source.py`
- `python3 -m json.tool docs/workstreams/imui-next-gap-audit-v1/WORKSTREAM.json > /dev/null`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`
