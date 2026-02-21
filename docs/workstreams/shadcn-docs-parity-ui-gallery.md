# shadcn/ui docs parity (UI Gallery)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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
| Accordion | Aligned (with extras) | Demo matches upstream `accordion-demo.tsx`; retains Fret-specific variants (multiple/disabled/borders/card/RTL) under **Extras**. | `apps/fret-ui-gallery/src/ui/previews/gallery/nav/accordion.rs` |
| Button | Aligned (with gaps) | Added RTL; “Link (render)” is a TODO because `Button::render/asChild` is not implemented in `fret-ui-shadcn` yet. | `apps/fret-ui-gallery/src/ui/pages/button.rs` |
| Button Group | Aligned | All sections use `DocSection::code` so Preview and Code stay coupled. | `apps/fret-ui-gallery/src/ui/previews/pages/components/basics/button_group.rs` |
| Form | Aligned (with gaps) | Upstream `FormDemo` mirrored via `FormState` + `FormRegistry`; `Textarea` placeholder is tracked as a gap. | `apps/fret-ui-gallery/src/ui/pages/form.rs` |
| Menubar | Aligned | Checkbox/Radio/Submenu/Icons/RTL examples mirror upstream intent. | `apps/fret-ui-gallery/src/ui/pages/menubar.rs` |
| Navigation Menu | Aligned (with gaps) | Demo + RTL match docs; doc-site `render` link composition is not modeled; use `NavigationMenuLink` + commands instead. | `apps/fret-ui-gallery/src/ui/pages/navigation_menu.rs` |
| Resizable | Aligned (with extras) | Matches upstream `resizable-demo.tsx` order; includes RTL section for direction-sensitive hit-testing. | `apps/fret-ui-gallery/src/ui/previews/gallery/forms/resizable.rs` |
| Slider | Aligned (with known gaps) | Core Radix contracts are covered (`vertical`, `dir`, `inverted`, `onValueCommit`, `minStepsBetweenThumbs`). | `apps/fret-ui-gallery/src/ui/pages/slider.rs` |
| Sonner | Aligned (with extras) | Demo mirrors upstream “type buttons”; extra sections cover global position + pinned/swipe-dismiss toasts. | `apps/fret-ui-gallery/src/ui/previews/gallery/overlays/sonner.rs` |
| Spinner | Aligned (with extras) | Sections mirror upstream structure; RTL + Extras include code tabs for parity review. | `apps/fret-ui-gallery/src/ui/previews/pages/components/basics/spinner.rs` |
| Table | Aligned (with extras) | Demo/footer/actions/RTL each include code tabs; actions column uses `DropdownMenu`. | `apps/fret-ui-gallery/src/ui/previews/gallery/table.rs` |
| Tabs | Aligned (with gaps) | Matches upstream `tabs-demo.tsx` ordering; password inputs are plain text (no masked input yet). | `apps/fret-ui-gallery/src/ui/previews/gallery/nav/tabs.rs` |
| Avatar | Aligned (with known gaps) | Demo order matches upstream; badge/group-count are tracked as explicit gap cards. | `apps/fret-ui-gallery/src/ui/pages/avatar.rs` |

## Next

Priority order (highest first):

1. **Charts**: upstream splits `chart-area`/`chart-bar`/`chart-line`; decide whether to add dedicated pages or keep a single `Chart` page with clearly labeled sections.
2. **Command + Form naming parity**: upstream uses `CommandDemo`/`FormDemo`; UI Gallery currently exposes `Command Palette` + `Forms` pages—consider adding lightweight aliases so “docs parity” scans are 1:1.
3. **Code coverage sweep**: keep “Preview ↔ Code” coupling strong by ensuring every non-Notes section uses `DocSection::code` (or is an explicit gap card).
