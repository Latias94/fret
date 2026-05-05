# ImUi Text Input Picker Keyboard Navigation v1

Status: Closed
Last updated: 2026-05-04

## Problem

The first picker recipe made completion/history candidates visible and clickable, but keyboard users
still could not move an active candidate and commit it without leaving the input field. Dear ImGui
exposes completion/history through callback flags; Fret keeps that policy in `fret-ui-kit::imui`
as a recipe over app-owned candidate data.

## Target

- Keep keyboard focus in the input field while a non-modal picker popup is open.
- Move the active visible candidate with ArrowDown and ArrowUp.
- Wrap active movement at list edges, matching existing IMUI popup row navigation expectations.
- Commit the active candidate with Enter or NumpadEnter.
- Keep Enter/Arrow unconsumed when the picker has no candidate to act on.
- Keep candidate/history storage and ranking app-owned.

## Ownership

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`: picker active-index state, popup-owner
  key handling, and selected-like active row rendering.
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`: internal element builder used so the picker can
  wrap input ownership without duplicating text-input policy.
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`: keyboard navigation options.
- `ecosystem/fret-imui/src/tests/models_text.rs`: app-facing behavior proof.

## Must-Be-True Outcomes

- ArrowDown selects the first visible completion candidate and subsequent ArrowDown advances it.
- ArrowUp wraps from the first history candidate to the last visible history candidate.
- Enter/NumpadEnter commits the active candidate to the same app-owned `Model<String>`.
- A picker with no visible candidates does not steal Enter from the underlying input submit path.
- `crates/fret-ui` remains a text editing mechanism layer and gains no candidate/history store.

## Non-Goals

- No mutable Dear ImGui callback buffer API.
- No runtime-owned completion/history storage.
- No editor ranking or fuzzy scoring policy in this slice.
- No full `aria-activedescendant`/screen-reader relationship audit in this slice.
- No multiline completion/history conflict policy.
