# UI Direction + RTL Parity (Fearless Refactor v1) — TODO

Tracking doc: `docs/workstreams/ui-direction-and-rtl-fearless-refactor-v1/DESIGN.md`

## A — Lock the minimal contract surface (kit + recipes)

- [ ] Write down the public “direction vocabulary” we expect shadcn recipes to use
  (provider install, local override, `Start/End` vs physical left/right).
- [ ] Add a small “authoring helpers” section (recommended builders / helpers) to avoid repeated
  per-component ad-hoc mappings.

## B — Parity matrix (direction-sensitive components)

Legend:

- Status: `OK` (aligned), `Drift` (known mismatch), `Unknown` (not audited)
- Gates: `Test` (unit test), `Diag` (scripted repro), `None` (no gate yet)

| Component / area | Direction-sensitive behavior | Status | Gates | Evidence / anchor |
| --- | --- | --- | --- | --- |
| Text (kit builders) | `TextAlign::Start/End` logical flip under RTL | OK | Test | `ecosystem/fret-ui-kit/src/ui.rs` |
| ScrollArea (gallery) | RTL padding + content alignment parity vs shadcn docs | OK | Diag | `apps/fret-ui-gallery/src/ui/snippets/scroll_area/rtl.rs` + `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-rtl-screenshot.json` |
| ScrollArea (demo) | Horizontal scroll “blank space” at max scroll | Drift | Diag | `apps/fret-ui-gallery/src/ui/snippets/scroll_area/horizontal.rs` + `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-horizontal-max-scroll-screenshot.json` |
| DropdownMenu (gallery) | Overlay root + `align=start` + RTL placement/parity | OK | Test + Diag | `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/rtl.rs` + `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` + `tools/diag-scripts/ui-gallery/dropdown-menu/ui-gallery-dropdown-menu-rtl-open-screenshot.json` |
| HoverCard (gallery) | Overlay placement parity under RTL | OK | Diag | `apps/fret-ui-gallery/src/ui/snippets/hover_card/rtl.rs` + `tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-rtl-open-screenshot.json` |
| Popper placement | `align=start/end` flips under RTL for vertical placements | OK | Test | `crates/fret-ui/src/overlay_placement/tests.rs` |
| DropdownMenu | `align=start` respects direction provider through overlay root | OK | Test | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` |
| NavigationMenu | viewport `align=start` respects direction provider | OK | Test | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` |
| Carousel | Drag sign + snap selection threshold under RTL | OK | Test | `ecosystem/fret-ui-shadcn/tests/carousel_direction_rtl.rs` + `ecosystem/fret-ui-shadcn/src/carousel.rs` |
| Tabs (gallery) | APG keynav (Left/Right flip under RTL) | OK | Diag | `apps/fret-ui-gallery/src/ui/snippets/tabs/extras.rs` + `tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-rtl-keynav-screenshot.json` |
| Slider / Range | Arrow keys + track fill direction | Unknown | None | — |
| Tabs (recipe) | Visual indicator + spacing parity under RTL | Unknown | None | — |
| Pagination | Chevron semantics + “next/prev” physical ordering | Unknown | None | — |

## C — Gates (add the missing ones)

- [x] Add at least 1 scripted diag gate that exercises direction across an overlay root boundary
  (provider installed in root, overlay created, ensure direction-sensitive behavior is still correct).
- [x] Add at least 1 scripted diag gate for a “direction-sensitive keyboard nav” component (tabs or
  slider), so future refactors can’t silently regress.

## D — Shadcn docs alignment hygiene

- [ ] For every shadcn doc page that includes an RTL example, ensure the gallery snippet:
  - [ ] installs direction the same way,
  - [ ] includes the same padding/wrappers (e.g. `p-4`),
  - [ ] and uses the same “start/end” semantics for alignment.
