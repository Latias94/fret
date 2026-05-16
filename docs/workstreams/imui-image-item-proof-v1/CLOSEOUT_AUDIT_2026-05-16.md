# ImUi Image Item Proof Closeout Audit - 2026-05-16

Status: closed

## Shipped Surface

- `fret-ui-kit::imui::ImageItemOptions`
- `fret-ui-kit::imui::ImageItemVariant`
- `UiWriterImUiFacadeExt::image_item(...)`
- `UiWriterImUiFacadeExt::image_item_with_options(...)`
- `UiWriterImUiFacadeExt::image_button(...)`
- `UiWriterImUiFacadeExt::image_button_with_options(...)`

The helper is intentionally policy-layer authoring over existing Fret image mechanisms:
`ImageId`, `ImageProps`, `ViewportFit`, `ImageSamplingHint`, and `UvRect`.

## Verdict

This lane is closed. It adds the missing response-bearing image item proof without importing Dear
ImGui texture-ID state, asset-loading policy, or a new `fret-imui` runtime surface.

Future work should start as new narrow follow-ons for:

- `ImageWithBg`-style background/tint policy,
- asset-loading cookbook proof,
- atlas/texture lifetime policy,
- image editor or preview-pane product behavior.

## Evidence

- Implementation: `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs`
- Facade API: `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- Options/export surface: `ecosystem/fret-ui-kit/src/imui/options/controls.rs`,
  `ecosystem/fret-ui-kit/src/imui.rs`
- Smoke tests: `ecosystem/fret-ui-kit/tests/imui_image_item_smoke.rs`
- Reference: `repo-ref/imgui/imgui.h`

## Gates

- `cargo fmt --package fret-ui-kit`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_image_item_smoke --no-fail-fast`
- `cargo nextest run -p fret-ui-kit --features imui image_item --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-image-item-proof-v1/WORKSTREAM.json`
