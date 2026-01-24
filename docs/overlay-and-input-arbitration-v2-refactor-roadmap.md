# Overlay & Input Arbitration v2 — Refactor Roadmap

Status: Draft (intended to land on `main` before implementation begins in a worktree branch)

This document is a concrete, test-driven refactor plan for Fret’s overlay substrate and input
arbitration, targeting editor-grade interaction **and** a general-purpose application framework
baseline.

Progress log (keep updated during implementation):

- `docs/workstreams/overlay-input-arbitration-v2.md`

It is aligned with the existing contract split:

- Runtime substrate (mechanisms): `crates/fret-ui`
- Policy + recipes (Radix/shadcn outcomes): `ecosystem/fret-ui-kit`, `ecosystem/fret-ui-shadcn`

References (existing accepted contracts):

- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Overlay policy boundary: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Outside press + click-through / consume / disableOutsidePointerEvents: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Docking vs overlays vs viewport capture arbitration: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Runtime contract map: `docs/runtime-contract-matrix.md`

Implementation anchors (current):

- Layer substrate: `crates/fret-ui/src/tree/layers.rs` (`push_overlay_root_ex`, `blocks_underlay_input`, `hit_testable`, outside-press flags)
- Dispatch: `crates/fret-ui/src/tree/dispatch.rs` + `crates/fret-ui/src/tree/mod.rs` (outside-press observer pass)
- Policy orchestration: `ecosystem/fret-ui-kit/src/overlay_controller.rs` + `ecosystem/fret-ui-kit/src/window_overlays/render.rs`

## Why v2 (Motivation)

Fret already has the right high-level architecture for overlays:

- window-scoped multi-root composition,
- deterministic routing across roots,
- a strict runtime/policy boundary.

The remaining risk is **behavioral drift** and **edge-case complexity** as more component recipes
and editor interactions are added (menus, dialogs, docking, viewports, tool capture).

This refactor aims to:

1. Make overlay lifecycle semantics explicit and uniform (no ad-hoc “almost interactive” states).
2. Make pointer occlusion semantics a first-class *mechanism* (not policy glue).
3. Lock cross-system arbitration with an executable conformance suite.
4. Improve ecosystem extensibility (new overlay kinds and policies without forking internals).

## Goals (P0)

- Preserve the core boundary: keep policies out of `crates/fret-ui`.
- Treat overlays as a stable, portable window-level contract for general-purpose UIs.
- Provide a clean mapping to Radix/shadcn overlay outcomes (DismissableLayer/FocusScope/Presence/Portal).
- Provide deterministic arbitration across:
  - pointer capture,
  - modal barrier,
  - docking drag sessions,
  - viewport tool capture,
  - non-modal overlays.
- Implement “disableOutsidePointerEvents, but allow scroll” as a runtime mechanism (GPUI-style outcome).
- Lock correctness with tests that survive fearless refactors.

## Non-Goals

- Replacing the declarative model or re-architecting the renderer.
- Implementing new shadcn components; this effort focuses on substrate + contracts + tests.
- Free-form, element-level painting escape hatches (GPUI `defer_draw` equivalents) unless strictly needed.

## Terms (v2 vocabulary)

To keep v2 refactors fearless, we use a small set of terms consistently across runtime, policy,
tests, and diagnostics:

- **Layer / root**: a window-scoped UI root in `UiTree` with explicit paint order (ADR 0011).
- **Modal barrier**: a layer with `blocks_underlay_input = true`. This is the *authoritative*
  “underlay inert” mechanism (pointer + keyboard + focus + semantics) while the barrier is present
  (ADR 0011 / ADR 0067).
- **Pointer occlusion**: a *pointer-only* underlay blocking mechanism for non-modal overlays
  (Radix `disableOutsidePointerEvents` outcome; ADR 0069). This is intentionally weaker than a
  modal barrier (no implied focus trap; no implied a11y inert).
- **Hit-testable**: whether the layer participates in hit-testing *inside* its subtree
  (`hit_testable = false` is “pointer transparent” for that layer).
- **Outside-press observer**: the runtime observer pass that can deliver a synthetic
  “pointer-down-outside” event to an overlay layer without stealing capture/focus (ADR 0069).
- **Presence**: policy-owned mount/paint vs interactive split (`OverlayPresence { present,
  interactive }`, ADR 0067).

## Portal / “escape painting” notes (non-normative)

GPUI’s `Window::with_content_mask` and `defer_draw` provide element-level “paint outside parent bounds”
escape hatches. In Fret, the **normative** escape hatch for most UI overlays is the window-scoped
multi-root overlay model (ADR 0011) plus an explicit placement solver (ADR 0064).

For a general-purpose UI framework, this is an intentional trade-off:

- Overlays/popovers/menus/tooltips should be expressed as overlay roots (portals), not as local
  absolute positioning inside potentially clipped dock panels.
- Per-element “paint escape” mechanisms may still be useful for specialized effects, but they are
  considered renderer/runtime *details* rather than the primary contract surface for overlay UI.

## Current Model (v1 summary)

### Layering

The runtime provides window-level layer roots with z-order and deterministic dispatch
(see ADR 0011). Overlays are installed by policy code via `UiTree::push_overlay_root_ex`.

Key runtime knobs (today):

- `blocks_underlay_input` (modal barrier scope)
- `hit_testable` (pointer transparent overlays)
- outside-press observer flags:
  - `wants_pointer_down_outside_events`
  - `consume_pointer_down_outside_events`
  - `pointer_down_outside_branches`

### Policy orchestration

The policy layer (`fret-ui-kit/window_overlays`) maps Radix outcomes onto the runtime substrate:

- Presence (mount vs interactive) is represented via `OverlayPresence { present, interactive }`.
- Menu-like overlays implement `disableOutsidePointerEvents` by enabling pointer occlusion
  (`PointerOcclusion::BlockMouseExceptScroll`) on the overlay layer.

This is directionally correct, but the barrier behavior is currently achieved by composing multiple
runtime flags. We want a simpler *mechanism vocabulary* that makes those compositions obvious and
hard to get wrong.

### Implementation snapshot (as of 2026-01)

The current `disableOutsidePointerEvents` outcome is implemented as a **separate, non-visual layer**
in `fret-ui-kit`:

- `ecosystem/fret-ui-kit/src/window_overlays/render.rs` installs an additional
  `pointer_barrier_layer` with:
  - `blocks_underlay_input = true` (reuses the modal barrier scoping mechanism),
  - `hit_testable = false` (so it does not become the hit-tested target).
- `crates/fret-ui/src/tree/dispatch.rs` contains a special-case for wheel events:
  - when the topmost barrier root is **hit-test-inert**, wheel hit-testing is re-run against
    hit-testable layers so the underlay scroll target can still receive `PointerEvent::Wheel`.

This works, and it is already covered by tests in `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`.
However, it has two structural drawbacks for a general-purpose framework baseline:

1. The semantics are implicit (a “barrier” that is not hit-testable is a *different* concept than a
   modal barrier, but the runtime does not model that explicitly).
2. The policy layer is forced to create extra layer roots to express a runtime outcome, which makes
   layering and debugging harder (especially once docking, viewports, and multi-pointer routing are
   involved).

## Proposed Changes (v2)

### 1) Runtime: Pointer Occlusion as a First-Class Mechanism (P0)

Add a small, explicit runtime mechanism to express occlusion outcomes in a stable vocabulary.

Proposed enum (names illustrative):

- `PointerOcclusion::None`
- `PointerOcclusion::BlockMouse`
- `PointerOcclusion::BlockMouseExceptScroll`

Key semantics:

- Applies at the window/layer substrate level (root scoping), not per-widget.
- Affects pointer interactions routed to underlay layers while an overlay is active.
- `BlockMouseExceptScroll` blocks hover/move/down/up outside the overlay, but allows wheel/scroll
  events to route to the underlay scroll target (matching established desktop UX and GPUI outcomes).

Interaction with existing knobs:

- `blocks_underlay_input` still wins: when a modal barrier is active, underlay layers are excluded
  by scope and occlusion is irrelevant for those layers.
- `hit_testable=false` remains “pointer transparent *within that layer’s subtree*”. Occlusion is
  about whether layers *behind* an overlay can be targeted when the pointer is *outside* the
  overlay subtree.
- Outside-press observer selection must remain compatible with occlusion: even when occlusion
  prevents a normal hit-tested target from being found, the observer pass still needs to run so
  policies can dismiss the overlay deterministically.

Routing sketch (non-normative algorithm outline):

1. Compute active input layer roots using the existing modal barrier scoping.
2. For pointer events without capture, iterate layers from top → bottom:
   - If the layer is not visible, skip.
   - If the layer is hit-testable and the event hits a node in that layer, select it as the target
     and stop (normal behavior).
   - If the layer has `PointerOcclusion != None` and **no hit occurred in this layer or above**:
     - For non-scroll events: stop and report “no underlay target” (underlay blocked).
     - For scroll-like events and `BlockMouseExceptScroll`: continue scanning lower layers.
3. Independently, run the outside-press observer selection pass (ADR 0069) against the topmost
   eligible overlay layer, even when the normal target is “none”.

Design constraints:

- Keep it mechanism-only: “when to enable occlusion” remains policy-owned.
- Keep it deterministic: scoping follows active layer order and the modal barrier rule.

Expected code touchpoints:

- Extend `UiLayer` and expose `UiTree::set_layer_pointer_occlusion(...)` (or equivalent).
- Update dispatch/hit-test to apply occlusion for non-modal layers without turning them into modal barriers.

Status: implemented in `crates/fret-ui` and adopted by `ecosystem/fret-ui-kit` (see ADR 0069).

### 2) Runtime: Make “Present vs Interactive” Routing Rules Hard (P0)

We must avoid ambiguous states where an overlay is painted but still interferes with input.

Contract:

- If an overlay is `present=true` but `interactive=false` (close transition), it must be:
  - pointer transparent for hit-testing,
  - excluded from outside-press observer selection,
  - excluded from any occlusion activation (unless it is a modal barrier that must remain authoritative).

Implementation strategy:

- Keep `present/interactive` in policy, but enforce the resulting runtime configuration via a single
  helper that sets all relevant runtime knobs consistently (visibility, hit-testability, observer flags, occlusion).
- Prefer “one helper per overlay kind” in `fret-ui-kit`, but backed by runtime-level invariants and tests.
   - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (`apply_non_modal_dismissible_layer_policy`),
     `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (`non_modal_overlay_disable_outside_pointer_events_does_not_block_underlay_while_closing`).
   - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (`apply_tooltip_layer_policy`,
     `apply_hover_layer_policy`), `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
     (`tooltip_does_not_request_observers_by_default`, `tooltip_does_not_request_observers_while_closing`,
     `hover_overlay_is_click_through_while_closing`).

### 3) Policy: Normalize Overlay Kinds + Capabilities (P0)

Lock a minimal set of overlay “kinds” at the policy surface, each mapping cleanly to runtime knobs:

- Modal (barrier-backed, focus trap expectations)
- Non-modal dismissible (outside press observer, click-through or consume)
- Menu-like non-modal (disableOutsidePointerEvents + allow scroll)
- Tooltip/hover (pointer transparent, no outside-press observer by default)
- Toast layer (non-blocking, focus behavior explicitly defined)

This keeps recipes consistent and reduces drift across ecosystem crates.

### 4) Arbitration: Executable Conformance Suite (P0)

Add a durable test suite that encodes cross-system arbitration, not just component outcomes.

Test categories:

1. **Pointer capture invariants**
   - Observer pass must not steal capture (already covered by v1-style tests; extend coverage).
2. **Modal barrier scoping**
   - Underlay is inert for pointer + focus + shortcuts while the barrier is present.
3. **DisableOutsidePointerEvents**
   - Underlay click/hover blocked while menu-like overlay is interactive.
   - Underlay scroll is still routed when configured as “except scroll”.
4. **Docking drag start/stop hygiene**
   - Starting dock drag closes or freezes non-modal overlays as configured.
   - Dock drag cannot be blocked by pointer-transparent overlays.
5. **Viewport capture interactions**
   - Viewport capture does not fight with docking drag capture; rules follow ADR 0072.
6. **Presence transitions**
   - `present=true && interactive=false` must be click-through and observer-inert for non-modals.

Where tests live:

- Runtime mechanism tests: `crates/fret-ui/src/tree/tests/` (dispatch, occlusion, barrier scoping).
- Policy tests: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (Radix mapping, focus restore, presence edges).
- Shadcn parity tests remain in `ecosystem/fret-ui-shadcn` as integration coverage.

### 4.1) Diagnostics-first testing (P0)

Conformance tests must fail in a way that is explainable without a debugger.

Minimum diagnostic anchors to require in tests:

- `UiTree::debug_layers_in_paint_order()` for asserting layer ordering and flags.
- `UiTree::debug_hit_test(...)` for asserting “which layer would win” at a given position.

Recommended additions during v2 implementation (not required to land this doc):

- A pointer-routing trace helper (e.g. “why was underlay blocked / which occluder stopped the scan”).
- A tiny event-script harness for dispatch tests (aligned with `docs/ui-diagnostics-and-scripted-tests.md`),
  so docking + viewport + overlays scenarios can be expressed as deterministic input scripts.

### 5) Ecosystem Extensibility: Stable Facades (P1, do early)

To support a general-purpose framework, ecosystem crates should not need to reach into
`window_overlays` internals.

Add a stable facade in `fret-ui-kit` for:

- submitting overlay requests (already exists conceptually via `OverlayController::request`),
- querying active overlay state (topmost dismissible, modal presence),
- optionally registering new overlay “kinds” (names + priority + default capabilities).

The intent is to keep `window_overlays` free to refactor while offering a stable extension seam.

## Migration Plan (Phased)

### Phase 0: Documentation + Test Inventory

- Land this document on `main`.
- Inventory existing tests that cover overlay dismissal/focus/observer behavior.
- Identify missing conformance cases (especially scroll pass-through).

### Phase 1: Runtime Occlusion Mechanism + Tests

- Implement `PointerOcclusion` mechanism in `crates/fret-ui`.
- Add focused runtime tests proving:
  - click/hover blocked,
  - wheel routed (except-scroll mode),
  - modal barrier still blocks everything.

### Phase 2: Policy Wiring + Presence Hardening

- Update `fret-ui-kit/window_overlays` to use the new mechanism for menu-like overlays.
- Ensure presence transitions configure runtime knobs consistently.
- Add policy-level tests for:
  - close transitions (present but non-interactive),
  - focus restoration correctness.

### Phase 3: Arbitration Conformance Suite

- Add docking + viewport capture interaction tests that exercise ADR 0072 edges.
- Ensure failures are deterministic and debuggable (include event scripts and state snapshots).

### Phase 4: Ecosystem Facade

- Introduce the stable extension facade in `fret-ui-kit`.
- Migrate internal callers to use the facade.

## Acceptance Criteria (Definition of Done)

- Menu-like overlays can block underlay pointer interaction while remaining scroll-friendly.
- Presence transitions never “eat” clicks or trigger outside-press observers during close animations.
- Docking and viewport capture behavior is deterministic and regression-tested (ADR 0072 conformance).
- No new policy behavior leaks into `crates/fret-ui`; runtime changes remain mechanism-only.
- Ecosystem crates can consume overlay services via stable `fret-ui-kit` surfaces without depending on `window_overlays` internals.

## Open Questions

- Should “except scroll” be the default for `disableOutsidePointerEvents` across all menu-like overlays,
  or opt-in per component recipe?
- Should the runtime treat scroll routing as “wheel only” or include touchpad pan/gesture events as well?
- Do we need a separate “keyboard occlusion” mechanism for non-modal overlays (e.g. prevent underlay shortcuts)
  without full modality, or is that strictly policy?
