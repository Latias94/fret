# Action Availability Coverage (Widget-Scoped Commands)

This note tracks the current "action availability" surface in Fret and highlights the remaining
gaps vs GPUI-style ergonomics.

Scope: this document focuses on `CommandScope::Widget` commands and the question:

> "Is this action available along the dispatch path to the current focus?"

## Current Mechanism

### Query API (in-process)

- `UiTree::command_availability` and `UiTree::is_command_available`
- GPUI naming parity aliases: `UiTree::action_availability` and `UiTree::is_action_available`
- Declarative policy hook: `ElementContext::command_on_command_availability_for` (lets component-layer
  surfaces participate in dispatch-path availability without adding new core widget types).

### Snapshot API (runner / menus / command palette)

- `UiTree::publish_window_command_action_availability_snapshot` publishes:
  - `WindowCommandActionAvailabilityService` (`HashMap<CommandId, bool>`)
- Consumers combine:
  - `WindowInputContextService` (`InputContext`)
  - `WindowCommandEnabledService` (explicit overrides)
  - `WindowCommandActionAvailabilityService` (dispatch-path availability)
  - via `WindowCommandGatingSnapshot`
  - Note: the availability map is best-effort and may omit commands with `NotHandled` availability
    (treat missing entries as "unknown", not "disabled"). `focus.next` / `focus.previous` are
    always published as `true/false` via the UiTree focus traversal fallback.
  - Providers should use `CommandAvailability::Blocked` (not `NotHandled`) for "owned but currently
    unavailable" states (e.g. `text.copy` with an empty selection) so menus/palettes can disable
    commands deterministically.

## Coverage Checklist (Core Widget Commands)

Source of truth: `crates/fret-app/src/core_commands.rs` (Widget scope)

### Focus

- `focus.next`
  - Availability must be `Available` when focus traversal can run even if no widget explicitly
    handles the command (UiTree default focus traversal contract).
- `focus.previous`
  - Same as `focus.next`.
- `focus.menu_bar`
  - Availability is app/runner specific; expected to be provided by a workspace shell or runner
    integration.
  - Default contract: shells that render an in-window menubar should publish
    `WindowMenuBarFocusService` so cross-surface gating can disable the command when no menubar is
    present.
    - Evidence: `ecosystem/fret-kit/src/workspace_shell.rs` (publishes service),
      `crates/fret-ui/src/tree/commands.rs` (publishes availability snapshot entry)

### Edit / Clipboard

- `edit.copy`
  - Availability should be `Available` when the focused widget exposes a non-empty selection or a
    copyable value.
  - Expected providers:
    - `BoundTextInput` / `BoundTextArea`
      - Evidence: `crates/fret-ui/src/text_input/bound.rs` and `crates/fret-ui/src/text_area/bound.rs`
      - Declarative wiring: `crates/fret-ui/src/declarative/host_widget.rs` (forwards command/availability)
    - `SelectableText` (read-only selection)
    - `NodeGraphCanvas` (non-text selection)
      - Evidence: `ecosystem/fret-node/src/ui/canvas/widget.rs`
      - Tests: `ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs`
    - `fret-ui-kit` list surfaces (non-text selection)
      - Evidence: `ecosystem/fret-ui-kit/src/declarative/list.rs` (`list_virtualized_copyable`)
      - Tests: `ecosystem/fret-ui-kit/src/declarative/list.rs` (`list_virtualized_copyable_reports_availability_and_emits_clipboard_text`)
  - Notes:
    - `text.copy` remains as a legacy alias for text-focused surfaces.
- `edit.cut`
  - Availability should be `Blocked` when the focused widget is read-only.
  - Expected providers:
    - `BoundTextInput` / `BoundTextArea`
      - Evidence: `crates/fret-ui/src/text_input/bound.rs` and `crates/fret-ui/src/text_area/bound.rs`
    - `NodeGraphCanvas`
      - Evidence: `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `edit.paste`
  - Availability depends on editability and clipboard capabilities.
  - Expected providers:
    - `BoundTextInput` / `BoundTextArea`
      - Evidence: `crates/fret-ui/src/text_input/bound.rs` and `crates/fret-ui/src/text_area/bound.rs`
- `edit.select_all`
  - Availability should be `Available` when the focused widget can select content.
  - Expected providers:
    - `BoundTextInput` / `BoundTextArea`
      - Evidence: `crates/fret-ui/src/text_input/bound.rs` and `crates/fret-ui/src/text_area/bound.rs`
    - `SelectableText`
    - `NodeGraphCanvas`
      - Evidence: `ecosystem/fret-node/src/ui/canvas/widget.rs`
  - Notes:
    - Prefer returning `Blocked` (not `NotHandled`) when the focused widget owns the command but has
      no selectable content (e.g. empty text), so command palette / menus can disable deterministically.

### Edit / Clear

- `text.clear`
  - Availability should be `Available` when the focused widget has any text to clear.
  - Expected providers:
    - `BoundTextInput` / `BoundTextArea`
      - Evidence: `crates/fret-ui/src/text_input/bound.rs` and `crates/fret-ui/src/text_area/bound.rs`

## Known Gaps / Next Targets

1) Broader "copy-like" semantics outside of text widgets (listbox item, table row)
   should implement `edit.copy` and provide availability evidence anchors.
