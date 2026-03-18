# Dataflow Authoring Surface (Fearless Refactor v1) — TODO

Status: Closed closeout tracker (selector/query closed; action write follow-on handed off)
Last updated: 2026-03-17

Companion docs:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `MIGRATION_MATRIX.md`
- `QUERY_READ_SURFACE_CLOSEOUT_2026-03-17.md`
- `ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md`
- `PROOF_SURFACE_AUDIT_2026-03-17.md`

Execution note on 2026-03-16:

- this lane exists because the broad authoring cleanup is already closed
- keep it narrow: `action`, `selector`, `query` default authoring only
- do not turn this into another route/history or storage-architecture reset

Execution note on 2026-03-17:

- `action` closeout is materially landed on the default app surface; the remaining open questions
  are narrower default-write/query follow-ons rather than alias cleanup
- `selector` now has a chosen LocalState-first default spelling:
  `cx.data().selector_layout(inputs, compute)`
- the hidden selector-layout substrate now stays name-honest as `LocalSelectorLayoutInputs` plus
  `LocalSelectorDepsBuilderExt`; these are implementation-facing support seams, not a second
  public selector teaching lane
- raw `cx.data().selector(...)` remains explicit for shared `Model<T>` signatures, global tokens,
  and direct advanced/component `ElementContext` work
- `query` read-side is now landed on the default app lane as `handle.read_layout(cx)`
- component/advanced `layout_query(cx)` remains intentionally explicit
- the hidden prelude import backing that app-lane read now stays name-honest as
  `QueryHandleReadLayoutExt`; the surface promise is the layout-phase fallback only, not a broad
  future query-read family
- the remaining query pressure is lifecycle/status branching, not proof that another default
  read-side helper family should be added here
- M4 audit is now landed:
  reusable ecosystem crates stay on direct engine/authoring bridges and router remains adjacent
- proof-surface audit is now landed:
  action/query are effectively proven on the current in-tree surfaces, and selector proof is now
  also closed on a non-Todo runtime example via `apps/fret-examples/src/hello_counter_demo.rs`
- the remaining default app-lane write-side follow-on now lives in
  `docs/workstreams/action-write-surface-fearless-refactor-v1/`

## Evidence set

### Generic app proof surfaces

- [x] `apps/fret-cookbook/examples/simple_todo.rs`
- [x] `apps/fret-examples/src/todo_demo.rs`
- [x] `apps/fret-cookbook/examples/query_basics.rs`
- [x] `apps/fret-cookbook/examples/form_basics.rs`
- [x] `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- [x] `apps/fretboard/src/scaffold/templates.rs`

### Editor-grade proof surfaces

- [x] `apps/fret-examples/src/genui_demo.rs`
- [x] `apps/fret-examples/src/imui_editor_proof_demo.rs`
- [x] `apps/fret-examples/src/workspace_shell_demo.rs`
- [x] `apps/fret-examples/src/launcher_utility_window_demo.rs`
- [x] `apps/fret-examples/src/query_async_tokio_demo.rs`

Selector-specific note:

- these proofs now cover both the default selector teaching path and non-Todo runtime adoption
- raw `cx.data().selector(...)` remains explicit for advanced/shared-model signatures by design

### Ecosystem / teaching proof surfaces

- [x] `docs/crate-usage-guide.md`
- [x] `docs/authoring-golden-path-v2.md`
- [x] `docs/examples/todo-app-golden-path.md`
- [x] first-party ecosystem adapter touchpoints under `ecosystem/*` once the target surfaces are chosen

### Router compatibility checks

- [x] `docs/workstreams/router-v1/router-v1.md`
- [x] `docs/workstreams/router-ui-v1/router-ui-v1.md`
- [x] `apps/fret-cookbook/examples/router_basics.rs`

Rule:

- router checks are compatibility checks only, not primary design drivers for this lane

## Current priority checklist

- [x] Freeze the scope and ownership rules.
- [x] Freeze the consumer matrix:
  - generic app
  - editor-grade app
  - reusable ecosystem crate
- [x] Inventory current action helper families on the default app path.
- [x] Decide the target collapse for:
  - one-slot local writes
  - multi-slot LocalState transactions
  - keyed payload row writes
  - app-only transient/effect handoff
  - explicit shared-model fallback
- [x] Inventory current LocalState-first selector choreography.
- [x] Decide the target collapse for default selector deps/reads.
- [x] Inventory current query read-side ceremony.
- [x] Decide the target collapse for default query reads without hiding key/policy/fetch.
- [x] Re-measure post-landing query pressure.
  - Conclusion: this lane stops at `handle.read_layout(cx)`.
  - Remaining loading/error/success branching stays explicit and is not evidence for
    `query_result(...)`, `map_status(...)`, or similar default-path helpers here.
- [x] Audit ecosystem adapter impact:
  - `ecosystem/fret`
  - `fret-selector`
  - `fret-query`
  - optional reusable-kit adapters
- [x] Audit editor-grade compatibility impact on shared-model surfaces.
- [x] Audit router compatibility after the default surface is chosen.
- [x] Update docs/templates/examples/gates together for each landed batch.
  - Current remaining discussion is formal closeout posture, not proof coverage or first-contact
    docs drift.

## M0 — Freeze the lane

- [x] Add the workstream directory and connect it from `docs/README.md`, `docs/roadmap.md`, and
  `docs/workstreams/README.md`.
- [x] Record the router-out-of-scope decision explicitly.
- [x] Record the reusable-ecosystem adaptation rules explicitly.

## M1 — Action surface

Follow-on note:

- remaining write-side productization work is now tracked in
  `docs/workstreams/action-write-surface-fearless-refactor-v1/`

- [x] Inventory current default action spellings.
- [x] Classify each current spelling as:
  - default
  - advanced/editor-grade
  - history-only/delete-ready
- [x] Choose the compact target posture for default app writes.
- [x] Prove it on both generic-app and editor-grade surfaces.

## M2 — Selector surface

- [x] Inventory LocalState-first selector dependency/read patterns.
- [x] Decide the target default selector posture.
- [x] Keep raw shared-model dependency signatures explicit.
- [x] Prove the result outside Todo.
  - Landed on 2026-03-17 in `apps/fret-examples/src/hello_counter_demo.rs`, with the
    corresponding source-policy expectation in `apps/fret-examples/src/lib.rs`.

## M3 — Query read surface

- [x] Inventory query read-side patterns on default app surfaces.
- [x] Decide the compact default read posture.
- [x] Keep full query engine semantics explicit.
- [x] Prove the result on both generic-app and editor-grade surfaces.

## M4 — Ecosystem adaptation and closeout

- [x] Update docs/templates/examples to the chosen posture.
- [x] Update source-policy tests or other gates.
- [x] Record which direct-crate ecosystem surfaces stay intentionally lower-level.
- [x] Record router compatibility outcomes without widening the lane scope.

## Standing rules

- [ ] No new helper should land from Todo-only pressure.
- [ ] No default-path helper should land without generic-app plus editor-grade evidence.
- [ ] No reusable ecosystem crate should be forced onto `fret` just to consume selector/query.
- [ ] No router/history/link API should be pulled into scope unless the lane is explicitly revised.
- [ ] No batch is complete until docs/templates/examples/gates agree on the same default story.
