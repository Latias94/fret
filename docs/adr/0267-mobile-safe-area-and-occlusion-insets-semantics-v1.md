# ADR 0267: Mobile Safe-Area and Occlusion Insets Semantics (v1)

Status: Proposed

## Context

Fret already exposes environment query keys for safe-area and occlusion insets (ADR 0232), and a
portable service for committing those values per window (`WindowMetricsService`). On mobile, these
values become “hard-to-change” quickly because they influence:

- keyboard avoidance behavior for text inputs,
- layout and hit-testing near system UI,
- candidate-window placement and caret anchoring (ADR 0012 / ADR 0261),
- and portability of shadcn/Radix recipes that assume a predictable visible rect.

If the semantics are not locked early, ecosystem code will encode inconsistent assumptions (e.g.
“occlusion includes safe area” vs “occlusion is only keyboard”), which will later require
large-scale refactors.

This ADR defines the v1 semantics for **safe-area** and **occlusion** insets, including the
unknown/known representation, coordinate model, and update expectations needed for mobile runners.

## Goals

1. Lock the meaning of `safe_area_insets` vs `occlusion_insets` for future Android/iOS runners.
2. Lock the representation of “unknown vs known-but-zero”.
3. Define the coordinate model and update timing so keyboard avoidance can be implemented in
   ecosystem crates without runner-specific hacks.
4. Keep `crates/fret-ui` mechanism-only (ADR 0066): no keyboard-avoidance policy in the runtime.

## Non-goals (v1)

- Prescribing a single keyboard-avoidance policy or gesture arena design.
- Guaranteeing perfect OEM/IME behavior on day one.
- A multi-window mobile navigation stack contract.

## Decision

### D1 — Definitions: safe-area vs occlusion are separate concepts

For each window, the runner may commit two independent inset values, both expressed as per-edge
`Edges<Px>` (top/left/bottom/right):

- **Safe-area insets** (`safe_area_insets`): areas that SHOULD be treated as reserved by the
  platform UI (status bar, notch/cutout, home indicator area, system gesture regions when
  applicable). The intent is “avoid drawing important UI under this region”.

- **Occlusion insets** (`occlusion_insets`): areas that are likely **not visible** or **not
  interactable** due to a transient obstruction (most notably the on-screen keyboard), or due to
  platform UI that overlays the app’s content area (best-effort).

Contract rule (v1):

- `safe_area_insets` MUST NOT implicitly include keyboard occlusion.
- `occlusion_insets` SHOULD represent *transient* occlusion (keyboard) when available; it MAY also
  include other transient overlays when the platform provides a unified “occluded region” API.

Consumers MUST treat these as two inputs and derive their own “content safe rect” and “visible
rect” policy as needed (see D5).

### D2 — Unknown vs known-but-zero is represented explicitly

Insets are committed via `WindowMetricsService`:

- `set_safe_area_insets(window, Option<Edges>)`
- `set_occlusion_insets(window, Option<Edges>)`

Representation rules:

- **Unknown**: the runner has not committed any value yet:
  - `WindowMetricsService::safe_area_insets_is_known(window) == false`
  - `WindowMetricsService::occlusion_insets_is_known(window) == false`
- **Known but none**: the runner has committed `None` (meaning “known to be absent / not applicable”):
  - `*_is_known(window) == true` and `*_insets(window) == None`
- **Known with value**: the runner has committed `Some(Edges { .. })`:
  - `*_is_known(window) == true` and `*_insets(window) == Some(edges)`

Rationale: mobile runners frequently cannot provide insets immediately at startup, and “unknown”
must not be conflated with “zero”.

### D3 — Coordinate model: window logical pixels

All insets in this ADR are in **window logical pixels** (DIP / logical px), consistent with ADR
0017 and ADR 0232.

Runners MUST convert platform-native pixel units into logical px using the current scale factor.

### D4 — Update timing and invalidation expectations

The runner commits safe-area and occlusion insets into `WindowMetricsService` whenever values
change (best-effort). The UI runtime consumes the committed values as part of the per-frame
environment snapshot (ADR 0232).

Normative expectations (v1):

- When either inset value changes for a window, the runtime MUST treat the corresponding
  environment query key (`SafeAreaInsets` / `OcclusionInsets`) as changed for that frame so
  observers can invalidate.
- Runners SHOULD request a redraw after committing insets changes (one-shot is sufficient).

Note: the exact invalidation level (Layout/Paint/HitTest) is determined by observers; the runtime
must provide dependency tracking so layout-sensitive consumers can relayout.

### D5 — Keyboard avoidance is ecosystem policy (recommended baseline algorithm)

Keyboard avoidance is policy-owned, but this ADR provides a recommended baseline algorithm that is
portable and uses only contract surfaces:

Inputs:

- `occlusion_insets.bottom` (when known),
- the focused text input caret/selection anchor rect (ADR 0012 / ADR 0261; e.g.
  `WindowTextInputSnapshot.ime_cursor_area`),
- `viewport_bounds_logical` (ADR 0232).

Recommended derived rects:

- `content_safe_rect = viewport_bounds_logical.deflate_edges(safe_area_insets.unwrap_or_zero())`
- `visible_rect = content_safe_rect.deflate_edges(occlusion_insets.unwrap_or_zero())`

Policy action (example):

- If `ime_cursor_area` falls outside `visible_rect`, request a scroll-into-view in the nearest
  scroll container (ecosystem policy; runtime provides scroll primitives).

This keeps the runtime mechanism-only while ensuring the minimum data flow is stable for mobile.

## Consequences

- Mobile runners can evolve their best-effort insets sourcing without breaking ecosystem code.
- Ecosystem components can implement keyboard avoidance without relying on platform paths or ad-hoc
  heuristics.
- “Unknown vs known-but-zero” is locked early, reducing future refactor risk.

## Implementation status (current)

As of 2026-02-12:

Implemented (evidence anchors):

- Insets are stored per-window in `WindowMetricsService`:
  - `crates/fret-core/src/window.rs`
- The runtime exposes safe-area/occlusion via environment queries (best-effort):
  - `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
  - `crates/fret-ui/src/elements/runtime.rs` (committed snapshot + dependency tracking)
- Diagnostics can override insets for scripted repros:
  - `crates/fret-runtime/src/effect.rs` (`Effect::WindowMetricsOverrideInsets`)

Known gaps (v1):

- Android/iOS runners are not yet fully audited for insets sourcing correctness and timing.
- A shared ecosystem “keyboard avoidance helper” policy should stay small and be validated against
  multiple real IMEs/OEMs.

## References

- ADR 0232: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- ADR 0012: `docs/adr/0012-keyboard-ime-and-text-input.md`
- ADR 0261: `docs/adr/0261-platform-text-input-client-interop-v1.md`
- ADR 0262: `docs/adr/0262-mobile-lifecycle-and-surface-policy-v1.md`
- ADR 0260: `docs/adr/0260-mobile-shell-runtime-bridge-v1.md`
