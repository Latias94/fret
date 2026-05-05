# ImUi Text Input Policy Depth v1

Status: Closed execution lane
Last updated: 2026-05-04

Closeout note (2026-05-04): this lane shipped its bounded read-only, select-all-on-focus,
multiline Tab, `PushID`, and cookbook-proof scope. Later completion/history, filter, undo, and
picker work lives in the narrower follow-ons listed in
`CLOSEOUT_AUDIT_2026-05-04.md`; new text editing depth should start another follow-on instead of
reopening this directory.

This is a narrow follow-on to `imui-editor-grade-product-closure-v1`. It owns the next text-input
depth slice needed for a Dear ImGui-class immediate-mode authoring surface without broadening
`fret-imui` into a policy-heavy component library.

## Why This Lane Exists

The local Dear ImGui reference exposes `ImGuiInputTextFlags_ReadOnly`,
`ImGuiInputTextFlags_AutoSelectAll`, and `ImGuiInputTextFlags_AllowTabInput` as first-class
`InputText` / `InputTextMultiline` flags (`repo-ref/imgui/imgui.h`,
`repo-ref/imgui/imgui_widgets.cpp`). Fret already has model-backed single-line and multiline text
controls plus command/selection infrastructure, but the immediate surface did not expose these
common authoring policies.

## Layer Ownership

- `crates/fret-ui` owns mechanism-level read-only text control behavior:
  - focus remains possible when the control is enabled and focusable;
  - pointer/keyboard selection and copy remain possible;
  - mutating text input, paste, cut, clear, delete, and platform text replacement are blocked;
  - semantics report the value as read-only and not editable.
- `crates/fret-ui` also owns the mechanism-level multiline Tab behavior:
  - text areas can be configured to either insert a tab character or leave Tab for traversal /
    higher-level policy;
  - the generic runtime mechanism remains policy-free.
- `ecosystem/fret-ui-kit::imui` owns immediate-mode policy options:
  - `InputTextOptions::read_only`;
  - `InputTextOptions::select_all_on_focus`;
  - matching textarea options;
  - `TextAreaOptions::allow_tab_input`, defaulting to false for Dear ImGui-style opt-in behavior.
- `ecosystem/fret-imui` remains a thin authoring facade and proof surface; it should not grow a
  second widget implementation.
- `ecosystem/fret-imui::ImUi::push_id` must preserve Dear ImGui-style explicit ID stack semantics:
  the caller-provided key owns item identity, not the Rust source location of the helper call.

## Must-Be-True Outcomes

- Read-only model-backed text inputs and textareas cannot mutate their model through text events,
  paste/cut/clear/delete commands, or platform text replacement hooks.
- Read-only fields remain focusable and selectable, and copy remains available when a selection
  exists and clipboard writing is supported.
- Select-all-on-focus is a kit/editor policy implemented through command/timer wiring, not a new
  runtime contract knob.
- A stale select-all-on-focus timer must not select text in a different control if focus moves
  before the timer fires; the timer records an element-scoped transient, and the next render only
  emits `edit.select_all` while the original element is still focused.
- IMUI response `changed()` stays false for blocked read-only edits.
- IMUI textarea Tab input stays opt-in: default Tab does not mutate the model, while
  `allow_tab_input=true` inserts `\t` and reports `changed()`.
- `PushID` keeps `changed()` stable across repeated render closures and collection reordering.

## Out Of Scope

- History/completion callbacks.
- Character filters and callback-edit surfaces.
- Undo/redo policy knobs.
- Numeric scalar text-edit fallback.
- No-horizontal-scroll, elide-left, and word-wrap flag parity.
- Docking/multi-window runner parity.
