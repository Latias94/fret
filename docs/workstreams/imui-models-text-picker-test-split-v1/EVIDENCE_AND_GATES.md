# ImUi Models Text Picker Test Split v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Reference Evidence

- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`: identifies `fret-imui` test architecture as a
  larger refactor hazard than missing top-level helper APIs.
- `.agents/skills/fret-fixture-driven-harnesses/SKILL.md`: fixture guidance used to keep this slice
  as Rust tests because the picker cases are procedural interactions rather than repeated data
  matrices.

## Implementation Anchors

- `ecosystem/fret-imui/src/tests/mod.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `ecosystem/fret-imui/src/tests/models_text_picker.rs`
- `docs/workstreams/imui-models-text-picker-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`

## Gates

```bash
cargo fmt --package fret-imui
cargo nextest run -p fret-imui models_text_picker --no-fail-fast
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-models-text-picker-test-split-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
