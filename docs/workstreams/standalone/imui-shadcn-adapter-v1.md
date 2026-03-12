# imui + shadcn Adapter Alignment v1

Status: Draft
Last updated: 2026-02-06

## 1) Why this exists

`fret-imui` is intentionally policy-light and keeps an immediate-mode control-flow surface.
`fret-ui-shadcn` is policy-heavy and visual (Radix/shadcn taxonomy + recipes).

This note documents a practical bridge pattern:

- keep interaction control flow in imui-style code,
- reuse shadcn visual components where it adds value,
- avoid duplicating state machines in two places.

## 2) Layering contract (author once)

- `ecosystem/fret-imui`
  - Owns immediate authoring frontend (`ImUi`, identity helpers, output sink).
  - Must stay independent from `fret-ui-kit` / `fret-ui-shadcn` policy internals.
- `ecosystem/fret-ui-kit` (`imui` feature)
  - Owns fa?ade adapters and response mapping (`button`, `checkbox_model`, `toggle_model`,
    `switch_model`, `slider_f32_model`, `select_model`, `input_text_model`, `textarea_model`).
  - Owns opt-in adapter seams for higher-level ecosystems.
- `ecosystem/fret-ui-shadcn`
  - Owns visual recipes/components and style tokens.
  - Should not require runtime-specific hacks from `fret-imui`.

Rule of thumb:

- behavior/state machine lives once (canonical surface),
- immediate fa?ade maps to that behavior,
- visual system can be swapped without rewriting interaction policy.

## 3) Minimal adapter pattern

Use `fret_imui::imui_vstack(...)` for control flow, then mix in:

1. fa?ade controls from `fret_ui_kit::imui::UiWriterImUiFacadeExt`,
2. shadcn visuals rendered as declarative elements (`into_element(cx)`) and pushed via `ui.add(...)`.

Reference demo:

- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-demo/src/bin/imui_shadcn_adapter_demo.rs`

Run:

```bash
cargo run -p fret-demo --bin imui_shadcn_adapter_demo
```

## 4) Migration checklist for third-party widget crates

When adding a third-party widget ecosystem on top of imui:

1. Keep widget policy in one canonical layer (prefer existing primitives/recipes).
2. Add a thin fa?ade API in `fret-ui-kit::imui` only when an immediate call shape is needed.
3. Preserve `Response` semantics (`clicked`, `changed`, focus/rect) in a predictable way.
4. Add one focused regression test in `ecosystem/fret-imui` for behavior gates.
5. Add one minimal demo proving integration ergonomics.

## 5) v1 scope note

This v1 adapter alignment intentionally focuses on ergonomics and ownership boundaries.
It does not attempt full API parity with every shadcn component in one iteration.
