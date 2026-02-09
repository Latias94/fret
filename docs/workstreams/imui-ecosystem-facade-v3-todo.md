# imui Ecosystem Facade v3 - TODO Tracker

Status: In progress (M0+ pending)
Last updated: 2026-02-07

This tracker covers:

- `docs/workstreams/imui-ecosystem-facade-v3.md`

Related:

- `docs/workstreams/imui-ecosystem-facade-v2.md` (baseline)
- `docs/workstreams/docking-multiwindow-imgui-parity.md` (OS-window tear-off parity)
- `docs/workstreams/code-editor-ecosystem-v1.md` (text/editor ecosystem)

Legend:

- [ ] open
- [~] in progress
- [x] done
- [!] blocked / needs decision

Tracking format:

- ID: `IMUIECO3-{area}-{nnn}`
- Areas:
  - `scope` (contracts and boundaries)
  - `float` (floating windows / z-order / focus)
  - `dock` (docking handshake touchpoints)
  - `adapter` (ecosystem ABI and seam evolution)
  - `resp` (response signal graduation decisions)
  - `text` (text/editor integration hooks)
  - `perf` (allocation/perf gates)
  - `test` (nextest/diag/compile gates)
  - `docs` (guides/migration)

---

## M0 - Scope lock + admission checklist

Exit criteria:

- v3 boundaries relative to docking and text ecosystems are explicit.
- breaking-change criteria for floating flags/behavior are documented.

- [~] IMUIECO3-scope-000 Lock v3 scope boundaries and explicit deferrals.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v3.md` (M0 contract notes: activation + breaking criteria).
- [~] IMUIECO3-docs-001 Add floating/z-order/focus admission checklist items.
  - Admission checklist (v3 floating):
    - Any change to activation / bring-to-front / focus choreography requires: at least 1 nextest +
      at least 1 diag script evidence path.
    - Any new `FloatingWindowOptions` field must document: default, breaking criteria, and
      interaction with overlays (dismiss/focus restore) when applicable.
    - Any behavior-changing default must update: this tracker + migration notes in the v3 note.
    - Any perf-sensitive wrapper change must add (or update) a cheap regression gate test.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v3.md` (M0 contract notes: activation + `no_inputs` semantics).
- [x] IMUIECO3-docs-002 Add explicit ImGui reference anchors and mapping notes.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v3.md` ("ImGui Reference Anchors (Audited)").

---

## M1 - Floating window primitives (ImGui-aligned, in-window)

Exit criteria:

- `window(...)`/floating surface has an explicit options/flags API (subset).
- bring-to-front + focus restore are deterministic and gated.
- at least one diag script covers floating + popup coexistence under the new rules.

- [~] IMUIECO3-float-010 Add `WindowFlags`/options surface for in-window floating windows.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`FloatingWindowOptions`, `window_ex`, `window_open_ex`).
  - Evidence: `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` (`activate_on_click`, `inputs_enabled` behavior).
- [x] IMUIECO3-float-011 Add deterministic bring-to-front + focus choreography for floatings.
  - Evidence: `crates/fret-ui/src/declarative/host_widget.rs` (capture-phase `PointerRegion` down hooks).
  - Evidence: `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` (content-click activation requests focus).
  - Evidence: `tools/diag-scripts/imui-float-window-activate-on-content-bring-to-front.json`.
- [x] IMUIECO3-float-012 Add minimal in-window z-order model that composes with overlay arbitration.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`floating_layer`, `FloatWindowLayerZOrder`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_layer_menu_outside_press_dismisses_without_activating_underlay`,
    `floating_layer_popover_outside_press_allows_underlay_activation_when_click_through`).
- [~] IMUIECO3-test-013 Add nextest gates for window flag semantics (close/collapse/resize/move).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (tests covering `activate_on_click`, `inputs_enabled`, `closable`, `movable`, `resizable`, `collapsible`).
- [x] IMUIECO3-test-014 Add/extend `fretboard diag` script(s) for floating + popup + drag/resize.
  - Evidence: `tools/diag-scripts/imui-float-window-activate-on-content-bring-to-front.json`
  - Evidence: `apps/fret-examples/src/imui_floating_windows_demo.rs`
  - Evidence: `crates/fret-ui/src/declarative/host_widget.rs`
- [x] IMUIECO3-float-015 Define a click-through policy surface aligned with ImGui `NoMouseInputs`/`NoInputs`.
  - Reference: `repo-ref/imgui/imgui.h` (`ImGuiWindowFlags_NoMouseInputs`, `ImGuiWindowFlags_NoInputs`).
  - Reference: `repo-ref/imgui/imgui.cpp` (hovered-viewport detection expects `ImGuiViewportFlags_NoInputs` to be honored).
  - Note: current `inputs_enabled=false` is "non-interactive" but not click-through (by design).
  - Evidence (initial click-through): `ecosystem/fret-ui-kit/src/imui.rs` (`FloatingWindowOptions.pointer_passthrough`,
    `FloatingAreaOptions.hit_test_passthrough`, `floating_area_show_ex` wrapping with `HitTestGate`).
  - Evidence (gate): `ecosystem/fret-imui/src/lib.rs` (`floating_window_pointer_passthrough_allows_underlay_hit_testing`).
  - Evidence (`NoInputs`): `ecosystem/fret-ui-kit/src/imui.rs` (`FloatingWindowOptions.no_inputs`, `FloatingAreaOptions.no_inputs`,
    `floating_area_show_ex` wrapping with `InteractivityGate`).
  - Evidence (tests): `ecosystem/fret-imui/src/lib.rs` (`floating_window_no_inputs_allows_underlay_hit_testing`,
    `floating_window_no_inputs_is_skipped_by_focus_traversal`, `no_inputs_is_click_through_and_skips_focus_traversal`).
- [x] IMUIECO3-float-016 Align activation semantics with ImGui `NoBringToFrontOnFocus` (focus vs z-order).
  - Reference: `repo-ref/imgui/imgui.h` (`ImGuiWindowFlags_NoBringToFrontOnFocus`).
  - Evidence (options): `ecosystem/fret-ui-kit/src/imui.rs` (`FloatingWindowOptions.focus_on_click`,
    `FloatingWindowOptions.activate_on_click`).
  - Evidence (wiring): `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` (content pointer-region requests focus
    independent of activation).
  - Evidence (test): `ecosystem/fret-imui/src/lib.rs` (`floating_window_focus_on_click_can_be_independent_from_z_order_activation`).
- [x] IMUIECO3-resp-017 Decide whether keyboard-nav highlight should participate in "hovered" (ImGui-style) or a separate signal.
  - Reference: `repo-ref/imgui/imgui.cpp` (`IsItemHovered`, `IsItemFocused`).
  - Decision: keep `hovered` pointer-driven; add a separate `nav_highlighted` signal + a helper that composes them.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt.nav_highlighted`, `ResponseExt.hovered_like_imgui`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`hit_test_passthrough_keeps_focus_traversal_and_nav_highlight`).
- [ ] IMUIECO3-scope-018 Add an explicit identity ergonomics note (and/or helper) covering ImGui `"##"`/`"###"` patterns.
  - Reference: `repo-ref/imgui/imgui.h` (label/ID guidance) + `repo-ref/imgui/imgui.cpp` (`PushID`, `GetID`).
  - Evidence (current Fret behavior): `ecosystem/fret-imui/src/lib.rs` (`id`, `for_each_keyed`, `for_each_unkeyed`) + `docs/workstreams/imui-authoring-facade-v1.md` (identity section).
- [x] IMUIECO3-float-019 Make drag threshold a theme/metric knob (align with ImGui `MouseDragThreshold`).
  - Reference: `repo-ref/imgui/imgui.h` (`ImGuiIO::MouseDragThreshold` default `6.0f`).
  - Evidence (token + default): `ecosystem/fret-ui-kit/src/theme_tokens.rs` (`component.imui.drag_threshold_px`),
    `ecosystem/fret-ui-kit/src/imui.rs` (`DEFAULT_DRAG_THRESHOLD_PX = 6.0`, `drag_threshold_sq_for`).
  - Evidence (test): `ecosystem/fret-imui/src/lib.rs` (`drag_threshold_metric_controls_drag_start`).
- [x] IMUIECO3-resp-020 Expose a facade-only ImGui-style hovered query helper (`ImGuiHoveredFlags_` subset).
  - Reference: `repo-ref/imgui/imgui.h` (`enum ImGuiHoveredFlags_`, incl. `AllowWhenDisabled`, `AllowWhenBlockedByPopup`,
    `DelayShort/DelayNormal`, `ForTooltip`, `NoNavOverride`).
  - Reference: `repo-ref/imgui/imgui.cpp` (`IsItemHovered` implements nav-highlight participation, delay gating, and disabled gating).
  - Decision: keep `fret-authoring::Response` stable/minimal; add a facade-only query helper surface on `ResponseExt`.
  - Implemented flags (best-effort):
    - `ALLOW_WHEN_DISABLED`: query hover even when the facade suppresses `core.hovered` for disabled widgets.
    - `ALLOW_WHEN_BLOCKED_BY_POPUP`: query hover even when popup policy suppresses hover (pointer occlusion / modal barriers).
    - `NO_NAV_OVERRIDE`: do not treat nav-highlight as hovered.
    - `FOR_TOOLTIP`: expands to `STATIONARY | DELAY_SHORT | ALLOW_WHEN_DISABLED`.
    - `STATIONARY` / `DELAY_SHORT` / `DELAY_NORMAL`: hover intent gating via element-owned timers.
    - `NO_SHARED_DELAY`: disables the window-scoped shared delay for the query (best-effort).
  - Evidence (API): `ecosystem/fret-ui-kit/src/imui.rs` (`ImUiHoveredFlags`, `ResponseExt::is_hovered`).
  - Evidence (mechanism): `crates/fret-ui/src/tree/dispatch.rs` (raw hovered pressable target selection),
    `crates/fret-ui/src/elements/cx.rs` (`PressableState.hovered_raw`) + `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt`).
  - Evidence (tests): `ecosystem/fret-imui/src/lib.rs` (`disabled_scope_blocks_underlay_and_suppresses_hover_and_click`,
    `hovered_for_tooltip_requires_stationary_and_delay_short_even_when_disabled`,
    `hovered_allow_when_blocked_by_popup_reads_underlay_hit_test`).
- [x] IMUIECO3-resp-021 Add a scoped disable helper aligned with ImGui `BeginDisabled` (and define how it affects responses).
  - Reference: `repo-ref/imgui/imgui.h` (`BeginDisabled`) + `repo-ref/imgui/imgui.cpp` (`BeginDisabled`, `EndDisabled`).
  - Decision: disabled items are inert and report `hovered=false` / `pressed=false` / `focused=false` / `clicked=false` by default
    (ImGui-style). Use `ResponseExt::is_hovered(ImUiHoveredFlags::ALLOW_WHEN_DISABLED)` when you need hover queries for disabled items.
  - Evidence (API): `ecosystem/fret-ui-kit/src/imui.rs` (`disabled_scope`, `begin_disabled`).
  - Evidence (policy): `ecosystem/fret-ui-kit/src/imui.rs` (`sanitize_response_for_enabled`, `component.imui.disabled_alpha`).
  - Evidence (tests): `ecosystem/fret-imui/src/lib.rs` (`disabled_scope_blocks_underlay_and_suppresses_hover_and_click`).
- [x] IMUIECO3-resp-022 Fill remaining `ImGuiHoveredFlags_` gaps (popup blocking + delays/stationary).
  - Reference: `repo-ref/imgui/imgui.h` (`AllowWhenBlockedByPopup`, `DelayShort/DelayNormal`, `Stationary`, `ForTooltip`).
  - Evidence (API): `ecosystem/fret-ui-kit/src/imui.rs` (`ImUiHoveredFlags`, `ResponseExt::is_hovered`).
  - Evidence (mechanism): `crates/fret-ui/src/tree/dispatch.rs` (hover allow-when-blocked target selection),
    `ecosystem/fret-ui-kit/src/imui.rs` (`install_hover_query_hooks_for_pressable`).
  - Evidence (tests): `ecosystem/fret-imui/src/lib.rs` (`hovered_allow_when_blocked_by_popup_reads_underlay_hit_test`,
    `hovered_for_tooltip_requires_stationary_and_delay_short_even_when_disabled`).
- [x] IMUIECO3-resp-023 Implement ImGui-style "shared hover delay" semantics (best-effort).
  - Reference: `repo-ref/imgui/imgui.h` (`ImGuiHoveredFlags_NoSharedDelay`, `ImGuiHoveredFlags_DelayShort/DelayNormal`)
    + `repo-ref/imgui/imgui.cpp` (`ImGui::IsItemHovered` shared delay timer logic).
  - Decision: add a window-scoped shared delay store to better match ImGui tooltip hand-feel, and keep `NO_SHARED_DELAY` as an
    escape hatch for query-time behavior.
  - Evidence (mechanism): `ecosystem/fret-ui-kit/src/imui.rs` (`ImUiSharedHoverDelayState`, `shared_hover_delay_on_hover_change`,
    `install_hover_query_hooks_for_pressable`).
  - Evidence (tests): `ecosystem/fret-imui/src/lib.rs` (`no_shared_delay_disables_window_scoped_hover_delay_sharing`).

---

## M2 - Docking/multi-window handshake (tracked, docking-owned)

Exit criteria:

- imui facade touchpoints needed for docking parity are listed and linked to docking workstreams.

- [x] IMUIECO3-dock-020 Document docking handshake touchpoints and required signals/metadata.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v3.md` (M2 touchpoints).
  - Evidence: `ecosystem/fret-docking/src/imui.rs`, `ecosystem/fret-docking/src/facade.rs`.

---

## M3 - Ecosystem extension ABI v1

Exit criteria:

- adapter seam template remains stable and is proven by a non-shadcn example.
- any metadata evolution is justified with duplication-reduction evidence.

- [x] IMUIECO3-adapter-030 Audit adapter seam v2 and list v3 ABI changes (if any).
  - Result: no v3 ABI changes required; v2 seam is the baseline.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v3.md` (M3 contract notes).
  - Evidence: `ecosystem/fret-ui-kit/src/imui/adapters.rs`.
- [x] IMUIECO3-adapter-031 Add one \"external widget crate\" style example (in-tree scaffold is OK).
  - Evidence: `ecosystem/fret-ui-kit/tests/imui_external_adapter_example.rs`.

---

## M4 - Text/editor bridge

Exit criteria:

- text/editor integration is explicit and delegated to the code-editor ecosystem (no fork).

- [ ] IMUIECO3-text-040 Define adapter hooks for editor-grade text surfaces.
- [ ] IMUIECO3-docs-041 Publish \"do not fork text engine\" integration guidance.

---

## M5 - Perf + regression gate upgrade

Exit criteria:

- perf gates are cheap, repeatable, and cover at least one floating hot path.

- [x] IMUIECO3-perf-050 Expand perf guard tests beyond the v2 smoke baseline (target floating hot paths).
  - Evidence: `ecosystem/fret-ui-kit/tests/imui_perf_guard_smoke.rs` (`floating_layer_z_order_does_not_clone_vec_each_frame`).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`FloatWindowLayerZOrderSnapshot`).
- [x] IMUIECO3-test-051 Add a small CI-friendly gate matrix (contracts + perf + diag scripts).
  - Evidence: `tools/diag_gate_imui_v3.ps1`.
