# App Render Grouped Helper Extension Naming Audit - 2026-04-19

## Question

After `UiCx<'a>` was removed from the default and advanced preludes, the app-facing grouped helper
extension traits still carried the old `UiCx` root:

- `UiCxActionsExt`
- `UiCxDataExt`

Those names were no longer honest. The traits work through `RenderContextAccess<'a, App>` and
therefore power the app render lane for `AppUi`, `AppRenderContext<'a>`,
`AppRenderCx<'a>`, and app-hosted component/snippet contexts.

## Decision

The canonical grouped helper extension names are now:

- `AppRenderActionsExt`
- `AppRenderDataExt`

The hidden grouped carriers follow the same naming:

- `AppRenderActions`
- `AppRenderActionLocal`
- `AppRenderLocalsWith`
- `AppRenderData`

The old `UiCx*` names remain only as explicit deprecated compatibility aliases. They are not
reexported by `fret::app::prelude::*` or `fret::advanced::prelude::*`.

## Evidence

- `ecosystem/fret/src/view.rs`
  - owns `AppRenderActionsExt` / `AppRenderDataExt`;
  - keeps `UiCxActionsExt` / `UiCxDataExt` as deprecated compatibility aliases;
  - renames the internal action-hook owner from `UiCx*` to `AppRender*`.
- `ecosystem/fret/src/lib.rs`
  - reexports `AppRenderActionsExt` / `AppRenderDataExt` through `fret::app`;
  - imports the canonical names in the default and advanced preludes;
  - keeps `UiCxActionsExt` / `UiCxDataExt` only for explicit compatibility imports.
- `apps/fret-ui-gallery/src/ui/snippets/**`
  - imports `fret::app::AppRenderActionsExt as _` instead of the old `UiCxActionsExt`.
- `apps/fret-examples/src/**`
  - imports `fret::advanced::view::AppRenderDataExt as _` where advanced examples need grouped
    data helpers.
- `docs/crate-usage-guide.md`, `docs/shadcn-declarative-progress.md`, and ADR 0319 now name
  `AppRenderActionsExt` / `AppRenderDataExt` as the canonical surface.

## Gates

- `cargo nextest run -p fret --test uicx_actions_surface --test uicx_data_surface --test crate_usage_grouped_query_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --no-fail-fast`
- `cargo nextest run -p fret-examples --lib --no-fail-fast`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

## Follow-On

The remaining `UiCx<'a>` alias can now be audited directly. This slice removes the more confusing
public trait-name leakage first, so the next decision can focus on whether the root alias should be
deprecated for one release window or deleted before publication.
