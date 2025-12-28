# Fret TODO Tracker (Review Findings)

This document tracks actionable TODOs discovered during architecture/doc/code review.
It complements (but does not replace) ADRs:

- ADRs define contracts and boundaries.
- This file lists concrete follow-up work, grouped by subsystem, and links back to the relevant ADRs.

## How to use

- Prefer turning P0 items into `Accepted` ADR decisions or conformance tests before adding new feature surface area.
- When an item is resolved, either delete it or move it into `docs/known-issues.md` (if it becomes a long-lived limitation).
- Deep-dive gap/backlog notes live under `docs/archive/backlog/` to keep `docs/` entrypoints small.

## P0 — IME / Text Input

- **Preedit-first key arbitration end-to-end (runner + routing)**
  - Problem: composing IME sessions must not lose `Tab/Space/Enter/NumpadEnter/Escape/Arrows/Backspace/...` to focus traversal or global shortcuts.
  - ADRs: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
  - Code: `crates/fret-runner-winit-wgpu/src/runner.rs`, `crates/fret-ui/src/tree.rs`, `crates/fret-ui/src/text_input.rs`, `crates/fret-ui/src/text_area.rs`
  - Current: `crates/fret-ui/src/tree.rs` defers shortcut matching for reserved keys and only falls back if the widget does not `stop_propagation`. `crates/fret-ui/src/text_input.rs` and `crates/fret-ui/src/text_area.rs` stop propagation for these keys while IME is composing (treat “composing” as `preedit` non-empty **or** preedit cursor metadata present).
  - Current: regression tests exist for:
    - composing: reserved keys suppress traversal/shortcuts (`tab_focus_next_is_suppressed_during_ime_composition`, `reserved_shortcuts_are_suppressed_during_ime_composition`);
    - not composing: `Tab` focus traversal works (`tab_focus_next_runs_when_text_input_not_composing`).

- **Define and validate blur/disable semantics for IME enablement**
  - Problem: ensure loss of focus reliably disables IME where appropriate, and avoid per-widget effect spam.
  - ADRs: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0020-focus-and-command-routing.md`
  - Code: `crates/fret-ui/src/tree.rs`
  - Current: `UiTree` owns `Effect::ImeAllow` and updates it on focus changes and at paint time; widgets only emit `Effect::ImeSetCursorArea` when the caret rect changes.

- **Multiline IME contract + conformance harness**
  - Goal: lock and validate multiline selection/composition/caret-rect behavior (scroll/wrap/DPI/preedit updates).
  - ADRs: `docs/adr/0071-text-input-multiline-composition-contract.md`, `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`, `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
  - Code: `crates/fret-ui/src/text_area.rs`, `crates/fret-render/src/text.rs`

## P0 — Fonts / Fallbacks / Missing Glyphs

- **Make the default font semantic (system UI font alias)**
  - Problem: relying on `FontId::default()` without a defined font family causes platform-dependent tofu and IME provisional-state breakage.
  - ADRs: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0006-text-system.md`
  - Code: `crates/fret-ui/src/theme.rs`, `crates/fret-render/src/text.rs`
  - Current: `crates/fret-render/src/text.rs` configures `cosmic-text`'s `fontdb` generic families at startup (preferring platform UI font families when present), so `Family::SansSerif` is no longer an implicit "Open Sans" placeholder.
  - Current: `TextStyle.font` now maps to `cosmic-text` generic families (`FontId::default()` -> sans, `FontId::serif()` -> serif, `FontId::monospace()` -> mono).
  - TODO: expose the default font stack at the theme/settings layer (and decide how user font loading maps to stable `FontId` values).

- **Fallback list participates in `TextBlobId` caching / invalidation**
  - Problem: changing configured fallbacks or font DB state must invalidate cached shaping/rasterization results.
  - ADRs: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
  - Code: `crates/fret-render/src/text.rs`
  - Current: `crates/fret-render/src/text.rs` includes a `font_stack_key` (derived from locale + configured generic families + fallback policy) in the `TextBlobKey` cache key.
  - TODO: when runtime font configuration becomes user-editable, add an explicit invalidation path that bumps the `font_stack_key` and clears cached blobs.

- **Emoji / variation selectors policy**
  - Goal: define baseline behavior for emoji fonts and variation selectors, and add a smoke test string that exercises it.
  - ADRs: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
  - Code: `crates/fret-render/src/text.rs`

## P0 — Docking / Overlays / Viewport Capture

- **Lock docking interaction arbitration (dock drag vs overlays vs viewport capture)**
  - Goal: prevent dismissal/capture conflicts and keep modal blocking rules intentional and consistent.
  - ADRs: `docs/adr/0072-docking-interaction-arbitration-matrix.md`, `docs/adr/0011-overlays-and-multi-root.md`, `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
  - Code: `crates/fret-ui/src/dock.rs`, `crates/fret-ui/src/tree.rs`, `crates/fret-components-ui/src/overlay_policy.rs`

- **Dock host keep-alive and early submission**
  - Goal: ensure dock hosts remain stable targets and do not "drop" docked content due to conditional submission.
  - ADRs: `docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`
  - Code: `crates/fret-ui/src/dock.rs`, runner/driver UI build order.

- **Programmatic close without one-frame tab "hole"**
  - Goal: add a `DockOp`/notify pattern so closing tabs from commands does not produce a transient no-selection/flicker.
  - ADRs: `docs/adr/0013-docking-ops-and-persistence.md`
  - Code: `crates/fret-ui/src/dock.rs`, app integration applying `DockOp` + invalidation.

## P0 - Scheduling / Render Lifecycle

- **Adopt the continuous frames lease contract across high-frequency subsystems**
  - Goal: use RAII `begin_continuous_frames` leases (ADR 0034) for viewport rendering, drags, and animations, and avoid ad-hoc RAF loops.
  - ADRs: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`
  - Code: `crates/fret-ui/src/elements.rs`, `crates/fret-runner-winit-wgpu/src/runner.rs`

- **Investigate "doesn't draw until hover" initial render regressions**
  - Symptom: some demo surfaces appear blank on startup and only render after pointer movement/hover.
  - Hypothesis: missing initial invalidation/redraw request, or render_root/layout/paint ordering drift.
  - ADRs: `docs/adr/0028-declarative-elements-and-element-state.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
  - TODO: add a tiny regression harness in `fret-demo` and lock this down with a deterministic first-frame draw rule.

## P1 — Accessibility (A11y) Conformance

- **Define minimum semantics for text fields (value/selection/composition)**
  - Goal: Narrator/AccessKit correctness for text editing and IME interaction.
  - ADRs: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`, `docs/adr/0071-text-input-multiline-composition-contract.md`
  - Code: `crates/fret-ui/src/tree.rs`, `crates/fret-platform/src/accessibility.rs`

- **Viewport semantics contract**
  - Goal: decide viewport role/actions (focus, scroll, basic labeling) and validate reachability under modal barriers.
  - ADRs: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`, `docs/adr/0007-viewport-surfaces.md`

## P1 — Tooling / Regression Harness

- **Add a repeatable IME regression checklist to the demo**
  - Goal: a short "manual test script" that can later be automated (Windows Japanese IME, caret placement, commit/cancel).
  - ADRs: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0071-text-input-multiline-composition-contract.md`
  - Code: `crates/fret-demo/src/components_gallery.rs` or `crates/fret-demo/src/ui_kit.rs` (choose a stable harness location).

- **Prefer `cargo nextest` for workspace tests**
  - Goal: make it easy to run conformance tests consistently.
  - Docs: `docs/README.md`, `docs/adr/README.md`

## P1 — Core Contract Drift

- **Formalize the vector path contract now that `SceneOp::Path` exists**
  - Problem: `fret-core::vector_path` and `SceneOp::Path` are implemented, but the contract is not yet locked at the ADR level (stroke joins/caps, AA expectations, transform interaction, caching keys).
  - ADRs: `docs/adr/0002-display-list.md`, `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
  - Code: `crates/fret-core/src/vector_path.rs`, `crates/fret-core/src/scene.rs`, `crates/fret-render/src/renderer.rs`, `crates/fret-ui-widgets/src/primitives/path.rs`

- **Clarify the runner vs platform split in docs and code**
  - Problem: `fret-platform` currently hosts the AccessKit bridge, while winit event mapping/effects draining live in `fret-runner-winit-wgpu`; keep responsibilities crisp to avoid duplicating window registries and event translation.
  - ADRs: `docs/adr/0003-platform-boundary.md`
  - Code: `crates/fret-platform/src/*`, `crates/fret-runner-winit-wgpu/src/runner.rs`

- **Decide whether `fret-platform::winit::WinitWindows` is part of the intended surface**
  - Problem: `crates/fret-platform/src/winit.rs` defines a window registry that is currently unused by `fret-runner-winit-wgpu`; decide whether to (a) use it, (b) keep it as internal scaffolding, or (c) remove it to avoid parallel window-tracking implementations.
  - ADRs: `docs/adr/0003-platform-boundary.md`
  - Code: `crates/fret-platform/src/winit.rs`, `crates/fret-runner-winit-wgpu/src/runner.rs`
