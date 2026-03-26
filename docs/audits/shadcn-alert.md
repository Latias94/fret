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
- Note: the component did not need new `fret-ui` runtime primitives; the pressure was on the
  recipe/public-surface layer and the docs teaching surface.

### Layout, semantics, and default-style ownership

- Pass: the root stamps `role="alert"` directly on the existing container instead of inserting an
  extra semantics wrapper.
- Pass: recipe-owned defaults align with the current new-york-v4 source for `w-full`, border,
  padding, icon slot spacing, destructive tinting, and absolute action positioning.
- Pass: caller-owned layout negotiation stays outside the recipe; examples still apply `max-w-*`
  from the page/snippet surface instead of baking width constraints into `Alert`.
- Pass: Base UI did not reveal any missing headless/runtime mechanism for this family. The current
  differences are recipe-level chrome or docs-surface choices, not hit-testing/focus/dismissal
  substrate gaps.
- Known gap: the current new-york-v4 `AlertTitle` baseline uses `line-clamp-1`, while the
  base/radix docs examples also demonstrate multiline titles. Fret intentionally keeps the
  new-york-v4 default for chrome parity and treats the multiline-title docs examples as a
  docs-surface divergence rather than a runtime bug.

### Gallery / docs parity

- Pass: the gallery now mirrors the shadcn docs path after `Installation`:
  `Demo`, `Usage`, `Basic`, `Destructive`, `Action`, `Custom Colors`, `RTL`, and `API Reference`.
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
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/alert.rs`
- Existing diag gate: `tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-action-text-non-overlap.json`
- New docs smoke gate: `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-docs-smoke.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
