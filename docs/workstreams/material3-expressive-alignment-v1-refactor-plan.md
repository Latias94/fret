# Material 3 Expressive Alignment v1 (Fearless Refactor Plan)

Goal: align **Material 3 Expressive** component outcomes (tokens, shapes, motion, semantics, interaction) in Fret while
preserving Fret's core design philosophy: **mechanisms in `crates/*`, policies + recipes in `ecosystem/*`**.

This workstream is scoped to **outcomes**, not 1:1 API compatibility with Compose/MUI.

## Source of truth (order of precedence)

1. Material 3 guidelines/spec (intent + UX constraints).
2. Compose Material3 (state machines, semantics, sizing, interaction policy).
3. MUI Material UI (web-specific defaults and interaction edge cases).
4. Base UI / Radix primitives (headless accessibility patterns, especially when MUI/Compose differ).

Local snapshots under `repo-ref/` are optional convenience only (see `docs/repo-ref.md`).

## Non-negotiable layering rules

- **Mechanisms** (`crates/*`): platform input, focus primitives, semantics tree, overlay roots, hit testing, paint
  primitives.
- **Shared policy** (`ecosystem/fret-ui-kit`): generic state machines (roving focus, dismissal, focus restore, etc).
- **Material policy + recipes** (`ecosystem/fret-ui-material3`):
  - token mapping (`md.comp.*` → `md.sys.*` fallback chains),
  - Material indication orchestration (state layers + ripples + focus ring),
  - component recipes (layout, chrome, semantics wiring),
  - stable `test_id` surfaces for diag automation.

If the mismatch is "interaction policy", it almost never belongs in `crates/fret-ui`.

## What "Expressive" means in this repo

- `DynamicVariant::Expressive` is the source of truth for **scheme/palette** differences (`md.sys.color.*`).
- Per-component `.expressive.*` token keys are only implemented when upstream token sets provide them (e.g. Material Web v30
  currently includes expressive component tokens only for `List`).
- Do not invent placeholder expressive component tokens. Prefer routing via `MaterialDesignVariant` and typed token
  modules.

See also: `docs/workstreams/material3-todo.md` (general MD3 alignment), `docs/workstreams/material3-refactor-plan.md`
(shared infrastructure).

## Delivery shape (the required 3-pack)

Every landed alignment change must ship as a 3-pack:

1. **Repro surface**: a small UI gallery demo page/section.
2. **Gate**: at least one deterministic regression gate:
   - headless test for deterministic logic/invariants, and/or
   - `tools/diag-scripts/*.json` script that asserts semantics/predicates with stable `test_id` selectors.
3. **Evidence anchors**: 1–3 pointers that make review fast:
   - upstream reference (doc/source file),
   - in-tree owner path + key symbol,
   - test/script path + exact command to run.

## Per-component alignment checklist

For each component:

- **Semantics**: correct `role`, `checked/selected/expanded`, labels, focusability, invoke behavior.
- **Interaction outcomes**:
  - hover/pressed/focus state layer color + opacity,
  - ripple clip policy (bounded/unbounded) and base opacity,
  - keyboard activation parity (Enter/Space; focus-visible gating).
- **Layout + sizing**:
  - container size + min touch target,
  - icon sizes, paddings, default density,
  - outline widths and corner radii precedence.
- **Motion**:
  - token-driven durations/easing (or a shared MotionScheme mapping),
  - interruption + settle rules (stable structure during press; stable geometry after settle).
- **Tokens**:
  - typed token module owns fallback chain and prevents drift,
  - no hard-coded numbers unless upstream is ambiguous (and then document it).
- **Automation**:
  - stable `test_id` on the interactive node,
  - diag script uses `test_id` selectors + predicates, not brittle geometry.

## Risks / known constraints

- Some upstream semantics use true tri-state (checked / unchecked / indeterminate). Fret currently maps indeterminate to
  `checked: None` (see `ecosystem/fret-ui-kit/src/primitives/checkbox.rs`). When OS-level accessibility needs a distinct
  "mixed" value, that becomes a **mechanism** task and must be handled as a contract change.

## References (in-tree)

- Architecture/layering: `docs/architecture.md`
- Material alignment backlog: `docs/workstreams/material3-todo.md`
- Diagnostics and scripted tests: `docs/ui-diagnostics-and-scripted-tests.md`
- Interactivity pseudoclasses stability: `docs/adr/0166-interactivity-pseudoclasses-and-structural-stability.md`

