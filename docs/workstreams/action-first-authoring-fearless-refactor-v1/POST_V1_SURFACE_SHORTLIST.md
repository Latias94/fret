# Action-First Authoring + View Runtime (Fearless Refactor v1) — Post-v1 Surface Shortlist

Status: draft recommendation
Last updated: 2026-03-09

Related:

- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Post-v1 proposal: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
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
4. revisit widget-local action sugar only if the above still leaves repeated handler noise,
5. keep macros and `DataTable` helper expansion deferred.

In practical terms, the next phase should optimize for **surface clarity and evidence-backed density
improvements**, not for adding more API names.

---

## Priority matrix

| Priority | Surface | Why it is still worth doing | Entry condition | Recommended outcome |
| --- | --- | --- | --- | --- |
| P0 | Default-path productization | The runtime path exists, but users still infer the intended story from scattered examples/docs. This is the largest remaining adoption risk. | Default/comparison/advanced taxonomy is still uneven across docs, templates, cookbook, and gallery. | Tighten the onboarding ladder and keep the same recommended path visible everywhere. |
| P1 | Invalidation ergonomics (`AFA-postv1-004`) | This is the closest remaining gap to the GPUI/Zed-style “state write => rerender” feel. | At least one real medium surface still needs explicit policy explanation around tracked local writes vs escape-hatch `notify()`. | Keep `notify()` as a low-level escape hatch; make tracked local writes the boring default. |
| P1 | Local-state ergonomics (`AFA-postv1-001`) | `LocalState<T>` is viable, but still model-backed and still visibly different from the north-star plain-Rust feel. | Evidence must come from real medium surfaces, not from synthetic micro helpers. | Improve the local-state boundary only if it reduces real coordination noise without weakening runtime semantics. |
| P2 | Builder-first last-mile seams (`AFA-postv1-002`) | Broad cleanup already landed; the remaining value is only in cross-cutting host/root seams that still force eager landing. | The seam must affect multiple real surfaces or block the default path directly. | Finish a small number of high-leverage root/container seams; stop leaf-by-leaf churn. |
| P3 | Widget-local action sugar (`AFA-postv1-003`) | This can reduce root-handler noise, but only after the default path is already stable. | At least two medium surfaces still look materially too noisy after productization + invalidation cleanup. | Add only a narrow `listener` / `dispatch` / `shortcut` layer; keep action identity and root handler semantics visible. |
| P4 | Hard-delete/quarantine follow-through | Important for facade hygiene, but not the next authoring-density win. | Deprecation windows and product-policy decisions must mature first. | Continue policy/gate work; do not let cleanup displace the higher-value authoring surfaces above. |

---

## Explicitly deferred surfaces

| Surface | Why it is not worth prioritizing now | Re-entry condition |
| --- | --- | --- |
| `DataTable` helper/macro expansion | The audit already shows the main pressure is business-table recipe assembly, not primitive table builders. The repo now has a curated default recipe plus a gallery slice and smoke gate. | Revisit only if the curated recipe still looks materially too noisy in multiple app-grade examples. |
| Broad macro design (`AFA-postv1-005`) | Macros would freeze a still-settling mental model too early. They are optional polish, not a v2 prerequisite. | Revisit only after productization + invalidation + widget-local sugar still leave repeated structural boilerplate. |
| Compat-runner removal | The current policy already says it is an intentional advanced interop seam for now. | Revisit only if the caller families shrink or a clear quarantine boundary exists. |
| `use_state` hard delete | Current policy keeps it as an explicit raw-model seam rather than a default local-state story. | Revisit only after the repo decides whether that explicit seam should remain permanent. |
| More `DataTable`-specific surface tweaks in this workstream | That would blur the line between business-table productization and the main action-first/view-runtime surface. | Track separately if a future business-table workstream is needed. |

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

### 3. Finish only the highest-leverage builder-first seams

Recommended deliverables:

- target only seams that still force eager `AnyElement` landing across multiple surfaces,
- avoid more local demo-by-demo cleanup unless it closes a reusable root/container seam.

### 4. Re-evaluate widget-local action sugar

Recommended deliverables:

- only after steps 1-3,
- only if at least two medium surfaces still look materially worse than the root-handler story.

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
