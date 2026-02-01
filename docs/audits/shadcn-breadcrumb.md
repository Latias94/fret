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
- Pass: Responsive truncation (`max-w-20 truncate md:max-w-none`) is representable via
  `BreadcrumbLink::truncate(true)` / `BreadcrumbPage::truncate(true)` combined with a `max_w` layout
  refinement (tests gate the mobile outcome).
- Note: Upstream uses `gap-1.5` with `sm:gap-2.5`. Our web goldens run at a desktop viewport, so the
  default gap is aligned to the `sm` outcome (`gap-2.5`) for 1:1 geometry conformance.

## Validation

- Web layout gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  - `web_vs_fret_layout_breadcrumb_separator_geometry`
  - `web_vs_fret_layout_breadcrumb_ellipsis_geometry`
  - `web_vs_fret_layout_breadcrumb_dropdown_trigger_geometry`
  - `web_vs_fret_layout_breadcrumb_demo_toggle_trigger_geometry`
  - `web_vs_fret_layout_breadcrumb_responsive_mobile_truncation_geometry`

- Web overlay placement gates (menu panel size + clamping): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  - `web_vs_fret_breadcrumb_demo_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_demo_small_viewport_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_dropdown_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_dropdown_small_viewport_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_responsive_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_responsive_mobile_drawer_overlay_insets_match`

- Web overlay chrome gates (menu panel border/background/shadow): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  - `web_vs_fret_breadcrumb_dropdown_surface_colors_match_web`
  - `web_vs_fret_breadcrumb_dropdown_surface_colors_match_web_dark`
  - `web_vs_fret_breadcrumb_dropdown_shadow_matches_web`
  - `web_vs_fret_breadcrumb_dropdown_shadow_matches_web_dark`
  - `web_vs_fret_breadcrumb_dropdown_small_viewport_surface_colors_match_web`
  - `web_vs_fret_breadcrumb_dropdown_small_viewport_surface_colors_match_web_dark`
  - `web_vs_fret_breadcrumb_dropdown_small_viewport_shadow_matches_web`
  - `web_vs_fret_breadcrumb_dropdown_small_viewport_shadow_matches_web_dark`
  - `web_vs_fret_breadcrumb_demo_surface_colors_match_web`
  - `web_vs_fret_breadcrumb_demo_surface_colors_match_web_dark`
  - `web_vs_fret_breadcrumb_demo_shadow_matches_web`
  - `web_vs_fret_breadcrumb_demo_shadow_matches_web_dark`
  - `web_vs_fret_breadcrumb_demo_small_viewport_surface_colors_match_web`
  - `web_vs_fret_breadcrumb_demo_small_viewport_surface_colors_match_web_dark`
  - `web_vs_fret_breadcrumb_demo_small_viewport_shadow_matches_web`
  - `web_vs_fret_breadcrumb_demo_small_viewport_shadow_matches_web_dark`
