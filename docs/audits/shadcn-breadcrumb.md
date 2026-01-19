# shadcn/ui v4 Audit — Breadcrumb

This audit compares Fret’s shadcn-aligned `Breadcrumb` against the upstream shadcn/ui v4 docs and
the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/breadcrumb.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/breadcrumb.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`
- Upstream-shaped primitives surface: `fret_ui_shadcn::breadcrumb::primitives::*`

## Audit checklist

### Layout & geometry (shadcn parity)

- Pass: Separator uses lucide `ChevronRight` at `size-3.5` (≈ `14px`), matching `BreadcrumbSeparator`.
- Pass: Custom separator examples (e.g. `breadcrumb-separator`) can be represented via
  `BreadcrumbSeparator::Icon { icon: ids::ui::SLASH, size: 14px }`.
- Pass: Ellipsis uses a `size-9` box (≈ `36x36`) with a centered lucide `MoreHorizontal` icon at `size-4`
  (≈ `16px`), matching `BreadcrumbEllipsis`.
- Note: Upstream uses `gap-1.5` with `sm:gap-2.5`. Our web goldens run at a desktop viewport, so the
  default gap is aligned to the `sm` outcome (`gap-2.5`) for 1:1 geometry conformance.

## Validation

- Web layout gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  - `web_vs_fret_layout_breadcrumb_separator_geometry`
  - `web_vs_fret_layout_breadcrumb_ellipsis_geometry`
  - `web_vs_fret_layout_breadcrumb_dropdown_trigger_geometry`
  - `web_vs_fret_layout_breadcrumb_demo_toggle_trigger_geometry`
