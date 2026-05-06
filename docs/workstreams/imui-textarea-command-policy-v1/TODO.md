# ImUi Textarea Command Policy v1 TODO

Status: Closed
Last updated: 2026-05-06

## M0 - Source Mapping

- [x] Confirm textarea command routing belongs in `fret-ui-kit::imui`, not `crates/fret-ui`.
- [x] Keep Dear ImGui mutable-buffer callback semantics out of scope.

## M1 - IMUI Option Surface

- [x] Add `TextAreaOptions::submit_command`.
- [x] Add `TextAreaOptions::cancel_command`.
- [x] Add `TextAreaSubmitKey` with Ctrl+Enter default and Enter opt-in.
- [x] Add repeat opt-in for submit/cancel command dispatch.

## M2 - Focused Key Policy

- [x] Dispatch submit on Ctrl+Enter by default.
- [x] Preserve unmodified Enter newline insertion by default.
- [x] Dispatch submit on unmodified Enter when opted in.
- [x] Dispatch cancel on unmodified Escape.
- [x] Ignore IME composition, Alt, Meta, and repeated keydown unless explicitly enabled.

## M3 - Gates And Closeout

- [x] Add focused `fret-imui` model tests.
- [x] Add `fret-ui-kit` public option smoke.
- [x] Update IMUI gap audit and workstream indexes.
- [x] Leave deeper multiline editing and callback-style gaps as separate follow-ons.
