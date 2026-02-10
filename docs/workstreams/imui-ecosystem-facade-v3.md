# imui Ecosystem Facade v3 (ImGui Parity + Ecosystem ABI + Perf Ceilings)

Status: Draft (workstream note; not an ADR)
Last updated: 2026-02-07

This workstream starts after `imui` ecosystem facade v2 is locked and complete.

v2 outcome (baseline for v3):

- `fret-authoring` defines the stable minimal authoring contract (`Response`, `UiWriter`).
- `fret-imui` remains a minimal immediate-mode frontend (policy-light).
- `fret-ui-kit::imui` hosts the egui/imgui-like facade wrappers behind an optional feature.
- popup/select, focus restore, and floating coexistence have regression gates.
- perf guidance is converted into enforceable gates.

Related:

- `docs/workstreams/imui-ecosystem-facade-v2.md`
- `docs/workstreams/imui-ecosystem-facade-v2-todo.md`
- `docs/workstreams/imui-ecosystem-facade-v2-m5-readiness-review.md`
- `docs/workstreams/imui-ecosystem-facade-perf-v1.md`
- `docs/workstreams/imui-imgui-parity-audit-v1.md` (behavior-focused parity audit notes)
- `docs/workstreams/docking-multiwindow-imgui-parity.md` (OS-window tear-off parity)
- `docs/workstreams/code-editor-ecosystem-v1.md` (text/editor ecosystem)

---

## 0) ImGui Reference Anchors (Audited)

This workstream uses Dear ImGui (C++) as the primary behavior reference for "immediate-mode editor UI feel".

Note: `repo-ref/imgui` is local state (not committed). When citing behavior, record the exact commit:

- `git -C repo-ref/imgui rev-parse --short HEAD` (example on one machine: `913a3c605`)
- If `repo-ref/` is not present in your worktree, run the same command against your local snapshot:
  - `git -C <path-to-imgui> rev-parse --short HEAD`
  - Windows tip: `New-Item -ItemType Junction repo-ref -Target <path-to-repo-ref>`

Authoritative anchors used for v3 audits:

- Window flags and semantics vocabulary:
  - `repo-ref/imgui/imgui.h` (`enum ImGuiWindowFlags_`, incl. `ImGuiWindowFlags_NoMove`, `NoResize`,
    `NoCollapse`, `NoBringToFrontOnFocus`, `NoMouseInputs`, `NoInputs`)
- Focus + z-order choreography:
  - `repo-ref/imgui/imgui.cpp` (`FocusWindow`, `BringWindowToDisplayFront`, `BringWindowToDisplayBehind`)
- Item query semantics (hover/active/focus/click):
  - `repo-ref/imgui/imgui.cpp` (`IsItemHovered`, `IsItemActive`, `IsItemFocused`, `IsItemClicked`)
- ID stack / stable identity patterns:
  - `repo-ref/imgui/imgui.cpp` (`PushID`, `PopID`, `GetID`)
  - `repo-ref/imgui/imgui.h` (FAQ-style guidance: loops should `PushID()` or use `"##suffix"` / `"###id"` patterns)
- Popups / context menus:
  - `repo-ref/imgui/imgui.cpp` (`OpenPopupEx`, `BeginPopupEx`, `BeginPopupContextItem`,
    `BeginPopupContextWindow`, `BeginPopupContextVoid`, `CloseCurrentPopup`, `ClosePopupToLevel`)
- Drag threshold defaults (policy knob):
  - `repo-ref/imgui/imgui.h` (`ImGuiIO::MouseDragThreshold` default `6.0f`)
- Multi-viewport / platform windows (docking + viewports):
  - `repo-ref/imgui/imgui.cpp` (`UpdateViewportsNewFrame`, `UpdateViewportsEndFrame`,
    `RenderPlatformWindowsDefault`)
  - `repo-ref/imgui/docs/BACKENDS.md` (backend contract, incl. honoring `ImGuiViewportFlags_NoInputs`)

Mapping notes (Fret vs ImGui):

- `FloatingWindowOptions` (`ecosystem/fret-ui-kit/src/imui.rs`) is intended to mirror a *subset* of `ImGuiWindowFlags_*`:
  - `movable` ↔ `ImGuiWindowFlags_NoMove`
  - `resizable` ↔ `ImGuiWindowFlags_NoResize`
  - `collapsible` ↔ `ImGuiWindowFlags_NoCollapse`
  - `activate_on_click` is closest to (but not identical to) `ImGuiWindowFlags_NoBringToFrontOnFocus`:
    - in Fret it gates bring-to-front activation for **content clicks**, **title-bar drag surfaces**, and
      **resize handles**.
  - `inputs_enabled=false` is closest in intent to `ImGuiWindowFlags_NoInputs` but **does not** currently
    implement ImGui's "mouse pass-through" (`NoMouseInputs`) by itself. In Fret v3, `no_inputs` means
    "rendered but non-interactive" and is intentionally **not click-through** by default.
  - `pointer_passthrough=true` provides an explicit click-through outcome (hit-test transparent subtree),
    and it is intended to mirror ImGui's `NoMouseInputs` outcome (pointer pass-through) while still
    allowing focus traversal / keyboard navigation.
- `ResponseExt` is the immediate-mode *return value* equivalent of calling a sequence of `IsItem*()` queries
  after rendering an ImGui item. Important divergence:
  - ImGui's `IsItemHovered()` can return true under keyboard/gamepad nav highlight. Today, Fret's
    `Response.core.hovered` is pointer-hover driven; use `ResponseExt.nav_highlighted` /
    `ResponseExt.hovered_like_imgui()` when you want nav-highlight-as-hover parity.
- Popup lifetime differs:
  - In ImGui, `OpenPopupEx()` marks a popup open in the open stack until closed by outside click, activation,
    or `CloseCurrentPopup()`.
  - In Fret, popup scopes are kept alive by `begin_popup_*` calls (`keep_alive_frame`) to avoid stale overlays
    when a declarative subtree stops submitting a popup.
- Context menu helpers differ:
  - ImGui commonly uses `BeginPopupContextItem(NULL)` (implicit "last item" identity).
  - Fret favors explicit identity: wrappers return `ResponseExt` (including `id` + last-bounds), and
    `begin_popup_context_menu(id, trigger_response, ...)` consumes the trigger response.

Known gaps vs ImGui (tracked in this v3 TODO):

- `NoMouseInputs` (pointer pass-through) vs `NoInputs` (pointer + focus traversal disabled) are split
  (`hit_test_passthrough` / `pointer_passthrough` vs `no_inputs`), but more ImGui-style flag surfaces are still missing.
- Focus vs z-order activation is split (`focus_on_click` vs `activate_on_click`), but this is still not a full
  `NoBringToFrontOnFocus` analog (ImGui’s window activation model is more nuanced).
- No ID-stack ergonomics (`PushID`-style nesting, `"##"` / `"###"` label parsing) at the facade level; callers must
  explicitly scope identity with `ui.push_id(...)` / `ui.keyed(...)`.
- `Response.core.hovered` remains pointer-hover driven, but `ResponseExt` now exposes an explicit `nav_highlighted`
  signal and a `hovered_like_imgui()` helper (pointer-hover OR nav-highlight).
- A facade-only hovered query helper exists: `ResponseExt::is_hovered(ImUiHoveredFlags)` implements a subset of
  ImGui-style hovered query flags (notably `ALLOW_WHEN_DISABLED`, `ALLOW_WHEN_BLOCKED_BY_POPUP`, `NO_NAV_OVERRIDE`,
  `NO_SHARED_DELAY`, and hover intent gating via `STATIONARY` / `DELAY_SHORT` / `DELAY_NORMAL` / `FOR_TOOLTIP`).
  A window-scoped shared hover delay is implemented (best-effort) to match ImGui tooltip hand-feel.
  Remaining gaps include most of `ImGuiHoveredFlags_` (window/root hierarchy flags, overlap rules, active-item suppression).
- Scoped disable helper is available (`disabled_scope` / `begin_disabled`), and disabled items suppress interaction
  signals by default (`hovered=false`, `clicked=false`, etc.).
- Drag threshold is expressed as a theme metric knob (`component.imui.drag_threshold_px`, default `6px`).

## 1) Why v3

v3 shifts focus from "stabilize the facade surface" (v2) to:

1) **ImGui-aligned floating window primitives** (hand-feel + flags + z-order + focus choreography).
2) **Ecosystem extension ABI** that makes third-party immediate wrappers predictable and cheap to adopt.
3) **Performance ceilings**: keep wrappers allocation-light and add regression gates for hot paths that
   can silently degrade as the facade grows.

This is intentionally "fearless refactor friendly" (pre-release), but still gate-driven: new behavior
must come with evidence (tests/diag) and explicit ownership decisions.

---

## 2) Invariants (Do Not Break)

- `fret-imui` remains policy-light and dependency-minimal.
- Canonical state machines remain single-sourced; facade wrappers map/report signals, not re-implement.
- OS-window promotion remains docking-owned (capability-driven); non-docking floatings stay in-window.
- "Immediate control flow" and "unified patch vocabulary" must remain composable (no parallel styling world).

---

## 3) Milestones (v3)

### M0 - v3 scope lock + admission checklist

- Confirm v3 boundaries relative to docking and text ecosystems.
- Define what is considered a breaking change for floating window flags/behavior.
- Add/refresh contribution checklist entries specific to floating/z-order/focus behavior.

M0 contract notes (normative for v3 work):

- **Activation / bring-to-front (in-window floating)**:
  - Default behavior is ImGui-like: pointer down anywhere inside a floating window (title bar or
    content) activates the window for z-order purposes when nested under a `floating_layer(...)`.
  - Activation must be recorded in a way that is robust to child controls stopping pointer event
    propagation (i.e. it must still activate when clicking a pressable inside the window).
  - Activation should also move keyboard focus into the floating surface (either the clicked control
    or a surface-level focus proxy) to keep shortcut routing deterministic.
  - Evidence: `crates/fret-ui/src/declarative/host_widget.rs` (capture-phase hook path for `PointerRegion`).
  - Evidence: `tools/diag-scripts/imui-float-window-activate-on-content-bring-to-front.json`.
  - `no_inputs` disables activation and all pointer interactions for the floating window surface.
- **Overlay composition (menu/popover vs floating z-order)**:
  - A menu-like overlay (`OverlayRequest::dismissible_menu`, `disableOutsidePointerEvents=true`) must
    dismiss on outside press without click-through: the underlay floating surface must not activate
    and in-window z-order must not change as a result of the outside press.
  - A click-through popover (`OverlayRequest::dismissible_popover`) dismisses on outside press and
    still allows the underlay floating surface to receive the click (activation / bring-to-front).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_layer_menu_outside_press_dismisses_without_activating_underlay`,
    `floating_layer_popover_outside_press_allows_underlay_activation_when_click_through`).
- **`no_inputs` semantics (portable minimum)**:
  - `no_inputs` means "rendered but non-interactive": the window surface must not activate,
    capture, drag, resize, or allow child pressables to receive pointer input.
  - `no_inputs` does **not** imply "click-through" by default. Hit-test passthrough is explicitly
    deferred until a capability-gated policy is designed.
- **No parallel runtime / no policy duplication**:
  - Immediate wrappers must not re-implement canonical component state machines.
  - Any new flags/options must remain a facade-layer policy surface (`fret-ui-kit::imui`), not a
    mechanism-layer contract.
- **Breaking criteria (v3 floating flags/behavior)**:
  - Changes to default activation, move/resize/collapse/close semantics, or focus/dismiss
    choreography are treated as breaking and require:
    - a TODO tracker update with evidence anchors (tests/diag/docs),
    - explicit migration notes when call-site expectations change.

## 0.1 Recently fixed regressions (carry-forward notes)

- Windows fractional DPI (150%, `scale_factor=1.5`) floating window text wrapping:
  - Pre-fix, wrapped `Text` could under-measure height during sizing and later paint a blob taller than its layout
    bounds, overlapping following items. This presented as "text shifts/misalignment after dragging" because the
    overlap is most visible during interactive move.
  - Fix landed in `crates/fret-ui/src/declarative/host_widget/measure.rs`: treat `Text`-like intrinsic height as
    independent of parent height constraints when `height=Auto` and no `max_height` is set.
  - Gate: `tools/diag-scripts/imui-float-window-text-wrap-no-overlap-150.json`.
- Windows fractional DPI title bar spill into content (fixed):
  - Pre-fix, title text could wrap during min-content probes and paint into the body area, which reads as a layout
    bug in immediate-mode demos (especially when dragging/resizing).
  - Fix landed in `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs`: clip title bar contents and force
    the title to a single line with truncation (`wrap=None`, `overflow=Ellipsis`) + `min_width=0` flex shrink.
  - Gate: `tools/diag-scripts/imui-float-window-titlebar-drag-screenshots.json`.

### M1 - Floating window primitives (ImGui-aligned, in-window)

Goal: bring the in-window floating `window(...)` surface closer to Dear ImGui semantics where it is
portable and layering-correct.

Examples of v3 M1 targets:

- A `WindowFlags`/options surface (subset) for `window(...)`:
  - title bar / close / collapse toggles,
  - resizable/movable constraints,
  - focus-on-appearing policy,
  - "no inputs" / pass-through behavior where applicable.
- Deterministic **bring-to-front** + focus restore behavior (within a window overlay stack).
- Minimal z-order model for in-window floating windows that composes with existing overlay rules.

### M2 - Docking/multi-window handshake (ImGui parity track)

Goal: align the docking tear-off/multi-viewport behavior with ImGui parity workstreams while keeping
ownership in the docking layer.

v3 only tracks the **imui facade touchpoints** (what wrappers need, what signals are required, what
to gate), but does not move docking policy into `imui`.

M2 touchpoints (normative for v3 work):

- **Embed the dock host (imui authoring)**:
  - Use `fret_docking::imui::dock_space_with(...)` to embed a dock space inside an immediate tree
    without re-implementing retained-bridge wiring in every app.
  - The dock host must be submitted every frame for every participating window (do not conditionally
    omit it when panels are hidden). See: `docs/docking-arbitration-checklist.md` (Driver integration checklist).
  - The `configure(app, window)` callback is the app seam to:
    - ensure panels exist (`DockManager::ensure_panel`),
    - ensure graph window roots are set,
    - update `ViewportPanel` targets/sizes for embedded engine viewports (ADR 0007 / ADR 0147).
  - Evidence: `ecosystem/fret-docking/src/imui.rs` (`dock_space_with`, `DockSpaceImUiOptions`).
- **Consume docking effects (runner/driver integration)**:
  - Docking UI emits `Effect::Dock(DockOp)` (ADR 0013). The runner/driver must consume it and apply
    mutations / translate tear-off requests into `WindowRequest::Create`.
  - Recommended driver façade: `fret_docking::DockingRuntime`:
    - `on_dock_op(...)` for `Effect::Dock(op)`,
    - `on_window_created(...)` for `CreateWindowKind::DockFloating` completion,
    - `before_close_window(...)` to merge/clean up when an OS window is closed.
  - Evidence: `ecosystem/fret-docking/src/facade.rs` (`DockingRuntime`),
    `ecosystem/fret-docking/src/runtime.rs` (`handle_dock_op`, tear-off fallback to in-window float),
    `crates/fret-runtime/src/effect.rs` (`Effect::Dock`).
- **Diagnostics + scripted regressions (hand-feel lock-in)**:
  - When triaging “why did it pick this drop target?”, capture a diagnostics bundle and inspect
    `debug.docking_interaction.dock_drop_resolve` (source + resolved target + candidate rects).
  - Multi-window note: scripted tests may need to attach to a specific window; use the schema v2
    top-level `window: "focused"` policy for docking repros.
  - Scripted repro (multi-window overlap hover): `tools/diag-scripts/imui-editor-proof-multiwindow-overlap-topmost-hover.json`.
  - Recommended `fretboard diag` gates for the repro:
    - `--check-dock-drop-resolve-min 1` (prove resolve diagnostics were emitted),
    - `--check-dock-drag-cross-window-max 0` (prove hover stayed in the topmost window).
- **Arbitration seams (docking vs overlays vs viewports)**:
  - Dock drag sessions are window-scoped and must close/suspend non-modal dismissable overlays in
    the same window to avoid fighting outside-press logic (ADR 0072).
  - While a dock drag session is active, docking suppresses forwarding pointer-move/wheel to embedded
    viewports in that window (ADR 0072; viewport forwarding ADR 0147).
  - Evidence: `docs/adr/0072-docking-interaction-arbitration-matrix.md`,
    `docs/workstreams/docking-multiviewport-arbitration-v1.md`.
- **Viewport overlay hooks (editor-owned policy)**:
  - Editor-grade viewport overlays (gizmo/marquee/selection) must remain app-owned and be injected via
    docking hooks instead of being re-implemented in `imui`.
  - Evidence: `ecosystem/fret-docking/src/dock/services.rs` (`DockViewportOverlayHooksService`),
    ADR 0075 (layering split).

### M3 - Ecosystem extension ABI v1 (adapter + metadata evolution)

Goal: make it easy for third-party crates to build immediate wrappers:

- keep the adapter seam thin and auditable,
- expand metadata only when it reduces duplication (focus/geometry/a11y intents),
- keep a stable template and at least one external-style example.

M3 contract notes (normative for v3 work):

- **v2 adapter seam is the v3 baseline**:
  - No ABI changes are required to unlock external adapters: `imui::adapters` is public and the
    seam contract remains limited to identity-in + signal-report-out + optional metadata.
  - External adapter helpers should be written as pure wrappers that:
    - call `ui.push_id(identity_key, |ui| canonical_wrapper(...))`,
    - call `report_adapter_signal(...)` once after render,
    - return the canonical `ResponseExt`.
- **Metadata evolution rule**:
  - Add new fields to `AdapterSignalMetadata` only when it demonstrably reduces policy duplication
    in external crates (e.g. focus restore choreography, geometry for anchoring/measurement).
  - Any metadata expansion must include: at least one external-style example update + evidence
    in this tracker.
- Evidence (external-style example): `ecosystem/fret-ui-kit/tests/imui_external_adapter_example.rs`.
- Evidence (contract types): `ecosystem/fret-ui-kit/src/imui/adapters.rs`.

### M4 - Text/editor bridge (integration, not re-implementation)

Goal: define adapter hooks for editor-grade text surfaces without forking the code editor ecosystem.

### M5 - Perf + regression gate upgrade

Goal: upgrade from "smoke gates" to a small matrix of cheap, high-signal gates:

- allocation regressions in hot wrappers,
- scripted diag paths for floating + popup + docking coexistence,
- at least one steady-state perf script reference for editor-grade demos.

---

## 4) Deliverables

- v3 design note + TODO tracker.
- M1 floating window flag surface + tests/diag evidence.
- Adapter ABI v1 guidance + example.
- Clear deferrals for OS-window promotion and text-editing depth.

---

## 5) Open Questions

1) Which ImGui window flags map cleanly to the current overlay + semantics substrate?
2) How should "no inputs / click-through" semantics be modeled across platforms and backends?
3) What is the smallest perf/per-frame allocation gate that is stable across OS + CI environments?
