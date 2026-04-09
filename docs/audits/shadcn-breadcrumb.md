# shadcn/ui v4 Audit - Breadcrumb


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Breadcrumb` against the upstream shadcn/ui v4 docs and
the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

There is no standalone `components/breadcrumb.mdx` page in the current v4 repo snapshot. Use
these sources instead:

- Base docs: `repo-ref/ui/apps/v4/content/docs/components/base/breadcrumb.mdx`
- Radix docs: `repo-ref/ui/apps/v4/content/docs/components/radix/breadcrumb.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/breadcrumb.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`
- Curated facade lane: `fret_ui_shadcn::facade::{BreadcrumbRoot, BreadcrumbItemPart, BreadcrumbSeparatorPart, BreadcrumbList, BreadcrumbLink, BreadcrumbPage, BreadcrumbEllipsis}`
- Upstream-shaped primitives surface: `fret_ui_shadcn::raw::breadcrumb::primitives::*`

## Audit checklist

### Authoring surface

- Pass: `Breadcrumb::new().items([...])` covers the common compact authoring path for standard breadcrumb trails.
- Pass: `BreadcrumbItem::new(...).href(...)`, non-link `BreadcrumbItem::new(...)` current-page text, and `BreadcrumbItem::ellipsis()` cover the important shadcn recipe outcomes.
- Pass: Upstream-shaped primitives remain available for cases that need more explicit composition.
- Pass: Gallery docs-path examples (`Usage`, `Basic`, `Custom Separator`, `Collapsed`) now stay on the curated facade aliases, while advanced overlay/router/responsive examples reopen raw breadcrumb primitives only when they actually need the low-level seam.
- Pass: `breadcrumb::primitives::BreadcrumbLink::children(...)` and `BreadcrumbPage::children(...)` now provide the narrow Rust-native equivalent of upstream arbitrary React children without introducing a generic `Slot` / `asChild` merge surface.
- Pass: Gallery `Usage` now prefers the curated facade aliases for the upstream-shaped composition lane, while raw primitives remain the explicit escape hatch for source-alignment-heavy examples and the compact builder remains an ergonomic Fret shorthand.
- Note: Gallery live previews keep `href("/")` examples deterministic by pairing them with a gallery command (`ui_gallery.app.open`) instead of falling through to `Effect::OpenUrl` during local/docs automation.
- Note: Because Fret already offers both the compact builder and primitives surface, it intentionally does not add a separate generic `compose()` builder here.

### Layout & geometry (shadcn parity)

- Pass: Separator uses lucide `ChevronRight` at `size-3.5` (about `14px`), matching `BreadcrumbSeparator`.
- Pass: Custom separator examples (e.g. `breadcrumb-separator`) can be represented via
  `BreadcrumbSeparator::Icon { icon: ids::ui::SLASH, size: 14px }`.
- Pass: Ellipsis uses a `size-9` box (about `36x36`) with a centered lucide `MoreHorizontal` icon at `size-4`
  (about `16px`), matching `BreadcrumbEllipsis`.
- Pass: Link items can attach `SemanticsRole::Link` + `value=href` when configured, and can fall back to
  `Effect::OpenUrl` when no explicit `on_activate` hook is provided.
- Pass: Root/list/item semantics now better approximate upstream `nav/ol/li` structure: `Breadcrumb`
  emits `Region + label("breadcrumb")`, `BreadcrumbList` emits `List`, and primitives `BreadcrumbItem`
  emits `ListItem` without changing layout defaults.
- Pass: Current-page and purely visual affordances now expose more accurate semantics outcomes:
  `BreadcrumbPage` approximates a disabled current-page link, while separators/ellipsis are hidden from
  the semantics tree like upstream `role="presentation" aria-hidden`.
- Pass: Responsive truncation (`max-w-20 truncate md:max-w-none`) is representable via
  `BreadcrumbLink::truncate(true)` / `BreadcrumbPage::truncate(true)` combined with a `max_w` layout
  refinement (tests gate the mobile outcome).
- Note: Upstream uses `gap-1.5` with `sm:gap-2.5`. Our web goldens run at a desktop viewport, so the
  default gap is aligned to the `sm` outcome (`gap-2.5`) for 1:1 geometry conformance.

## Validation

- Focused native correctness gate: `cargo test -p fret-ui-shadcn --lib breadcrumb_ -- --nocapture`
  - Covers current-page semantics, list/item semantics, hover tweening, `href` link semantics, and the `BreadcrumbLink::children(...) + on_click(...)` no-`OpenUrl` fallback invariant.

- Gallery authoring-surface gate: `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies breadcrumb`
  - Verifies the copyable `Usage` snippet stays on the curated facade lane while advanced docs-parity examples keep the explicit raw breadcrumb escape hatch.

- Gallery live-preview gate: `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-usage-home-command.json --launch -- cargo run -p fret-ui-gallery --release`
  - Verifies the docs `Usage` section keeps `Home` as a semantic link while routing activation through `ui_gallery.app.open` (`ui-gallery-status-last-action` contains `cmd.open`) instead of opening `/` in the system browser.

- Web layout gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  - `web_vs_fret_layout_breadcrumb_separator_geometry`
  - `web_vs_fret_layout_breadcrumb_ellipsis_geometry`
  - `web_vs_fret_layout_breadcrumb_dropdown_trigger_geometry`
  - `web_vs_fret_layout_breadcrumb_demo_toggle_trigger_geometry`
  - `web_vs_fret_layout_breadcrumb_responsive_mobile_truncation_geometry`

- Web overlay placement gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  - `web_vs_fret_breadcrumb_demo_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_demo_small_viewport_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_dropdown_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_dropdown_small_viewport_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_responsive_overlay_placement_matches`
  - `web_vs_fret_breadcrumb_responsive_mobile_drawer_overlay_insets_match`

- Web overlay chrome gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
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

## Conclusion

- Result: Default-style ownership already matched upstream: wrap/gap/truncation lived in the right layer.
- Result: The actionable drift was semantics parity (`nav/ol/li`, current page, presentation-only affordances), not layout policy.
- Result: A future dedicated `Navigation` landmark role in `fret-core` would allow an even tighter match than today's `Region + label("breadcrumb")` approximation.
