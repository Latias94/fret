# Action-First Authoring + View Runtime (Fearless Refactor v1) — Post-v1 Surface Shortlist

Status: draft recommendation
Last updated: 2026-03-09

Related:

- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Post-v1 proposal: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
- `notify()` policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/NOTIFY_POLICY_DECISION_DRAFT.md`
- Hard-delete execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- Command-first widget audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`
- DataTable audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_AUTHORING_AUDIT.md`
- DataTable golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_GOLDEN_PATH.md`

---

## Purpose

This note answers a narrow planning question:

> After v1 and after the `DataTable` audit/default-recipe pass, which post-v1 authoring surfaces are
> still worth investing in, and which ones should be explicitly deferred?

The intent is to stop treating every remaining density complaint as equally urgent.

---

## Current conclusion

As of 2026-03-09, the repo does **not** need another broad helper-expansion pass.

The highest-value post-v1 work is now:

1. make the existing default path product-like and hard to misuse,
2. improve invalidation/local-state ergonomics only where medium-surface evidence still shows a real
   cliff,
3. finish only the builder-first seams that still block cross-cutting default authoring,
4. revisit keyed-list / payload-row handler ergonomics only if the above still leaves repeated keyed-list handler noise,
5. keep macros and `DataTable` helper expansion deferred.

In practical terms, the next phase should optimize for **surface clarity and evidence-backed density
improvements**, not for adding more API names.

Builder-seam inventory update (as of 2026-03-09):

- `Alert` seam is now closed via `Alert::build(...)` / `AlertAction::build(...)`.
- `ScrollArea` seam is now closed via `ScrollArea::build(...)`.
- `Field` seam is now closed via `FieldSet::build(...)` / `FieldGroup::build(...)` /
  `Field::build(...)`.
- The heaviest remaining raw counts in app/snippet code are now mostly **adoption** of already
  existing builders (`Card` / `CardHeader` / `CardContent`) rather than missing API seams.
- Therefore the next worthwhile move is **not** another new family by default; it is to keep the
  default path/product story stable and only reopen a new seam if repeated cross-surface evidence
  still shows one missing runtime-owned boundary.

---

## Priority matrix

| Priority | Surface | Why it is still worth doing | Entry condition | Recommended outcome |
| --- | --- | --- | --- | --- |
| P0 | Default-path productization | The runtime path exists, but users still infer the intended story from scattered examples/docs. This is the largest remaining adoption risk. | Default/comparison/advanced taxonomy is still uneven across docs, templates, cookbook, and gallery. | Tighten the onboarding ladder and keep the same recommended path visible everywhere. |
| P1 | Invalidation ergonomics (`AFA-postv1-004`) | This remains important because it defines the default rerender rule, even though the next move now looks more like policy/productization than API expansion. | Keep the policy explicit: tracked writes rerender by default, `notify()` stays an escape hatch, and only reopen API work if a new real medium surface contradicts that rule. | Keep `notify()` as a low-level escape hatch; avoid adding another generic helper unless evidence changes. |
| P1 | Local-state ergonomics (`AFA-postv1-001`) | `LocalState<T>` is viable and default-ready, but the remaining distance is now architectural: it is still model-backed and still visibly different from the north-star plain-Rust feel. | Reopen additive API work only if a real medium surface is still blocked after the current `use_local*` path; otherwise treat this as a runtime/ownership question rather than a helper backlog item. | Do not add more sugar by default; only pursue further work with a stronger runtime-level proposal. |
| P2 | Builder-first last-mile seams (`AFA-postv1-002`, maintenance mode) | The broad cleanup plus the later `Alert` / `ScrollArea` / `Field` closures mean this is no longer an active expansion track. | Reopen only if a new cross-surface host/root seam still forces eager landing across multiple real default-facing surfaces. | Keep existing builders; avoid more family-by-family expansion without new evidence. |
| P3 | Keyed-list / payload-row handler ergonomics (`AFA-postv1-003`, maintenance mode) | A deliberately narrow helper already landed and current todo-like surfaces adopted it, so this is no longer an active expansion track. | Reopen only if a new medium surface shows the same row-local pressure beyond the existing todo-like evidence; command/query/form surfaces still do not count. | Keep the current helper as-is; do not widen keyed-list sugar without new evidence. |
| P4 | Hard-delete/quarantine follow-through | Important for facade hygiene, but not the next authoring-density win. | Deprecation windows and product-policy decisions must mature first. | Continue policy/gate work; do not let cleanup displace the higher-value authoring surfaces above. |

---

## Explicitly deferred surfaces

| Surface | Why it is not worth prioritizing now | Re-entry condition |
| --- | --- | --- |
| `DataTable` helper/macro expansion | The audit already shows the main pressure is business-table recipe assembly, not primitive table builders. The repo now has a curated default recipe plus a gallery slice and smoke gate. | Revisit only if the curated recipe still looks materially too noisy in multiple app-grade examples. |
| Broad macro design (`AFA-postv1-005`) | Macros would freeze a still-settling mental model too early. They are optional polish, not a v2 prerequisite. | Revisit only after productization + invalidation + keyed-list / payload-row ergonomics still leave repeated structural boilerplate. |
| Broader keyed-list / payload-row sugar beyond `on_payload_action_notify_local_update_if::<...>(...)` | The current evidence slice is already covered by the narrow helper, and the remaining visible root handler tables are intentional for action identity / ownership clarity. | Revisit only if a second non-todo medium surface shows the same row-local handler-placement noise. |
| Compat-runner removal | The current policy already says it is an intentional advanced interop seam for now. | Revisit only if the caller families shrink or a clear quarantine boundary exists. |
| `use_state` hard delete | Current policy keeps it as an explicit raw-model seam rather than a default local-state story. | Revisit only after the repo decides whether that explicit seam should remain permanent. |
| More `DataTable`-specific surface tweaks in this workstream | That would blur the line between business-table productization and the main action-first/view-runtime surface. | Track separately if a future business-table workstream is needed. |
| Broad `Card`/`CardHeader`/`CardContent` rewrite as a new API task | Those builders already exist; the remaining volume is mostly migration/adoption churn, not proof that another authoring surface is missing. | Revisit only if a narrow default-facing sub-seam appears that the current card builders still cannot express cleanly. |

---

## What “productize the default path” means

This is the next concrete surface, even though it is not a new API:

1. make `hello` -> `simple_todo` -> app-grade todo the obvious onboarding ladder,
2. keep docs, templates, cookbook, and gallery aligned on the same default/comparison/advanced
   taxonomy,
3. keep helper visibility intentionally narrow,
4. keep advanced/reference surfaces clearly labeled so they stop polluting first-contact decisions.

This is the highest-value post-v1 move because it turns the current runtime/API progress into a
predictable product surface instead of leaving it as an expert-only reading exercise.

---

## Concrete next implementation order

### 1. Productize the current default path first

Recommended deliverables:

- tighten docs/examples/template wording around the onboarding ladder,
- make gallery/cookbook pages explicitly distinguish default vs comparison vs advanced,
- keep policy gates aligned with that default story.

### 2. Re-open invalidation/local-state only where evidence remains

Recommended deliverables:

- one focused medium-surface review,
- one small runtime/API move only if it clearly removes repeated tracked-write noise,
- diagnostics evidence showing rebuild reasons stay explainable.

### 3. Re-open builder-first seam work only with new evidence

Recommended deliverables:

- none by default,
- reopen only if a new seam still forces eager `AnyElement` landing across multiple surfaces,
- treat families with existing builders but low migration (for example `Card`) as adoption work,
  not as automatic new-surface candidates.

### 4. Re-open keyed-list / payload-row handler ergonomics only with new evidence

Recommended deliverables:

- none by default,
- reopen only after steps 1-3 and only if a new medium surface still looks materially worse than
  the current root-handler story even with the existing narrow helper.

### 5. Leave macros and `DataTable` expansion out of the critical path

Recommended deliverables:

- none for now beyond keeping the docs decision stable.

---

## Practical rule for accepting a new post-v1 surface

Do not promote a new post-v1 authoring surface unless all of the following are true:

1. at least two real surfaces need it,
2. the existing path is clearly noisier in code review, not just in theory,
3. the new surface does not hide action identity, key context, invalidation ownership, or cache
   boundaries,
4. the change improves the default product surface rather than just one advanced component family.

That rule is how the repo avoids repeating the helper-surface sprawl that v1 just removed.
