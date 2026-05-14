# ImUi Facade Value Model Owner Split v1 - Design

Status: closed
Last updated: 2026-05-13

This lane is a narrow follow-on from `imui-facade-boolean-wrapper-owner-split-v1`. It owns only the
private source owner for slider/combo model `ImUiFacade` inherent wrappers.

## In Scope

- Move `slider_f32_model(...)`, `slider_f32_model_with_options(...)`, `combo_model(...)`, and
  `combo_model_with_options(...)` out of `facade_writer.rs`.
- Keep the methods as inherent `ImUiFacade` methods.
- Preserve public names, `fret::imui` re-export paths, `fret-imui`, and `crates/fret-ui` runtime
  contracts.

## Out Of Scope

- New slider, combo, or value editing behavior.
- New model/state ownership.
- Table wrappers, debug draw, docking, multi-window, or runtime contract changes.
