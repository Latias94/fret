# UI Direction + RTL Parity (Fearless Refactor v1)

Status: Draft (workstream note)

This workstream locks down a **direction substrate** (LTR/RTL) that is consistent across:

- authoring defaults (`fret-ui-kit` builders),
- shadcn recipes (`fret-ui-shadcn`),
- and direction-sensitive mechanisms (placement math, scroll/drag sign, key navigation).

The goal is to make direction-related refactors **fearless** by leaving behind:

- a small set of invariants (“what must always be true”),
- a component parity matrix,
- and regression gates (unit tests + `fretboard-dev diag` scripts) for the failure-prone cases.

## Scope / layering

Direction is a cross-cutting concern, so we must be explicit about ownership:

- `crates/*`: mechanisms + hard contracts (e.g. placement math correctness, input routing).
- `ecosystem/fret-ui-kit`: headless policy + authoring defaults (e.g. what `TextAlign::Start` means).
- `ecosystem/fret-ui-shadcn`: shadcn composition + sizing + padding defaults (RTL demos must match docs).

This workstream intentionally prefers **policy-layer closure** first (kit/shadcn), and only escalates
to a `crates/*` contract change when we can’t get stable parity outcomes without it.

## Source of truth

- Radix direction semantics audit: `docs/audits/radix-direction.md`
- shadcn docs parity tracker: `docs/workstreams/standalone/ui-gallery-shadcn-docs-alignment-v4-todo.md`

## Public direction vocabulary

The direction substrate is intentionally small:

- `LayoutDirection::{Ltr, Rtl}` is the public direction enum used by authoring code.
- `DirectionProvider` / `with_direction_provider(...)` install an inherited direction for a
  subtree.
- `use_direction(local, inherited)` resolves `local || inherited || Ltr`.
- `use_direction_in_scope(cx, local)` is the common helper for recipes that need to resolve the
  active provider plus an optional local override.
- `TextAlign::Start` and `TextAlign::End` are logical. Use them when the desired outcome should
  flip under RTL.
- Physical `Left` / `Right` decisions must be explicit when the geometry should not flip.
- Overlay and placement helpers should take an explicit `LayoutDirection` rather than reading
  hidden ambient state after entering a portal/root boundary.

This workstream does not attempt to define a general CSS writing-mode contract. If we need one,
that is a separate mechanism discussion.

## Authoring helpers

Recommended call-site patterns:

- At the recipe root, prefer `DirectionProvider::new(dir)` or
  `with_direction_provider(cx, dir, ...)` when a subtree needs an inherited direction.
- Inside the recipe, prefer `use_direction(cx, None)` / `use_direction_in_scope(cx, None)` over
  ad-hoc branching.
- For start/end placement or alignment, prefer the shared logical helpers in `fret-ui-kit` and
  `fret-ui-shadcn` instead of repeating left/right mappings per component.
- Keep physical left/right branches explicit only where the component truly needs them.

## Invariants (must hold)

1. **Logical direction resolution**
   - When a direction provider exists, the effective direction is resolved like Radix:
     local override wins, otherwise inherit, otherwise default to LTR.
2. **Logical alignment**
   - `TextAlign::Start` and `TextAlign::End` are *logical* (Start/End flip under RTL).
   - “Physical left/right” alignment must be explicit (or expressed via a helper).
3. **Recipes match upstream composition**
   - If an upstream shadcn example wraps scroll content in `p-4`, our RTL snippet must do the same.
4. **Direction-sensitive interactions are gated**
   - Drag sign (e.g. carousel) and “start/end placement” (popper) have at least one regression gate.
5. **Direction does not silently disappear across roots**
   - If overlay roots do not inherit provider state, recipes must thread direction explicitly and be
     tested (see `docs/audits/radix-direction.md` “Gaps / intentional differences”).

## Current state (known anchors)

- Direction substrate (Radix-named facade):
  - `ecosystem/fret-ui-kit/src/primitives/direction.rs`
- Placement already consumes `LayoutDirection`:
  - `crates/fret-ui/src/overlay_placement/`
- Text logical alignment (policy default) landed in kit builders:
  - `ecosystem/fret-ui-kit/src/ui.rs`
- Slider / Range direction tests now cover horizontal RTL key mapping and inverted pointer/key
  direction rules:
  - `ecosystem/fret-ui-shadcn/src/slider.rs`
- Pagination direction tests now cover logical previous/next chevrons plus physical text/icon order:
  - `ecosystem/fret-ui-shadcn/src/pagination.rs`
- Tabs direction tests now cover RTL shared-indicator pill paint alignment and trigger spacing:
  - `ecosystem/fret-ui-shadcn/src/tabs.rs`
- ScrollArea RTL snippet updated to match upstream padding expectations:
  - `apps/fret-ui-gallery/src/ui/snippets/scroll_area/rtl.rs`

## Open questions / risks

- Should “logical alignment flips under RTL” live in:
  - (A) authoring defaults (`fret-ui-kit`) only (current), or
  - (B) a deeper text contract (thread `LayoutDirection` into text constraints/measurement),
    which would be a broader mechanism change?
- Provider inheritance across overlay roots:
  - Should this be solved by explicitly threading direction (policy), or by changing the provider
    rooting model (mechanism)?
