# ImUi Text Control Chrome Stability v1 - TODO

Status: closed
Last updated: 2026-04-28

## Current Slice

- [x] Route the issue as a narrow follow-on instead of reopening the closed control-chrome lane.
- [x] Add IMUI-specific text input and textarea chrome helpers.
- [x] Remove shadcn input recipe chrome from IMUI text controls.
- [x] Add direct invariant tests for rendered `TextInputProps` and `TextAreaProps`.
- [x] Run the focused unit gates.
- [x] Run the package check, format check, JSON/catalog gates, and diff check.

## Follow-On Backlog

- [ ] Start a narrower follow-on for a diagnostics script only if a future visual report cannot be proven by the unit
  chrome invariants plus the existing bounds test.
- [ ] Audit any future IMUI field-like helper before sharing chrome with shadcn recipes.
- [x] Close this lane once the focused gates pass and no broader text-control API work is needed.
