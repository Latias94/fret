# App Composition Density Follow-on v1 — TODO

Status: closeout tracker

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
- [ ] Audit repeated app-shell composition density.
  - Inventory wrapper-only `container` / `v_flex` / `h_flex` / `ui::single` transport chains.
  - Classify each repeated pattern as:
    - docs/adoption discipline,
    - first-party wrapper rule,
    - or real shared helper gap.
  - Require at least one non-Todo proof surface before promoting any shared helper.
- [ ] Audit repeated query invalidation shell.
  - Inventory default app-lane `with_query_client(...)` + redraw call sites.
  - Decide whether a grouped helper belongs on `cx.data()` or another app-facing authoring seam.
  - Keep `key` vs `namespace` explicit if a helper lands.
  - Keep raw `fret-query` ownership unchanged.
- [ ] Keep router out of scope.
  - Use `apps/fret-cookbook/examples/router_basics.rs` only as a boundary check.
  - If router later needs shorter app-shell ergonomics, open a router-specific lane instead.
- [ ] Delete displaced first-contact wording once a better story lands.
  - Because the repo is pre-release, prefer hard deletion over compatibility aliases.
- [ ] Update docs/examples/templates/gates together for each landed batch.

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
- [ ] Remove displaced first-contact wording from the affected docs/examples.

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
- [ ] If a grouped helper does **not** land, record the explicit rationale and keep raw client
  plumbing as an intentional advanced/app-shell seam instead of leaving the question implicit.

## M3 — Delete and lock

- [ ] Update docs indices and authoring guides to the chosen narrower story.
- [ ] Refresh source-policy/tests that protect the default app-lane guidance.
- [ ] Remove old first-contact wording from docs/examples/templates once the replacement is proven.
- [ ] Record any consciously retained raw seam as explicit advanced/reference context rather than
  as a co-equal default path.
