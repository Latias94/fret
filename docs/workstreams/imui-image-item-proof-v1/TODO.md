# ImUi Image Item Proof TODO

Status: closed
Last updated: 2026-05-16

Status note (2026-05-16): all implementation tasks below landed in the closeout slice.

- [x] Resolve this as a narrow follow-on from `imui-imgui-gap-closure-v1`, not a broad widget
  backlog or a reopen of debug-draw image support.
- [x] Add `ImageItemOptions` and `ImageItemVariant` to `fret-ui-kit::imui`.
- [x] Add facade helpers for `image_item`, `image_item_with_options`, `image_button`, and
  `image_button_with_options`.
- [x] Reuse shared pressable item behavior so image items report `ResponseExt` hover, click,
  context-menu, and drag-derived signals consistently with the rest of IMUI.
- [x] Add public smoke coverage for defaults and facade API compilation.
- [x] Run focused gates and record final results in `EVIDENCE_AND_GATES.md`.
- [x] Close this lane once the implementation slice lands; future background/tint/asset-loading
  depth should start as a separate follow-on.
