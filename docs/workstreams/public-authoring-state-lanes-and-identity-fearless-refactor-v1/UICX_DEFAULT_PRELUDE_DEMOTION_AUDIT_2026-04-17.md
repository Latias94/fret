# UiCx Default Prelude Demotion Audit — 2026-04-17

Status: Frozen follow-on

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `docs/first-hour.md`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`

## Scope

Close one specific default-lane ambiguity:

- should `AppComponentCx` keep arriving through `fret::app::prelude::*`,
- even after `AppRenderCx<'a>` is already the blessed concrete closure-local carrier?

This note does not remove the compatibility alias itself, does not delete `AppUi` `Deref`, and
does not widen the advanced lane back into the default prelude.

## Assumptions-first checkpoint

1. `AppComponentCx` is now compatibility vocabulary, not the taught default helper name.
   Confidence: Confident.
   Evidence: `docs/authoring-golden-path-v2.md`, `docs/first-hour.md`,
   `docs/crate-usage-guide.md`, `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`.
2. `AppRenderCx<'a>` already exists exactly to carry the default concrete helper story without
   reopening raw `ElementContext<App>` naming.
   Confidence: Confident.
   Evidence: `ecosystem/fret/src/lib.rs`, `ecosystem/fret/tests/render_authoring_capability_surface.rs`.
3. The default prelude should reinforce the blessed lane rather than smuggling compatibility names
   back into autocomplete.
   Confidence: Likely.
   Evidence: `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`,
   `ecosystem/fret/src/lib.rs`.

## Findings

### 1) Keeping `UiCx` in the default prelude continued to blur the teaching lane

Before this slice, the repo simultaneously said:

- prefer `AppRenderContext<'a>` for named helpers,
- prefer `AppRenderCx<'a>` for concrete closure-local helpers,
- keep `UiCx` only as the compatibility old-name alias,

but `fret::app::prelude::*` still reexported `UiCx`.

That meant the default import surface still taught two concrete helper names at once:

- the new blessed one (`AppRenderCx`),
- and the compatibility old name (`UiCx`).

### 2) The compatibility alias still has value, just not as a default import

Current evidence does not justify deleting `UiCx` outright:

- advanced/reference examples still used it intentionally,
- migration-era helper families still exist,
- and the alias is harmless when it is treated as explicit compatibility vocabulary.

The mismatch was narrower:

- `UiCx` was arriving by default app prelude autocomplete even though the docs now say it is not
  the taught default.

### 3) The correct step is prelude demotion, not alias deletion

The repo now takes the narrower, correct action:

- `fret::app::prelude::*` keeps `AppRenderCx<'a>`,
- `fret::app::prelude::*` no longer reexports `UiCx`,
- `UiCx` remains available only via explicit import or the advanced lane.

This sharpens the default teaching surface without pretending the compatibility alias can already
be deleted everywhere.

## Evidence

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `docs/first-hour.md`

## Gate commands

- `cargo nextest run -p fret --test render_authoring_capability_surface --test advanced_prelude_surface --test raw_state_advanced_surface_docs`

## Outcome

The default extracted-helper import story is now narrower:

1. `AppRenderCx<'a>` is the default concrete closure-local helper carrier on `fret::app::prelude::*`.
2. `UiCx` remains only as a compatibility old-name alias.
3. Using `UiCx` now requires explicit import or advanced-lane intent instead of coming from the
   default app prelude automatically.
