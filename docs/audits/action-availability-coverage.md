# Action Availability Coverage (Widget-Scoped Commands)

This note tracks the current "action availability" surface in Fret and highlights the remaining
gaps vs GPUI-style ergonomics.

Scope: this document focuses on `CommandScope::Widget` commands and the question:

> "Is this action available along the dispatch path to the current focus?"

## Current Mechanism

### Query API (in-process)

- `UiTree::command_availability` and `UiTree::is_command_available`
- GPUI naming parity aliases: `UiTree::action_availability` and `UiTree::is_action_available`

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

### Edit / Clipboard

- `text.copy`
  - Availability should be `Available` when the focused widget exposes a non-empty selection or a
    copyable value.
  - Expected providers:
    - `BoundTextInput` / `BoundTextArea`
      - Evidence: `crates/fret-ui/src/text_input/bound.rs` and `crates/fret-ui/src/text_area/bound.rs`
      - Declarative wiring: `crates/fret-ui/src/declarative/host_widget.rs` (forwards command/availability)
    - `SelectableText` (read-only selection)
- `text.cut`
  - Availability should be `Blocked` when the focused widget is read-only.
  - Expected providers:
    - `BoundTextInput` / `BoundTextArea`
      - Evidence: `crates/fret-ui/src/text_input/bound.rs` and `crates/fret-ui/src/text_area/bound.rs`
- `text.paste`
  - Availability depends on editability and clipboard capabilities.
  - Expected providers:
    - `BoundTextInput` / `BoundTextArea`
      - Evidence: `crates/fret-ui/src/text_input/bound.rs` and `crates/fret-ui/src/text_area/bound.rs`
- `text.select_all`
  - Availability should be `Available` when the focused widget can select content.
  - Expected providers:
    - `BoundTextInput` / `BoundTextArea`
      - Evidence: `crates/fret-ui/src/text_input/bound.rs` and `crates/fret-ui/src/text_area/bound.rs`
    - `SelectableText`
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

1) Broader "copy-like" semantics outside of text widgets (node graph, listbox item, table row)
   should decide whether to:
   - reuse `text.copy`, or
   - introduce a more general `edit.copy` command family.
2) `focus.menu_bar` availability needs an explicit contract between runner shells and UI-kit.
