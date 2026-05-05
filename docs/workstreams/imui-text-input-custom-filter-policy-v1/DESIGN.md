# ImUi Text Input Custom Filter Policy v1 Design

Status: Closed
Last updated: 2026-05-04

## Problem

Dear ImGui exposes `ImGuiInputTextFlags_CallbackCharFilter`, where user code can replace or discard
incoming characters. Fret should support the useful policy outcome without copying the mutable
buffer callback shape into the runtime.

## Layer Decision

- `crates/fret-ui` remains unchanged for this slice; the existing `TextInputInsertFilter` is the
  mechanism.
- `fret-ui-kit::imui` owns the policy surface through `InputTextCustomFilter`.
- `fret-imui` remains thin and only verifies that the app-facing immediate-mode path works.

## Semantics

- Named filters run first.
- `InputTextCustomFilter` receives the named-filtered insertion text.
- Returning an empty string rejects the insertion.
- The custom filter does not receive or mutate the whole buffer, cursor, selection, or undo state.

## Must-Be-True Outcomes

- IMUI authors can install a custom insertion filter through `InputTextOptions`.
- Custom filtering composes with named filters in the same order Dear ImGui uses: generic filters,
  then custom filter.
- Runtime and `fret-imui` contracts do not grow a mutable text-buffer callback.

## Non-Goals

- No Dear ImGui callback data struct.
- No cursor/selection mutation from the filter.
- No undo-stack or completion/history recipe changes.
