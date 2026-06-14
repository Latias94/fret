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
- `UiTree::publish_window_command_action_availability_snapshot_filtered` publishes the same
  snapshot shape for a caller-owned command set:
  - it is intended for app/driver surfaces that already know the exact command family they consume,
  - it sorts/dedupes the requested command ids for stable snapshot signatures,
  - it ignores unregistered and non-widget commands,
  - omitted commands remain "unknown" to consumers rather than disabled.
- Consumers combine:
  - `WindowInputContextService` (`InputContext`)
  - `WindowCommandEnabledService` (explicit overrides)
  - `WindowCommandActionAvailabilityService` (dispatch-path availability)
  - via `WindowCommandGatingSnapshot`
  - Note: the retained-runtime helper publishes registered widget-scoped commands and treats
    `NotHandled` on the current dispatch path as "unavailable" (`false`). Filtered/app-owned
    publishers may still omit commands, and consumers should treat missing entries as "unknown",
    not "disabled".
  - Snapshot availability is dispatch-path scoped (current focus/default route plus explicit
    `focus.next` / `focus.previous` and `focus.menu_bar` hooks); it intentionally does not scan
    unfocused subtrees.
  - View/app-owned typed action handlers that are intentionally outside the focused widget route
    must publish an explicit action-route fallback root. Snapshot publication and dispatch both
    consult those roots before the no-focus subtree fallback, which keeps menus, command palettes,
    overlays, and cached `AppUi` view handlers aligned without turning availability into an
    arbitrary whole-tree scan.
    - Evidence: `ElementContext::action_route_fallback_root`,
      `UiTree::command_availability_in_action_route_fallback_roots`,
      `install_app_ui_action_handlers_for_owner`,
      `app_ui_unit_action_handler_publishes_available_snapshot_when_focus_exists`.
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
    - Evidence: `apps/fret-ui-gallery/src/driver/menubar.rs` (publishes service for the first-party in-window menubar),
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
    - `fret-ui-kit` table surfaces (non-text selection)
      - Evidence: `ecosystem/fret-ui-kit/src/declarative/table.rs` (`table_virtualized_copyable`)
      - Tests: `ecosystem/fret-ui-kit/src/declarative/table.rs` (`table_virtualized_copyable_reports_availability_and_emits_clipboard_text`)
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
