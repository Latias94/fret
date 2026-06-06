# ImUi Floating Layer Dismissal Test Split v1 Closeout Audit - 2026-06-06

Status: Closed.

This lane closes a mechanical `fret-imui` floating-layer proof split. It does not change overlay
behavior, public APIs, or runtime contracts.

## What Shipped

- Kept `ecosystem/fret-imui/src/tests/floating/layer_dismissal.rs` as a module hub.
- Moved the menu non-click-through outside-press proof into
  `ecosystem/fret-imui/src/tests/floating/layer_dismissal/menu.rs`.
- Moved the click-through popover outside-press proof into
  `ecosystem/fret-imui/src/tests/floating/layer_dismissal/popover.rs`.
- Added source-gate anchors that keep the hub/menu/popover proof owner split visible.

## Proof

- `cargo fmt --package fret-imui` passed.
- `cargo nextest run -p fret-imui floating::layer_dismissal --no-fail-fast` passed:
  2 tests run, 2 passed, 187 skipped.
- `python tools/gate_imui_workstream_source.py` passed.
- `python tools/check_workstream_catalog.py` passed.
- `python -m json.tool docs/workstreams/imui-floating-layer-dismissal-test-split-v1/WORKSTREAM.json`
  passed.
- `git diff --check` passed.

## Remaining Work

Start narrower follow-ons for behavior changes such as nested overlay dismissal, focus restoration,
or pointer-capture arbitration. Do not add those cases back into the `layer_dismissal.rs` hub.
