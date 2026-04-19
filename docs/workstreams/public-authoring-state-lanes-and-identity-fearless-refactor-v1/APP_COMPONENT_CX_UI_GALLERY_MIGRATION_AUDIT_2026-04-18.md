# AppComponentCx UI Gallery Migration Audit — 2026-04-18

## Scope

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `apps/fret-ui-gallery/src/ui/**`
- `apps/fret-ui-gallery/tests/**`

## Outcome

- `fret` now exposes `AppComponentCx<'a>` as the explicit app-hosted component/snippet helper
  alias.
- `UiCx<'a>` remains only as the compatibility old-name alias and now points to
  `AppComponentCx<'a>`.
- First-party UI Gallery snippet/page/internal-preview surfaces now use `AppComponentCx<'a>`
  instead of teaching `UiCx<'a>` on the shipped exemplar surface.
- The remaining `AppComponentCx` tail is no longer first-party gallery authoring; it is now concentrated in
  explicit compatibility / advanced/reference surfaces outside the gallery.

## Evidence anchors

- Facade alias split:
  `ecosystem/fret/src/lib.rs`
- Facade source-policy gate:
  `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- Gallery scaffold + page/snippet/internal-preview usage:
  `apps/fret-ui-gallery/src/ui/doc_layout.rs`
- Gallery source-policy gates:
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
  `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs`
  `apps/fret-ui-gallery/tests/support/mod.rs`

## Gates

- `cargo nextest run -p fret --test render_authoring_capability_surface --test advanced_prelude_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --test ui_authoring_surface_internal_previews --test ui_snippets_deny_gallery_internal_imports`
- `rg -n "\\bUiCx\\b" apps/fret-ui-gallery/src apps/fret-ui-gallery/tests`

## Remaining follow-on

- Audit and classify the remaining explicit-import `AppComponentCx` tail in:
  - `apps/fret-examples` advanced/reference demos
  - `apps/fret-cookbook` advanced examples
- Decide case by case whether each remaining helper should stay explicit compatibility/advanced or
  migrate to `AppRenderContext<'a>`, `AppRenderCx<'a>`, or `AppComponentCx<'a>`.
