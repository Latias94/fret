# Ecosystem Adaptation And Router Audit — 2026-03-17

Status: M4 audit note
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- `MIGRATION_MATRIX.md`
- `QUERY_READ_SURFACE_CLOSEOUT_2026-03-17.md`
- `docs/workstreams/router-v1/router-v1.md`
- `docs/workstreams/router-ui-v1/router-ui-v1.md`

## Why this note exists

Milestones 1 through 3 narrowed the default app-facing dataflow story.

Milestone 4 must confirm that this narrower story does **not** accidentally pull reusable ecosystem
crates, router crates, or editor-grade surfaces onto the wrong layer just because `fret` now ships
more opinionated app-lane helpers.

This note records that audit result.

## Audit scope

This audit checks four questions:

1. Does `ecosystem/fret` stay the only owner of default app-lane selector/query sugar?
2. Can reusable ecosystem crates still consume `fret-selector` / `fret-query` directly?
3. Do editor-grade surfaces still keep their explicit advanced/shared-model posture?
4. Does router remain compatible without becoming a design driver for this lane?

## Finding 1: `ecosystem/fret` remains the app-lane owner

The app-facing sugar continues to live only in `ecosystem/fret`:

- `fret` feature flags keep selector/query sugar behind `state-selector` / `state-query`
- `cx.data().selector_layout(...)` is owned by the app-facing facade
- `handle.read_layout(cx)` is owned by the app-facing facade

Evidence anchors:

- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`

Decision:

- keep app-lane density helpers in `ecosystem/fret`
- do not move `LocalState<T>`-specific teaching helpers into `fret-selector`, `fret-query`, or
  router crates

## Finding 2: reusable ecosystem crates still stay on direct-crate surfaces

First-party reusable ecosystem crates already consume selector/query/router capabilities through
direct engine dependencies or thin authoring bridges rather than through `fret`:

- `fret-markdown`
  - `mathjax-svg` / `mermaid` opt into `fret-query`
  - query reads stay on explicit `handle.layout_query(cx).cloned().unwrap_or_default()`
- `fret-ui-editor`
  - `state-selector` / `state-query` remain optional direct dependencies
  - selector helpers use `ElementContext::use_selector(...)`, not `fret` app sugar
- `fret-ui-assets`
  - `query-integration` is explicitly optional
  - baseline asset loading does not require `fret-query`
- `fret-authoring`
  - keeps raw `use_selector(...)` authoring bridges for immediate-mode / advanced surfaces
- `fret-router`
  - keeps `query-integration` as an optional direct feature on the router crate

Evidence anchors:

- `ecosystem/fret-markdown/Cargo.toml`
- `ecosystem/fret-markdown/src/mermaid_svg_support.rs`
- `ecosystem/fret-ui-editor/Cargo.toml`
- `ecosystem/fret-ui-editor/src/state/mod.rs`
- `ecosystem/fret-ui-assets/Cargo.toml`
- `ecosystem/fret-authoring/Cargo.toml`
- `ecosystem/fret-authoring/src/selector.rs`
- `ecosystem/fret-router/Cargo.toml`
- `ecosystem/fret-router/src/lib.rs`

Decision:

- reusable ecosystem crates keep direct `fret-selector` / `fret-query` / router usage as the
  baseline
- optional adapters are acceptable, but they must remain thin and must not force a hard dependency
  on `fret`

## Finding 3: editor-grade surfaces remain explicit

The editor-grade proof/demo surfaces checked in this audit do not depend on the new default
app-lane selector/query helpers:

- `apps/fret-examples/src/genui_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/launcher_utility_window_demo.rs`

Audit observation:

- no `selector_layout(...)`
- no `read_layout(cx)`
- no pressure to rewrite advanced/editor-grade code into the default app lane

Complementary evidence:

- `apps/fret-cookbook/examples/router_basics.rs` still uses explicit model/action/router seams
  such as `RouterUiStore`, `snapshot_model.watch(...)`, and `cx.actions().models::<...>(...)`

Decision:

- keep editor-grade/shared-model surfaces explicit
- do not use this lane to make advanced/component/router code look identical to the default app
  lane

## Finding 4: router is compatible but remains adjacent

Router adoption remains explicit in both docs and example code:

- router docs teach `RouteCodec`, `RouterUiStore`, `RouterOutlet`, and explicit link/history
  helpers under the `fret::router::*` seam
- `apps/fret-cookbook/examples/router_basics.rs` uses typed routing and explicit router actions
  without depending on `selector_layout(...)` or `handle.read_layout(cx)`
- router/query integration is still optional and route-driven:
  `fret-router` exposes query-key / invalidation / prefetch planning behind the
  `query-integration` feature instead of turning route state into part of the default dataflow
  helper budget

Evidence anchors:

- `docs/workstreams/router-v1/router-v1.md`
- `docs/workstreams/router-ui-v1/router-ui-v1.md`
- `apps/fret-cookbook/examples/router_basics.rs`
- `ecosystem/fret-router/src/query_integration.rs`
- `ecosystem/fret-router-ui/src/lib.rs`

Decision:

- router compatibility is confirmed
- router does not reopen this workstream's scope
- future router ergonomics should continue to land in router workstreams, not here

## Locked closeout rules

After this audit, the lane should treat these rules as fixed:

- `ecosystem/fret` owns default app-lane selector/query sugar
- direct `fret-selector` / `fret-query` / router crate usage remains valid and intentionally
  lower-level
- reusable ecosystem crates must not be forced onto `fret`
- router remains an adjacent capability, not a justification for widening the default dataflow
  surface

## What this audit does not claim

This note does **not** claim that the entire workstream is fully closed.

Remaining closeout still depends on:

- finishing the outstanding non-Todo proof surfaces called out in `TODO.md`
- keeping docs/templates/examples/gates aligned
- deleting any old default-looking spellings that survive only by inertia

## Reopen trigger

Reopen M4 only if new evidence shows one of these failures:

- a reusable ecosystem crate now requires `fret` just to consume selector/query/router features
- an editor-grade surface is forced onto the default app lane to stay ergonomic
- router adoption starts requiring new default dataflow helpers instead of explicit router seams
