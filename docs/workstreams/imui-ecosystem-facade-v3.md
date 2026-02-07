# imui Ecosystem Facade v3 (ImGui Parity + Ecosystem ABI + Perf Ceilings)

Status: Draft (workstream note; not an ADR)
Last updated: 2026-02-06

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
- `docs/workstreams/docking-multiwindow-imgui-parity.md` (OS-window tear-off parity)
- `docs/workstreams/code-editor-ecosystem-v1.md` (text/editor ecosystem)

---

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

### M3 - Ecosystem extension ABI v1 (adapter + metadata evolution)

Goal: make it easy for third-party crates to build immediate wrappers:

- keep the adapter seam thin and auditable,
- expand metadata only when it reduces duplication (focus/geometry/a11y intents),
- keep a stable template and at least one external-style example.

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
