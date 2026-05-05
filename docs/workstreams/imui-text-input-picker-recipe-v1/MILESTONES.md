# ImUi Text Input Picker Recipe v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M1 - Recipe Surface

Outcome: app authors can call a named helper instead of manually wiring input + popup + selectable.

Exit criteria:

- Completion and history helper methods exist on `UiWriterImUiFacadeExt` / `ImUiFacade`.
- Options cover filtering, max visible items, popup policy, open policy, and test ids.
- Response reports input response, picked value, picked index, and open state.

## M2 - Behavior

Outcome: the helper is usable for common completion/history flows.

Exit criteria:

- Completion filters candidates against the current model value.
- Candidate click commits to the model and reports changed.
- Exact-match completion values do not immediately reopen the popup.
- History can show unfiltered entries for empty input.

## M3 - Gate And Closeout

Outcome: behavior is locked by tests and the lane is closed with evidence.

Exit criteria:

- Targeted picker tests pass.
- Full `models_text` tests pass.
- Workstream JSON, catalog, layering, skills validation, and whitespace gates are recorded.
