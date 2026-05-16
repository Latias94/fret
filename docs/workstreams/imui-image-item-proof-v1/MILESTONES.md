# ImUi Image Item Proof Milestones

Status: closed
Last updated: 2026-05-16

Status note (2026-05-16): this lane is closed. Future image background/tint, asset-loading, atlas,
or editing behavior belongs in a new proof-led follow-on.

## M0 - Boundary Freeze

Exit criteria:

- The lane is recorded as a narrow follow-on.
- Ownership is limited to `fret-ui-kit::imui`.
- Dear ImGui image APIs are used as reference evidence, not copied as a texture-ID runtime model.

Result: done. The target surface is an additive policy-layer helper over existing Fret image
mechanisms.

## M1 - API And Behavior Slice

Exit criteria:

- Public facade helpers compile through `UiWriterImUiFacadeExt`.
- Plain image item and image button defaults are distinct.
- Response assembly reuses shared IMUI item behavior.
- Image fit, sampling, opacity, and UV options flow into `ImageProps`.

Result: done. `UiWriterImUiFacadeExt` exposes plain image item and image button helpers, and both
paths use the shared `ResponseExt` assembly behavior.

## M2 - Gate And Closeout

Exit criteria:

- Focused `fret-ui-kit` IMUI smoke tests pass.
- Workstream catalog/source gates pass.
- This lane records closeout and does not become a bucket for background/tint/asset-loader work.

Result: done. Focused smoke/helper gates and workstream source/catalog gates passed.
