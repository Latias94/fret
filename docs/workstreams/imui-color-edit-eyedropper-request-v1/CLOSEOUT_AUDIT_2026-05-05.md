# ImUi Color Edit Eyedropper Request v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `ColorEditEyedropperRequest` with alpha-preserving sampled-color application rules.
- Added `OnColorEditEyedropper` and `ColorEditOptions::on_eyedropper` as an opt-in app-owned hook.
- Added a popup `Eyedropper` command row when the hook is present.
- Applied synchronous returned samples through the existing color model, draft text, and validation
  state update path.
- Kept asynchronous or OS-backed screen sampling app-owned by allowing the hook to return `None`.
- Kept the implementation in `fret-ui-editor`; no runtime, platform, renderer, or global mode
  contract was added.

## Evidence

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/popup/eyedropper.rs`
- `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `docs/adr/0120-offscreen-rendering-frame-capture-and-readback.md`
- `docs/adr/0125-frame-capture-options-and-determinism-v1.md`

## Gates Run

```bash
cargo fmt --package fret-ui-editor -- --check
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
python -m json.tool docs/workstreams/imui-color-edit-eyedropper-request-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/check_layering.py
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Residual Gaps

- Native/web platform screen sampling remains unimplemented as a framework-owned contract.
- Deeper side-preview polish.
- Higher-fidelity picker visual parity.
