# AppRenderContext Facade Audit — 2026-04-16

Status: façade-surface follow-on for the active public authoring lane

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/EXTRACTED_HELPER_RENDER_GUIDANCE_AUDIT_2026-04-16.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `apps/fret-cookbook/src/scaffold.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`

## Question

After freezing the wording that default extracted helpers should no longer teach `UiCx`, does the
repo need one named app-facing helper trait, or should it keep teaching the generic
`RenderContextAccess<'a, App>` spelling directly?

## Verdict

Land one small façade trait:

- `fret::app::AppRenderContext<'a>` is now the named default extracted-helper lane.
- `RenderContextAccess<'a, App>` remains the underlying generic capability.
- `UiCx` stays as the compatibility raw alias.

This is the correct follow-on because it gives the default lane a stable app-facing noun without
introducing a new wrapper type or changing behavior.

## Findings

### 1) The current default helper story had already become repetitive enough to justify a façade

Evidence from first-party proof surfaces:

- `apps/fret-cookbook/src/scaffold.rs`
- `apps/fret-examples/src/api_workbench_lite_demo.rs`
- `docs/first-hour.md`
- `docs/authoring-golden-path-v2.md`

These surfaces were already repeating `RenderContextAccess<'a, App>` as the default helper
signature.

Conclusion:

- the lane had a stable capability,
- but not yet a stable app-facing name.

### 2) A blanket trait is enough; a wrapper type is still unnecessary

The repo does not need:

- another concrete context carrier,
- or a second helper wrapper alongside `AppUi` and `UiCx`.

It only needs a named façade over the existing capability contract:

- `AppRenderContext<'a>: RenderContextAccess<'a, App>`

Conclusion:

- this keeps the type-level target explicit,
- while preserving the earlier audit's warning against inventing another wrapper surface.

### 3) The right proof surfaces are the cookbook scaffold and the real consumer probe

`apps/fret-cookbook/src/scaffold.rs` proves the default shared lesson framing.

`apps/fret-examples/src/api_workbench_lite_demo.rs` proves the lane against a real tool-app
consumer surface rather than a toy helper.

Conclusion:

- those two surfaces are enough for the first façade landing,
- without forcing a broad repo-wide migration of existing `UiCx` stock.

## Landed Slice

- `ecosystem/fret/src/view.rs` now defines `AppRenderContext<'a>`.
- `ecosystem/fret/src/lib.rs` reexports it on `fret::app` and `fret::app::prelude::*`.
- `apps/fret-cookbook/src/scaffold.rs` now uses `AppRenderContext<'a>`.
- `apps/fret-examples/src/api_workbench_lite_demo.rs` now uses `AppRenderContext<'a>`.
- source-policy tests and first-contact docs now name the same façade surface.

## Repro, Gate, Evidence

Repro target:

- inspect extracted helper signatures in `apps/fret-cookbook/src/scaffold.rs`
- inspect extracted helper signatures in `apps/fret-examples/src/api_workbench_lite_demo.rs`

Primary gates:

- `cargo nextest run -p fret --test render_authoring_capability_surface --test raw_state_advanced_surface_docs --test crate_usage_grouped_query_surface`
- `cargo nextest run -p fret-cookbook --lib shared_scaffold_prefers_explicit_app_context_access_for_cookbook_page_shells`
- `cargo nextest run -p fret-examples --lib api_workbench_lite_demo_uses_query_for_sqlite_reads_and_mutation_for_explicit_submit`

What these prove:

- the default extracted-helper lane now has an app-facing name,
- cookbook and consumer-probe proof surfaces both use that name,
- and the repo no longer needs to teach `UiCx` or the generic capability spelling as the first
  extracted-helper surface for ordinary app code.
