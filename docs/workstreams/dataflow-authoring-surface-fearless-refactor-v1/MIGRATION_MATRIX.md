# Dataflow Authoring Surface (Fearless Refactor v1) — Migration Matrix

Status: execution tracker
Last updated: 2026-03-17

This file is the execution-oriented companion to:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- `ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md`
- `PROOF_SURFACE_AUDIT_2026-03-17.md`

Closeout note on 2026-03-17:

- selector/query rows in this file remain authoritative for the dataflow lane
- action-related rows now act as the historical handoff snapshot to
  `docs/workstreams/action-write-surface-fearless-refactor-v1/`

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
| Handed off | active execution moved to a narrower follow-on lane; this file remains the historical handoff snapshot |

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
| `cx.actions().payload_locals::<A>(...)` | former rare multi-local payload transaction path | demoted out of first-contact docs first, then deleted in the post-closeout payload-surface cleanup because no first-party proof appeared | Deleted |
| `cx.actions().payload::<A>()` chain | deleted from production code after its last advanced proof returned to raw `on_payload_action_notify::<A>(...)` | no grouped payload-chain surface remains; keep payload-side shared-model coordination on the raw notify hook when genuinely needed | Deleted |
| `cx.actions().models::<A>(...)` | shared-graph/editor-grade coordination path | explicit advanced/editor-grade lane | Kept intentionally |
| widget `.action(...)` / `.action_payload(...)` / `.listen(...)` and host-side `cx.actions().action(...)` / `.action_payload(...)` / `.listen(...)` | current activation-bridge vocabulary | keep as the only activation-glue happy path | Kept intentionally |
| widget `.dispatch::<A>()` / `.dispatch_payload::<A>(...)` and host-side `cx.actions().dispatch::<A>()` / `.dispatch_payload::<A>(...)` | removed from the shipped app-facing surface; only historical workstream records still mention them | stay deleted and keep them out of current docs/tests | Deleted |
| `cx.actions().listener(...)` alias | duplicate alias over `listen(...)`, with no in-tree usage | delete outright | Deleted |
| `cx.data().selector(...)` + `DepsBuilder::new(cx)` + `local_layout_rev(...)` + `layout_in(cx)` | no longer the default Todo/template/runtime teaching path; first-party LocalState-first callsites now teach `cx.data().selector_layout(...)` while preserving raw `selector(...)` for explicit signatures | replace with one app-facing LocalState-first selector path owned by `ecosystem/fret` | Migrated |
| `cx.data().selector_keyed(...)` | explicit keyed selector escape hatch | keep explicit for keyed/looped call sites | Kept intentionally |
| `cx.data().query(...)` / `query_async(...)` | current explicit create-side query surface | keep key/policy/fetch visible | Kept intentionally |
| `cx.data().query_async_local(...)` | explicit `!Send`/wasm query path | keep explicit and non-default | Kept intentionally |
| `handle.layout(cx).value_or_default()` on `QueryHandle<T>` | no longer the default app teaching surface; `ecosystem/fret` now teaches `handle.read_layout(cx)` while keeping raw tracked reads available for explicit/advanced use | explicit raw tracked-read fallback, not the default app lane | Quarantined |

## Surface Lanes

| Lane | Current surface | Target surface | Migration tactic | Delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| One-slot LocalState writes | small family split across `local_update`, `local_set`, and `toggle_local_bool` | one clearly chosen default one-slot write posture | inventory first, then either freeze this family as the intentional default budget or replace it with one narrower authoring shape on `ecosystem/fret` only | first-contact docs/templates/examples no longer teach several near-equal one-slot helpers | Handed off | `ecosystem/fret/src/view.rs`, `apps/fret-cookbook/examples/hello.rs`, `apps/fret-cookbook/examples/toggle_basics.rs`, `apps/fretboard/src/scaffold/templates.rs`, `docs/workstreams/action-write-surface-fearless-refactor-v1/DESIGN.md` |
| Multi-slot LocalState transactions | `cx.actions().locals::<A>(...)` | one canonical LocalState transaction story | treat `locals::<A>(...)` as the baseline until a better default is proven on non-Todo surfaces; do not grow parallel transaction families | no first-party default surface teaches a competing LocalState transaction helper for the same common case | Migrated | `apps/fret-examples/src/todo_demo.rs`, `apps/fret-cookbook/examples/form_basics.rs`, `docs/authoring-golden-path-v2.md` |
| Keyed payload row writes | `payload_local_update_if::<A>(...)` as default, with duplicate multi-local payload helpers and the lower-level payload chain now deleted | one canonical row-write happy path, no duplicate multi-local helper retained by default, and raw `on_payload_action_notify::<A>(...)` kept as the explicit payload-side fallback | keep the happy path on keyed row writes, delete duplicate zero-proof helpers, and keep raw payload notify explicit instead of grouped | templates/docs/examples stop teaching `payload::<A>()` as a co-equal choice and post-closeout cleanup removes the remaining payload chain from production code | Migrated | `apps/fret-examples/src/todo_demo.rs`, `apps/fretboard/src/scaffold/templates.rs`, `docs/examples/todo-app-golden-path.md`, `apps/fret-cookbook/examples/payload_actions_basics.rs`, `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md` |
| Shared-model coordination | `cx.actions().models::<A>(...)` | explicit advanced/editor-grade lane | keep explicit and do not fold into LocalState-first sugar | docs/templates keep it off the default happy path while preserving advanced/editor-grade examples | Migrated | `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`, `apps/fret-cookbook/examples/router_basics.rs`, `docs/crate-usage-guide.md` |
| Activation glue aliases | `action/action_payload/listen` plus former `dispatch/dispatch_payload` aliases | keep `action/action_payload/listen`; keep the deleted aliases from reappearing | remove the aliases from code, docs, and source gates together | no first-party docs/examples/tests teach or depend on `dispatch*` | Deleted | `ecosystem/fret/src/view.rs`, `docs/authoring-golden-path-v2.md`, `docs/crate-usage-guide.md`, `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` |
| LocalState-first selector deps/reads | app-facing `selector_layout(...)` helper is landed in `ecosystem/fret`; raw `DepsBuilder` choreography remains for explicit shared-model/global signatures | one app-facing LocalState-first selector surface that does not teach dependency-builder internals on first contact | keep the shortening only in `ecosystem/fret`; keep `fret-selector` narrow and raw; reopen only if new evidence shows first-contact default surfaces drifting again | first-contact docs/templates/examples no longer require `DepsBuilder::new(cx)` as the default selector story, and a non-Todo runtime proof exists | Migrated | `apps/fretboard/src/scaffold/templates.rs`, `docs/examples/todo-app-golden-path.md`, `docs/authoring-golden-path-v2.md`, `docs/crate-usage-guide.md`, `ecosystem/fret/src/view.rs`, `apps/fret-examples/src/hello_counter_demo.rs`, `apps/fret-examples/src/lib.rs`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/PROOF_SURFACE_AUDIT_2026-03-17.md` |
| Keyed/advanced selector path | `selector_keyed(...)` plus raw selector-engine dependency signatures | explicit advanced/reference lane | keep explicit and separate from the default selector sugar | direct `fret-selector` consumers remain unaffected | Kept intentionally | `ecosystem/fret/src/view.rs`, `ecosystem/fret-selector/src/ui.rs` |
| Reusable ecosystem selector/query surfaces | first-party reusable crates already stay on direct `fret-selector` / `fret-query` or thin authoring bridges (`fret-markdown`, `fret-ui-editor`, `fret-ui-assets`, `fret-authoring`, router query integration) | keep those surfaces explicit and optional instead of routing them through `fret` | document and audit the boundary; do not force a facade dependency for reusable crates | direct-crate ecosystem consumers remain layer-correct and do not need `fret` just to consume selector/query/router capabilities | Migrated | `ecosystem/fret-markdown/Cargo.toml`, `ecosystem/fret-markdown/src/mermaid_svg_support.rs`, `ecosystem/fret-ui-editor/src/state/mod.rs`, `ecosystem/fret-ui-assets/Cargo.toml`, `ecosystem/fret-authoring/src/selector.rs`, `ecosystem/fret-router/src/lib.rs`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md` |
| Query create side | `query(...)`, `query_async(...)`, `query_async_local(...)` | keep explicit key + policy + fetch | keep create-side semantics visible and avoid hiding policy/fetch behind app sugar | first-party docs continue to show explicit create-side semantics | Migrated | `ecosystem/fret/src/view.rs`, `docs/integrating-tokio-and-reqwest.md`, `docs/integrating-sqlite-and-sqlx.md` |
| Query read side | app-facing `handle.read_layout(cx)` helper is now the shipped default app-lane read posture; explicit `QueryStatus` branching remains unchanged | one lower-noise default read posture that still exposes state-machine ownership | keep the raw handle/state path available for explicit/advanced surfaces; only reopen with new cross-surface evidence that the remaining pressure is not just lifecycle ownership | Todo/query docs/templates teach `handle.read_layout(cx)` as the default story and keep raw handle reads secondary | Migrated | `apps/fretboard/src/scaffold/templates.rs`, `apps/fret-cookbook/examples/query_basics.rs`, `apps/fret-examples/src/query_async_tokio_demo.rs`, `apps/fret-examples/src/query_demo.rs`, `ecosystem/fret/src/view.rs`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/QUERY_READ_SURFACE_CLOSEOUT_2026-03-17.md` |
| Docs/templates/examples/gates | first-contact docs/templates/gates now align on `selector_layout(...)` for LocalState-first selectors, raw `selector(...)` for explicit signatures, and `read_layout(cx)` for the default query read path | one fully aligned default story for the landed batches | keep source-policy checks and README/rustdoc wording aligned whenever the default surface shifts | first-hour, golden-path, Todo docs, scaffold templates, crate docs, and source-policy gates tell the same default story for the currently landed batches | Migrated | `docs/authoring-golden-path-v2.md`, `docs/examples/todo-app-golden-path.md`, `docs/crate-usage-guide.md`, `apps/fretboard/src/scaffold/templates.rs`, `ecosystem/fret/src/lib.rs`, `ecosystem/fret/README.md` |
| Router compatibility | route-aware docs/examples keep route/history/link work on explicit router seams and do not require `selector_layout(...)` / `read_layout(cx)` to stay usable | compatibility check only, not a source of new requirements | keep route/history/link work in router lanes and treat any future dataflow pressure as separate evidence | router examples still compile/read correctly without expanding this lane scope | Migrated | `docs/workstreams/router-v1/router-v1.md`, `docs/workstreams/router-ui-v1/router-ui-v1.md`, `apps/fret-cookbook/examples/router_basics.rs`, `ecosystem/fret-router/src/query_integration.rs`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md` |

## Hard Delete / Hard Quarantine Matrix

| Old symbol / posture | Replacement / posture | Remove or quarantine when | Status |
| --- | --- | --- | --- |
| `cx.actions().listener(...)` | `cx.actions().listen(...)` | immediately; there is no in-tree usage keeping the alias alive | Deleted |
| widget `.dispatch::<A>()` / `.dispatch_payload::<A>(...)` | widget `.action(...)` / `.action_payload(...)` / `.listen(...)` | first-party docs/tests no longer teach the explicit alias path and gates keep it out of the default story | Deleted |
| host-side `cx.actions().dispatch::<A>()` / `.dispatch_payload::<A>(...)` | `cx.actions().action(...)` / `.action_payload(...)` / `.listen(...)` | first-party docs/tests stop teaching it and no curated app surface needs the alias wording | Deleted |
| first-contact `cx.actions().payload::<A>()` teaching | `payload_local_update_if::<A>(...)` for the happy path; no grouped payload-chain helper remains | templates/docs/examples no longer present `payload::<A>()` as a co-equal default path and production code no longer ships the chain | Deleted |
| first-contact `DepsBuilder::new(cx)` + `local_layout_rev(...)` + `layout_in(cx)` selector teaching | new app-facing LocalState-first selector sugar (`cx.data().selector_layout(...)`) | source gates prevent raw builder teaching on the default path and the repo has landed a non-Todo runtime proof for the replacement | Migrated |
| first-contact `handle.layout(cx).value_or_default()` query teaching | app-facing `handle.read_layout(cx)` on the default `fret` lane | once the replacement exists and keeps query-state ownership explicit | Migrated |

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
