# ADR 0313: Window Hit Test Regions v1 (Passthrough Except Interactive Regions)

Status: Accepted

## Context

ADR 0324 defines a portable, capability-gated window hit-testing facet:

- `WindowStyleRequest.hit_test = Normal | PassthroughAll`

This enables fully click-through utility windows, but it does not support a common desktop posture:

- A visually large overlay window that should be *click-through in most places* while still
  accepting pointer interaction in a small set of bounded areas (e.g. a compact control panel).

Without OS-level region hit testing, a window that "ignores" pointer events in UI-space still
blocks underlying windows: pointer events are delivered to the top-level OS window first, and
there is no portable way to "forward" them to whatever is behind it.

We therefore need a portable mechanism to express "passthrough except for these regions" while:

- staying orthogonal to `transparent` and background materials (ADR 0139 / ADR 0310),
- remaining capability-gated and deterministically degradable (ADR 0054),
- remaining observable via diagnostics and scripted predicates (ADR 0036),
- avoiding per-pixel shaped windows in v1.

## Goals

1. Extend the window hit-test contract with a small, portable region vocabulary.
2. Keep the API surface minimal and intent-oriented (rect / rounded-rect / union).
3. Specify deterministic clamping behavior when regions are unsupported.
4. Require diagnostics observability of requested vs effective hit-test regions.

## Non-goals

- Per-pixel hit testing based on alpha masks.
- Arbitrary path shapes and boolean operations beyond union.
- A guarantee that region semantics are identical across all platforms/compositors.

## Decision

### 1) Extend `WindowHitTestRequestV1` with a regions variant

Extend `fret_runtime::WindowHitTestRequestV1` with:

- `PassthroughRegions { regions: Vec<WindowHitTestRegionV1> }`

Semantics:

- When `PassthroughRegions` is effective, the OS window is treated as **passthrough by default**
  and **interactive only within** the union of `regions`.

### 2) Define `WindowHitTestRegionV1`

`WindowHitTestRegionV1` is a simple, portable shape vocabulary:

- `Rect { x, y, width, height }`
- `RRect { x, y, width, height, radius }` (single radius for simplicity)

All values are in **logical pixels** in **window client coordinates**:

- `(0, 0)` is the top-left of the client area.
- Coordinates are independent of OS window decorations (client-only).

Regions are interpreted as a **union**:

- If any region contains the pointer point, the window is interactive at that point.
- Otherwise, the window is passthrough at that point.

Validation/clamping (normative):

- Negative or non-finite sizes clamp to an empty region.
- Radius is clamped to `[0, min(width, height) / 2]`.

### 3) Capability keys

This ADR standardizes the existing capability key:

- `ui.window.hit_test.passthrough_regions`

Runners must:

1. Advertise whether regions are supported.
2. Clamp region requests deterministically.
3. Expose requested/effective state via diagnostics.

### 4) Deterministic degradation (normative)

If `PassthroughRegions` is requested but `ui.window.hit_test.passthrough_regions == false`, the
runner must clamp deterministically:

- If `ui.window.hit_test.passthrough_all == true`, clamp to `PassthroughAll`.
- Otherwise, clamp to `Normal`.

Rationale:

- Prefer preserving "click-through" posture where possible.
- If click-through cannot be provided, prefer a usable window over a permanently non-interactive
  one.

### 5) Patchability

Region hit testing is runtime patchable on platforms where the backend supports it. Where the
backend only supports create-time hit-test regions, runners may ignore runtime patches, but must
reflect the effective/clamped state in diagnostics.

### 6) Observability (required)

Diagnostics must be able to answer per window:

- What hit-test policy and regions were requested?
- What policy/regions are effective (after clamping/validation)?
- If clamped, which capability was missing and which fallback was chosen?

This ADR does not require scripts to assert exact region geometry in v1, but does require a stable
“region signature” (e.g. canonicalized region list + hash) so scripts can gate that regions were
applied and remain stable across refactors.

## Consequences

Pros:

- Enables true “passthrough except controls” utility windows without native-handle escape hatches.
- Keeps the contract small and capability-gated.
- Unlocks deterministic scripted regression gates for overlay interaction posture.

Cons:

- Region support is uneven across platforms; the contract must remain best-effort.
- Frequent region updates can be expensive or unsupported; some designs may still require a
  multi-window composition fallback.
