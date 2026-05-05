# ImUi Text Input Policy Depth v1 TODO

Status: Closed
Last updated: 2026-05-04

Closeout note (2026-05-04): this lane is closed. The original bounded scope is complete, and the
callback/filter/undo/picker pressure was split into narrower follow-ons. Future text depth should
start from a new workstream instead of adding TODOs here.

## M1 - Read-Only And Focus Selection Slice

- [x] Add read-only mechanism support to `TextInputProps`, `TextAreaProps`, `TextInput`,
  `TextArea`, and their bound model wrappers.
- [x] Block mutating events, commands, clipboard paste/cut/clear, and platform text replacement
  while preserving focus, selection, and copy.
- [x] Add IMUI options for `read_only` and `select_all_on_focus`.
- [x] Implement select-all-on-focus in `fret-ui-kit::imui` through command/timer wiring.
- [x] Keep `ImUi::push_id` identity key-driven instead of caller-location-driven.
- [x] Add focused tests for read-only model protection, focus-time select-all behavior, stale
  select-all timer focus migration, and `PushID` changed-signal stability.
- [x] Run the gates listed in `EVIDENCE_AND_GATES.md`.

## M2 - Public Cookbook Proof

- [x] Wire `InputTextOptions::select_all_on_focus` and `InputTextOptions::read_only` through the
  app-facing `fret::imui::{prelude::*, kit}` surface in `imui_action_basics`.
- [x] Extend the cookbook authoring-surface policy test so the example keeps teaching the root
  `fret::imui` facade instead of direct `fret-ui-kit` / `fret-imui` internals.

## M3 - Multiline Tab Input Policy Slice

- [x] Add mechanism-level `TextAreaProps::allow_tab_input` / `TextArea::set_allow_tab_input`.
- [x] Keep generic runtime text area Tab insertion available when the mechanism flag is true.
- [x] Expose `TextAreaOptions::allow_tab_input` in `fret-ui-kit::imui`.
- [x] Default IMUI textarea Tab behavior to no mutation, matching Dear ImGui's opt-in
  `AllowTabInput` posture.
- [x] Add runtime and IMUI regression tests for blocked/default Tab and opt-in Tab insertion.

## Later Follow-Ons

- [x] Audit whether history/completion callback policy belongs in `fret-ui-kit::imui` or an editor
  adapter layer.
  - Result: command-oriented single-line Tab/Up/Down routing belongs in `fret-ui-kit::imui`;
    completion/history UI and mutable-buffer callbacks stay as separate editor/app follow-ons.
  - Evidence: `docs/workstreams/imui-text-input-history-completion-policy-v1/`.
- [x] Decide how to model character filters without pushing callback-heavy Dear ImGui semantics
  into `crates/fret-ui`.
  - Result: named filters and custom insertion filters landed in
    `docs/workstreams/imui-text-input-filter-policy-v1/` and
    `docs/workstreams/imui-text-input-custom-filter-policy-v1/`.
- [ ] Start a numeric/scalar text-edit lane once drag/slider editing policy has a second proof
  surface.
- [ ] Start a narrower multiline wrap/no-horizontal-scroll/elide-left follow-on only after a real
  demo needs it.
