# App Composition Density Follow-on v1 — TODO

Status: maintenance-only closeout tracker

Companion docs:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `APP_SHELL_COMPOSITION_AUDIT_2026-03-17.md`
- `QUERY_INVALIDATION_SHELL_AUDIT_2026-03-17.md`
- `DEFAULT_TODO_AUTHORING_STATUS_2026-03-17.md`

Handoff note on 2026-03-18:

- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/` has now formally
  closed its generic fearless-shrink lane after the last zero-proof `fret::actions::*` cleanup,
- this folder is therefore the primary owner for any remaining default app-lane density work that
  is not actually a write-budget, selector/query, or `into_element` redesign question,
- router remains an explicit adjacent seam and should still stay out of scope here,
- do not treat this handoff as permission to widen grouped app nouns or reopen the
  app/component/advanced lane split.

## Execution checklist

- [x] Freeze the evidence set for this lane.
  - Primary detection surface:
    - `apps/fret-examples/src/todo_demo.rs`
    - `apps/fret-cookbook/examples/simple_todo.rs`
    - `docs/examples/todo-app-golden-path.md`
    - `apps/fret-cookbook/examples/query_basics.rs`
    - `apps/fret-examples/src/query_demo.rs`
    - `apps/fret-examples/src/query_async_tokio_demo.rs`
  - Secondary proof / migration surface:
    - `docs/integrating-sqlite-and-sqlx.md`
    - `docs/integrating-tokio-and-reqwest.md`
    - `apps/fret-examples/src/async_playground_demo.rs`
    - `ecosystem/fret-authoring/src/query.rs`
- [x] Keep the lane budget frozen.
  - Do not reopen selector/query read surfaces.
  - Do not reopen `cx.actions()` helper growth.
  - Do not widen `fret::app::prelude::*`.
  - Do not pull router into this lane.
- [x] Audit repeated app-shell composition density.
  - Inventory wrapper-only `container` / `v_flex` / `h_flex` / `ui::single` transport chains.
  - Classify each repeated pattern as:
    - docs/adoption discipline,
    - first-party wrapper rule,
    - or real shared helper gap.
  - Require at least one non-Todo proof surface before promoting any shared helper.
- [x] Audit repeated query invalidation shell.
  - Inventory default app-lane `with_query_client(...)` + redraw call sites.
  - Decide whether a grouped helper belongs on `cx.data()` or another app-facing authoring seam.
  - Keep `key` vs `namespace` explicit if a helper lands.
  - Keep raw `fret-query` ownership unchanged.
- [x] Keep router out of scope.
  - Use `apps/fret-cookbook/examples/router_basics.rs` only as a boundary check.
  - If router later needs shorter app-shell ergonomics, open a router-specific lane instead.
  - Closed on 2026-03-18:
    - router remains an explicit adjacent seam,
    - no default app-lane helper decision in this folder now depends on router pressure.
- [x] Delete displaced first-contact wording once a better story lands.
  - Closed on 2026-03-18:
    - canonical docs/examples/templates now teach `ui::single(cx, child)` for the one-child
      landing case,
    - app-lane query invalidation now teaches
      `cx.data().invalidate_query(...)` / `cx.data().invalidate_query_namespace(...)`,
    - raw `with_query_client(...)` is kept only as the explicit pure app/driver seam.
- [x] Update docs/examples/templates/gates together for each landed batch.
  - Closed on 2026-03-18:
    - canonical examples/templates and the `ecosystem/fret/tests/*` source-policy checks now all
      point at the same narrowed default app-lane story,
    - remaining work in this folder is maintenance-only drift control rather than another
      product-surface batch.

## M0 — Freeze the lane

- [x] Add the workstream directory.
- [x] Add `DESIGN.md`, `TARGET_INTERFACE_STATE.md`, `MILESTONES.md`, and `TODO.md`.
- [x] Add the minimal docs index pointers needed for discoverability.

## M1 — App-shell composition density

- [x] Inventory the repeated wrapper patterns on Todo and query examples.
- [x] Decide whether the best first fix is:
  - better local helper extraction,
  - tighter first-party wrapper rules,
  - or one narrow shared helper.
- [x] Prove the same pressure on at least one non-Todo default app surface.
- [x] Land the smallest justified reduction.
  - 2026-03-17 audit result:
    - no new shared `fret` helper is justified,
    - cookbook scaffolds already cover the canonical default app lane,
    - remaining density is first-party example/scaffold discipline rather than framework surface debt.
- [x] Remove displaced first-contact wording from the affected docs/examples.
  - Closed on 2026-03-18:
    - the first-contact default app-lane surfaces now consistently teach the no-new-API M1
      verdict for shell composition and the grouped M2 query invalidation posture.

## M2 — Query invalidation shell

- [x] Inventory repeated query invalidation shell code on the default app lane.
- [x] Decide whether existing lower-level authoring helpers already point to the right shape.
- [x] Freeze the ownership rule for any grouped invalidation helper.
  - App-facing grouped helper: acceptable.
  - `fret-query` ownership expansion: not acceptable for this lane.
- [x] If a grouped helper lands, migrate first-party evidence surfaces in this order:
  - `apps/fret-cookbook/examples/query_basics.rs`
  - `apps/fret-examples/src/query_demo.rs`
  - `apps/fret-examples/src/query_async_tokio_demo.rs`
  - `docs/integrating-sqlite-and-sqlx.md`
  - `docs/integrating-tokio-and-reqwest.md`
- [x] Grouped-helper decision recorded.
  - Closed on 2026-03-18:
    - the grouped helper did land on `cx.data()`,
    - the retained raw client plumbing is now explicitly documented as the pure app/driver seam
      rather than as a missing default app-path helper.

## M3 — Delete and lock

- [x] Update docs indices and authoring guides to the chosen narrower story.
- [x] Refresh source-policy/tests that protect the default app-lane guidance.
- [x] Remove old first-contact wording from docs/examples/templates once the replacement is proven.
- [x] Record any consciously retained raw seam as explicit advanced/reference context rather than
  as a co-equal default path.
  - Closed on 2026-03-18:
    - grouped app-lane docs now teach `cx.data().invalidate_query*`,
    - `crate-usage-guide`, canonical examples, and the `uicx_data_surface` /
      `crate_usage_grouped_query_surface` tests lock the same posture,
    - `with_query_client(...)` remains documented only for pure app/driver code.
