# Upstream Audit: Zed / GPUI (Focus + Overlay Mechanisms)

This note summarizes upstream mechanisms in Zed's `gpui` that are relevant to Fret's focus,
overlay, and event dispatch correctness work (A/B/C phases of this workstream).

The intent is not to copy APIs, but to identify proven invariants and the failure modes they
prevent, then map those outcomes to Fret's mechanism layer without leaking component policy into
`crates/fret-ui`.

## Scope

Focus and overlay failure-prone areas:

- focus identity and focus containment queries
- dispatch-time snapshot coherence (avoid consulting live mutable structures mid-dispatch)
- pointer dispatch ordering (capture vs bubble) for outside-press style interactions
- interaction barriers (block mouse while optionally allowing scroll)

## Key upstream observations (mechanisms)

### 1) Focus containment is answered against a per-frame dispatch snapshot (not a live tree)

`FocusHandle::contains(...)` and related queries operate against the **most recently rendered
frame's dispatch tree**, not an in-flight or live mutating structure. This reduces the chance that
event routing or containment checks observe a transient inconsistent state.

Upstream anchors (local reference checkout):

- `F:\\SourceCodes\\Rust\\fret\\repo-ref\\zed\\crates\\gpui\\src\\window.rs` (`FocusHandle`, `FocusId::contains`, `rendered_frame.dispatch_tree`)
- `F:\\SourceCodes\\Rust\\fret\\repo-ref\\zed\\crates\\gpui\\src\\key_dispatch.rs` (`DispatchTree`, `DispatchNodeId` not stable between frames)

Mapping to Fret:

- This aligns with Phase C: build (or at least anchor) containment and routing decisions on a
  coherent dispatch snapshot rather than retained `parent` pointers.

### 2) Dispatch uses a capture + bubble model with deterministic ordering

Mouse dispatch in GPUI runs:

1. Capture phase first (listeners iterated in a deterministic order), described as useful for
   "detecting events outside of a given Bounds".
2. Bubble phase second (reverse order), where most normal handlers do their work.

Upstream anchor:

- `F:\\SourceCodes\\Rust\\fret\\repo-ref\\zed\\crates\\gpui\\src\\window.rs` (`dispatch_mouse_event`)

Mapping to Fret:

- Phase C should ensure Fret's dispatch snapshot provides a stable, per-dispatch ordering for
  outside-press style mechanisms (DismissibleLayer / overlay stacks), so "outside" detection does
  not depend on any live-tree traversal.

### 3) Overlays commonly need "block mouse, allow scroll" semantics

GPUI's hit-testing distinguishes:

- `is_hovered(...)`: used for interaction directly under the mouse (hover styles, clicks, tooltips)
- `should_handle_scroll(...)`: used for `ScrollWheelEvent` to locate an outer scroll container even
  when a front layer blocks mouse interactions

And it supports hitbox behaviors that occlude elements behind an overlay:

- `HitboxBehavior::BlockMouse`
- `HitboxBehavior::BlockMouseExceptScroll`

Upstream anchor:

- `F:\\SourceCodes\\Rust\\fret\\repo-ref\\zed\\crates\\gpui\\src\\window.rs` (`Hitbox::is_hovered`, `Hitbox::should_handle_scroll`, `HitboxBehavior`)

Mapping to Fret:

- Fret currently has barrier-style mechanisms for modal/overlay stacks. We should explicitly decide
  whether scroll wheel can "tunnel" to underlying scroll containers when a barrier is active.
  If yes, the mechanism layer needs a separate scroll hit-test pathway (or an explicit exception)
  analogous to `should_handle_scroll`.
  Policy (when to allow) still belongs in `ecosystem/*`.

### 4) Focus identity is explicit and ref-counted; dispatch node IDs are ephemeral

GPUI separates:

- stable focus identity (`FocusId` + `FocusHandle` stored in a `FocusMap`)
- per-frame dispatch node identity (`DispatchNodeId` is explicitly not stable across frames)

Upstream anchors:

- `F:\\SourceCodes\\Rust\\fret\\repo-ref\\zed\\crates\\gpui\\src\\window.rs` (`FocusHandle`, `WeakFocusHandle`, ref-counting)
- `F:\\SourceCodes\\Rust\\fret\\repo-ref\\zed\\crates\\gpui\\src\\key_dispatch.rs` (`DispatchNodeId` not stable)

Mapping to Fret:

- This is consistent with "state outlives a frame" while "dispatch graph is per-frame/per-dispatch".
  Fret's `GlobalElementId`/`NodeId` split should be treated similarly: stable identities for state,
  ephemeral snapshot IDs for dispatch traversal.

## Likely failure modes to watch in Fret

These are the UI-framework "classic" traps that the upstream architecture tends to avoid:

- Containment checks consulting live retained parent pointers mid-dispatch (transient drift can
  produce false negatives and leak focus/capture outside an active overlay scope).
- Outside-press dismissal observing inconsistent ordering (a background handler sees the event
  before a foreground barrier/dismissible layer).
- Barrier semantics that unintentionally block scroll wheel (UI feels "frozen" under overlays even
  when scroll should continue to work).
- Assuming a stable "dispatch node id" across frames (breaks focus restore / action routing when
  nodes are rebuilt or subtrees are reused).

## Actionable follow-ups for this workstream

1. Phase C perf probe: measure dispatch snapshot build cost and worst-frame impact in UI gallery
   overlay scenarios; gate regressions with `fretboard-dev diag perf`.
2. Clarify barrier scroll semantics:
   - Decide mechanism contract: whether scroll is blocked by default, allowed by default, or
     allowed only with an explicit policy knob.
   - Add a regression test + a scripted diag repro for the chosen semantics.
3. Audit remaining dispatch paths for "live tree" membership queries and migrate to snapshots where
   possible (maintain the A/B "child-edge reachability" invariant).

