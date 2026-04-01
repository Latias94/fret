# Shadow Surface (Fearless Refactor v1) — Design

Status: Complete (v1 closure landed; future upgrades need a new workstream or ADR)

Last updated: 2026-04-01

## Context

Fret currently has two shadow stories:

- a portable component-level `ShadowStyle` / `paint_shadow(...)` path used by recipes and generic
  container chrome,
- a bounded blur-based `DropShadowV1` effect path implemented in `fret-render-wgpu`.

Both paths are valid historical steps, but the current shipped shadow surface has drifted in ways
that are now visible in shadcn parity work:

1. Preset drift
   - `shadow-sm` used by shadcn `Card` is not numerically aligned with the current upstream
     `new-york-v4` source and web golden output.
2. Theme drift
   - shadcn theme seeding currently provides radius/ring/component sizing defaults, but not the
     `component.shadow.*` values needed to make shadow presets explicit and reviewable.
3. Gate drift
   - card/control chrome tests currently prove border/radius/focus outcomes more strongly than
     shadow footprint/softness outcomes.
4. Contract drift
   - ADR 0060 locked a no-blur portable baseline before `DropShadowV1` existed.
   - `DropShadowV1` now exists and is production-ready in the renderer, but the relationship
     between the two shadow surfaces is not yet documented as a current source-of-truth decision.

Because the repo is still pre-open-source, we can refactor aggressively toward the correct shape.
The point of this workstream is to make that refactor boring, staged, and evidence-backed rather
than ad hoc.

## Source of Truth

### Local references

- `docs/adr/0060-shadows-and-elevation.md`
- `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1.md`
- `docs/runtime-contract-matrix.md`
- `docs/tailwind-semantics-alignment.md`
- `docs/audits/shadcn-card.md`
- `docs/audits/shadcn-new-york-v4-alignment.md`
- `goldens/shadcn-web/v4/new-york-v4/card-demo.json`
- `repo-ref/ui/apps/v4/registry/new-york-v4/ui/card.tsx`
- `ecosystem/fret-ui-kit/src/declarative/style.rs`
- `ecosystem/fret-ui-shadcn/src/card.rs`
- `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`
- `crates/fret-ui/src/paint.rs`
- `crates/fret-core/src/scene/mod.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs`

### Related workstreams

- `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1.md`
- `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/shadcn-source-alignment-v1/TODO.md`

## Problem Statement

### What is wrong today

The main issue is not that Fret lacks any shadow system. The issue is that the active shadow
surfaces do not currently form one coherent, source-aligned story.

Concretely:

1. The recipe-level shadcn/Tailwind presets are partly hard-coded and partly inferred.
   - This makes `shadow-xs/sm/md/lg/xl` easy to drift away from upstream values.
2. The portable component-level painter still approximates softness with layered quads.
   - That was the right baseline when blur effects were not generally available.
   - It is now an explicit quality tradeoff, not an invisible implementation detail.
3. The blur-based renderer path exists, but component-level recipes do not yet have a documented
   ownership rule for when to stay on `ShadowStyle` versus when to move to `DropShadowV1`.
4. Some audit docs still read as if shadow parity is already closed, even where no shadow-specific
   gate exists.

### Why this matters

Without a single reviewable shadow plan:

- shadcn parity work keeps rediscovering the same drift,
- theme authors cannot tune elevation explicitly,
- deleting old code is risky because we have not first classified which parts are:
  - incorrect,
  - portable but low fidelity,
  - or still required by non-shadcn consumers.

That is exactly the kind of ambiguity that turns a safe pre-open-source cleanup into churn.

## Goals

1. Make shadow preset values explicit and source-aligned.
   - `shadow-xs/sm/md/lg/xl` should match the chosen upstream source of truth where those presets
     are used by first-party shadcn recipes.
2. Add real regression gates for shadow footprint/chrome parity on non-overlay components.
   - Card is the first target.
3. Clarify the long-term ownership split between:
   - portable recipe shadows (`ShadowStyle`),
   - bounded renderer blur shadows (`DropShadowV1`).
4. Delete or rewrite drift-prone code only after it is classified and gated.
5. Leave docs in a state where future refactors do not have to re-litigate the same shadow story.

## Non-goals

1. A general-purpose lighting system.
2. Arbitrary CSS `filter: drop-shadow(...)` parity with all browser semantics.
3. Inner shadows or arbitrary path-shadow parity in this v1 workstream.
4. Blindly deleting `ShadowStyle` just because `DropShadowV1` exists.
5. Moving shadcn interaction policy into `crates/fret-ui`.

## Invariants

1. Mechanism vs policy ownership remains intact.
   - `crates/*` own mechanism surfaces and renderer contracts.
   - `ecosystem/*` owns recipe-level preset mapping and shadcn parity.

2. Source alignment is evidence-first.
   - When a shadcn recipe says `shadow-sm`, the chosen source of truth is the actual upstream web
     source/golden outcome, not local memory or "close enough" numbers.

3. Portable fallback remains explicit until replaced on purpose.
   - If `ShadowStyle` remains the portable baseline, docs must say so.
   - If it is superseded in some lanes by `DropShadowV1`, degradation and backend coverage must be
     explicit.

4. Pre-open-source status permits deletion of wrong surfaces.
   - We should remove stale helpers, stale docs, and dead mappings once gates prove the new shape.
   - We should not preserve wrong code for compatibility theater.

5. Every shadow claim should have one of:
   - a web golden gate,
   - a focused conformance test,
   - or an explicit "not parity-gated yet" note.

## Locked v1 Decisions

1. Card shadow drift is a real closure item, not a subjective polish complaint.
2. `shadow-sm` alignment must be fixed before claiming shadcn card chrome parity again.
3. New shadcn theme seeding may add `component.shadow.*` tokens instead of relying purely on
   hidden hard-coded fallback values.
4. The workstream may delete drift-prone preset helpers or stale docs once:
   - replacement behavior is landed,
   - a gate exists,
   - and non-shadcn consumers have been audited.
5. A contract change to `ShadowStyle` itself requires an ADR update or superseding ADR.

## Target v1 Architecture

### 1. Explicit preset layer

Recipe-level shadow presets should be explicit, token-driven, and auditable:

- `ecosystem/fret-ui-kit/src/declarative/style.rs` remains the preset mapping surface,
- shadcn theme seeding provides the actual `component.shadow.*` numbers when the style baseline is
  `new-york-v4`,
- fallback literals remain only as a reviewed last resort.

### 2. Clear dual-surface story

Status note (2026-04-01): v1 now explicitly chooses the coexistence posture.

- `ShadowStyle` remains the portable component baseline for theme-token-driven box/container chrome.
- `DropShadowV1` remains the effect-backed blur path for surfaces that already own explicit effect
  bounds/intermediates and want content-derived blur.
- Current effect-backed proof surfaces are editor/canvas-style consumers (for example node static
  node shadow and wire glow), not generic shadcn surface presets.
- No renderer-side implicit promotion from `ShadowStyle` to `DropShadowV1` is part of v1.
- A future upgrade posture remains possible, but it now requires a new contract update plus
  deterministic degradation, conformance, and perf evidence before replacing recipe presets.

### 3. Gate-first cleanup

Status note (2026-04-01): the post-gate cleanup audit now classifies the remaining manual shadow
sites as intentional product-owned surfaces, animation helpers, design-system-specific mappings, or
an intentional shared fallback. The generic toast fallback shadow in
`ecosystem/fret-ui-kit/src/window_overlays/render.rs` is no longer an open deletion candidate:
dedicated Sonner light/dark gates now prove it against the checked-in web baseline, so v1 retains
it intentionally as the generic toast chrome baseline.

Before deleting old code:

- land a card shadow gate,
- land at least one additional non-overlay control-chrome shadow gate,
- then classify delete-ready surfaces and stale docs.

## Fearless Refactor Posture

This workstream is intentionally more willing to delete than a public compatibility cycle would be.
That does not mean "delete everything old." It means:

- delete code that is demonstrably wrong,
- delete documentation that claims parity we do not currently prove,
- delete helper layers that only preserve drift,
- but keep portable mechanisms until a better ownership split is proven and documented.

The standard for deletion is therefore:

1. classify the surface,
2. add or reuse a gate,
3. land the replacement,
4. delete the stale layer,
5. update evidence anchors.

## Deliverables

Minimum deliverables for v1:

- `DESIGN.md`, `TODO.md`, `MILESTONES.md`
- at least one card shadow parity gate
- a consumer inventory for `ShadowStyle` vs `DropShadowV1`
- shadow token seeding or an explicit decision not to seed it
- updated shadow-related audit/docs status notes where parity claims were stale
