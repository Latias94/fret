# App-Shell Composition Audit — 2026-03-17

Status: M1 audit note
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- `apps/fret-cookbook/src/scaffold.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-cookbook/examples/hello_counter.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `docs/examples/todo-app-golden-path.md`

## Why this note exists

M1 asked whether the remaining app-shell composition density on the default app lane was:

- a real framework-level helper gap,
- only docs/adoption drift,
- or a first-party scaffold discipline problem.

This note records that audit result.

## Audit scope

The audit intentionally checks only the **default app lane**:

- `use fret::app::prelude::*;`
- `View` + `AppUi<'_, '_>`
- page/root wrappers that only center, pad, and background one primary surface

It does **not** use these as design drivers:

- advanced/manual `ElementContext<'_, App>` roots,
- direct `UiTree` examples,
- router-specific shells,
- or specialized utility-window / platform-material shells.

## Evidence set

Primary default app-facing proof surfaces:

- `apps/fret-cookbook/src/scaffold.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-cookbook/examples/hello_counter.rs`
- `docs/examples/todo-app-golden-path.md`

Secondary app-facing proof surfaces:

- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fret-examples/src/todo_demo.rs`

Excluded boundary surfaces:

- `apps/fret-examples/src/cjk_conformance_demo.rs`
- `apps/fret-examples/src/emoji_conformance_demo.rs`
- `apps/fret-cookbook/examples/utility_window_materials_windows.rs`

These may repeat similar wrapper shapes, but they sit on advanced/manual/specialized ownership
lanes and therefore should not drive shared app-lane helper design.

## Finding 1: the canonical default app path is already mostly scaffolded

The strongest evidence from cookbook is that the default app lane already converges on shared
app-owned page scaffolds:

- `fret_cookbook::scaffold::centered_page_background(...)`
- `fret_cookbook::scaffold::centered_page_muted(...)`

These are used across a broad app-facing set, including:

- `simple_todo`
- `query_basics`
- `hello_counter`
- many additional cookbook examples outside Todo

Conclusion:

- the broad default app lane does **not** currently need a new framework helper to express
  centered one-surface pages.

## Finding 2: the remaining repeated wrapper shape is first-party shell composition, not framework API pressure

The repeated shell that still looks dense is essentially:

- background container,
- page padding,
- one centered flex shell,
- one typed child landing.

`apps/fret-examples/src/todo_demo.rs::todo_page(...)` still spells that shape locally, but it is:

- app-owned,
- theme-token specific,
- test-id specific,
- and page-shell specific.

That makes it a poor candidate for promotion into `ecosystem/fret` because it would move page
policy and presentation defaults into the facade layer instead of keeping them app-owned.

`apps/fret-examples/src/simple_todo_demo.rs` now lands on the same default app lane and reaches the
same conclusion from a public starter-style demo surface rather than from an advanced/manual shell.

Conclusion:

- the remaining density is real to read,
- but it does **not** justify a new shared `fret` API.

## Finding 3: `ui::single(cx, child)` already closed the framework-level single-child question

At the framework level, the main composition closure question was already solved by:

- `ui::single(cx, child)` for one-child landing,
- `ui::children![cx; ...]` for collections,
- typed `impl UiChild` helpers on the app lane.

The audit did not find a second repeated framework-level gap beyond that.

What remains is wrapper structure discipline in first-party shells, not a missing runtime/facade
primitive.

Conclusion:

- do not add another shared composition helper to `fret` for this lane.

## Finding 4: advanced/manual shells should stay excluded

The repeated `ui::container(...) -> ui::v_flex(...) -> ui::single(...)` shape also appears in
manual `ElementContext<'_, App>` roots and specialized window shells.

Those surfaces remain intentionally explicit because they own different constraints:

- direct-root/manual tree assembly,
- platform/window-material behavior,
- or advanced/manual host composition.

Conclusion:

- similar-looking wrapper shapes outside the default app lane are not evidence for widening the
  app-facing grouped surface.

## Decision from this audit

Treat M1 as closed with a **no new shared API** verdict:

- the default app lane already has the right framework primitive budget,
- cookbook already proves the reusable first-party page-shell answer,
- and the remaining duplication belongs to first-party example/scaffold discipline rather than to
  `ecosystem/fret`.

## Execution consequence

From this point forward:

1. do not add a new shared app-lane composition helper to `fret` for this question,
2. prefer app-owned or example-owned shell helpers when a first-party surface wants a centered
   page wrapper,
3. keep `ui::single(cx, child)` as the framework-level one-child rule,
4. treat any future cleanup here as first-party adoption / local helper extraction unless fresh
   cross-surface evidence shows a new framework-level gap.

## What remains open

This note closes the M1 design question, but not the entire workstream.

Remaining work is now narrower:

- keep M2's grouped query invalidation result aligned,
- and use M3 only for delete/lock discipline if first-party docs/examples drift again.
