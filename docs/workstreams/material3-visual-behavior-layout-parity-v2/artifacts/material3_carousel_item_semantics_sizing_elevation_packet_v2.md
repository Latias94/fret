# Material3 Carousel Item Semantics, Sizing, And Elevation Packet v2

Date: 2026-05-29

## Truth

- Non-interactive carousel items are surfaces/groups, not disabled buttons.
- Interactive carousel items keep button semantics, enabled/disabled state, focus ring, state layer,
  and bounded ripple behavior.
- Explicit `CarouselItem::width` / `height` constrains both the root automation node and the
  recipe-owned `.chrome` node.
- Interactive carousel item hover elevation animates into Material level1 instead of snapping on
  the first hover frame.

## Sources

- Compose Material3 `Carousel.kt`: carousel container owns pager state, snap/fling behavior,
  keyline sizing, item masking, and `Role.Carousel`; item content is rendered through a scoped box.
- Compose Material3 `CarouselItemScope.kt`: item masking is a carousel-scope helper, not a
  standalone item surface concern.
- Material Web v30 tokens: `md.comp.carousel-item.*` defines item surface shape, outline,
  disabled opacity, state-layer opacities, and hover elevation.
- Fret shadcn Carousel: a mature Fret-side exemplar where carousel container behavior and slide
  semantics live at the carousel recipe layer, while item sizing remains caller/container owned.

MUI Material UI is not available in this worktree's `repo-ref/`; this packet used the local
Compose and generated Material Web token snapshots.

## Layer Finding

This packet found a Material recipe gap, not a core or kit mechanism gap:

- Recipe semantics: `CarouselItem` reused a `Pressable` wrapper for static content, so static
  items inherited default button semantics and disabled control flags.
- Recipe motion wiring: item hover elevation resolved the right token, but painted the target
  elevation immediately instead of going through the shared Material elevation runtime.
- Layout ownership: Compose carousel item sizing and mask motion are container/strategy owned.
  Standalone Fret `CarouselItem` keeps width/height explicit and caller-owned rather than growing a
  partial carousel engine in the item recipe.

The existing `fret-ui` pressable, semantics decoration, frame scheduling, and node-bound inspection
mechanisms were sufficient.

## Artifacts

- `ecosystem/fret-ui-material3/src/carousel_item.rs`
- `ecosystem/fret-ui-material3/tests/carousel_item_state.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test carousel_item_state
```

Failed because static carousel items exposed `Button` semantics, and interactive items painted
hover shadows on the first hover frame.

Green gates:

```powershell
cargo nextest run -p fret-ui-material3 --test carousel_item_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_carousel_item_suite_goldens_v1
```

## Residual Risk

- This packet closes the standalone item recipe, not a full Material carousel container.
- Carousel scrolling, keyline sizing, item masking/parallax, snap/fling behavior, and container
  `Role.Carousel` semantics remain future Material carousel-container work if Fret adds that
  recipe. Until then, shadcn Carousel remains the in-tree carousel behavior engine.
