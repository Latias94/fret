# Foundation Closure (P0) — Milestones & Exit Criteria

Status: Draft (workstream tracker; ADRs remain the source of truth)

This document is a **cross-workstream milestone tracker** for “foundation-first” closure:

- lock hard-to-change contracts early (ADRs),
- keep `crates/fret-ui` mechanism-only (ADR 0066 / ADR 0074),
- and ensure we have **executable regressions** before scaling component surface area.

This is intentionally not a replacement for:

- Roadmap: `docs/roadmap.md`
- Module closure index: `docs/golden-architecture.md`
- UI closure map: `docs/ui-closure-map.md`
- GPUI parity workstream: `docs/workstreams/gpui-parity-refactor.md`

Tracking:

- TODO tracker (keep updated): `docs/workstreams/foundation-closure-p0-todo.md`

## Why this tracker exists

Many critical gaps are **cross-cutting** (UI runtime ↔ runner ↔ ecosystem policy ↔ diagnostics).
Individual workstreams already track details, but we still need a single “P0 closure checklist”
to prevent:

- accidental policy leaks into `fret-ui`,
- late-stage performance rewrites (layout, view-cache, virtualization),
- platform-specific forks due to missing capability modeling,
- untestable interaction drift (menus/overlays/docking/viewport tools).

## P0 closure definition (what “done enough to scale” means)

A P0 module is “closed enough to scale” when:

1) **Contract is locked** (Accepted ADR, or an explicitly documented decision gate).
2) **Mechanism/policy boundary is enforced** (no shadcn/Radix policy in `crates/fret-ui`).
3) **At least one executable regression exists** (unit/integration test or scripted diagnostics).
4) **Observability exists** (bundle fields / stats / logs sufficient to debug drift).
5) **Portability is modeled** (capability quality signals instead of ad-hoc platform branches).

## Milestones (P0)

Each milestone below should be treated as “ship before scaling component surface area” for editor-grade UX and
general-purpose applications.

### M0 — Layout Engine v2 closure (multi-viewport roots)

Goal: a window-scoped layout engine with deterministic viewport roots and explicit barrier rules.

Primary references:

- Roadmap: `docs/layout-engine-refactor-roadmap.md`
- Inventory: `docs/layout-engine-v2-migration-inventory.md`
- ADR gate: `docs/adr/0116-window-scoped-layout-engine-and-viewport-roots.md`

Exit criteria (summary):

- `AvailableSpace` preserved end-to-end (no “huge definite probes”).
- Non-reentrant intrinsic measurement enforced (leaf-only `measure_in`).
- Viewport roots are first-class and “flush only invalidated roots” is test-locked.
- Docking/scroll/virtualization/viewport-surface remain explicit barriers with clean interop.

Validation anchors:

- `cargo nextest run -p fret-ui`
- `cargo run -p fret-demo --bin docking_arbitration_demo` (manual checklist: `docs/docking-arbitration-checklist.md`)

### M1 — Overlay + input arbitration closure (cross-system determinism)

Goal: overlay lifecycle and pointer occlusion semantics are stable and regression-tested, including under view-cache reuse.

Primary references:

- Roadmap: `docs/overlay-and-input-arbitration-v2-refactor-roadmap.md`
- Workstream: `docs/workstreams/overlay-input-arbitration-v2.md`
- TODO tracker: `docs/workstreams/overlay-input-arbitration-v2-todo.md`
- Contract gates: `docs/adr/0067-*`, `docs/adr/0069-*`, `docs/adr/0072-*`, `docs/adr/1157-*`

Exit criteria (summary):

- “present vs interactive” invariants are enforced consistently (painted-but-closing is click-through and observer-inert).
- `PointerOcclusion` is a mechanism (not policy glue) and supports “block mouse except scroll” outcome.
- Conformance suite covers: modal barrier scoping, outside press observer behavior, capture cancellation, menu-like occlusion.

Validation anchors:

- `cargo nextest run -p fret-ui`
- `cargo nextest run -p fret-ui-shadcn` (Radix/shadcn state gates)
- `cargo run -p fretboard -- diag suite ui-gallery` (cached+uncached matrix recommended)

### M2 — View-cache + notify + prepaint closure (fearless refactor safety net)

Goal: a GPUI-aligned “dirty views + reuse ranges” story that remains correct under refactors, and an incremental path to
prepaint-derived windows for large surfaces.

Primary references:

- Workstream: `docs/workstreams/gpui-parity-refactor.md`
- TODO tracker: `docs/workstreams/gpui-parity-refactor-todo.md`
- Contract gates:
  - `docs/adr/0180-dirty-views-and-notify-gpui-aligned.md`
  - `docs/adr/0182-prepaint-interaction-stream-and-range-reuse.md`
  - `docs/adr/0190-prepaint-windowed-virtual-surfaces.md`
  - `docs/adr/0192-retained-windowed-surface-hosts.md`

Exit criteria (summary):

- View-cache reuse is behavior-preserving (semantics + routing + overlay behavior) under scripted diagnostics matrix.
- Prepaint/interaction stream carries enough information to avoid “tree re-walk drift” in hot paths (hit testing, coordinate mapping).
- A clear migration plan exists for “windowed surfaces” (VirtualList, code view, markdown, node graph) with at least one stress harness.

Validation anchors:

- `cargo run -p fretboard -- diag matrix ui-gallery`
- `cargo nextest run -p fret-ui`

### M3 — Text system v2 closure (UI authoring + quality baseline)

Goal: span-based text authoring surfaces are usable by ecosystem crates (markdown/code view) and quality knobs are stable.

Primary references:

- Workstream: `docs/workstreams/text-system-v2-parley.md`
- Contract gate: `docs/adr/0157-text-system-v2-parley-attributed-spans-and-quality-baseline.md`
- Text editing contracts: `docs/adr/0044-*`, `docs/adr/0045-*`, `docs/adr/0046-*`, `docs/adr/0071-*`

Exit criteria (summary):

- UI surfaces can author attributed spans deterministically (no ad-hoc per-crate hacks for code/markdown).
- Theme-only changes do not force reshaping/re-wrapping (paint-only path is validated).
- Conformance strings (mixed script + emoji + IME) exist and are stable across platforms/bundles.

Validation anchors:

- `cargo nextest run -p fret-render`
- Demos:
  - `cargo run -p fret-demo --bin emoji_conformance_demo`
  - `cargo run -p fret-demo --bin cjk_conformance_demo`

### M4 — Multi-window tear-off closure (capability-driven cross-platform behavior)

Goal: tear-off and cross-window docking behavior is deterministic and capability-modeled (no platform forks inside components).

Primary references:

- Workstream: `docs/workstreams/docking-multiwindow-imgui-parity.md`
- TODO tracker: `docs/workstreams/docking-multiwindow-imgui-parity-todo.md`
- Contract gates: `docs/adr/0013-*`, `docs/adr/0041-*`, `docs/adr/0017-*`, `docs/adr/0084-*`

Exit criteria (summary):

- “re-dock last tab closes empty floating window” is implemented and regression-tested.
- “close floating window merges content back” is implemented and regression-tested.
- “hovered window selection / set_outer_position / z-level” differences are represented as capability quality signals
  (prefer amending ADR 0054 / ADR 0084 instead of adding a new ADR).

Validation anchors:

- Demos:
  - `cargo run -p fret-demo --bin docking_demo`
  - `cargo run -p fret-demo --bin docking_arbitration_demo`
- Platform acceptance: keep macOS in the loop for multi-window behavior (z-order/focus/hovered-window quality).
