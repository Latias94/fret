# imui Ecosystem Facade (egui/imgui-like ergonomics) v1 - TODO Tracker

Status: Draft
Last updated: 2026-02-06

This tracker covers the work described in:

- `docs/workstreams/imui-ecosystem-facade-v1.md`

Related:

- `docs/workstreams/imui-authoring-facade-v2.md` (implemented; authoring frontend baseline)
- `docs/adr/0175-unified-authoring-builder-surface-v1.md` (patch vocabulary)
- Docking parity: `docs/workstreams/docking-multiwindow-imgui-parity.md`
- Overlays policy split: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`

Legend:

- [ ] open
- [~] in progress
- [x] done
- [!] blocked / needs decision

Tracking format:

- ID: `IMUIECO-{area}-{nnn}`
- Areas:
  - `scope` (taxonomy, ownership, invariants)
  - `api` (public facade surface)
  - `resp` (`Response` parity expansion)
  - `controls` (buttons/inputs/etc)
  - `overlays` (menus/popovers/tooltips)
  - `float` (floating windows/areas)
  - `perf` (allocation/perf guidance, caching)
  - `demo` (proof apps)
  - `test` (nextest/diag/compile smoke)
  - `docs` (guides, migration notes)

---

## M-1 - Fearless refactor (pre-open-source)

Exit criteria:

- The shared `Response` contract is owned by `ecosystem/fret-authoring` (not `fret-imui`).
- `ecosystem/fret-imui` is policy-light (builder + identity + output sink).
- The facade surface is hosted in `ecosystem/fret-ui-kit` behind the existing `imui` feature.

- [x] IMUIECO-scope-000 Move minimal `Response` to `ecosystem/fret-authoring` (breaking change OK).
  - Evidence: `ecosystem/fret-authoring/src/lib.rs` (`Response`, `UiWriter`).
- [x] IMUIECO-scope-001 Slim `ecosystem/fret-imui` dependencies (move policy/widget helpers to ui-kit where appropriate).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (crate docs: policy-light stance).
- [x] IMUIECO-scope-002 Decide whether to extract the facade into a dedicated crate later (default: keep in `fret-ui-kit` for v1).
  - Decision: keep in `fret-ui-kit` (feature `imui`) for v1; revisit if surface size grows.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (feature-gated facade module).

---

## M0 - Lock remaining seams (decisions first)

Exit criteria:

- Delegation strategy for `Response` is chosen (no duplicated widget policy).
- Scope is documented: what lives in `fret-imui` vs `fret-ui-kit` (imui facade) vs `fret-ui-shadcn` (visuals).

- [x] IMUIECO-scope-010 Decide whether shadcn integration is via an optional feature or a dedicated adapter module/crate.
  - Decision: dedicated adapter module under `fret-ui-kit` imui facade (`fret_ui_kit::imui::adapters::shadcn`), not a frontend feature toggle on `fret-imui`.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v1.md` (section 5.6.1).
- [x] IMUIECO-scope-011 Choose a canonical delegation seam for returning `Response` from canonical components.
  - Decision: element-id based delegation + reporter hook; canonical components own state machines, facade only maps signals to `ResponseExt`.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v1.md` (section 5.6.2).
- [x] IMUIECO-scope-012 Decide whether "tear-off to OS window" is docking-only for v1 (recommended) or generalized.
  - Decision: docking-only for v1; non-docking `ui.window(...)` / `ui.area(...)` stay in-window.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v1.md` (section 5.6.3).
- [x] IMUIECO-scope-013 Define the `ResponseExt` signal storage model (transient vs element-local state) and document it.
  - Decision: hybrid model (clear-on-read transients for edge events, element-local state for sessions, last-bounds two-frame stabilization for geometry).
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v1.md` (section 5.6.4).
- [x] IMUIECO-scope-014 Define a tiered delegation rule for official widgets (primitive wrappers vs canonical component adapters).
  - Decision: Tier 1 canonical adapters by default, Tier 2 primitive fallback only when necessary, Tier 3 parallel complex-policy implementation disallowed in v1.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v1.md` (section 5.6.5).

---

## M1 - Response parity expansion (egui/imgui-style signals)

Exit criteria:

- The facade exposes a richer `Response` surface that covers the most common immediate-mode queries.

- [x] IMUIECO-resp-010 Add click variants (secondary + double click).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt::{secondary_clicked,double_clicked}`, `UiWriterImUiFacadeExt::button`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`right_click_sets_context_menu_requested_true_once`, `double_click_sets_double_clicked_true_once`).
- [x] IMUIECO-resp-010b Add long-press / press-and-hold signal (touch-first; schedule-aware).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt::{long_pressed,press_holding}`, timer-backed scheduling).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`long_press_sets_long_pressed_true_once_and_reports_holding`).
  - Evidence: `crates/fret-ui/src/declarative/host_widget/event/hooks.rs` (`TimerHookHost::record_transient_event` keeps timer hooks parity with pointer/key hooks).
  - Evidence: `apps/fret-examples/src/imui_response_signals_demo.rs` (demo report includes `long` + `holding`).
- [x] IMUIECO-resp-011 Add drag lifecycle signals (started/dragging/stopped) and useful geometry/delta fields.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt::drag`, `UiWriterImUiFacadeExt::button`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`drag_started_stopped_and_delta_are_consistent`).
- [x] IMUIECO-resp-012 Add context-menu request signal (right click / keyboard menu key).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt::context_menu_requested`, key hook + right-click hook).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`right_click_sets_context_menu_requested_true_once`, `shift_f10_sets_context_menu_requested_true_once`).
- [x] IMUIECO-resp-012b Add context-menu anchor + trigger identity for popup alignment.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt::{id,context_menu_anchor}`).
- [x] IMUIECO-resp-013 Document "two-frame stabilization" where geometry is sourced from last-frame bounds.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v1.md` (section 5.3).

---

## M2 - Controls + containers (immediate-mode standard library)

Exit criteria:

- Common controls are authorable in a single immediate-mode style, backed by canonical components.

- [~] IMUIECO-controls-020 Button/checkbox/toggle wrappers that return `Response` without duplicating policy.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::{button,checkbox_model}`)
- [x] IMUIECO-controls-021 Input/textarea wrappers (coordinate with the code editor ecosystem; don't duplicate).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`InputTextOptions`, `TextAreaOptions`, `UiWriterImUiFacadeExt::{input_text_model,textarea_model}`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`input_text_model_reports_changed_once_after_text_input`, `textarea_model_reports_changed_once_after_text_input`).
- [ ] IMUIECO-controls-022 Slider/select/switch wrappers (shadcn-aligned when enabled).
- [x] IMUIECO-api-023 Container helpers (`horizontal`, `vertical`, `grid`, `scroll`) that prefer `UiBuilder` patch vocabulary.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`HorizontalOptions`, `VerticalOptions`, `GridOptions`, `ScrollOptions`, `UiWriterImUiFacadeExt::{horizontal,vertical,grid,scroll}`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`container_helpers_layout_horizontal_vertical_grid_and_scroll`).
- [x] IMUIECO-api-024 `push_id` / scoped identity helpers mirroring egui/imgui patterns.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ImUiFacade::{id,push_id,for_each_keyed}`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`ImUi::push_id`, `push_id_keeps_changed_signal_stable_after_reorder`).

---

## M3 - Overlays + floating windows/areas (ImGui-aligned)

Exit criteria:

- Overlays have convenient immediate-mode entry points with correct dismissal/focus policy.
- Floating windows/areas exist as first-class outcomes in-window, aligned with ImGui-style UX.

- [x] IMUIECO-overlays-030 Menu/popover/tooltip convenience wrappers built on `OverlayController`.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::{open_popup_at,begin_popup_menu,begin_popup_context_menu,menu_item}`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`context_menu_popup_opens_on_right_click_and_closes_on_outside_click`).
- [x] IMUIECO-overlays-031 Align menu-like popup focus policy (Radix-style initial focus + restore).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`begin_popup_menu_ex`: `primitives::menu::root::dismissible_menu_request_with_modal` + `MenuInitialFocusTargets`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`context_menu_popup_keyboard_open_focuses_first_item_and_escape_restores_trigger_focus`).
- [x] IMUIECO-overlays-032 Add a minimal `BeginPopupModal`-style primitive (ImGui-aligned).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`open_popup`, `begin_popup_modal(_ex)`, `PopupModalOptions`).
- [x] IMUIECO-overlays-033 Add minimal menu keyboard navigation (roving focus) for imui popups.
  - Target: ArrowUp/ArrowDown + Home/End to move focus between menu items.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`menu_item_impl`: key handlers + `ImUiMenuNavState`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`context_menu_popup_arrow_keys_move_focus_between_items`).
- [x] IMUIECO-overlays-034 Add checkbox/radio menu items (ImGui-style) with semantics.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`menu_item_checkbox_ex`, `menu_item_radio_ex`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`menu_item_checkbox_stamps_semantics_checked_state`).
- [x] IMUIECO-float-031 Implement a floating **area** primitive in `fret-ui-kit` (policy-heavy):
  - move (drag) + z-order + focus activation,
  - predictable hit-testing with overlays,
  - degrade cleanly when multi-window is unavailable (always in-window).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::{floating_area,floating_area_drag_surface_ex}`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_area_moves_when_dragging_drag_surface`, `floating_area_bring_to_front_updates_hit_test_order`).
- [x] IMUIECO-float-032a Add a minimal floating window skeleton (in-window) with a draggable title bar.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::floating_window`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_window_moves_when_dragging_title_bar`).
- [x] IMUIECO-float-032b Add ImGui-style `open` model + close button for floating windows.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::floating_window_open`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_window_close_button_sets_open_false`).
- [x] IMUIECO-float-032c Add `Esc`-to-close for ImGui-style `open` windows (when focused).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`floating_window_open`: `KeyCode::Escape` handler).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_window_escape_sets_open_false_after_focusing_title_bar`).
- [x] IMUIECO-float-032d Add bring-to-front z-order management for floating windows (ImGui-style activation).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::floating_layer`, `KEY_FLOAT_WINDOW_ACTIVATE`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_layer_bring_to_front_updates_hit_test_order`).
- [x] IMUIECO-float-032e Add minimal resize handles for floating windows (v1: edges + corners; diagonal cursor supported; min/max configurable).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::floating_window_resizable`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_window_resizes_when_dragging_corner_handle`).
- [x] IMUIECO-float-032f Add ImGui-style title-bar double-click collapse/expand for floating windows.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED`, `FloatingWindowResponse::collapsed`, double-click hook in `floating_area_drag_surface_element`).
  - Evidence: `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` (collapse toggle + collapsed layout branch + resize handles hidden while collapsed).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_window_title_bar_double_click_toggles_collapsed`).
  - Evidence: `apps/fret-examples/src/imui_floating_windows_demo.rs` (demo hint includes double-click collapse/expand behavior).
- [x] IMUIECO-float-032 Layer a floating **window chrome** policy on top of the area:
  - title bar (drag surface), close button, Esc-to-close,
  - resize handles + resize session state,
  - z-order activation when nested in `floating_layer(...)`.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`floating_window_impl_on_area` using `floating_area_ex`).
  - Evidence: `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` (`render_floating_window_in_area`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (floating window tests still pass under nextest).
- [x] IMUIECO-float-033 Add `fret-ui-kit` immediate wrappers (`ui.area(...)`, `ui.window(...)`) returning meaningful interaction results.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`FloatingAreaResponse`, `FloatingWindowResponse`).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::{area,window,window_open,window_resizable}`).
- [ ] IMUIECO-float-034 Decide OS-window promotion scope:
  - docking-only for v1 (recommended), or
  - generalized capability-gated promotion later.

---

## M4 - Demos, tests, and perf guidance

Exit criteria:

- Demos exist and are stable proof points.
- Basic tests exist to prevent regressions in signals and floating behavior.
- Perf guidance is written down (allocation patterns, caching boundaries, virtualization).

- [x] IMUIECO-demo-040 Add a minimal demo showing `Response` parity signals (click/drag/context menu).
  - Evidence: `apps/fret-demo/src/bin/imui_response_signals_demo.rs`
  - Evidence: `apps/fret-examples/src/imui_response_signals_demo.rs`
- [x] IMUIECO-demo-041 Add a floating-window demo (in-window float + overlay interactions).
  - Evidence: `apps/fret-demo/src/bin/imui_floating_windows_demo.rs`
  - Evidence: `apps/fret-examples/src/imui_floating_windows_demo.rs`
- [x] IMUIECO-test-042 Add nextest coverage for facade crates (smoke + key behavior tests):
  - click variants are delivered once (clear-on-read),
  - drag lifecycle is consistent across frames,
  - context-menu request is stable (mouse + keyboard if supported).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (tests: `click_sets_clicked_true_once`, `double_click_sets_double_clicked_true_once`, `right_click_sets_context_menu_requested_true_once`, `shift_f10_sets_context_menu_requested_true_once`, `drag_started_stopped_and_delta_are_consistent`)
- [x] IMUIECO-test-043 Add a wasm compile smoke harness for the facade surface.
  - Evidence: `.github/workflows/ci.yml` (`Wasm Compile Smoke (imui facade)`)
  - Evidence (local): `cargo check -p fret-authoring -p fret-imui -p fret-ui-kit --features imui --target wasm32-unknown-unknown`
- [ ] IMUIECO-perf-044 Add a short perf guide (avoid allocations, prefer keyed identity, use virtualization/caching).
- [ ] IMUIECO-docs-045 Document extension guidelines for third-party widget crates (author once, adapter modules).
- [x] IMUIECO-test-046 Add one `fretboard diag` script covering floating window drag/resize + overlay coexistence (regression gate).
  - Evidence: `tools/diag-scripts/imui-float-window-drag-resize-context-menu.json`
