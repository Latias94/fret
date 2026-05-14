# ImUi Facade Text Model Owner Split v1 - Design

Status: closed
Last updated: 2026-05-13

This lane is a narrow follow-on from `imui-facade-disclosure-owner-split-v1`. It owns only the
private source owner for text and textarea model-backed `ImUiFacade` inherent wrappers.

## In Scope

- Move `input_text_model(...)`, `input_text_model_with_options(...)`,
  `input_text_completion_model(...)`, `input_text_completion_model_with_options(...)`,
  `input_text_history_model(...)`, `input_text_history_model_with_options(...)`,
  `textarea_model(...)`, and `textarea_model_with_options(...)` out of `facade_writer.rs`.
- Keep the methods as inherent `ImUiFacade` methods.
- Preserve public names, `fret::imui` re-export paths, `fret-imui`, and `crates/fret-ui` runtime
  contracts.

## Out Of Scope

- New text input behavior.
- New completion/history picker policy.
- Boolean/model wrappers, table wrappers, debug draw, docking, multi-window, or runtime contract
  changes.
