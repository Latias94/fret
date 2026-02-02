# Immediate-Mode Authoring Facade ("imui") v1 — TODO Tracker

Status: Active (workstream tracker)
Last updated: 2026-02-02

This is the checkbox tracker companion to:

- `docs/workstreams/imui-authoring-facade-v1.md`

Legend:

- [ ] pending
- [~] in progress
- [x] done
- [!] blocked / needs decision

---

## Tracking Format

- ID: `IMUI-{area}-{nnn}`
- Status: `[ ]` pending, `[~]` in progress, `[x]` done, `[!]` blocked / needs decision

Areas:

- `scope` (scope, contracts, invariants)
- `api` (public API shape)
- `id` (identity, keying, keyed loops)
- `layout` (layout helpers)
- `widget` (widgets and interaction responses)
- `interop` (escape hatches and bridges)
- `eco` (official ecosystem adoption + conventions)
- `test` (tests and harnesses)
- `docs` (documentation and guides)

Milestone rule of thumb:

- **M0–M2** lock the seams and “make it hard to do the wrong thing”.
- **M3–M5** deliver a minimal, testable authoring surface (Hello World + correctness).
- **M6–M8** validate the ecosystem story and editor-grade proof points (docking + viewport surfaces).

---

## M0 — Scope and Contracts (no surprises later)

Exit criteria:

- A1/A2 split is explicit and referenced by new crate docs.
- “Do not break” invariants are written down (identity, overlays, multi-window).
- The v1 public surface is small and stable enough for third-party experiments.

- [x] IMUI-scope-001 Confirm A1/A2 split:
  - A1: `ecosystem/fret-imui` (policy-free façade + minimal foundation)
  - A2: `fret-ui-kit` / `fret-ui-shadcn` adapters (recipes/policy)
- [x] IMUI-scope-002 Write down the “do not break” invariants:
  - stable identity under dynamic collections,
  - multi-window + multi-root overlays remain first-class,
  - docking and engine viewport surfaces remain supported.
- [x] IMUI-scope-003 Decide the public “escape hatch” shape:
  - `cx_mut()`,
  - `mount(...)` bridge helper.
- [x] IMUI-scope-004 Declare wasm constraints for v1:
  - no TLS-based global collectors,
  - no platform-specific types in public signatures.

---

## M1 — Crate Skeleton (A1)

Exit criteria:

- `ecosystem/fret-imui` exists and compiles in both native and wasm builds.
- Public symbols are namespaced and re-exported via a `prelude` module.

- [x] IMUI-api-010 Add `ecosystem/fret-imui` crate with:
  - `ImUi<'a, H: UiHost>`,
  - `Response`,
  - `imui(cx, |ui| ...) -> Elements`.
- [x] IMUI-api-011 Add sink-based entry point:
  - `imui_build(cx, out: &mut Vec<AnyElement>, |ui| ...)`.
- [x] IMUI-api-012 Ensure `fret-imui` depends only on:
  - `fret-ui`, `fret-core` (and `fret-runtime` only if strictly needed).
- [x] IMUI-api-013 Add a minimal prelude module for imports (`fret_imui::prelude::*`).

---

## M2 — Identity and Loops (push_id parity)

Exit criteria:

- There is exactly one “golden path” to stable identity (`ui.id(...)`).
- Dynamic list rendering has an ergonomic keyed helper (optional, but discoverable).

- [x] IMUI-id-020 Implement `ui.id(key, |ui| ...)` as the canonical keyed scope.
  - Decision: delegate to `ElementContext::keyed(...)` so hashing matches `fret-ui` stable hashing (FNV-1a 64).
- [x] IMUI-id-021 Provide optional helpers that enforce keys:
  - `ui.for_each_keyed(...)` (or similar).
- [x] IMUI-id-022 Add debug guidance for unkeyed loops:
  - expose `ui.for_each_unkeyed(...)` as an explicit opt-in,
  - when a collection is rendered without keys and order changes, surface a warning (align with existing policy).
- [x] IMUI-id-023 Document key stability rules and recommended key types (persisted editor layout guidance).
- [x] IMUI-id-024 Define canonical persisted panel identity conventions (`PanelKind` / `PanelKey`, namespacing, migration expectations).

---

## M3 — Minimal Layout + Widgets (Hello World)

Exit criteria:

- A tiny “Hello World” UI can be written using only `fret-imui` + `fret-ui`.
- All APIs work in both native and wasm demo shells (no runner-specific surface).

- [x] IMUI-layout-030 Layout:
  - `ui.row(|ui| ...)` (and `row_build` if needed)
  - `ui.column(|ui| ...)` (and `column_build` if needed)
  - `ui.separator()`
- [x] IMUI-widget-031 Text:
  - `ui.text("...")` (simple text leaf)
- [x] IMUI-widget-032 Button:
  - `ui.button("...") -> Response`
- [x] IMUI-widget-033 Checkbox:
  - `ui.checkbox_model("...", &Model<bool>) -> Response` (or `changed()` semantics)
- [x] IMUI-docs-034 Add a minimal “Hello World” snippet to `docs/workstreams/imui-authoring-facade-v1.md`.

---

## M4 — Response Semantics (clicked/changed correctness)

Exit criteria:

- `clicked/changed` are deterministic, single-fire per interaction, and do not double-trigger under wrappers.
- `Response` is sufficient for third-party widgets without reaching into `ElementContext` for common cases.

- [x] IMUI-api-040 Define `Response` v1 fields and semantics:
  - `clicked`, `changed`, `hovered`, `pressed`, `focused`
  - optional: `rect` for popover placement
- [x] IMUI-widget-041 Implement edge-trigger storage as:
  - element-local “pending flag” cleared on read (recommended), with optional “frame id” guard to avoid repeats.
- [x] IMUI-test-042 Add at least 3 smoke tests (UI tree harness) for:
  - click → clicked true once,
  - holding press does not repeat clicked,
  - checkbox toggling sets changed exactly once.
- [ ] IMUI-test-043 Add a wasm-targeted smoke harness entry if the existing test harness cannot execute wasm (at minimum:
  compile-only + `cargo test` coverage in non-wasm where possible).

---

## M5 — Interop Bridges (don’t strand existing ecosystem)

Exit criteria:

- Existing declarative builders can be embedded without rewriting.
- Advanced widgets can still be authored via the substrate without breaking the imui composition story.

- [x] IMUI-interop-050 `ui.mount(...)` helper to embed existing declarative builders.
- [x] IMUI-interop-051 `ui.cx_mut()` helper for direct access to `ElementContext` and advanced widgets (canvas/viewport surfaces).
- [ ] IMUI-interop-052 Document “when to drop to `cx`” (canvas, viewport surfaces, docking host integration).
- [x] IMUI-interop-053 Add a feature-gated retained subtree host element (`fret-ui/unstable-retained-bridge`) so
  retained widget subtrees (e.g. `fret-node`) can be embedded in imui without a rewrite.
- [x] IMUI-docs-054 Document retained subtree embedding (node graph adapter snippet).
- [x] IMUI-interop-055 Lock retained-subtree stability policy (remain feature-gated until M7 proof points).

---

## M6 — Ecosystem Adoption Plan (official crates)

Goal: prove that imui is a viable composition surface by adapting 2–3 ecosystem crates.

Exit criteria:

- At least 2 ecosystem crates ship an `imui` feature gate and can be used from an imui app without pulling in heavy
  default features.

- [x] IMUI-eco-060 Define a standard feature gate naming convention for ecosystem crates:
  - `headless`, `ui`, `imui` (imui implies ui).
- [~] IMUI-eco-061 Add `imui` adapters for official crates:
  - [x] `fret-markdown` (render markdown inside an imui container)
  - [x] `fret-code-view` (render code blocks inside an imui container)
  - [x] `fret-docking` (embed a docking host inside imui)
  - [ ] `fret-plot` or `fret-chart` (render a plot widget inside imui)
  - [x] `fret-node` (render a node graph editor surface inside imui)
- [x] IMUI-eco-063 Add an imui demo that embeds `fret-node` via `RetainedSubtree` (proof that retained interop works).
- [x] IMUI-docs-062 Add a short third-party guide:
  - “How to write an imui widget crate”
  - “How to add `imui` feature gates without forcing heavy deps”
  - (Initial checklist lives in `docs/workstreams/imui-authoring-facade-v1.md`.)

---

## M7 — Multi-window + Docking + Viewport Surfaces (editor-grade proof points)

Exit criteria:

- A single demo proves imui does not block editor-grade requirements:
  - multiple windows,
  - docking host,
  - embedded viewport surfaces with correct focus/input boundaries.

- [x] IMUI-test-070 Add an example that demonstrates (native):
  - two windows (`AppWindowId`) each hosting imui,
  - a docking host (ecosystem) within imui,
  - an engine viewport surface (`ViewportSurface`) embedded in a docked panel.
  - evidence:
    - `apps/fret-examples/src/imui_editor_proof_demo.rs`
    - `apps/fret-demo/src/bin/imui_editor_proof_demo.rs`
    - run: `cargo run -p fret-demo --bin imui_editor_proof_demo`
- [ ] IMUI-id-071 Validate identity stability for docked panels:
  - panels keyed by stable `PanelKey` (or string) using `ui.id(...)`.
- [x] IMUI-test-072 Add a regression harness note for wasm/mobile fallback behavior:
  - multi-window layouts degrade into in-window floatings (no OS window requirement).
  - verify:
    - set `FRET_IMUI_EDITOR_PROOF_SINGLE_WINDOW=1` and run `cargo run -p fret-demo --bin imui_editor_proof_demo`
    - attempt a dock panel tear-off; it should **not** emit OS window creation and should remain as an in-window floating
    - rationale: `fret-docking::runtime::handle_dock_op` degrades when `PlatformCapabilities.ui.multi_window` /
      `window_tear_off` are false (or hover detection is unavailable)

---

## M8 — Documentation (golden path)

Exit criteria:

- There is a single “golden path” doc snippet that can be copied into demos.
- The extension and layering rules are explicit, so third-party ecosystems do not fork the surface.

- [ ] IMUI-docs-080 Add a concise “Golden Path” section with:
  - how to call `render_root` + `imui/imui_build`,
  - how to use `ui.id(...)` correctly,
  - how to embed existing ecosystem widgets via `ui.mount(...)`.
- [ ] IMUI-docs-081 Add a short “FAQ / gotchas”:
  - “Why keys matter”
  - “How clicked/changed works”
  - “When to drop to `cx_mut()`”
