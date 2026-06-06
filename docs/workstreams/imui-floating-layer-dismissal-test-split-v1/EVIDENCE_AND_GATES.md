# ImUi Floating Layer Dismissal Test Split v1 Evidence And Gates

Status: Closed
Last updated: 2026-06-06

## Reference Evidence

- `docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json`: umbrella IMUI maintenance lane for
  keeping proof surfaces reviewable without moving policy into `fret-imui`.
- `docs/workstreams/imui-models-text-final-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`: previous
  narrow IMUI test-architecture split pattern.
- `ecosystem/fret-imui/src/tests/floating/mod.rs`: floating proof module registration.

## Implementation Anchors

- `ecosystem/fret-imui/src/tests/floating/layer_dismissal.rs`
- `ecosystem/fret-imui/src/tests/floating/layer_dismissal/menu.rs`
- `ecosystem/fret-imui/src/tests/floating/layer_dismissal/popover.rs`
- `tools/gate_imui_workstream_source.py`
- `docs/workstreams/imui-floating-layer-dismissal-test-split-v1/CLOSEOUT_AUDIT_2026-06-06.md`

## Gates

```bash
cargo fmt --package fret-imui
cargo nextest run -p fret-imui floating::layer_dismissal --no-fail-fast
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-floating-layer-dismissal-test-split-v1/WORKSTREAM.json
git diff --check
```

## Fresh Verification - 2026-06-06

- Passed: `cargo fmt --package fret-imui`.
- Passed: `cargo nextest run -p fret-imui floating::layer_dismissal --no-fail-fast`.
  - `fret-imui tests::floating::layer_dismissal::menu::floating_layer_menu_outside_press_dismisses_without_activating_underlay`
  - `fret-imui tests::floating::layer_dismissal::popover::floating_layer_popover_outside_press_allows_underlay_activation_when_click_through`
  - Result: 2 passed, 187 skipped.
- Passed: `python tools/gate_imui_workstream_source.py`.
- Passed: `python tools/check_workstream_catalog.py`.
- Passed: `python -m json.tool docs/workstreams/imui-floating-layer-dismissal-test-split-v1/WORKSTREAM.json`.
- Passed: `git diff --check`.
