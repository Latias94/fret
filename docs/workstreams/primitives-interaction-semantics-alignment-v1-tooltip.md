# Primitives Interaction Semantics Alignment v1 — Tooltip (Audit Sheet)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Active (workstream note; not a contract)

Baseline: Radix Tooltip outcomes (provider delays, skip-delay window, pointer/keyboard triggers).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/tooltip/src/*`

---

## Current Fret implementation anchors

- Primitives/policy:
  - `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`
  - `ecosystem/fret-ui-kit/src/tooltip_provider.rs`
  - delay-group state machine: `ecosystem/fret-ui-headless/src/tooltip_delay_group.rs`
  - shared hover intent helper: `ecosystem/fret-ui-kit/src/primitives/hover_intent.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/tooltip.rs`

Time-source note (audit focus):

- Internally, Tooltip interaction remains **tick-based** (`open_delay_ticks*` / `close_delay_ticks*`) with a deterministic delay-group state machine.
- The shadcn recipe now surfaces **semantic wall-clock configuration** (`Duration`) for:
  - `TooltipProvider::{delay, close_delay, timeout_duration}`
  - `Tooltip::{open_delay, close_delay}`
- `Duration` is mapped to ticks using `WindowFrameClockService` (prefers fixed delta via `FRET_DIAG_FIXED_FRAME_DELTA_MS`), with a stable 60Hz fallback for headless/test environments to avoid flake.

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` (tooltip fixtures)

Evidence (Radix web timeline parity gates):

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`:
  - `tooltip-example.tooltip.hover-show-hide.light`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `tooltip-example.tooltip.hover-show-hide.light`

Scripted repros (existing):

- `tools/diag-scripts/ui-gallery-tooltip-repeat-hover.json`
- `tools/diag-scripts/ui-gallery-tooltip-docs-smoke.json`
- `tools/diag-scripts/ui-gallery-tooltip-hovercard-scroll-clamp.json`
- `tools/diag-scripts/ui-gallery-tooltip-delay-group-skip-delay.json`
- `tools/diag-scripts/ui-gallery-overlay-modals-visible.json` (tooltip open snapshot)

---

## Outcome model (what we must preserve)

State:

- `open`
- provider delay state (open delay, skip-delay window)
- pointer hover intent state (enter/leave, grace windows if any)

Invariants:

- Provider delays are semantic (`Duration`) and consistent across instances.
- Keyboard focus-triggered tooltip is distinct from hover-triggered tooltip.
- Scroll/viewport clamp behavior is stable.

### Provider delay-group model (Radix `TooltipProvider`)

Radix tooltip providers apply a “delay then skip-delay window” behavior across instances:

- First open after idle uses `delay`.
- After a tooltip closes, subsequent tooltip hovers within a short `skip-delay` window open immediately.

In Fret, this is implemented as a deterministic tick-based state machine:

- Config: `TooltipDelayGroupConfig { delay_ticks, skip_delay_ticks }`
- State: `TooltipDelayGroupState { last_closed_at }`
- Transition: `note_closed(now)` records `last_closed_at`
- Decision: `open_delay_ticks(now, cfg)` returns `0` if `now - last_closed_at <= skip_delay_ticks`, else `delay_ticks`.

Anchors:

- `ecosystem/fret-ui-headless/src/tooltip_delay_group.rs`
- provider wiring + stack service: `ecosystem/fret-ui-kit/src/tooltip_provider.rs`

### Trigger gating model (hover/focus vs explicit dismiss)

Fret models Radix-like tooltip “open affordance” gates at the trigger:

- Hover-open can be suppressed after an explicit dismiss (escape / outside press) until the pointer leaves the trigger.
- Mouse hover-open is gated on “has seen pointer move” to avoid instant open on synthetic hover transitions.

Anchors:

- `TooltipTriggerEventModels` + `tooltip_trigger_update_gates`: `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`

---

## Audit checklist (dimension-driven)

- [x] `M` Document provider + delay group semantics (delay, skip-delay window).
- [x] `I` Surface `Duration` delay configuration at recipe/policy boundary.
- [x] `G` Keep a diag script gating repeat-hover stability and skip-delay behavior.
