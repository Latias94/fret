# Carousel Embla parity (v2) — API + events (workstream design)

Status: Draft

This document captures a **workstream-level** design for a Rust-native `CarouselApi` surface and
its observable event semantics (`select` / `reInit`) while we are still iterating inside
`ecosystem/*`. If/when this surface becomes stable and must be treated as a long-lived contract, we
can promote the key parts into an ADR.

## Why not an ADR (yet)

- The v2 work is still evolving (API shape and ergonomics are not stable).
- The implementation lives in `ecosystem/*` and is expected to iterate quickly.
- We already have regression protection via unit tests + `fretboard diag` scripted gates.

## Layering (non-negotiable)

- `crates/fret-ui`: mechanisms only (routing, capture/cancel semantics, hit-testing).
- `ecosystem/fret-ui-headless`: engine math + deterministic state machines (Embla-aligned helpers).
- `ecosystem/fret-ui-shadcn`: recipes + tokens + ergonomics + docs-aligned demos.

No component policy or physics should move into `crates/fret-ui`.

## Upstream reference outcomes (shadcn/ui v4)

Shadcn’s `Carousel` uses Embla and relies on:

- `setApi(api)` to obtain a handle
- `api.on('select', ...)` to update the “Slide N of M” counter
- `api.on('reInit', ...)` to update `canScrollPrev/Next` (and counters) after geometry changes

Local snapshots:

- Docs: `repo-ref/ui/apps/v4/content/docs/components/carousel.mdx`
- Component: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/carousel.tsx`

## Current in-tree baseline

### Snapshot surface (shipping today)

`ecosystem/fret-ui-shadcn::CarouselApiSnapshot` is a **deterministic** snapshot intended for UI
gallery demos and basic “API” examples without exposing Embla’s imperative API.

It now includes MVP event observability:

- `select_generation`: increments when the selected index changes
- `reinit_generation`: increments when the carousel re-initializes due to geometry changes

This provides an “effect-like” hook in Rust: render code can remember the last generation and react
when it changes.

### Internal engine re-init (shipping today)

The recipe rebuilds the headless engine derived state when measured geometry changes:

- `Engine::reinit(...)` recomputes limit/targets/bounds while clamping `location/target` to the new
  limit.
- The recipe triggers re-init when snaps/view size/max offset changes.

## Goals (v2)

1) Provide a Rust-native `CarouselApi` handle with ergonomic methods:
   - `scroll_prev`, `scroll_next`, `scroll_to(index)`
   - `selected_index`, `snap_count`, `can_scroll_prev`, `can_scroll_next`
2) Provide **observable** and **stable** event semantics:
   - `select`
   - `re_init`
3) Avoid storing arbitrary closures inside models.
4) Keep layering intact (no policy creep into `crates/fret-ui`).

## Non-goals (v2)

- Perfect 1:1 mirroring of Embla’s JS API types.
- Requiring React-style effect lifetimes or callback patterns.

## Event semantics (what “correct” means)

### `select`

Contract:

- A `select` event is emitted when `selected_index` changes.
- It should be “exactly once” per index transition (no duplicates per frame).

MVP implementation approach:

- Increment `select_generation` when `selected_index` changes (after clamping / snap selection).

### `reInit`

Contract:

- A `reInit` event is emitted when geometry changes cause a re-initialization that can affect:
  - snap list
  - scroll limits / bounds config
  - view size
- It must be safe to emit multiple times during continuous resize.

MVP implementation approach:

- Increment `reinit_generation` when the recipe observes a meaningful geometry delta
  (snaps/maxOffset/viewSize changed and view size is measurable).

Ordering:

- If a re-init causes a selected index change, a `select` event must also occur.
- The exact ordering between `reInit` and `select` is not required to match Embla perfectly, but it
  must be stable and documented.

## Resize throttling (open question)

Embla can emit `reInit` during resize; in Rust/native we may want stronger guarantees to reduce
noise:

- Option A: emit `reInit` for every geometry change (simple, possibly noisy).
- Option B: coalesce resize-triggered re-init events into “at most once per N frames”.
- Option C: coalesce into “once after stable frames” (debounce).

Decision should be driven by:

- UI gallery demo stability
- diagnostics bundle size/variance
- real app responsiveness during interactive resize

## Proposed evolution path

### Phase 1 (now): generation counters + gates

- Treat `CarouselApiSnapshot` generations as the MVP “event system”.
- Update UI gallery API demo to optionally use generations (no API handle yet).
- Add/keep diag gates:
  - inertia pixels changed
  - resize during engine-driven motion does not panic and content stays visible

### Phase 2: `CarouselApi` handle

Add a small handle that:

- reads from models (snapshot) for queries
- writes via actions/effects for commands
- exposes a subscription mechanism (likely still generation-based, or an explicit queue)

### Phase 3: stabilize and consider ADR

When:

- other crates/components depend on this surface,
- and we want the surface to be hard-to-change,

then promote the key parts to an ADR.

## Gates (must remain executable)

- Unit tests (recommended):
  - `select_generation` increments exactly once per index change.
  - `reinit_generation` increments on snap/viewport changes.
- Diag scripts:
  - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-inertia-pixels-changed.json`
  - `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-reinit-resize-gate.json`

