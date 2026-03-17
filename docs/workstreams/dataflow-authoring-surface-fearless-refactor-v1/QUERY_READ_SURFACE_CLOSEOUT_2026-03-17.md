# Query Read Surface Closeout — 2026-03-17

Status: Landed direction note
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- `MIGRATION_MATRIX.md`

## Why this note exists

Milestone 3 is no longer a design exploration.

The default app-lane query read posture has landed in the facade, in first-party docs, in scaffold
templates, and in proof surfaces. Older workstream notes still describe query read-side work as
"cleanup first" or imply that more default-path helper families may still be needed.

This note closes that ambiguity.

## Landed posture

- query creation stays explicit on `cx.data().query(...)`, `query_async(...)`, and
  `query_async_local(...)`
- the default `fret` app-lane read spelling is `handle.read_layout(cx)` for the common
  `QueryState::<T>::default()` fallback case
- component/advanced/declarative surfaces keep explicit
  `handle.layout_query(cx).value_or_default()` reads
- raw tracked reads such as `handle.layout(cx).value_or_default()` remain available but are no
  longer the taught default app path

## What this lane is intentionally not doing

- no `query_result(...)`, `query_state(...)`, `map_status(...)`, or `when_success(...)`
- no `fret-query` engine changes
- no attempt to hide `QueryStatus` / `data` / `error` lifecycle ownership
- no router/history expansion

## Why the lane stops here

After the default fallback collapse, the remaining query verbosity is mostly real lifecycle work:

- deciding how loading should render,
- deciding how errors should render or retry,
- deciding how success data should join the surrounding UI state.

That is not accidental read plumbing anymore. It is explicit application or component policy.

If future evidence shows repeated lifecycle noise on multiple non-trivial surfaces, it should be
handled by a later focused workstream with fresh evidence. It should not be solved here by growing
another default query-helper family.

## Evidence anchors

- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `docs/authoring-golden-path-v2.md`
- `docs/crate-usage-guide.md`
- `docs/integrating-tokio-and-reqwest.md`

## Exit rule

Treat Milestone 3 as landed.

Remaining work for this broader lane now belongs to:

- ecosystem adaptation,
- editor-grade compatibility auditing,
- router compatibility auditing,
- and keeping docs/templates/gates aligned.
