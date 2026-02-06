# imui Ecosystem Facade (egui/imgui-like ergonomics) v1

Status: Draft (workstream note; not an ADR)
Last updated: 2026-02-06

This document proposes an ecosystem-level “batteries included” facade built on top of Fret’s
immediate-mode authoring surface (`imui`) that targets **egui/Dear ImGui-style ergonomics**
without turning `imui` into a second runtime.

The central decision: keep `ecosystem/fret-imui` **policy-light and minimal**, and move “feels like
egui/imgui” convenience (richer `Response` signals, floating windows/areas, menus, adapters for
common controls) into **ecosystem facade crates**.

Status snapshot (2026-02-06):

- The minimal shared `Response` contract lives in `ecosystem/fret-authoring`.
- `ecosystem/fret-imui` is intentionally policy-light (authoring frontend entry points + identity helpers).
- The initial egui/imgui-like convenience surface is hosted in `ecosystem/fret-ui-kit` behind its `imui` feature.
- `ResponseExt` now covers common v1 signals (secondary + double click, drag lifecycle + deltas, context-menu request + anchor).
- A minimal menu-like popup primitive exists (`open_popup_at` + `begin_popup_menu` / `begin_popup_context_menu` + `menu_item`), built on `OverlayController`.
  - Menu popups now use Radix-aligned initial focus policy (pointer-open focuses the menu container; keyboard-open focuses the first focusable item when available).
  - `Escape` closes menu popups via `DismissableLayer`, and focus restore routes back to the trigger when appropriate.
  - Menu popups now support minimal roving keyboard navigation (ArrowUp/ArrowDown + Home/End).
  - Menu items support checkbox/radio semantics (`menu_item_checkbox_ex`, `menu_item_radio_ex`).
- A minimal modal popup primitive exists (`open_popup` + `begin_popup_modal`), built on `OverlayRequest::modal`.
  - Default policy: `Escape` closes; outside presses are ignored (unless explicitly enabled via options).
- A minimal in-window floating area primitive exists (`floating_area` + `floating_area_drag_surface_ex`):
  - drag move + element-local position state,
  - opt-in `floating_layer(...)` for bring-to-front z-order management.
- A minimal in-window floating window primitive exists (`floating_window` / `floating_window_open`):
  - draggable title bar + element-local position state (window chrome layered on top of the same floating area state),
  - optional ImGui-style `open` model + close button,
  - `Esc`-to-close when the title bar is focused,
  - opt-in `floating_layer(...)` for bring-to-front z-order management.
  - optional v1 resize handles via `floating_window_resizable(...)` (edges + corners; diagonal cursor supported).
- `ui.area(...)` / `ui.window(...)` wrappers return meaningful reports (`FloatingAreaResponse`, `FloatingWindowResponse`) for persistence/debugging.
- A minimal diagnostics demo + scripted repro exists for floating window drag/resize + context-menu overlay coexistence.
  - Demo: `cargo run -p fret-demo --bin imui_floating_windows_demo`
  - Script: `tools/diag-scripts/imui-float-window-drag-resize-context-menu.json`
- A minimal response-signals demo exists for click variants + drag lifecycle + context-menu requests.
  - Demo: `cargo run -p fret-demo --bin imui_response_signals_demo`

Tracking:

- TODO tracker: `docs/workstreams/imui-ecosystem-facade-v1-todo.md`
- imui authoring facade v2 (implemented): `docs/workstreams/imui-authoring-facade-v2.md`
- Unified authoring builder surface (ADR): `docs/adr/0175-unified-authoring-builder-surface-v1.md`
- Docking + multi-window parity (ImGui-aligned): `docs/workstreams/docking-multiwindow-imgui-parity.md`
- macOS multi-window parity anchors: `docs/workstreams/macos-docking-multiwindow-imgui-parity.md`
- Overlays policy split (Radix-aligned): `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Upstream references (local snapshots):
  - egui: `repo-ref/egui/`
  - Dear ImGui (C++): `repo-ref/imgui/`
  - Dear ImGui Rust binding workspace: `repo-ref/dear-imgui-rs/`

---

## 1) Motivation

Fret’s `imui` v2 workstream established a clear stance:

- `imui` is an **authoring frontend** (control flow + `Response`-like signals),
- it compiles down to Fret’s **declarative element tree** (`AnyElement`),
- and it must not become a parallel runtime or a competing patch vocabulary.

What’s still missing is the “standard library” layer that makes egui/imgui ecosystems feel
productive:

- a richer `Response` surface (drag/click variants/context menu/etc.),
- floating windows/areas as first-class authoring outcomes,
- overlay/menu/tooltip convenience APIs,
- “adapter-first” patterns that encourage third-party widget crates to integrate cleanly.

Today, most of the substrate exists (pressables, overlays, placement solver, canvas, scroll,
virtualization, docking, multi-window runner support), but the ergonomics are spread across layers.
The result is that immediate-style code often drops to `cx_mut()` quickly, and ecosystem crates have
no unified “immediate-mode convenience” home.

This workstream is about **shaping that convenience layer** while preserving Fret’s layering and
avoiding duplicated state machines.

---

## 2) Invariants (Do Not Break)

These are hard constraints aligned with the existing `imui` v2 plan and core ADR boundaries:

1) **No second runtime**
   - The facade must still compile to the declarative element taxonomy mounted into `UiTree`
     (see ADR 0028/0039 and `docs/workstreams/imui-authoring-facade-v2.md`).

2) **Stable identity remains canonical**
   - Dynamic collections must use keyed identity (`ui.id(...)` / `UiWriter::keyed(...)`).
   - No new hashing scheme or alternative ID stack.

3) **Canonical widget rule (official crates)**
   - One source-of-truth implementation per widget.
   - Immediate-mode entry points are thin adapters that delegate, not parallel implementations.

4) **Single patch vocabulary**
   - `ui()` / `UiBuilder<T>` (ADR 0175) remains the canonical patch chain for chrome/layout.
   - The facade should not introduce a separate “tailwind-ish” patch language.

5) **Policy stays in ecosystem crates**
   - Mechanisms live in `crates/fret-ui`.
   - Policies/recipes (dismiss, focus restore, hover intent, default sizes) live in `ecosystem/`.

6) **Multi-window and degradation**
   - Floating behavior must degrade predictably when multi-window is unavailable
     (e.g. wasm/mobile → in-window floating).
   - Docking parity and multi-viewport semantics remain aligned with the existing workstreams.

---

## 3) Design Goals

### 3.1 Ergonomics goals (egui/imgui “feel”)

- Provide a `Ui`-like surface that supports:
  - `Response`-driven widgets (click variants, drag state, rects),
  - common containers (horizontal/vertical, grids, scroll),
  - overlays (menus/popovers/tooltips) with correct dismissal/focus policy,
  - floating windows/areas.

### 3.2 Ecosystem + extension goals

- Third-party widget crates should be able to:
  - depend on `fret-authoring::UiWriter` (minimal contract),
  - optionally provide an `imui` feature with adapters that return `Response`-like signals,
  - reuse the unified patch vocabulary (`UiBuilder`) for chrome/layout.

### 3.3 Performance goals

- Avoid introducing per-widget allocations that would make “every-frame UI” expensive.
- Prefer clear-on-read transient signals (frame-local events) over persistent per-widget state.
- Keep geometry queries explicit: placement based on last-frame bounds is acceptable, but should be
  documented as a two-frame stabilization in some cases.

---

## 4) Non-Goals

- Building a monolithic component library in `crates/fret-ui`.
- Re-creating the full egui/imgui API surface verbatim.
- Re-implementing docking in a new facade (reuse `ecosystem/fret-docking`).
- Owning the full text editor ecosystem inside this workstream (there is already an in-progress
  `code-editor-ecosystem-v1` worktree; we should integrate, not duplicate).

---

## 5) Proposed Shape (Ecosystem Facade Crates)

### 5.1 Crate taxonomy (recommended)

Because we are pre-open-source, we can choose the “fearless refactor” path that minimizes long-term
surface duplication:

- move the shared `Response` contract into `ecosystem/fret-authoring`,
- keep `ecosystem/fret-imui` policy-light (builder + identity + output sink),
- host the egui/imgui-like convenience layer in `ecosystem/fret-ui-kit` behind its existing `imui`
  feature (and split into a dedicated crate later only if it proves necessary).

Recommended taxonomy (v1):

- `ecosystem/fret-authoring`
  - owns: `UiWriter` and a minimal, ecosystem-friendly `Response` contract.
- `ecosystem/fret-imui`
  - owns: `ImUi`, `imui/imui_build/imui_vstack`, identity helpers (`id`, keyed/unkeyed loops).
  - does **not** own: styling presets or policy-heavy widget semantics.
- `ecosystem/fret-ui-kit` (feature: `imui`)
  - owns: the immediate-mode “standard library” facade (containers, overlay conveniences, floating
    windows/areas, and `Response`-returning control wrappers when practical).
- `ecosystem/fret-ui-shadcn`
  - owns: shadcn-aligned visuals and canonical components.
  - optional: expose immediate-mode adapter modules that integrate with the facade without
    duplicating state machines.

Rationale:

- This matches the current dependency direction: `fret-ui-kit` already has an `imui` feature that
  depends on `fret-authoring`, while `fret-imui` intentionally does not depend on `fret-ui-kit`.
- It avoids introducing a new crate prematurely. If/when the facade surface grows large, we can
  extract it into `ecosystem/fret-imui-kit` without changing the underlying ownership rules.

### 5.2 Public surface sketch

The facade should be usable from any authoring frontend that implements `UiWriter`:

- Wrapper type:
  - `struct Ui<'a, H, W: UiWriter<H>> { ... }`
- Or extension traits:
  - `trait UiWriterImUiKitExt<H: UiHost>: UiWriter<H> { ... }`

The wrapper approach reduces “extension trait sprawl” while keeping call sites egui-like:

```rust
fret_imui::imui_vstack(cx, |ui| {
    use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;

    ui.text("Inspector");
    if ui.button("Reset").clicked() {
        // ...
    }
});
```

### 5.3 `Response` parity plan

egui’s `Response` is rich (click variants, drag lifecycle, etc.), and Dear ImGui’s item API exposes
similar concepts. Fret’s current `fret-imui::Response` is intentionally minimal.

To get egui/imgui-like ergonomics, we should expand `Response` **in the facade layer first** (and
only stabilize/move contracts later), focusing on:

- click variants: primary/secondary, double click, long press (touch),
- drag lifecycle: started/dragging/stopped + drag delta,
- context-menu request signal,
- wheel/scroll hit signal (where relevant),
- geometry: `rect` (already present) and possibly an `interact_rect` concept.

Pre-open-source recommendation:

- Move the minimal `Response` contract to `ecosystem/fret-authoring` early, before third-party
  crates depend on it.
- Keep richer / more opinionated signals in the facade layer (ui-kit) until they settle.

Signal storage model (recommended):

- Use **transient events (clear-on-read)** for one-shot signals (clicked variants, context-menu request).
- Use **element-local state** (`ElementContext::with_state_for`) for continuous interactions (drag in-progress,
  accumulated delta, resize session bookkeeping).
- Source geometry from `last_bounds_for_element` (two-frame stabilization): treat the first frame as “no rect”
  and document this explicitly where it matters (overlays/floating).

### 5.4 Canonical-widget delegation (no duplicated state machines)

The hardest design seam is: “how do we return `Response` while delegating to the canonical
implementation?”

We want to avoid writing two button state machines:

- one inside `fret-ui-shadcn::Button` (declarative),
- and another inside the immediate-mode facade (e.g. `fret-ui-kit` `imui` helpers).

Two viable strategies:

1) **Expose an adapter seam from canonical components** (preferred long-term)
   - Provide a `with_id` / “signal sink” hook in canonical components (or shared helpers) so the
     facade can capture the element id and read transient signals without duplicating logic.
   - This keeps the canonical component as the only policy owner.

2) **Facade-built widgets using mechanism primitives + shared styling helpers** (fallback)
   - Build immediate widgets directly using `Pressable`/`TextInput` primitives, but reuse the same
     style/token resolver helpers as shadcn (to avoid visual drift).
   - Accept the maintenance cost explicitly and keep the set small (only primitives, not complex
     composites).

This workstream should choose (1) where possible, and reserve (2) for places where (1) would create
an unreasonable amount of “callback plumbing”.

### 5.5 Floating windows/areas (ImGui-aligned)

We want a first-class floating surface similar to:

- egui: `Area` / `Window` (in-window floating containers; not OS windows),
- Dear ImGui docking branch: floating windows that can become platform viewports when multi-viewport
  is enabled.

In Fret terms, the plan is:

- Implement policy-heavy floating window behavior in `ecosystem/fret-ui-kit` (not in `fret-imui`):
  - draggable title bar,
  - resize handles,
  - close button + Escape-to-close,
  - focus/restore policy,
  - placement constraints and z-order arbitration with other overlays.
- Expose immediate-mode wrappers in `fret-ui-kit` (feature: `imui`):
  - `ui.window("Title", |ui| { ... })`
  - `ui.area("id", |ui| { ... })`

Multi-window / “tear-off” alignment:

- Keep docking’s existing “tear-off to OS window” path as the baseline for editor-grade workflows.
- For non-docking floating windows, consider an optional “promote to OS window” path gated by
  `PlatformCapabilities` (and degrade to in-window floating when unavailable), but treat that as a
  later milestone (it touches runner/window lifecycle seams).

Upstream reference anchors (Dear ImGui docking branch):

- Viewports and platform window lifecycle:
  - `repo-ref/imgui/imgui.h` (Viewports section; `ImGuiViewportFlags_*`, `RenderPlatformWindowsDefault`, platform callbacks)
  - `repo-ref/imgui/imgui.cpp` (`UpdateViewportsNewFrame`, `UpdateViewportsEndFrame`, `RenderPlatformWindowsDefault`)
- Window move/resize interaction:
  - `repo-ref/imgui/imgui.cpp` (`StartMouseMovingWindow`, `UpdateMouseMovingWindowNewFrame`, `UpdateMouseMovingWindowEndFrame`)
  - `repo-ref/imgui/imgui_internal.h` (declarations)
- Hovered-viewport detection (backend contract):
  - `repo-ref/imgui/docs/BACKENDS.md` (HasMouseHoveredViewport, `io.AddMouseViewportEvent`, and handling `ImGuiViewportFlags_NoInputs`)

---

## 6) Text Editing Integration (Explicit Dependency)

There is an in-progress text editor ecosystem worktree (`code-editor-ecosystem-v1`).

This workstream must not duplicate that effort. Instead:

- The facade layer (ui-kit `imui` helpers) should provide adapter slots for “text surfaces” once
  the code editor ecosystem is merged (or a stable surface is identified).
- Immediate-mode wrappers should target editor-grade outcomes (selection, IME, commands routing)
  and remain thin.

---

## 7) Acceptance Criteria

Minimum “v1 done” outcomes:

- A documented immediate-mode facade surface exists in `ecosystem/` with clear layering rules:
  - v1 target: `ecosystem/fret-ui-kit` (feature: `imui`),
  - optional later extraction: `ecosystem/fret-imui-kit` if the surface grows large.
- A richer `Response` surface is available to immediate-mode callers (at least click + drag).
- A floating window/area primitive exists (in-window floating), aligned with ImGui-style behavior
  and Fret’s overlay contracts.
- Demos exist that exercise:
  - `Response` signals,
  - floating windows,
  - coexistence with docking/multi-window (no regressions).
- `cargo nextest run` passes for affected crates.

---

## 8) Risks and Mitigations

1) **Fragmentation of authoring surfaces**
   - Mitigation: keep `UiWriter` as the shared minimal contract; keep `UiBuilder` as the shared patch vocabulary.

2) **Duplicated widget implementations**
   - Mitigation: prioritize “adapter seams” inside canonical components; keep fallback wrappers minimal.

3) **Performance regressions**
   - Mitigation: avoid per-frame allocations in wrappers, use transient signals, and adopt
     virtualization/caching guidance in docs and demos.

4) **API churn in `Response`**
   - Mitigation: keep `Response` expansion in the facade crate until it stabilizes; only then
     consider moving/shared-contract decisions.

---

## 9) Open Questions

1) What is the best “canonical delegation seam” for returning `Response` without duplication?
   - `with_id` callbacks,
   - explicit “signal sink” plumbing,
   - or a small public mechanism API for transient event recording/querying.

2) How far should non-docking floating windows go toward OS-window tear-off (ImGui multi-viewport)?
   - Keep docking-only for v1,
   - or add a capability-gated path in ui-kit + runner for general floating surfaces.

3) How should `ResponseExt` evolve before we consider moving any portion into the shared contract?
   - Keep all “rich” signals in the facade indefinitely, or
   - graduate a minimal `ResponseCore` + stable “interaction session” types once patterns settle.
