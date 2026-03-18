# Action-First Authoring + View Runtime (Fearless Refactor v1) — Post-v1 Surface Shortlist

Status: draft recommendation
Last updated: 2026-03-16

Related:

- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Shared-surface evidence matrix: `docs/workstreams/action-first-authoring-fearless-refactor-v1/SHARED_SURFACE_EVIDENCE_MATRIX_2026-03-16.md`
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

As of 2026-03-12, the repo still does **not** need another broad helper-expansion pass.

The highest-value post-v1 work is now:

1. collapse the remaining conversion-surface taxonomy through the dedicated `into-element`
   workstream,
2. reopen keyed/list/build-sink density on the canonical trio
   (`simple_todo_v2_target`, `todo_demo`, scaffold template),
3. make the existing default path product-like and hard to misuse around that same evidence set,
4. improve invalidation/local-state ergonomics only where medium-surface evidence still shows a real
   cliff after the first two steps,
5. keep broad builder-family expansion, macros, and `DataTable` helper expansion deferred.

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
| P0 | Conversion-surface cleanup (`into-element` workstream) | This is the clearest remaining reason Fret still feels less unified than GPUI when writing helpers/snippets/components. | App-facing surfaces are already narrow enough that the remaining pain is mostly on helper/component lanes. | Collapse the public conversion story to one concept and keep raw `AnyElement` explicit. |
| P1 | Keyed/list/build-sink density (`AFA-postv1-003`, reopened) | The canonical trio still exposes visible list wiring and sink boilerplate that shapes day-to-day authoring feel. | Treat `simple_todo_v2_target`, `todo_demo`, and the scaffold template as the evidence set; do not wait for unrelated medium surfaces first. | Land one narrow list-authoring improvement and keep action identity explicit. |
| P1 | Default-path productization | The runtime path exists, but users still infer the intended story from scattered examples/docs. This is the largest remaining adoption risk once the conversion story is clearer. | Default/comparison/advanced taxonomy is still uneven across docs, templates, cookbook, and gallery. | Tighten the onboarding ladder and keep the same recommended path visible everywhere, especially across the canonical trio. |
| P2 | Invalidation ergonomics (`AFA-postv1-004`) | This remains important because it defines the default rerender rule, even though the next move now looks more like policy/productization than API expansion. | Keep the policy explicit: tracked writes rerender by default, `notify()` stays an escape hatch, and only reopen API work if a new real medium surface contradicts that rule after P0/P1 land. | Keep `notify()` as a low-level escape hatch; avoid adding another generic helper unless evidence changes. |
| P2 | Local-state ergonomics (`AFA-postv1-001`) | `LocalState<T>` is viable and default-ready, but the remaining distance is now architectural: it is still model-backed and still visibly different from the north-star plain-Rust feel. | Reopen additive API work only if a real medium surface is still blocked after the current `use_local*` path plus the P0/P1 productization work; otherwise treat this as a runtime/ownership question rather than a helper backlog item. | Do not add more sugar by default; only pursue further work with a stronger runtime-level proposal. |
| P3 | Builder-first last-mile seams outside keyed lists (`AFA-postv1-002`, maintenance mode) | The broad cleanup plus the later `Alert` / `ScrollArea` / `Field` closures mean this is no longer an active expansion track. | Reopen only if a new cross-surface host/root seam still forces eager landing across multiple real default-facing surfaces outside the canonical keyed/list slice. | Keep existing builders; avoid more family-by-family expansion without new evidence. |
| P4 | Hard-delete/quarantine follow-through | Important for facade hygiene, but not the next authoring-density win. | Deprecation windows and product-policy decisions must mature first. | Continue policy/gate work; do not let cleanup displace the higher-value authoring surfaces above. |

---

## Current ceremony candidate triage (2026-03-16)

This triage answers a narrower question than the priority matrix:

> When a maintainer sees visible authoring ceremony today, should the next move be docs-only,
> local helper/recipe adoption, or a possible shared public-surface change?

Use the buckets below to avoid turning every Todo-shaped complaint into framework API expansion.
For the current evidence basis behind the "consider shared public surface only with more evidence"
bucket, see `SHARED_SURFACE_EVIDENCE_MATRIX_2026-03-16.md`.

| Bucket | Candidate | Current evidence | Recommended handling |
| --- | --- | --- | --- |
| Docs-only / productization | Default vs richer-rung framing (`hello` / `simple-todo` / `todo`) | The code path is already settled; users still mainly get confused when a richer third rung or a comparison surface is read as the default first-hour path. | Treat this as docs/template/gallery wording discipline; do not add API just to make the ladder easier to explain. |
| Docs-only / productization | Default vs comparison vs advanced taxonomy drift | Current ingress drift is wording-level rather than structural. The remaining failures are usually labeling drift in READMEs, examples indexes, or workstream notes. | Fix the wording and source-policy gates first. If the same API still feels wrong after taxonomy is stable, reopen the API question separately. |
| Local helper / recipe adoption | Single-child wrapper landing and root/page-shell cleanup | The repo already has `ui::single(cx, child)` and the default app lane now teaches `impl UiChild` helpers. Remaining noise is mostly uneven adoption in first-party examples/docs. | Keep migrating call sites and snippets onto the existing helper; do not mint another conversion family for this. |
| Local helper / recipe adoption | Native widget action-slot adoption instead of activation-bridge growth | First-party default widgets already prefer native `.action(...)` / `.action_payload(...)` slots. Remaining friction usually comes from stale examples or recipe-local wrappers, not from a missing default bridge. | Prefer recipe/widget-local narrowing or docs cleanup. Do not grow `AppActivateExt` just to shorten one surface. |
| Local helper / recipe adoption | Existing builder adoption on medium surfaces | `Alert`, `ScrollArea`, and `Field` now already have the needed builders; remaining noise in surfaces such as `form_basics` is mostly adoption and snippet cleanup, not a missing public seam. | Use the existing builders and local wrapper cleanup before proposing more family-level API. |
| Consider shared public surface only with more evidence | Tracked-value read density (`state.layout(cx).value_*`, watched-local reads) | Repeats on Todo plus non-Todo surfaces such as `form_basics` and `assets_reload_epoch_basics`, but the current path is at least coherent and source-aligned. | Keep as a candidate, but require another real medium surface and a concrete design that does not hide invalidation ownership before widening the public surface. |
| Consider shared public surface only with more evidence | Coordinated `locals_with((...)).on::<A>(...)` capture ceremony | Repeats on the canonical Todo compare set and appears in non-Todo form flows too. The pressure is real, but it is easy to overreact with broad sugar that obscures write ownership. | Only reopen with a narrow proposal that improves at least one additional non-Todo real surface and preserves action identity + transaction ownership. |
| Consider shared public surface only with more evidence | Keyed-row payload mutation density | Strongest evidence still comes from `simple_todo_v2_target`, `todo_demo`, and the scaffold template. Outside that Todo-shaped lane, the pressure is not yet broad enough to justify generic sugar. | Keep this on the canonical trio productization lane first. Do not promote a repo-wide helper until the same pain clearly repeats on another real default-facing surface. |
| Adjacent dedicated track (not a fresh generic helper pass) | Conversion-surface collapse / `.into_element(cx)` vocabulary | The biggest remaining write-UI feel gap still spans helper/component lanes, but it already has a dedicated owner in the `into-element` workstream. | Route this work through the dedicated conversion-surface track; do not answer it by adding unrelated action/local-state sugar. |
| Explicitly not a default-path ceremony target | Advanced/runtime-owned friction in `advanced::prelude::*` surfaces | Examples such as `async_playground_demo` still show heavy raw landing and runtime-owned seams, but that is expected on advanced/manual-assembly lanes. | Do not use these surfaces as the primary justification for widening the default app-facing API. Handle them on their own advanced/runtime tracks. |

Short rule:

- if the pain is wording/taxonomy, fix docs;
- if the pain is uneven use of an already-shipped helper, fix adoption;
- if the pain still remains on multiple real default-facing surfaces after those two steps, only
  then consider widening the shared public surface.

---

## Explicitly deferred surfaces

| Surface | Why it is not worth prioritizing now | Re-entry condition |
| --- | --- | --- |
| `DataTable` helper/macro expansion | The audit already shows the main pressure is business-table recipe assembly, not primitive table builders. The repo now has a curated default recipe plus a gallery slice and smoke gate. | Revisit only if the curated recipe still looks materially too noisy in multiple app-grade examples. |
| Broad macro design (`AFA-postv1-005`) | Macros would freeze a still-settling mental model too early. They are optional polish, not a v2 prerequisite. | Revisit only after productization + invalidation + keyed-list / payload-row ergonomics still leave repeated structural boilerplate. |
| Broad keyed-list / payload-row sugar beyond a narrow canonical-trio fix | The repo should improve list authoring feel, but only through the smallest change that helps the canonical trio without hiding action identity or widening generic sugar everywhere. | Revisit broader expansion only if the canonical trio still looks materially too noisy after the first narrow fix. |
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

### 1. Collapse the conversion surface first

Recommended deliverables:

- land the focused `into-element` cleanup,
- keep app-facing `Ui` / `UiChild` and one component conversion concept as the taught story,
- prevent helper/component docs from reintroducing legacy split conversion names.

### 2. Reopen keyed/list/build-sink density on the canonical trio

Recommended deliverables:

- keep `simple_todo_v2_target`, `todo_demo`, and the scaffold template aligned on one intended
  writing style,
- evaluate one narrow list-authoring improvement (`ui::for_each_keyed`, `ui::keyed_column`, or an
  equivalent sink-friendly helper family) against those three surfaces,
- keep action identity explicit instead of hiding it behind broad sugar.

### 3. Productize the current default path next

Recommended deliverables:

- tighten docs/examples/template wording around the onboarding ladder,
- make gallery/cookbook pages explicitly distinguish default vs comparison vs advanced,
- keep policy gates aligned with that default story.

### 4. Re-open invalidation/local-state only where evidence remains

Recommended deliverables:

- one focused medium-surface review,
- one small runtime/API move only if it clearly removes repeated tracked-write noise,
- diagnostics evidence showing rebuild reasons stay explainable.

### 5. Re-open builder-first seam work outside keyed lists only with new evidence

Recommended deliverables:

- none by default,
- reopen only if a new seam still forces eager `AnyElement` landing across multiple surfaces,
- treat families with existing builders but low migration (for example `Card`) as adoption work,
  not as automatic new-surface candidates.

### 6. Leave macros and `DataTable` expansion out of the critical path

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
