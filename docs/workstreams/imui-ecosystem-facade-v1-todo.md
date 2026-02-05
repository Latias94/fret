# imui Ecosystem Facade (egui/imgui-like ergonomics) v1 - TODO Tracker

Status: Draft
Last updated: 2026-02-05

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

- [ ] IMUIECO-scope-010 Decide whether shadcn integration is via an optional feature or a dedicated adapter module/crate.
- [ ] IMUIECO-scope-011 Choose a canonical delegation seam for returning `Response` from canonical components.
- [ ] IMUIECO-scope-012 Decide whether “tear-off to OS window” is docking-only for v1 (recommended) or generalized.
- [ ] IMUIECO-scope-013 Define the `ResponseExt` signal storage model (transient vs element-local state) and document it.
- [ ] IMUIECO-scope-014 Define a tiered delegation rule for official widgets (primitive wrappers vs canonical component adapters).

---

## M1 - Response parity expansion (egui/imgui-style signals)

Exit criteria:

- The facade exposes a richer `Response` surface that covers the most common immediate-mode queries.

- [x] IMUIECO-resp-010 Add click variants (secondary + double click).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt::{secondary_clicked,double_clicked}`, `UiWriterImUiFacadeExt::button`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`right_click_sets_context_menu_requested_true_once`, `double_click_sets_double_clicked_true_once`).
- [ ] IMUIECO-resp-010b Add long-press / press-and-hold signal (touch-first; schedule-aware).
- [x] IMUIECO-resp-011 Add drag lifecycle signals (started/dragging/stopped) and useful geometry/delta fields.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt::drag`, `UiWriterImUiFacadeExt::button`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`drag_started_stopped_and_delta_are_consistent`).
- [x] IMUIECO-resp-012 Add context-menu request signal (right click / keyboard menu key).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt::context_menu_requested`, key hook + right-click hook).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`right_click_sets_context_menu_requested_true_once`, `shift_f10_sets_context_menu_requested_true_once`).
- [x] IMUIECO-resp-013 Document “two-frame stabilization” where geometry is sourced from last-frame bounds.
  - Evidence: `docs/workstreams/imui-ecosystem-facade-v1.md` (section 5.3).

---

## M2 - Controls + containers (immediate-mode standard library)

Exit criteria:

- Common controls are authorable in a single immediate-mode style, backed by canonical components.

- [~] IMUIECO-controls-020 Button/checkbox/toggle wrappers that return `Response` without duplicating policy.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::{button,checkbox_model}`)
- [ ] IMUIECO-controls-021 Input/textarea wrappers (coordinate with the code editor ecosystem; don’t duplicate).
- [ ] IMUIECO-controls-022 Slider/select/switch wrappers (shadcn-aligned when enabled).
- [ ] IMUIECO-api-023 Container helpers (`horizontal`, `vertical`, `grid`, `scroll`) that prefer `UiBuilder` patch vocabulary.
- [ ] IMUIECO-api-024 `push_id` / scoped identity helpers mirroring egui/imgui patterns.

---

## M3 - Overlays + floating windows/areas (ImGui-aligned)

Exit criteria:

- Overlays have convenient immediate-mode entry points with correct dismissal/focus policy.
- Floating windows/areas exist as first-class outcomes in-window, aligned with ImGui-style UX.

- [ ] IMUIECO-overlays-030 Menu/popover/tooltip convenience wrappers built on `OverlayController`.
- [ ] IMUIECO-float-031 Implement a floating **area** primitive in `fret-ui-kit` (policy-heavy):
  - move (drag) + z-order + focus activation,
  - predictable hit-testing with overlays,
  - degrade cleanly when multi-window is unavailable (always in-window).
- [x] IMUIECO-float-032a Add a minimal floating window skeleton (in-window) with a draggable title bar.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::floating_window`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_window_moves_when_dragging_title_bar`).
- [x] IMUIECO-float-032b Add ImGui-style `open` model + close button for floating windows.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::floating_window_open`).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`floating_window_close_button_sets_open_false`).
- [ ] IMUIECO-float-032 Layer a floating **window chrome** policy on top of the area:
  - title bar, close button, Esc-to-close,
  - resize handles + resize session state,
  - focus trap/restore when appropriate (aligned with overlay policy).
  - Note: `Esc-to-close` is implemented for `floating_window_open` when the title bar is focused.
- [~] IMUIECO-float-033 Add `fret-ui-kit` immediate wrappers (`ui.area(...)`, `ui.window(...)`) returning meaningful interaction results.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt::floating_window`).
- [ ] IMUIECO-float-034 Decide OS-window promotion scope:
  - docking-only for v1 (recommended), or
  - generalized capability-gated promotion later.

---

## M4 - Demos, tests, and perf guidance

Exit criteria:

- Demos exist and are stable proof points.
- Basic tests exist to prevent regressions in signals and floating behavior.
- Perf guidance is written down (allocation patterns, caching boundaries, virtualization).

- [ ] IMUIECO-demo-040 Add a minimal demo showing `Response` parity signals (click/drag/context menu).
- [ ] IMUIECO-demo-041 Add a floating-window demo (in-window float + overlay interactions).
- [ ] IMUIECO-test-042 Add nextest coverage for facade crates (smoke + key behavior tests):
  - click variants are delivered once (clear-on-read),
  - drag lifecycle is consistent across frames,
  - context-menu request is stable (mouse + keyboard if supported).
- [ ] IMUIECO-test-043 Add a wasm compile smoke harness for the facade surface.
- [ ] IMUIECO-perf-044 Add a short perf guide (avoid allocations, prefer keyed identity, use virtualization/caching).
- [ ] IMUIECO-docs-045 Document extension guidelines for third-party widget crates (author once, adapter modules).
- [ ] IMUIECO-test-046 Add one `fretboard diag` script covering floating window drag/resize + overlay coexistence (regression gate).
