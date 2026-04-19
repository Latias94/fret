# App Render Cx Concrete Carrier Audit — 2026-04-16

Status: landed narrow proof for the active public authoring lane

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/UICX_CLOSURE_CONCRETE_TYPE_PRESSURE_AUDIT_2026-04-16.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_RENDER_CONTEXT_FACADE_AUDIT_2026-04-16.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `apps/fret-examples/src/hello_world_compare_demo.rs`
- `apps/fret-examples/src/lib.rs`

## Question

After landing `AppRenderContext<'a>` as the named helper façade, should closure-heavy default
authoring keep leaning on compatibility `AppComponentCx`, or does the app-facing lane need its own concrete
helper carrier?

## Verdict

The default app-facing lane should own a concrete helper carrier:

- `AppRenderContext<'a>` remains the correct surface for named helper signatures,
- `AppRenderCx<'a>` becomes the default concrete carrier for closure-local or inline helper
  families,
- `UiCx` remains only as the compatibility old-name alias during migration.

This answers the ergonomics question without reopening the broader `AppUi` `Deref` debate.

## Why this is the correct narrow step

`AppRenderContext<'a>` solved the naming problem for extracted helper functions, but it did not
solve ordinary Rust syntax pressure in closure-heavy helpers where a concrete parameter type reads
materially better than a generic trait bound.

Landing `AppRenderCx<'a>` does three things:

1. it gives the default lane an app-facing concrete type name that does not teach raw
   `ElementContext<App>` spelling,
2. it stops overloading `AppComponentCx` as both the compatibility old name and the only concrete helper
   carrier,
3. it keeps the underlying capability model unchanged because `RenderContextAccess<'a, App>`
   remains the real contract.

## Proof surface

The narrow proof in this slice is `apps/fret-examples/src/hello_world_compare_demo.rs`:

- the closure-local swatch helper now uses `&mut AppRenderCx<'_>`,
- the outer extracted helper boundary still uses
  `fret::app::ElementContextAccess<'a, KernelApp>`,
- and the demo therefore proves that the app-facing concrete carrier can coexist with the
  capability-first late-landing story.

The corresponding source-policy gate lives in `apps/fret-examples/src/lib.rs` and forbids the old
`AppComponentCx` spelling from drifting back into that concrete proof.

## Implication for the lane

This change does **not** prove that the repo should now delete `AppComponentCx` everywhere or remove
`AppUi`'s `Deref`.

It proves something narrower and more useful:

- the named-helper story is `AppRenderContext<'a>`,
- the concrete closure-helper story is `AppRenderCx<'a>`,
- and `AppComponentCx` is now clearly only a migration-era old name.
