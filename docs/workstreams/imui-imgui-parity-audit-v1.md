# imui ↔ Dear ImGui Parity Audit (v1)

Status: Draft (audit note; not an ADR)
Last updated: 2026-02-08

This document records a *behavior-focused* audit of Fret's immediate-mode ecosystem facade
(`ecosystem/fret-ui-kit::imui` + `ecosystem/fret-imui`) against Dear ImGui (C++).

It is not a goal to re-create ImGui 1:1. The goal is to identify **hand-feel gaps** that matter for
editor-grade UI, and to make the remaining divergences **explicit and tracked**.

ImGui reference snapshot (local, not committed):

- `git -C repo-ref/imgui rev-parse --short HEAD` (example audited commit: `913a3c605`)

Related workstreams:

- `docs/workstreams/imui-ecosystem-facade-v3.md` (tracks parity-related work at the ecosystem facade layer)
- `docs/workstreams/docking-multiwindow-imgui-parity.md` (docking + multi-viewport parity; runner/platform owned)

---

## 0) Scope and Reading Guide

This audit focuses on the `imui` facade *as an authoring surface* and the interaction outcomes it
produces. We intentionally do **not** move docking/viewports policy into `imui`.

Implementation anchors (Fret):

- Facade surface + primitives: `ecosystem/fret-ui-kit/src/imui.rs`
- Floating window chrome policy: `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs`
- Regression tests for facade semantics: `ecosystem/fret-imui/src/lib.rs`

Primary upstream anchors (ImGui):

- Window flags: `repo-ref/imgui/imgui.h` (`enum ImGuiWindowFlags_`, `NoMouseInputs`, `NoInputs`, `NoBringToFrontOnFocus`)
- Hovered semantics: `repo-ref/imgui/imgui.h` (`enum ImGuiHoveredFlags_`) + `repo-ref/imgui/imgui.cpp` (`IsItemHovered`)
- Disabled semantics: `repo-ref/imgui/imgui.h` (`BeginDisabled`) + `repo-ref/imgui/imgui.cpp` (`BeginDisabled`, `EndDisabled`)
- ID stack guidance: `repo-ref/imgui/imgui.h` (ID stack notes: `PushID()` + `"##xx"` patterns)
- Popup lifetime: `repo-ref/imgui/imgui.cpp` (`OpenPopupEx`, `BeginPopupContextItem/Window/Void`, `CloseCurrentPopup`)
- Drag threshold: `repo-ref/imgui/imgui.h` (`ImGuiIO::MouseDragThreshold` default `6.0f`)

---

## 1) Parity Matrix (Outcomes, not APIs)

Legend:

- **Aligned**: same outcome in typical editor usage.
- **Partial**: outcome achievable but requires different wiring or has known edge differences.
- **Gap**: no direct way to achieve the outcome today without runtime-level changes.
- **Intentional divergence**: different by design; documented so adapters can compensate.

### 1.1 Floating windows / activation / pass-through

- **Bring-to-front activation gating**: **Aligned**
  - Fret: `FloatingWindowOptions.activate_on_click` gates activation for content, title bar, and resize handles.
  - ImGui anchor: `ImGuiWindowFlags_NoBringToFrontOnFocus`.
- **Inputs disabled vs click-through**: **Aligned**
  - ImGui distinguishes `NoMouseInputs` (mouse pass-through) from `NoInputs` (mouse + nav disabled).
  - Fret exposes:
    - `inputs_enabled=false`: rendered but non-interactive (blocks hits; not click-through by default).
    - `pointer_passthrough=true`: pointer pass-through (hit-test transparent subtree) while still allowing focus traversal.
    - `no_inputs=true`: click-through and skipped by focus traversal.
- **Focus vs z-order split**: **Partial**
  - ImGui separates “taking focus” from “bringing to front” more cleanly than current `imui` wrappers.

### 1.2 Item query semantics (hover/active/focus/click)

- **Basic signals** (`hovered`, `pressed`, `focused`, `clicked`, `changed`): **Partial**
  - Fret: stable minimal contract in `fret-authoring::Response` plus `fret-ui-kit::imui::ResponseExt`.
  - ImGui: `IsItemHovered/Active/Focused/Clicked` plus status flags and per-query flags.
  - Gap: ImGui’s per-query flags (e.g. `AllowWhenBlockedByPopup`, `AllowWhenDisabled`, hover delays) have no direct surface today.
- **Nav-highlight participates in hovered**: **Partial**
  - ImGui: `IsItemHovered()` can return true when keyboard/gamepad nav highlight is on the item (unless overridden).
  - Fret: `Response.hovered` remains pointer-hover driven, but `ResponseExt` now exposes `nav_highlighted` and a
    convenience helper `hovered_like_imgui()` (pointer-hover OR nav-highlight).

### 1.3 Disabled semantics

- **Scoped disabling** (`BeginDisabled`): **Gap**
  - ImGui: `BeginDisabled()` affects interactions and modifies visual alpha.
  - Fret `imui` facade: per-widget `enabled` exists for some widgets (menus/select/input), but there is no
    “disable a subtree” wrapper that matches ImGui’s ergonomics and semantics.
  - Note: `InteractivityGate` can make a subtree inert, but the facade lacks a first-class helper and
    does not define how “disabled” interacts with hover/tooltip signals.

### 1.4 Identity / ID stack ergonomics

- **Stable identity in loops**: **Aligned (explicit)**, **divergent ergonomics**
  - ImGui: `PushID()/PopID()` or `"Label##suffix"` patterns.
  - Fret: explicit keyed scopes (`ui.push_id(...)`, `ui.id(...)`, `for_each_keyed`).
  - Gap (ergonomics): no helper/note translating ImGui `"##"` / `"###"` patterns into Fret’s keyed scopes.

### 1.5 Popups / context menus

- **Context menu basic outcome**: **Aligned**
  - Fret: `ResponseExt.context_menu_requested` + `begin_popup_context_menu(...)`.
  - ImGui: `BeginPopupContextItem/Window/Void` + `OpenPopupEx`.
- **Implicit "last item" identity**: **Intentional divergence**
  - ImGui: `BeginPopupContextItem(NULL)` often relies on “last item ID”.
  - Fret: favors explicit identity (`ResponseExt.id` + `ResponseExt.core.rect`) passed as the trigger.

### 1.6 Drag thresholds and pointer policy knobs

- **Drag threshold default**: **Aligned (policy knob)**
  - ImGui default: `ImGuiIO::MouseDragThreshold = 6.0f`.
  - Fret `imui` facade default is `6.0` and is configurable via theme metric `component.imui.drag_threshold_px`.

### 1.7 Large surfaces not mirrored (explicitly deferred)

The following ImGui surfaces are intentionally **not** mirrored by the current `imui` facade
surface and should be treated as “missing until proven necessary”:

- **Style stack** (`PushStyleVar/PopStyleVar`, `PushStyleColor/PopStyleColor`, `SetNextItemWidth`, etc.)
  - Fret expresses visual policy via theme tokens and declarative refinement rather than an immediate style stack.
- **Tables / columns / tree nodes / tab bars** (submission-driven spatial widgets)
  - Fret has declarative infrastructure for these patterns, but there is no ImGui-style immediate wrapper set yet.
- **Drag-and-drop payload API** (`BeginDragDropSource/Target`, payload types)
  - Fret has runtime-owned drag sessions and docking-owned drag policy; an ImGui-like payload API would need a
    dedicated adapter story and must respect cross-window drag contracts (ADR 0041).

---

## 2) Recommendations (What to Do Next)

P0 (unblock editor hand-feel parity):

1) Done: define the `NoMouseInputs` vs `NoInputs` split explicitly (`pointer_passthrough` vs `no_inputs`).
2) Done: make drag threshold configurable (theme/metric), with ImGui-aligned default `6.0`.

P1 (ecosystem ergonomics):

1) Publish an “ImGui ID patterns ↔ Fret keyed scopes” guide and/or a helper that makes the keyed path frictionless.
2) Add an `imui`-level scoped disabling helper that documents hover/tooltip behavior.
3) Consider a tooltip helper aligned with ImGui’s `ForTooltip` hover-delay defaults.

Tracking source of truth for implementation work:

- `docs/workstreams/imui-ecosystem-facade-v3-todo.md`
