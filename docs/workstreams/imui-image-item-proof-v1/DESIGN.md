# ImUi Image Item Proof v1

Status: closed execution follow-on
Last updated: 2026-05-16

Status note (2026-05-16): this lane closed after landing the focused response-bearing image item /
image button proof. Start a new narrow follow-on for background/tint parity, asset-loading
teaching, image atlas policy, or broader image editing behavior.

## Scope

This lane owns the first standalone IMUI image widget admitted from the Dear ImGui gap catalog.

The target is intentionally narrow:

- add response-bearing image item authoring to `fret-ui-kit::imui`,
- expose a button variant for image-button use cases,
- use Fret's existing `ImageId`, `ImageProps`, `ViewportFit`, `ImageSamplingHint`, and `UvRect`
  mechanism vocabulary,
- keep asset loading, image registration, metadata lookup, and texture lifetime app-owned,
- keep `fret-imui` policy-light and unchanged as a runtime owner.

## Assumptions

- Confident: the component catalog calls standalone image widget growth a candidate, not a broad
  widget-backlog mandate.
  Evidence: `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`.
  Consequence if wrong: this lane should stop at documentation and not add public helpers.
- Confident: Fret already has the correct mechanism-level image representation.
  Evidence: `crates/fret-ui/src/element.rs` (`ImageProps`) and `crates/fret-ui/src/elements/cx.rs`
  (`ElementContext::image_props`).
  Consequence if wrong: the mechanism gap belongs in `crates/fret-ui`, not this policy-layer lane.
- Likely: response-bearing image item should reuse the shared IMUI pressable behavior.
  Evidence: button/selectable/debug-draw interaction helpers already populate `ResponseExt` through
  shared item behavior in `ecosystem/fret-ui-kit/src/imui/item_behavior.rs`.
  Consequence if wrong: image item would drift from hover/click/context/drag response vocabulary and
  need a follow-up refactor back into the shared behavior path.

## Target API

`UiWriterImUiFacadeExt` gains:

- `image_item(id, image, size) -> ResponseExt`
- `image_item_with_options(id, image, size, options) -> ResponseExt`
- `image_button(id, image, size) -> ResponseExt`
- `image_button_with_options(id, image, size, options) -> ResponseExt`

`ImageItemOptions` carries:

- enabled/focusable policy,
- `ImageItemVariant::{Image, Button}`,
- image fit/sampling/opacity/UV,
- optional accessibility label and test id.

Plain image items default to non-focusable image semantics while still reporting pointer hover,
click, context-menu, and drag-derived response signals. Image buttons default to focusable button
semantics and use the existing IMUI button chrome.

## Non-Goals

- No Dear ImGui `ImTextureID` / `ImTextureRef` runtime stack.
- No asset loader or image registration tutorial.
- No `ImageWithBg` tint/background parity in this slice.
- No image editor, preview-pane, atlas, or GPU texture lifetime policy.
- No `fret-imui` dependency on `fret-ui-kit`.
