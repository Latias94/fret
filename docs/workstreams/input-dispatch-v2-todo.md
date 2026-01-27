# Input Dispatch v2 — TODO Tracker

Status: Complete (v2 shipped; keep updated if new v2 gaps are discovered)

This document tracks executable TODOs for the Input Dispatch v2 workstream.

- Narrative plan: `docs/workstreams/input-dispatch-v2.md`
- Contract gate: `docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
- Coverage audit: `docs/audits/action-availability-coverage.md`

## Active Branch Notes

This workstream has ongoing parallel work. When marking items as `[~]`, prefer noting the active branch name in the TODO
bullet itself (to avoid implying it already landed on `main`).

## Tracking Format

Each TODO is labeled:

- ID: `IDV2-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Verified Building Blocks)

Keep this list short and evidence-backed:

- Dispatch phases exist (`Preview/Capture/Bubble`):
  - Evidence: `crates/fret-runtime/src/input.rs`, `crates/fret-ui/src/tree/dispatch.rs`
- Observer-only pass is type-isolated (`ObserverCx` + `Widget::event_observer`):
  - Evidence: `crates/fret-ui/src/widget.rs`, `crates/fret-ui/src/tree/dispatch.rs`
- Default-action suppression is mechanism-owned (`prevent_default(DefaultAction)`):
  - Evidence: `crates/fret-runtime/src/input.rs`, `crates/fret-ui/src/widget.rs`, `crates/fret-ui/src/tree/dispatch.rs`
- Runner-friendly gating aggregation exists (`WindowCommandGatingSnapshot`):
  - Evidence: `crates/fret-runtime/src/window_command_gating.rs`, `crates/fret-launch/src/runner/desktop/*_menu.rs`

## MVP0 — Mechanism Contract (Must Stay Stable)

- [x] IDV2-rt-001 Define `InputDispatchPhase::{Preview,Capture,Bubble}`.
  - Evidence: `crates/fret-runtime/src/input.rs`
- [x] IDV2-ui-002 Route outside-press/click-through via Preview + `ObserverCx` (no routing mutations).
  - Evidence: `crates/fret-ui/src/widget.rs`, `crates/fret-ui/src/tree/dispatch.rs`, tests `crates/fret-ui/src/tree/tests/outside_press.rs`
- [x] IDV2-ui-003 Implement `prevent_default(DefaultAction)` and apply `DefaultAction::FocusOnPointerDown` by default.
  - Evidence: `crates/fret-runtime/src/input.rs`, `crates/fret-ui/src/widget.rs`, tests `crates/fret-ui/src/tree/tests/prevent_default.rs`
- [x] IDV2-ui-004 Implement capture-phase dispatch for key down/up and core pointer interactions.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs`, tests `crates/fret-ui/src/tree/tests/dispatch_phase.rs`
- [x] IDV2-ui-005 Preserve element-owned timer routing (needed for hover delays / submenu policies).
  - Evidence: `crates/fret-ui/src/elements/access.rs` (`timer_target_node`), `crates/fret-ui/src/tree/dispatch.rs` (`Event::Timer`)

## MVP0.1 — Default Actions Expansion (Keep Mechanism/Policy Boundary Clean)

- [x] IDV2-def-006 Decide which behaviors qualify as mechanism-owned default actions (vs ecosystem policy).
  - Decision tracker: `docs/workstreams/default-actions-v2-todo-input-dispatch-v2.md`
  - Notes: keep v2 limited to `DefaultAction::FocusOnPointerDown` for now to avoid smuggling Radix/shadcn policies into `crates/fret-ui`.
- [x] IDV2-def-007 Expand default actions incrementally with tests (e.g. selection start, scroll routing), if justified.
  - Decision: defer until we have a concrete, boundary-safe, cross-surface default.
  - Evidence: `docs/workstreams/default-actions-v2-todo-input-dispatch-v2.md`

## MVP1 — Action Availability (GPUI `is_action_available` Parity)

- [x] IDV2-cmd-010 Add tri-state availability query (`Available/Blocked/NotHandled`) on the dispatch path.
  - Evidence: `crates/fret-ui/src/widget.rs` (`CommandAvailability`), `crates/fret-ui/src/tree/commands.rs`
- [x] IDV2-cmd-011 Add runner bridge snapshot (`WindowCommandActionAvailabilityService`).
  - Evidence: `crates/fret-runtime/src/window_command_action_availability.rs`
- [x] IDV2-cmd-012 Expose GPUI naming parity aliases.
  - Evidence: `crates/fret-ui/src/tree/commands.rs` (`is_action_available` alias)

## MVP2 — Cross-Surface Gating (Single Source of Truth)

- [x] IDV2-gate-020 Aggregate gating inputs (when + overrides + availability) as data-only snapshot.
  - Evidence: `crates/fret-runtime/src/window_command_gating.rs` (`WindowCommandGatingSnapshot`)
- [x] IDV2-gate-021 Use gating snapshot for OS menu validation (native runner).
  - Evidence: `crates/fret-launch/src/runner/desktop/macos_menu.rs`, `crates/fret-launch/src/runner/desktop/windows_menu.rs`
- [x] IDV2-gate-022 Ensure command palette uses the same gating snapshot path (no divergent heuristics).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (publishes `WindowCommandGatingService` snapshot),
    `ecosystem/fret-ui-shadcn/src/command.rs` (consumes gating snapshot)

## MVP3 — Frozen Gating Target (Overlay Does Not Pollute Availability)

- [x] IDV2-freeze-030 Freeze command palette gating while open (editor-style discoverability).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (`WindowCommandGatingService::push_snapshot`)
- [x] IDV2-freeze-031 Support nested overlays (stackable gating snapshots per window).
  - Evidence: `crates/fret-runtime/src/window_command_gating.rs` (`push_snapshot`, `pop_snapshot`, `WindowCommandGatingHandle`),
    `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (command palette pushes a snapshot and stores a handle; closes pops it),
    `ecosystem/fret-ui-shadcn/src/command.rs` (gating override tests use `push_snapshot`)

## MVP4 — Coverage Targets (Keep Expanding Incrementally)

- [x] IDV2-avail-040 Core text commands availability is selection/editability/capability-sensitive.
  - Tracker: `docs/audits/action-availability-coverage.md`
  - Evidence anchors: `crates/fret-ui/src/text_input/bound.rs`, `crates/fret-ui/src/text_area/bound.rs`, `crates/fret-ui/src/declarative/host_widget.rs`
  - Tests: `crates/fret-ui/src/declarative/tests/interactions.rs` (`text_input_select_all_is_blocked_when_empty`, `text_area_select_all_is_blocked_when_empty`)
- [x] IDV2-avail-041 Define a general “copy-like” command family outside text widgets (tables/lists/node graphs).
  - Evidence: `crates/fret-app/src/core_commands.rs` (`edit.{copy,cut,paste,select_all}`),
    `crates/fret-ui/src/text_input/widget.rs` + `crates/fret-ui/src/text_area/widget.rs` (text alias),
    `ecosystem/fret-node/src/ui/canvas/widget.rs` (node graph availability + command routing),
    `ecosystem/fret-ui-kit/src/declarative/list.rs` (`list_virtualized_copyable`),
    `ecosystem/fret-ui-kit/src/declarative/table.rs` (`table_virtualized_copyable`)
  - Tests: `crates/fret-ui/src/declarative/tests/interactions.rs` (`text_input_supports_edit_select_all_and_copy`),
    `ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs`,
    `ecosystem/fret-ui-kit/src/declarative/list.rs` (`list_virtualized_copyable_reports_availability_and_emits_clipboard_text`),
    `ecosystem/fret-ui-kit/src/declarative/table.rs` (`table_virtualized_copyable_reports_availability_and_emits_clipboard_text`)
- [x] IDV2-avail-042 Define `focus.menu_bar` contract between runner shells and UI-kit.
  - Evidence: `crates/fret-runtime/src/window_menu_bar_focus.rs`, `crates/fret-ui/src/tree/commands.rs`, `ecosystem/fret-kit/src/workspace_shell.rs`
  - Tests: `crates/fret-ui/src/tree/tests/window_command_action_availability_snapshot.rs` (`action_availability_snapshot_publishes_focus_menu_bar_gating`)
- [x] IDV2-avail-043 Allow declarative components to report command availability via policy hooks.
  - Evidence: `crates/fret-ui/src/action.rs` (`OnCommandAvailability`, `CommandAvailabilityActionCx`),
    `crates/fret-ui/src/elements/cx.rs` (`command_on_command_availability_for`),
    `crates/fret-ui/src/declarative/host_widget.rs` (invokes hook during `command_availability`)
  - Tests: `crates/fret-ui/src/declarative/tests/interactions.rs` (`declarative_command_availability_hooks_participate_in_dispatch_path_queries`)

## MVP5 — Overlay / Menu Parity (Radix-shadcn Hand Feel)

- [x] IDV2-ovl-050 Normalize “present vs interactive” for overlay close transitions (click-through + no observer routing).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/state.rs`, `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- [x] IDV2-ovl-051 Stabilize submenu safe-hover + timer routing (menu hover intent under caching/multi-layer routing).
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menu/*`, shadcn tests in `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- [x] IDV2-ovl-052 Lock menu open modality + entry focus (pointer-open vs keyboard-open) as a reusable policy contract.
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menu/root.rs` (`MenuInitialFocusTargets`, modality-gated `initial_focus`),
    `ecosystem/fret-ui-shadcn/src/{dropdown_menu.rs,menubar.rs,context_menu.rs}` (wires focus targets).
  - Conformance: shadcn tests cover keyboard-open entry focus and pointer-open “focus content, not first item” for
    DropdownMenu / Menubar / ContextMenu.
  - Notes: keep policy in `ecosystem/*`, but ensure mechanism hooks exist (`prevent_default`, focus hooks, timers, auto-focus hooks).
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menu/root.rs` (plumbs `on_open_auto_focus/on_close_auto_focus`),
    tests `ecosystem/fret-ui-shadcn/src/{dropdown_menu.rs,menubar.rs,context_menu.rs}` (auto-focus preventDefault conformance).
- [x] IDV2-ovl-054 Align Select open modality + entry focus (pointer focuses listbox; keyboard focuses selected entry).
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/select.rs` (`SelectInitialFocusTargets`),
    `ecosystem/fret-ui-shadcn/src/select.rs` (wires `SelectInitialFocusTargets` into overlay requests).
  - Conformance: `ecosystem/fret-ui-shadcn/src/select.rs` tests `select_pointer_open_focuses_listbox_container` and
    `select_keyboard_open_focuses_selected_entry`.
- [x] IDV2-ovl-055 Align Combobox open modality + entry focus (pointer/keyboard open autofocuses the search input).
  - Evidence: `ecosystem/fret-ui-shadcn/src/combobox.rs` (combobox is `Popover` + `CommandPalette` with `input_role=ComboBox`).
  - Conformance: `ecosystem/fret-ui-shadcn/src/combobox.rs` tests `combobox_pointer_open_auto_focuses_search_input` and
    `combobox_keyboard_open_auto_focuses_search_input`.
- [x] IDV2-ovl-053 Decide hover/tooltip request caching policy under view caching (avoid stale overlays).
  - Notes: align with overlay presence (`present` vs `interactive`) so close transitions remain click-through.

## Open Questions (Keep Short)

- Resolved: `WindowCommandGatingService` is stack-based per window so nested overlays can publish gating snapshots without clobbering each other (IDV2-freeze-031).
- Resolved: UI diagnostics expose a command gating trace for debugging cross-surface enablement.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`command_gating_trace_for_window`, `command_gating_decision_trace`)
