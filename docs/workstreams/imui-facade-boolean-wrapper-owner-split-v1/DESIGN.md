# ImUi Facade Boolean Wrapper Owner Split v1 - Design

Status: closed
Last updated: 2026-05-13

This lane is a narrow follow-on from `imui-facade-text-model-owner-split-v1`. It owns only the
private source owner for checkbox/radio/switch `ImUiFacade` inherent wrappers.

## In Scope

- Move `checkbox_model(...)`, `checkbox_model_with_options(...)`, `radio(...)`,
  `radio_with_options(...)`, `switch_model(...)`, and `switch_model_with_options(...)` out of
  `facade_writer.rs`.
- Keep the methods as inherent `ImUiFacade` methods.
- Preserve public names, `fret::imui` re-export paths, `fret-imui`, and `crates/fret-ui` runtime
  contracts.

## Out Of Scope

- New boolean control behavior.
- New model/state ownership.
- Slider/combo model wrappers, table wrappers, debug draw, docking, multi-window, or runtime
  contract changes.
