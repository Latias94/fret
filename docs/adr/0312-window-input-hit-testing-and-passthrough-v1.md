# ADR 0312: Window Input Hit Testing and Passthrough v1 (Window-Level)

Status: Proposed

## Context

ADR 0139 introduced the need for click-through utility windows. Many overlays are still served by
simple window-level passthrough, but we want an explicit, diagnosable OS hit-test contract that is
orthogonal to transparency and background materials:

- A visually transparent / material-backed window that is only interactive in a bounded panel area.
- A frameless window that should ignore pointer events in "empty" regions but accept them in controls.

Importantly, input hit testing must remain **orthogonal** to:

- window surface alpha composition (`transparent`),
- OS backdrop materials (ADR 0310),
- renderer-level effects inside the window.

We want a portable contract that:

- keeps native handle APIs out of portable crates (ADR 0090),
- is capability-gated and degrades deterministically (ADR 0054),
- is observable via diagnostics (ADR 0036),
- does not require per-pixel shaped windows in v1.

## Goals

1. Define a portable, capability-gated contract for window-level pointer passthrough.
2. Keep the mechanism layer free of application policy (panel sizing, interaction rules).
3. Require per-window observability of effective/clamped hit-test policy.

## Non-goals

- Per-pixel hit testing based on alpha masks or arbitrary paths.
- Exact parity of region semantics across X11/Wayland/macOS/Windows (best-effort, capability-gated).
- Keyboard focus semantics (handled by `ActivationPolicy` and focus routing).

## Decision

### 1) Introduce a dedicated hit-test policy facet (orthogonal to `mouse`)

Add a new optional facet to `WindowStyleRequest`:

- `hit_test: Option<WindowHitTestRequestV1>`

This facet is intentionally separate from the existing `mouse` facet. `mouse` remains a high-level
policy vocabulary for pointer routing. `hit_test` is a window-level OS hit-test contract used for
click-through utility windows.

### 2) Define `WindowHitTestRequestV1`

`WindowHitTestRequestV1` is intent-oriented and capability-gated:

- `Normal` (default): window participates in pointer hit testing normally.
- `PassthroughAll`: the entire window ignores pointer hit testing (click-through).

Region-based hit testing is intentionally deferred: v1 focuses on a deterministic, window-level
contract that can be implemented safely across backends. A follow-up ADR may extend `hit_test`
with a region vocabulary once the runner plumbing and diagnostics evidence are stable (ADR 0313).

### 3) Capability keys

Add capability keys (ADR 0054):

- `ui.window.hit_test.passthrough_all`
- `ui.window.hit_test.passthrough_regions`

`ui.window.hit_test.passthrough_regions` is reserved for a future region-based extension. Runners
must report it as unavailable until there is an end-to-end implementation. See ADR 0313 for the
region vocabulary and degradation rules.

Runners must:

1. Advertise available keys at startup.
2. Clamp requests deterministically.
3. Expose effective/clamped hit-test state per window via diagnostics.

### 5) Observability (required)

Runners must expose per-window information that can answer:

- What hit-test policy was requested?
- What policy is effective/clamped?
- If clamped, which capability was missing?

### 6) Patchability

`hit_test` is runtime patchable on platforms where the backend supports it. Where it is create-time
only, runners may ignore runtime patches (and must reflect that in the effective snapshot).

## Consequences

Pros:

- Makes utility windows practical without native handle escape hatches.
- Keeps input semantics orthogonal to transparency/material contracts.
- Enables deterministic scripted regression gates via diagnostics predicates.

Cons:

- Cross-platform support for region-based hit testing is uneven; v1 must be conservative.
- Without per-pixel hit testing, some visual designs still require in-window composition to avoid
  "dead zones".
