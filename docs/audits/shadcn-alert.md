# shadcn/ui v4 Audit - Alert

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- MUI Base UI: https://github.com/mui/base-ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Alert` against the upstream shadcn/ui v4 docs and base
example implementations in `repo-ref/ui`, using Base UI only as a secondary headless check when
deciding whether any missing runtime mechanism work exists.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/alert.mdx`
- Docs page (radix): `repo-ref/ui/apps/v4/content/docs/components/radix/alert.mdx`
- Current visual baseline: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert.tsx`
- Base/radix recipe surface: `repo-ref/ui/apps/v4/registry/bases/radix/ui/alert.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/bases/radix/examples/alert-example.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/alert.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/alert.rs`

## Audit checklist

### Authoring surface

- Pass: `Alert::new([...])` and `Alert::build(...)` cover the expected root composition lane.
- Pass: `AlertAction::build(...)` keeps the top-right action slot on a typed builder-first path,
  while `AlertAction::new([...])` remains valid for already-landed content.
- Pass: `AlertTitle::new(...)` preserves the compact title lane, while
  `AlertTitle::new_children(...)` and `AlertTitle::build(...)` cover attributed or precomposed
  title content.
- Pass: `AlertDescription::new(...)` preserves the plain-text lane, while
  `AlertDescription::new_children(...)` and `AlertDescription::build(...)` cover multi-paragraph or
  composed description content.
- Note: the first Alert audit was too optimistic about the runtime boundary. The later Card/slot
  parity work proved that nearby grid families also depend on runtime alignment surfaces
  (`justify-items`, grid `align-self`, grid `justify-self`). That runtime closure is now exercised
  by Alert as a recipe consumer: the chrome root remains a `Container`, while the source-aligned
  slot geometry now lives in an inner grid instead of the older flex approximation.

### Layout, semantics, and default-style ownership

- Pass: the root stamps `role="alert"` directly on the existing container instead of inserting an
  extra semantics wrapper.
- Pass: recipe-owned defaults align with the current new-york-v4 source for `w-full`, border,
  padding, icon slot spacing, destructive tinting, and absolute action positioning.
- Pass: the content lane now translates the upstream `grid-cols-[0_1fr]` /
  `grid-cols-[calc(var(--spacing)*4)_1fr]` shape directly, including source-aligned `col-start-2`
  placement for content and independent `gap-y-0.5` / conditional `gap-x-3`.
- Pass: `AlertDescription` now mirrors the upstream single-column grid stack (`gap-1`,
  `justify-items-start`) instead of relying on a flex approximation.
- Pass: `AlertAction` remains an explicit recipe extension slot for the base/radix docs surface,
  and its absolute positioning now coexists with the inner grid translation instead of blocking it.
- Pass: caller-owned layout negotiation stays outside the recipe; examples still apply `max-w-*`
  from the page/snippet surface instead of baking width constraints into `Alert`.
- Pass: Base UI still does not reveal any missing hit-testing/focus/dismissal mechanism for this
  family.
- Known gap: the current new-york-v4 `AlertTitle` baseline uses `line-clamp-1`, while the
  base/radix docs examples also demonstrate multiline titles. Fret intentionally keeps the
  new-york-v4 default for chrome parity and treats the multiline-title docs examples as a
  docs-surface divergence rather than a runtime bug.

### Gallery / docs parity

- Pass: the gallery now mirrors the shadcn docs path after `Installation`:
  `Demo`, `Usage`, `Basic`, `Destructive`, `Action`, `Custom Colors`, `RTL`, and `API Reference`.
- Pass: `Demo` now stays on the upstream `alert-demo` three-row composition; richer title,
  description, and interactive-link teaching remain explicit under later `Fret Extras` sections
  instead of drifting into the docs path.
- Pass: `Action` now stays on the upstream two-row `With Actions` example (`Undo` button row plus
  `Badge` row), and the diagnostics/test-id surface uses matching action names.
- Pass: Fret-only follow-ups now stay explicit after `API Reference` under `Fret Extras`, so the
  upstream docs path remains readable while composed-content guidance stays copyable.
- Pass: `Usage` now teaches the builder-first root/slot composition path, and the copyable snippets
  no longer need to hand-land rich title/description children just to keep the intended alert
  authoring surface.
- Pass: the page now exposes stable docs-oriented anchors such as
  `ui-gallery-alert-usage-content`, `ui-gallery-alert-api-reference-content`, and
  `ui-gallery-alert-rich-title-content` for deterministic diagnostics.

## Validation

- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app alert_snippets_prefer_ui_cx_on_the_default_app_surface`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app alert_page_uses_typed_doc_sections_for_app_facing_snippets`
- `cargo test -p fret-ui-shadcn --lib alert::tests::alert_title_build_collects_children_on_builder_path`
- `cargo test -p fret-ui-shadcn --lib alert::tests::alert_description_build_collects_children_on_builder_path`
- `cargo nextest run -p fret-ui-shadcn --lib -E 'test(alert_description_children_scope_inherited_text_style) or test(alert_description_uses_source_aligned_grid_stack) or test(alert_title_truncates_by_default_like_current_shadcn) or test(alert_root_content_grid_tracks_and_gaps_match_current_shadcn_source) or test(alert_with_icon_uses_source_aligned_grid_columns_and_icon_slot) or test(alert_build_collects_children_on_builder_path) or test(alert_with_action_translates_absolute_action_offsets_to_padding_box_in_ltr) or test(alert_with_action_translates_absolute_action_offsets_to_padding_box_in_rtl)'`
- `cargo nextest run -p fret-ui-shadcn --lib -E 'test(alert_with_action_reserves_right_padding_like_shadcn_in_ltr) or test(alert_with_action_reserves_left_padding_like_shadcn_in_rtl) or test(alert_action_explicit_right_override_wins_over_rtl_fallback) or test(alert_action_uses_upstream_offsets_and_merges_layout_refinement) or test(alert_action_uses_logical_end_offsets_in_rtl) or test(alert_action_build_preserves_upstream_offsets) or test(alert_action_build_uses_logical_end_offsets_in_rtl) or test(alert_forces_icon_to_inherit_current_color) or test(alert_attaches_foreground_to_main_content_without_wrapper)'`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout --test web_vs_fret_control_chrome -E 'test(web_vs_fret_layout_alert_demo_alerts_are_w_full_like_web) or test(web_vs_fret_alert_demo_chrome_matches) or test(web_vs_fret_alert_demo_icon_geometry_matches) or test(web_vs_fret_alert_destructive_chrome_matches)'`
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/alert.rs`
- Existing diag gate: `tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-action-text-non-overlap.json`
- New docs smoke gate: `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-docs-smoke.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
