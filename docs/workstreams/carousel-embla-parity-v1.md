# Carousel Embla parity (v1)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- `embla-carousel`: https://github.com/search?q=embla-carousel&type=repositories
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
## Goal

Align Fret’s shadcn-style `Carousel` behavior and API surface with Embla Carousel semantics (the reference implementation used by shadcn/ui),
while keeping Fret’s layering contract intact:

- `crates/fret-ui`: mechanisms (event routing, capture, cancel semantics, hit-testing contracts)
- `ecosystem/fret-ui-kit`: headless interaction policy/state machines (optional for v1, recommended for v2)
- `ecosystem/fret-ui-shadcn`: composition + tokens + shadcn taxonomy + recipes

This workstream is **not** “port Embla line-by-line”. It is “port the *capabilities* and the *interaction semantics*” to a custom-rendered,
GPU-first UI runtime.

## Scope (v1)

### In scope

- Drag semantics parity:
  - `dragThreshold`-style arming threshold (Embla default is `10px`).
  - Dragging starting on interactive descendants (buttons/inputs) should be supported without accidental activation.
  - Parent drag winning a gesture should cancel the descendant press/gesture in a deterministic way.
- Pointer-event contract closure for drag arbitration:
  - Observe pointer moves in capture phase (opt-in) to decide whether to steal capture.
  - When capture switches, previous capture target receives `PointerCancel`.
- Docs-aligned UI gallery presentation for carousel (shadcn docs layout and examples).
- Regression gates:
  - Unit/integration tests for “drag from descendant pressable suppresses activation”.
  - A `fretboard diag` script that exercises the same scenario on UI gallery.

### Out of scope (v1)

- Embla plugin system (autoplay, etc).
- Full Embla API parity (`setApi`, events, `slidesInView`, etc).
- Advanced physics/scroll-body modeling, momentum, overscroll.
- Looping (`loop: true`) and contain-scroll variants (`trimSnaps`/`keepSnaps`) beyond a minimal snap behavior.
- Touch-specific scroll-prevention heuristics (vs cross-axis diff) beyond what Fret already supports.

## References (source of truth)

### Upstream / reference code

- Embla options defaults, including `dragThreshold=10`:
  - `repo-ref/embla-carousel/packages/embla-carousel/src/components/Options.ts`
- Embla click-prevention logic (conceptually “cancel descendant click when drag wins”):
  - `repo-ref/embla-carousel/packages/embla-carousel/src/components/DragHandler.ts`
- shadcn/ui composition:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/carousel.tsx`

### In-tree implementation + gates

- shadcn carousel implementation:
  - `ecosystem/fret-ui-shadcn/src/carousel.rs`
- UI gallery page:
  - `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- Test: drag from inner pressable does not activate:
  - `ecosystem/fret-ui-shadcn/tests/carousel_pointer_passthrough.rs`
- Diag script gate:
  - `tools/diag-scripts/ui-gallery-carousel-demo-swipe-and-buttons.json`

### Mechanism prerequisites (Fret)

This workstream assumes the following mechanism contracts exist (or are introduced as part of the work):

- Pointer regions can opt in to receiving capture-phase pointer moves.
- When pointer capture switches, the previous capture target receives a `PointerCancel` event so pressed/drag state is cleared.

## Design notes (how to translate Embla to a custom renderer)

### Embla’s mental model

Embla is built on DOM events and uses:

- An arming threshold (`dragThreshold`) before classifying the gesture as a drag.
- Once classified as a drag, it prevents clicks by capturing/canceling behavior (“preventClick”).
- It attaches handlers at the root, and relies on a browser event model where capture and bubbling are first-class.

### Fret translation (mechanism-first)

To reproduce Embla semantics in a custom renderer, we need:

1. **Gesture arming**: do not steal capture on `PointerDown`. Start “armed”, record start position and start offset.
2. **Capture-phase observation**: observe pointer `Move` *even when a descendant currently holds capture*.
3. **Steal capture on threshold**: when delta exceeds `CAROUSEL_DRAG_START_PX`, the carousel steals pointer capture.
4. **Cancel previous target**: switching capture triggers `PointerCancel` for the previously captured element, suppressing activation.

This mirrors common UI frameworks:

- “Gesture arena” (parent/child compete until one wins).
- “Routed events” capture/bubble with explicit cancel semantics.

## Definition of done (v1)

- Dragging from an interactive descendant (e.g. an inner button) does not activate the descendant when the carousel starts dragging.
- Clicking the inner button without crossing the drag threshold still activates it.
- UI gallery demo contains a stable `test_id` surface to exercise the behavior.
- At least one deterministic Rust test + one diag script gate exist and are green locally.

## Next steps (v2+ preview)

If we want closer Embla parity without polluting `fret-ui-shadcn` with physics/math details:

- Move scroll-body, snap selection, and edge constraints to a headless engine in `ecosystem/fret-ui-kit` (or `fret-ui-headless` if it fits).
- Keep `fret-ui-shadcn` as pure composition + styling, and treat the engine output as input (offset/index/events).

