# Dataflow Authoring Surface (Fearless Refactor v1) — Milestones

Status: Active closeout lane
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `MIGRATION_MATRIX.md`
- `QUERY_READ_SURFACE_CLOSEOUT_2026-03-17.md`
- `ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md`
- `PROOF_SURFACE_AUDIT_2026-03-17.md`

## Current execution stance (2026-03-16)

This is the next focused authoring lane after the broader authoring closeout chain.

Read it as:

- narrow and evidence-driven,
- app-facing by default,
- explicitly compatible with editor-grade and reusable-ecosystem consumers,
- and intentionally separate from router/history scope.

## Current execution stance (2026-03-17)

- Milestone 1 is in late closeout: dispatch-style activation aliases are gone and the default app
  lane now teaches one narrower action-first vocabulary.
- Milestone 2 proof coverage is now landed: `cx.data().selector_layout(inputs, compute)` is the
  chosen LocalState-first default selector spelling, while raw `cx.data().selector(...)` remains
  the explicit shared-model/global-signature lane and `hello_counter_demo` now proves the default
  posture on a non-Todo runtime surface.
- Milestone 3 is now landed: `handle.read_layout(cx)` is the default app-lane query read posture
  for the common `QueryState::<T>::default()` fallback case while keeping `QueryStatus` ownership
  explicit.
- Milestone 4 audit is now landed: reusable ecosystem crates remain on direct selector/query/router
  surfaces and router compatibility is confirmed without widening this lane's scope.
- The remaining work for this lane is now closeout-oriented:
  legacy default-looking spellings still need final classification/delete-ready decisions where
  applicable, but selector proof is no longer an open blocker.

## Milestone 0 — Freeze the lane

Outcome:

- maintainers agree on scope, ownership, proof surfaces, and the router boundary.

Deliverables:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- `MIGRATION_MATRIX.md`
- docs index / roadmap updates

Exit criteria:

- reviewers can answer what belongs here versus:
  - closed broad authoring lanes,
  - router workstreams,
  - and domain-engine work in `fret-selector` / `fret-query`.

## Milestone 1 — Collapse the action surface

Outcome:

- the default app lane has one compact action language for common local-state writes.

Deliverables:

- current action helper inventory
- chosen target surface for:
  - one-slot local writes,
  - multi-slot LocalState transactions,
  - keyed payload row writes,
  - transients,
  - explicit shared-model fallback
- migration plan for docs/templates/examples/source-gates

Exit criteria:

- the default app path no longer teaches several near-equivalent mutation helper families for the
  same common use case,
- shared-model coordination is still clearly explicit.

## Milestone 2 — Collapse the LocalState-first selector surface

Outcome:

- the default app lane has one compact LocalState-first derived-state story.

Deliverables:

- LocalState-first selector dependency/read inventory
- chosen target surface for default app authoring
  - current chosen spelling: `cx.data().selector_layout(inputs, compute)` for LocalState-first
    selectors on the `fret` app lane
- explicit advanced fallback story for shared-model dependency signatures

Exit criteria:

- default app docs/templates/examples no longer require raw dependency-builder knowledge as the
  first-contact selector story and teach `selector_layout(...)` for LocalState-first inputs,
- reusable ecosystem crates can still use `fret-selector` directly without app-facade coupling.

Proof closeout on 2026-03-17:

- `apps/fret-examples/src/hello_counter_demo.rs` now uses `selector_layout(...)` on a real runtime
  demo, closing the non-Todo proof gap for this milestone.

## Milestone 3 — Collapse the query read-side surface

Outcome:

- query remains explicit but the default read side is materially lower-noise.

Status:

- Landed on 2026-03-17.
- Remaining work now belongs to Milestone 4 closeout and compatibility audit, not to more query
  read-side sugar design.

Deliverables:

- query read-pattern inventory
- chosen default read-side posture
  - current chosen spelling: `handle.read_layout(cx)` on the default `fret` app lane
- explicit advanced handle/state-machine fallback posture

Exit criteria:

- key/policy/fetch remain visible,
- common loading/error/success reads on the default app path are shorter and the default docs teach
  `read_layout(cx)` for the ordinary fallback case,
- reusable ecosystem crates can still consume `fret-query` directly.

## Milestone 4 — Ecosystem adaptation and closeout

Outcome:

- the chosen dataflow dialect works across generic apps, editor-grade apps, and reusable ecosystem
  adapters.

Status:

- M4 audit landed on 2026-03-17 via
  `ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md`.
- Remaining closeout is about docs/source-gate alignment and unresolved proof surfaces from
  earlier milestones, not about inventing more ecosystem/router-specific helper APIs.

Deliverables:

- ecosystem adaptation audit
- docs/templates/examples/source-policy updates
- compatibility audit against router/query/selector workstreams

Exit criteria:

- the same default teaching story appears in first-contact docs, scaffold templates, cookbook
  surfaces, and first-party demos,
- reusable ecosystem crates are not forced onto the wrong dependency tier,
- router remains compatible but separate.

Current remaining closeout gap:

- first-contact docs/templates/gates are aligned for the landed batches,
- and the remaining work is now about final closeout posture for still-classified legacy spellings,
  not missing selector proof.
