# shadcn/ui docs parity (UI Gallery)

## Goal

Make `apps/fret-ui-gallery` behave like a shadcn/ui doc site:

- The **Preview** content mirrors the shadcn/ui v4 docs **Examples** order.
- Gaps are explicitly tracked (so we don’t “accidentally diverge”).
- Behavior fixes are backed by diagnostics (layout dumps / targeted tests) before changing styling tokens.

## Sources of truth

- Docs structure + example ordering: `repo-ref/ui/apps/v4/content/docs/components/base/*.mdx`
- Example content: `repo-ref/ui/apps/v4/examples/base/*.tsx`
- Interaction semantics reference: `repo-ref/primitives` (Radix/Base primitives) when behavior is unclear.

## Working rules

- Prefer **repro → diag → fix**:
  - Repro in UI Gallery first.
  - If layout looks suspicious, dump Taffy: `FRET_TAFFY_DUMP=1` (optionally scope with `FRET_TAFFY_DUMP_ROOT*`).
  - If behavior is contract-level (hover/press/focus), add a small `cargo nextest` regression test.
- Keep “mechanism” fixes in `fret-ui` / `fret-ui-kit` minimal and verified; keep styling/recipes in `fret-ui-shadcn`.

## Progress

Status legend:

- **Aligned**: matches the shadcn docs’ example order and intent.
- **Aligned (with gaps)**: ordering matches; some shadcn-only APIs (e.g. `render/asChild`) not present yet.
- **TODO**: preview exists but doesn’t follow the docs yet (or is a placeholder).

| Page | Status | Notes / gaps | Evidence |
|------|--------|--------------|----------|
| Button | Aligned (with gaps) | Added RTL; “Link (render)” is a TODO because `Button::render/asChild` is not implemented in `fret-ui-shadcn` yet. | `apps/fret-ui-gallery/src/ui.rs` |
| Menubar | Aligned | Added examples: Checkbox, Radio, Submenu, With Icons, RTL. | `apps/fret-ui-gallery/src/ui.rs` |
| Navigation Menu | Aligned (with gaps) | Demo + RTL match docs. Doc-site `render` link composition is not modeled; use `NavigationMenuLink` + commands instead. | `apps/fret-ui-gallery/src/ui.rs` |
| Slider | Aligned (with known gaps) | Core Radix contracts are covered (`vertical`, `dir`, `inverted`, `onValueCommit`, `minStepsBetweenThumbs`); continue parity checks with scripted gallery sweeps to catch visual drift in app-level composition. | `ecosystem/fret-ui-shadcn/src/slider.rs` |
| Avatar | Aligned (with known gaps) | Added retry for transient image registration failures during early runner initialization (`renderer/wgpu not initialized`) to prevent a blank first avatar in UI Gallery. | `apps/fret-ui-gallery/src/driver.rs` |

## Next

Priority order (highest first):

1. **Sidebar**: align examples and verify hover/active chrome across all items (repro + hit-test diagnostics if needed).
2. **Native Select**: upgrade from “smoke stubs” to doc-shaped previews.
3. **Pagination**: mirror docs ordering (and add RTL if applicable).
4. **Gallery sweep gate**: keep a lightweight scripted sweep for Slider + Avatar (`tools/diag-scripts/ui-gallery-slider-and-avatar-screenshots.json`) and run it when semantics/layout wrappers change.
