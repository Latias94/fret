# ImUi Models Text Filter Test Split v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Reference Evidence

- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`: identifies `fret-imui` test architecture as a
  larger refactor hazard than missing top-level helper APIs.
- `docs/workstreams/imui-models-text-picker-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`: previous
  narrow split proving the same decomposition approach.

## Implementation Anchors

- `ecosystem/fret-imui/src/tests/mod.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `ecosystem/fret-imui/src/tests/models_text_filters.rs`
- `docs/workstreams/imui-models-text-filter-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`

## Gates

```bash
cargo fmt --package fret-imui
cargo nextest run -p fret-imui models_text_filters --no-fail-fast
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-models-text-filter-test-split-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
