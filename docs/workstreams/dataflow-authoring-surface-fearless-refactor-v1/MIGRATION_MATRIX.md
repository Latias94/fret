# Dataflow Authoring Surface (Fearless Refactor v1) — Migration Matrix

Status: execution tracker
Last updated: 2026-03-16

This file is the execution-oriented companion to:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`

Its purpose is to keep this lane concrete:

- which surfaces are under review,
- which old spellings are merely quarantined versus actually delete-ready,
- which migration order is allowed,
- and what proof is required before hard deletion.

This workstream is pre-release and does **not** optimize for compatibility.
Temporary adapters are acceptable only as short-lived in-repo scaffolding.
Once the new default path lands, the displaced default-looking path should be removed rather than
documented as an equal alternative.

## Status Legend

| Status | Meaning |
| --- | --- |
| Not started | no migration code or source migration landed |
| In progress | target posture is chosen or emerging, and first-party migration is underway |
| Migrated | official first-party docs/templates/examples use the target posture |
| Delete-ready | migration is complete and the old default-looking path can be removed |
| Deleted | old path is gone |
| Kept intentionally | explicit advanced/reference surface that should remain public |
| Quarantined | still exists, but only as a lower-level/reference surface rather than a default teaching path |

## Global Delete Rule

An old dataflow spelling is eligible for hard deletion only when all of the following are true:

1. scaffold templates are migrated,
2. first-contact docs are migrated,
3. default cookbook/examples are migrated,
4. first-party demos/galleries that teach the same posture are migrated,
5. a gate exists that prevents the old spelling from silently returning,
6. router-aware surfaces were checked for compatibility if the migration touched route-linked dataflow.

## Global No-Compat Rule

Use these rules while executing the lane:

1. Do not keep two default-looking spellings alive just because both still work.
2. If a surface is kept, classify it as either:
   - default,
   - advanced/editor-grade,
   - or reference/bridge-only.
3. If a surface is bridge-only, keep it out of templates, first-hour docs, and canonical Todo-like
   examples.
4. If a surface survives only for reusable ecosystem crates or router/query/selector direct usage,
   keep that reason explicit in docs and tests.

## Current Classification Snapshot (2026-03-16)

| Surface | Current reality | Intended posture | Status |
| --- | --- | --- | --- |
| `cx.actions().local_update::<A>(...)`, `local_set::<A, T>(...)`, `toggle_local_bool::<A>(...)` | still taught across `hello`, `toggle`, `overlay`, `query`, and template surfaces as the one-slot LocalState write family | collapse to one clearly productized one-slot write story or explicitly freeze this small family and stop growing it | In progress |
| `cx.actions().locals::<A>(...)` | current canonical coordinated LocalState transaction path on Todo/form/template surfaces | baseline default until a clearly better LocalState-first transaction surface is proven | Kept intentionally |
| `cx.actions().payload_local_update_if::<A>(...)` | current canonical keyed-row payload write path on Todo/template surfaces | baseline default row-write path unless a shorter but equally explicit replacement is proven | Kept intentionally |
| `cx.actions().payload_locals::<A>(...)` | current rare multi-local payload transaction path | explicit non-happy-path/advanced default companion, not the first row-write story | Kept intentionally |
| `cx.actions().payload::<A>()` chain | still live in cookbook/reference surfaces and a few non-default demos | lower-level/reference-only surface; keep it out of first-contact authoring | Quarantined |
| `cx.actions().models::<A>(...)` | shared-graph/editor-grade coordination path | explicit advanced/editor-grade lane | Kept intentionally |
| widget `.action(...)` / `.action_payload(...)` / `.listen(...)` and host-side `cx.actions().action(...)` / `.action_payload(...)` / `.listen(...)` | current activation-bridge vocabulary | keep as the only activation-glue happy path | Kept intentionally |
| widget `.dispatch::<A>()` / `.dispatch_payload::<A>(...)` and host-side `cx.actions().dispatch::<A>()` / `.dispatch_payload::<A>(...)` | removed from the shipped app-facing surface; only historical workstream records still mention them | stay deleted and keep them out of current docs/tests | Deleted |
| `cx.actions().listener(...)` alias | duplicate alias over `listen(...)`, with no in-tree usage | delete outright | Deleted |
| `cx.data().selector(...)` + `DepsBuilder::new(cx)` + `local_layout_rev(...)` + `layout_in(cx)` | no longer the default Todo/template teaching path; the repo is moving those LocalState-first callsites to `cx.data().selector_layout(...)` while preserving raw `selector(...)` for explicit signatures | replace with one app-facing LocalState-first selector path owned by `ecosystem/fret` | In progress |
| `cx.data().selector_keyed(...)` | explicit keyed selector escape hatch | keep explicit for keyed/looped call sites | Kept intentionally |
| `cx.data().query(...)` / `query_async(...)` | current explicit create-side query surface | keep key/policy/fetch visible | Kept intentionally |
| `cx.data().query_async_local(...)` | explicit `!Send`/wasm query path | keep explicit and non-default | Kept intentionally |
| `handle.layout(cx).value_or_default()` on `QueryHandle<T>` | current default read-side query posture | may collapse to a shorter default read surface if it stays explicit about query-state ownership | Not started |

## Surface Lanes

| Lane | Current surface | Target surface | Migration tactic | Delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| One-slot LocalState writes | small family split across `local_update`, `local_set`, and `toggle_local_bool` | one clearly chosen default one-slot write posture | inventory first, then either freeze this family as the intentional default budget or replace it with one narrower authoring shape on `ecosystem/fret` only | first-contact docs/templates/examples no longer teach several near-equal one-slot helpers | In progress | `ecosystem/fret/src/view.rs`, `apps/fret-cookbook/examples/hello.rs`, `apps/fret-cookbook/examples/toggle_basics.rs`, `apps/fretboard/src/scaffold/templates.rs` |
| Multi-slot LocalState transactions | `cx.actions().locals::<A>(...)` | one canonical LocalState transaction story | treat `locals::<A>(...)` as the baseline until a better default is proven on non-Todo surfaces; do not grow parallel transaction families | no first-party default surface teaches a competing LocalState transaction helper for the same common case | Migrated | `apps/fret-examples/src/todo_demo.rs`, `apps/fret-cookbook/examples/form_basics.rs`, `docs/authoring-golden-path-v2.md` |
| Keyed payload row writes | `payload_local_update_if::<A>(...)` as default, `payload_locals::<A>(...)` as rarer companion, plus the lower-level `payload::<A>()` chain | one canonical row-write happy path, one explicit multi-local fallback, lower-level payload chain kept out of first-contact docs | keep the happy path on keyed row writes, quarantine `payload::<A>()`, and prove any shortening on Todo plus at least one non-list surface | templates/docs/examples stop teaching `payload::<A>()` as a co-equal choice | In progress | `apps/fret-examples/src/todo_demo.rs`, `apps/fretboard/src/scaffold/templates.rs`, `docs/examples/todo-app-golden-path.md`, `apps/fret-cookbook/examples/payload_actions_basics.rs` |
| Shared-model coordination | `cx.actions().models::<A>(...)` | explicit advanced/editor-grade lane | keep explicit and do not fold into LocalState-first sugar | docs/templates keep it off the default happy path while preserving advanced/editor-grade examples | Migrated | `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`, `apps/fret-cookbook/examples/router_basics.rs`, `docs/crate-usage-guide.md` |
| Activation glue aliases | `action/action_payload/listen` plus former `dispatch/dispatch_payload` aliases | keep `action/action_payload/listen`; keep the deleted aliases from reappearing | remove the aliases from code, docs, and source gates together | no first-party docs/examples/tests teach or depend on `dispatch*` | Deleted | `ecosystem/fret/src/view.rs`, `docs/authoring-golden-path-v2.md`, `docs/crate-usage-guide.md`, `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` |
| LocalState-first selector deps/reads | app-facing `selector_layout(...)` helper is being introduced in `ecosystem/fret`; raw `DepsBuilder` choreography remains for explicit shared-model/global signatures | one app-facing LocalState-first selector surface that does not teach dependency-builder internals on first contact | add the shortening only in `ecosystem/fret`; keep `fret-selector` narrow and raw | Todo/template/docs no longer require `DepsBuilder::new(cx)` as the default selector story | In progress | `apps/fretboard/src/scaffold/templates.rs`, `docs/examples/todo-app-golden-path.md`, `docs/authoring-golden-path-v2.md`, `docs/crate-usage-guide.md`, `ecosystem/fret/src/view.rs` |
| Keyed/advanced selector path | `selector_keyed(...)` plus raw selector-engine dependency signatures | explicit advanced/reference lane | keep explicit and separate from the default selector sugar | direct `fret-selector` consumers remain unaffected | Kept intentionally | `ecosystem/fret/src/view.rs`, `ecosystem/fret-selector/src/ui.rs` |
| Query create side | `query(...)`, `query_async(...)`, `query_async_local(...)` | keep explicit key + policy + fetch | keep create-side semantics visible and avoid hiding policy/fetch behind app sugar | first-party docs continue to show explicit create-side semantics | Migrated | `ecosystem/fret/src/view.rs`, `docs/integrating-tokio-and-reqwest.md`, `docs/integrating-sqlite-and-sqlx.md` |
| Query read side | `handle.layout(cx).value_or_default()` + explicit `QueryStatus` branching | one lower-noise default read posture that still exposes state-machine ownership | only add read-side sugar if it proves out on generic-app and editor-grade surfaces; keep the raw handle/state path available | Todo/query docs/templates no longer need the raw handle-read choreography as the default story | Not started | `apps/fretboard/src/scaffold/templates.rs`, `apps/fret-cookbook/examples/query_basics.rs`, `apps/fret-examples/src/query_async_tokio_demo.rs` |
| Docs/templates/examples/gates | templates and core docs are moving to `selector_layout(...)`, but some source gates and secondary docs still mention the older grouped selector spelling as the default | one fully aligned default story | migrate in lockstep: code, templates, docs, source gates, then hard delete | first-hour, golden-path, Todo docs, scaffold templates, cookbook, and UI Gallery gates all tell the same story | In progress | `docs/authoring-golden-path-v2.md`, `docs/examples/todo-app-golden-path.md`, `docs/crate-usage-guide.md`, `apps/fretboard/src/scaffold/templates.rs`, `ecosystem/fret/src/lib.rs` |
| Router compatibility | route-aware demos currently consume action/query/state surfaces separately | compatibility check only, not a source of new requirements | audit after target surfaces are chosen; keep route/history/link work in router lanes | router examples still compile/read correctly without expanding this lane scope | Not started | `docs/workstreams/router-v1/router-v1.md`, `docs/workstreams/router-ui-v1/router-ui-v1.md`, `apps/fret-cookbook/examples/router_basics.rs` |

## Hard Delete / Hard Quarantine Matrix

| Old symbol / posture | Replacement / posture | Remove or quarantine when | Status |
| --- | --- | --- | --- |
| `cx.actions().listener(...)` | `cx.actions().listen(...)` | immediately; there is no in-tree usage keeping the alias alive | Deleted |
| widget `.dispatch::<A>()` / `.dispatch_payload::<A>(...)` | widget `.action(...)` / `.action_payload(...)` / `.listen(...)` | first-party docs/tests no longer teach the explicit alias path and gates keep it out of the default story | Deleted |
| host-side `cx.actions().dispatch::<A>()` / `.dispatch_payload::<A>(...)` | `cx.actions().action(...)` / `.action_payload(...)` / `.listen(...)` | first-party docs/tests stop teaching it and no curated app surface needs the alias wording | Deleted |
| first-contact `cx.actions().payload::<A>()` teaching | `payload_local_update_if::<A>(...)` for the happy path; `payload_locals::<A>(...)` for the rarer coordinated payload case | templates/docs/examples no longer present `payload::<A>()` as a co-equal default path | Quarantined |
| first-contact `DepsBuilder::new(cx)` + `local_layout_rev(...)` + `layout_in(cx)` selector teaching | new app-facing LocalState-first selector sugar (`cx.data().selector_layout(...)`) | the new selector surface is proven on generic-app and editor-grade surfaces and source gates prevent raw builder teaching on the default path | In progress |
| first-contact `handle.layout(cx).value_or_default()` query teaching | a proven lower-noise default query read posture, if one lands | only after a replacement exists and keeps query-state ownership explicit | Not started |

## Current Execution Order

Execute the lane in this order unless a concrete cross-surface proof forces revision:

1. `action`
   - finish inventory,
   - decide the one-slot write budget,
   - delete redundant activation aliases,
   - keep `models::<A>(...)` explicit.
2. `selector`
   - collapse the LocalState-first deps/read story on the app lane,
   - keep raw selector-engine signatures available for advanced/reusable consumers.
3. `query`
   - preserve explicit create-side semantics,
   - collapse only the read-side ceremony if the proof surfaces justify it.
4. ecosystem adaptation
   - confirm what belongs in `ecosystem/fret`,
   - keep `fret-selector` / `fret-query` narrow,
   - avoid forcing reusable crates onto the `fret` facade.
5. docs/templates/examples/gates
   - migrate canonical sources together,
   - then hard-delete displaced default-looking spellings.
6. router compatibility audit
   - verify composability,
   - do not reopen router/history/link design inside this lane.

## Revalidation Bundle

Run this bundle before changing a row from `In progress` to `Migrated` or `Delete-ready`:

- `cargo nextest run -p fret --lib --no-fail-fast`
- `cargo test -p fretboard scaffold --lib`
- `cargo check -p fret-examples --all-targets`

## Completion Rule

This migration matrix is complete when every row that still describes a default-looking old
dataflow surface is either:

- `Deleted`, or
- intentionally classified as `Kept intentionally` / `Quarantined` with a narrow, explicit reason.
