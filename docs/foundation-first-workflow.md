# Foundation-First Workflow (Plan C)

This project optimizes for a long-lived, cross-platform UI framework. The core risk is baking
policy-heavy behavior into the runtime too early, which later blocks performance work, a11y work,
and cross-platform portability.

Plan C is a “foundation-first, component-validated” loop:

- Keep `crates/fret-ui` small and mechanism-only (ADR 0066 / ADR 0074).
- Build a small number of high-leverage shadcn-aligned surfaces to *validate* the mechanisms.
- Scale component count only after the mechanisms are stable and tested.

## Layer Boundaries (What Goes Where)

### `fret-ui` (runtime / mechanism layer)

Allowed:

- Tree + layout + paint substrate (Taffy integration, display list building).
- Event routing primitives (pointer routing, key routing, focus primitives).
- Overlay/layer substrate (multi-root, barrier gating rules).
- Semantics snapshot and platform bridge boundary (ADR 0033).
- Small, renderer-friendly primitives that are hard to duplicate (clips, shadows, focus ring ops).

Not allowed:

- “What does activation do?” policy (model writes, toggles, overlay dismissal rules).
- Widget-specific keyboard strategies (roving, typeahead buffering, menu/listbox navigation rules).
- Shadcn/Radix/APG “outcome” semantics embedded as runtime defaults.

Mechanism-only APIs must remain easy to optimize: fewer public knobs, fewer side effects, and
explicit invalidation contracts.

### `fret-ui-kit` (headless + reusable policy helpers)

Allowed:

- Headless interaction policies expressed via action hooks (ADR 0074):
  - roving navigation state machines,
  - typeahead buffering/matching,
  - dismissal policy wiring (Escape/outside press), when the outcome is component-owned,
  - composite widget focus strategies (e.g. cmdk).
- Small declarative helpers that are not runtime contracts.

### `fret-ui-shadcn` (shadcn/ui-aligned surface)

Allowed:

- shadcn naming/taxonomy and visual recipes (tokens, variants, composition).
- Components built on `fret-ui-kit` headless helpers + `fret-ui` primitives.

Not allowed:

- New runtime APIs “because one component needs it” without an ADR and tests.

## “Add Runtime API” Gate (Hard-to-Change Contract)

If a component seems to need a new `fret-ui` capability:

1) Identify the upstream reference (GPUI / Radix / APG / AccessKit / Tailwind) and link it.
2) Decide whether it is mechanism or policy. If it’s policy, keep it in components.
3) If it’s mechanism, write/update an ADR and land a minimal test.
4) Only then broaden usage across the component surface.

## Validation Targets (Small but High Leverage)

Use a few components as acceptance tests for the substrate:

- **Command palette / listbox (cmdk)**: validates `active_descendant` semantics (ADR 0073) and
  keyboard routing while preserving IME/caret behavior.
- **Popover / tooltip**: validates anchored placement contract, portal/layering, and dismissal.
- **Menu / menubar**: validates roving + typeahead policy and focus/escape boundaries.

Each validation surface should have:

- a minimal demo entry (if UI behavior is hard to test),
- at least one unit/integration test for the hard-to-change contract.

## Current State (2025-12-29)

- Interaction policy leaks are removed from `fret-ui` (ADR 0074; MVP 68).
- `active_descendant` is implemented at the semantics schema + platform bridge level (ADR 0073,
  Phase A). The remaining work is component-layer cmdk policy wiring.

## Tracking (P0 closure)

For a cross-workstream “done enough to scale” checklist (milestones + exit criteria), see:

- `docs/workstreams/foundation-closure-p0.md`
- `docs/workstreams/foundation-closure-p0-todo.md`

