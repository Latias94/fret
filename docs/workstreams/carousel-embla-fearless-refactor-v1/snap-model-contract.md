# Snap Model Contract (v1)

This workstream locks the deterministic, headless snap model used by the shadcn-style Carousel
recipe. The intent is to match Embla's snap/contain outcomes while keeping all policy in ecosystem
crates.

Source of truth:

- Implementation: `ecosystem/fret-ui-headless/src/carousel.rs` (`snap_model_1d`)
- Regression tests: `ecosystem/fret-ui-headless/src/carousel.rs` (`mod tests`)
- Upstream references:
  - Embla core: `repo-ref/embla-carousel/packages/embla-carousel/src/components/*`
  - shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/carousel.mdx`

## Coordinate conventions

- All values are in the carousel's **main axis** (X for horizontal, Y for vertical).
- `CarouselSlide1D.start` is measured from the viewport start edge when the track offset is `0`.
- A snap offset `snaps_px[k]` is a **positive** number that should be applied as:
  - `transform: translate(-snaps_px[k])` (i.e. increasing offset moves content left/up).
- `0` means "unshifted track".

## Inputs

### Slide geometry

- `slides`: ordered list of slide rects, in the same coordinate space and axis.
- `start_gap` / `end_gap`: additional leading/trailing whitespace (e.g. margins). These are used
  by `slidesToScroll=auto` grouping and by the `max_offset_px` computation.

### Grouping (`slidesToScroll`)

- `Fixed(n)`: groups slides in chunks of size `n` (last group may be smaller).
- `Auto`: groups as many slides as fit in `view_size`, using `pixel_tolerance_px` as the threshold.
  This is the deterministic analogue of Embla's "slides to scroll" auto grouping.

### Alignment (`align`)

Alignment is measured per group using the group size:

- `Start`: align to viewport start.
- `Center`: center within the viewport.
- `End`: align to viewport end.
- `Custom`: callback for bespoke alignment.

### Containment (`containScroll`)

Containment is applied only when `!loop_enabled` (v1 does not implement loop behavior).

- `None`: no containment. Snap offsets may be outside `[0, max_offset_px]`.
- `KeepSnaps`: clamp snaps to `[0, max_offset_px]`, preserving snap count (duplicates allowed).
- `TrimSnaps`: clamp snaps, then trim duplicates at the ends. Edge slide groups are expanded so that
  every slide maps to a valid snap.

### Pixel tolerance

`pixel_tolerance_px` participates in:

- Fit short-circuit: if `content_size <= view_size + pixel_tolerance_px`, returns a single snap
  (`[0]`) and groups all slides under it.
- Auto grouping threshold.
- Containment edge snapping: values within `~1px` of the edges may snap to exactly `0` or `max`.

## Outputs / invariants

- `snaps_px` is never empty (empty slide input returns `[0]`).
- `max_offset_px = max(content_size - view_size, 0)`, where `content_size` includes `end_gap`.
- When containment is enabled (`containScroll != None` and `!loop_enabled`):
  - `snaps_px[0] == 0` and the last snap is `max_offset_px`.
  - middle snaps are clamped and rounded to 3 decimals for determinism.
- `slides_by_snap.len() == snaps_px.len()` after trimming/adjustment.
- `snap_by_slide[i]` is always a valid index into `snaps_px`.

## Known gaps (v1)

- Looping behavior (`loop_enabled=true`) is not implemented beyond disabling containment.
- Start-gap inference from layout is recipe-specific; the snap model expects explicit gaps if needed.

