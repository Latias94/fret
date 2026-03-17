# Action Write Surface (Fearless Refactor v1) — Milestones

Status: active closeout lane
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`

## Current execution stance (2026-03-17)

- selector/query default authoring now has its own closed dataflow closeout record
- the old broad action-first lane remains closed and should not be reused as a generic backlog
- this lane owns only the remaining default app-lane write-surface questions on `cx.actions()`
- router, selector, query, widget activation slots, and `LocalState<T>` architecture remain out of
  scope unless fresh evidence reopens them separately

## Milestone 0 — Freeze scope and evidence

Outcome:

- maintainers agree that this lane is about default write-side authoring only

Deliverables:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- docs index / roadmap updates

Exit criteria:

- reviewers can answer what belongs here versus:
  - the closed action-first lane,
  - dataflow selector/query closeout,
  - router workstreams,
  - and local-state architecture follow-ons

## Milestone 1 — Decide the single-local write budget

Outcome:

- the repo freezes the current one-slot helper family as intentional unless fresh cross-surface
  evidence justifies reopening it

Deliverables:

- inventory of current one-slot write teaching surfaces
- classification of:
  - `local_update::<A>(...)`
  - `local_set::<A, T>(...)`
  - `toggle_local_bool::<A>(...)`
- explicit keep-vs-replace decision

Exit criteria:

- first-contact docs/templates/examples no longer imply that the one-slot write story is still in
  flux
- no new parallel one-slot helper family is needed to explain ordinary app writes

Current decision on 2026-03-17:

- M1 audit lands on "freeze the current trio":
  - `local_update::<A>(...)`
  - `local_set::<A, T>(...)`
  - `toggle_local_bool::<A>(...)`
- `locals::<A>(...)` remains the primary transaction story for coordinated writes

## Milestone 2 — Decide the payload row-write budget

Outcome:

- the repo freezes the current payload row-write posture as intentional:
  `payload_local_update_if::<A>(...)` is the default row-write path,
  `payload_locals::<A>(...)` stays off the first-contact lane until proven,
  and `payload::<A>()` remains quarantined

Deliverables:

- inventory of default-facing keyed payload row-write callsites
- explicit default vs secondary classification for:
  - `payload_local_update_if::<A>(...)`
  - `payload_locals::<A>(...)`
  - lower-level `payload::<A>()` references
- evidence review across generic-app and non-Todo surfaces

Exit criteria:

- first-contact docs/templates/examples teach one obvious row-write story
- Todo pressure alone is not the only justification for the shipped surface

Current M2 decision on 2026-03-17:

- `payload_local_update_if::<A>(...)` is strongly proven as the default row-write path
- `payload::<A>()` remains successfully quarantined
- route 2 landed:
  `payload_locals::<A>(...)` is demoted out of first-contact docs/templates and retained only as
  an explicit advanced/reference seam until a real first-party proof surface exists

## Milestone 3 — Closeout and gate alignment

Outcome:

- the chosen write-side budget is reflected consistently across docs, templates, examples, and
  source-policy tests

Deliverables:

- docs/template/example updates
- source-policy gate updates
- explicit retained-seam note for any advanced/reference helpers that remain public

Exit criteria:

- the same write-side story appears in:
  - `docs/first-hour.md`
  - `docs/authoring-golden-path-v2.md`
  - `docs/examples/todo-app-golden-path.md`
  - `docs/crate-usage-guide.md`
  - `apps/fretboard/src/scaffold/templates.rs`
  - first-party demo/cookbook examples that teach the default lane
