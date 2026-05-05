# ImUi Color Edit Popup Numeric Split v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-05

## Implementation Anchors

- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/numeric.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`

## Gates

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python tools/check_workstream_catalog.py
git diff --check
```
