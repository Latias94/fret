# UiCx Advanced Prelude and First-Party Tail Audit - 2026-04-18

## Goal

Finish the next `UiCx` retirement step after the UI Gallery migration:

- keep `UiCx<'a>` only as an explicit-import compatibility alias,
- stop exporting it from `fret::advanced::prelude::*`,
- and move the remaining first-party app-hosted examples/cookbook helper tail to
  `AppComponentCx<'a>`.

This is intentionally not an `AppComponentCx` deletion pass. `AppComponentCx<'a>` is now the
canonical name for app-hosted component/snippet helpers; `UiCx<'a>` is the old name.

## Decision

The advanced prelude should export the canonical app-hosted helper context:

- `fret::advanced::prelude::*` exports `AppComponentCx<'a>`,
- `fret::advanced::prelude::*` no longer exports `UiCx<'a>`,
- `fret::app::prelude::*` still stays narrower and does not export either `UiCx` or
  `AppComponentCx`,
- `UiCx<'a>` remains available only from the root/facade as a compatibility alias for explicit
  imports during migration.

## Evidence

- `ecosystem/fret/src/lib.rs`
  - `AppComponentCx<'a>` remains the canonical app-hosted component/snippet alias.
  - `UiCx<'a>` remains a root compatibility alias to `AppComponentCx<'a>`.
  - `advanced::prelude` now reexports `AppComponentCx` and not `UiCx`.
- `apps/fret-examples/src/{assets_demo.rs,custom_effect_v1_demo.rs,custom_effect_v2_demo.rs,custom_effect_v3_demo.rs,genui_demo.rs,imui_editor_proof_demo.rs,liquid_glass_demo.rs,markdown_demo.rs,postprocess_theme_demo.rs}`
  - Remaining first-party advanced/reference helper contexts now use `AppComponentCx<'a>`.
- `apps/fret-cookbook/examples/{app_owned_bundle_assets_basics.rs,assets_reload_epoch_basics.rs,chart_interactions_basics.rs,customv1_basics.rs,drop_shadow_basics.rs,effects_layer_basics.rs,icons_and_assets_basics.rs}`
  - Remaining cookbook helper contexts now use `AppComponentCx<'a>`.
- `apps/fret-examples/src/lib.rs`
  - Source-policy gates now require `AppComponentCx<'a>` on the advanced helper tail.
- `apps/fret-cookbook/src/lib.rs`
  - Source-policy gates now require `AppComponentCx<'a>` on the cookbook advanced helper tail.
- `ecosystem/fret/tests/advanced_prelude_surface.rs`
  - The advanced prelude gate requires `AppComponentCx` and forbids `UiCx`.

## Gates

- `cargo nextest run -p fret-examples --lib --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib`
- `cargo nextest run -p fret --test render_authoring_capability_surface --test advanced_prelude_surface --test raw_state_advanced_surface_docs --no-fail-fast`

## Outcome

`UiCx` is no longer taught by any first-party example/cookbook runtime source and is no longer
available through the advanced prelude. The remaining alias is now limited to explicit import /
compatibility use, which leaves a smaller final closeout question:

- keep `UiCx<'a>` as a short migration alias with explicit deprecation wording, or
- delete it once downstream/internal references have migrated.
