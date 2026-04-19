# Closeout Audit - 2026-04-19

## Verdict

This lane is ready to close.

The repo now has one explicit default app-facing render-authoring story, one explicit advanced/raw
story, and one explicit compatibility story. The remaining `UiCx<'a>` alias no longer carries any
first-party teaching or architecture weight; it is retained only as a deprecated compatibility
alias until a narrower release-facing removal lane is justified.

## Final target interface

- root render surface:
  - `fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui`
- named default-path helper surface:
  - `Cx: fret::app::AppRenderContext<'a>`
- concrete closure-local helper surface:
  - `&mut fret::app::AppRenderCx<'_>`
- app-hosted snippet/page/component surface:
  - `&mut fret::AppComponentCx<'_>`
- grouped app-render helper namespaces:
  - `AppRenderActionsExt`
  - `AppRenderDataExt`
- deprecated compatibility-only names:
  - `UiCx<'a>`
  - `UiCxActionsExt`
  - `UiCxDataExt`

## Migration results

- `AppUi` no longer depends on `Deref` / `DerefMut` to inherit raw `ElementContext` authoring.
- `fret::app::prelude::*` teaches `AppRenderCx<'a>` and does not reexport `UiCx<'a>`.
- `fret::advanced::prelude::*` teaches `AppComponentCx<'a>` and does not reexport `UiCx<'a>`.
- first-party examples, cookbook/runtime tails, and UI Gallery snippets/pages now use the canonical
  app-facing helper aliases instead of teaching `UiCx<'a>`.
- grouped helper namespaces are now `AppRenderActionsExt` / `AppRenderDataExt`.
- default teaching-snippet tooling now enforces `AppComponentCx<'_>` instead of `UiCx<'_>`.

## Retained advanced seams

These remain intentional and are not regressions:

- explicit raw-model access remains on the advanced surface,
- explicit `cx.elements()` boundaries remain the raw-owner seam for advanced/manual builders,
- explicit raw helper seams such as the markdown image-hook `UiCx<'_>` helper stay classified as
  advanced/compatibility, not default teaching,
- and the deprecated `UiCx<'a>` alias remains available only for explicit compatibility imports.

## Why `UiCx` was not deleted in this lane

Deleting `UiCx<'a>` is now a release-facing decision, not a framework-architecture decision:

- the repo no longer depends on it internally,
- the published-crate workflow in `release-plz.toml` means alias removal should be decided with
  release policy on purpose,
- and the current deprecation already gives downstream users the correct replacement path.

That makes immediate deletion lower-signal than closing this broad lane and opening a narrower
follow-on if release evidence later says the alias should be removed.

## Closure evidence

Commands run for the final slice:

```bash
cargo fmt --package fret
python3 tools/gate_no_raw_app_context_in_default_teaching_snippets.py
cargo nextest run -p fret --test render_authoring_capability_surface --test raw_state_advanced_surface_docs --test uicx_actions_surface --test uicx_data_surface --test crate_usage_grouped_query_surface --no-fail-fast
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```

Key evidence anchors:

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `tools/gate_no_raw_app_context_in_default_teaching_snippets.py`
- `tools/pre_release.py`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`

## Deferred follow-on

If the repo later wants to remove `UiCx<'a>` outright:

- open a new narrow release-facing follow-on lane,
- decide the removal window explicitly,
- and use downstream evidence rather than reopening this broad authoring-surface lane.
