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

- [ ] IMUIECO-scope-000 Move minimal `Response` to `ecosystem/fret-authoring` (breaking change OK).
- [ ] IMUIECO-scope-001 Slim `ecosystem/fret-imui` dependencies (move policy/widget helpers to ui-kit where appropriate).
- [ ] IMUIECO-scope-002 Decide whether to extract the facade into a dedicated crate later (default: keep in `fret-ui-kit` for v1).

---

## M0 - Lock remaining seams (decisions first)

Exit criteria:

- Delegation strategy for `Response` is chosen (no duplicated widget policy).
- Scope is documented: what lives in `fret-imui` vs `fret-ui-kit` (imui facade) vs `fret-ui-shadcn` (visuals).

- [ ] IMUIECO-scope-010 Decide whether shadcn integration is via an optional feature or a dedicated adapter module/crate.
- [ ] IMUIECO-scope-011 Choose a canonical delegation seam for returning `Response` from canonical components.
- [ ] IMUIECO-scope-012 Decide whether “tear-off to OS window” is docking-only for v1 (recommended) or generalized.

---

## M1 - Response parity expansion (egui/imgui-style signals)

Exit criteria:

- The facade exposes a richer `Response` surface that covers the most common immediate-mode queries.

- [~] IMUIECO-resp-010 Add click variants (secondary, double click, long press where applicable).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`ResponseExt`, `UiWriterImUiFacadeExt::button`)
- [ ] IMUIECO-resp-011 Add drag lifecycle signals (started/dragging/stopped) and useful geometry/delta fields.
- [ ] IMUIECO-resp-012 Add context-menu request signal (right click / keyboard menu key).
- [ ] IMUIECO-resp-013 Document “two-frame stabilization” where geometry is sourced from last-frame bounds.

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
- [ ] IMUIECO-float-031 Implement a floating window/area primitive in `fret-ui-kit` (policy-heavy).
- [ ] IMUIECO-float-032 Add `fret-ui-kit` immediate wrappers (`ui.window(...)`, `ui.area(...)`) returning a meaningful `Response`.
- [ ] IMUIECO-float-033 (If generalized) Add a capability-gated “promote to OS window” path; otherwise keep docking-only.

---

## M4 - Demos, tests, and perf guidance

Exit criteria:

- Demos exist and are stable proof points.
- Basic tests exist to prevent regressions in signals and floating behavior.
- Perf guidance is written down (allocation patterns, caching boundaries, virtualization).

- [ ] IMUIECO-demo-040 Add a minimal demo showing `Response` parity signals (click/drag/context menu).
- [ ] IMUIECO-demo-041 Add a floating-window demo (in-window float + overlay interactions).
- [ ] IMUIECO-test-042 Add nextest coverage for facade crates (smoke + key behavior tests).
- [ ] IMUIECO-test-043 Add a wasm compile smoke harness for the facade surface.
- [ ] IMUIECO-perf-044 Add a short perf guide (avoid allocations, prefer keyed identity, use virtualization/caching).
- [ ] IMUIECO-docs-045 Document extension guidelines for third-party widget crates (author once, adapter modules).
