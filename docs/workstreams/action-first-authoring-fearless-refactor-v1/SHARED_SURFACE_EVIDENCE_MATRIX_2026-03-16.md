# Action-First Authoring + View Runtime (Fearless Refactor v1) — Shared Surface Evidence Matrix (2026-03-16)

Status: draft, planning evidence note
Last updated: 2026-03-16

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION_AUDIT_2026-03-10.md`
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`

---

## Purpose

This note answers one narrow planning question:

> Which remaining authoring-ceremony complaints are credible candidates for shared public-surface
> discussion, and which ones should stay on docs/adoption/productization or adjacent-track
> ownership?

The goal is to keep post-v1 work evidence-driven.
The repo should not widen shared public surface just because a Todo-shaped compare set still looks
busy.

---

## Decision rule

Apply the following rule before proposing a new shared surface:

1. use the canonical trio to find default-path friction,
2. try docs/productization, existing helper adoption, or recipe/local-helper cleanup first,
3. require at least one additional real non-Todo default-facing surface before widening shared
   public API,
4. exclude advanced/manual-assembly surfaces from default app-lane API justification unless the
   problem is explicitly owned by an advanced/runtime track.

This keeps the workstream aligned with the owner rule already recorded in:

- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`

---

## Evidence set used here

### Canonical trio (primary default-path evidence set)

- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

Use this set to identify where ordinary app authoring still feels denser than intended.

### Second evidence surfaces (required before shared public-surface widening)

- `apps/fret-cookbook/examples/form_basics.rs`
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`

Use these to test whether a complaint is still visible outside the Todo-shaped compare set.

### Explicitly excluded as default-path API justification

- `apps/fret-examples/src/async_playground_demo.rs`

This surface intentionally imports `advanced::prelude::*` and still shows heavy
manual-assembly/`.into_element(cx)` usage.
It is useful as conversion-surface evidence, but not as a reason to widen the default app-facing
API.

---

## Matrix

| Candidate | Todo evidence | Non-Todo evidence | Counter-evidence / why not enough | Current disposition | Next action |
| --- | --- | --- | --- | --- | --- |
| Tracked-value read density (`layout(cx).value_*`, watched-local reads) | Repeats in `simple_todo_v2_target.rs`, `todo_demo.rs`, and scaffold templates via `*.layout(cx).value_or_default()` / `value_or(...)`. | Repeats in `form_basics.rs` via `cx.state().watch(&error_state).layout().value_or_default()` and in `assets_reload_epoch_basics.rs` via `cx.state().watch(&bumps_state).layout().value_or(0)`. | The current path is at least coherent and source-aligned. A premature helper could hide tracked-read ownership or blur invalidation semantics. | Keep as a possible shared-surface candidate with more evidence. | Observe at least one more real medium default-facing surface after current productization work. Only reopen with a design that keeps tracked ownership explicit. |
| Coordinated `locals_with((...)).on::<A>(...)` capture ceremony | Repeats in the canonical trio for `Add` / `ClearDone` flows. | Repeats in `form_basics.rs` for `Submit` / `Reset` flows. | This closure shape carries multi-slot transaction ownership. Broad sugar here could hide action identity or make writes feel implicit in a way the runtime cannot explain well. | Keep as a possible shared-surface candidate with more evidence. | Reopen only with a narrow proposal that also improves form-like flows, not just Todo flows, while preserving action identity and transaction boundaries. |
| Keyed-row payload mutation density (`action_payload(...)` + `payload_local_update_if::<A>(...)`) | Strongly repeated in `simple_todo_v2_target.rs`, `todo_demo.rs`, and scaffold templates; this is where the pressure is currently clearest. | No equally strong second evidence surface yet. The current pressure is still mostly concentrated in Todo-shaped dynamic-list examples. | This is the easiest place to overfit Todo compare code and accidentally mint generic sugar for one lane. The current evidence still looks like canonical-trio productization pressure, not repo-wide API proof. | Keep on the canonical-trio productization lane for now. | Continue aligning `simple_todo_v2_target`, `todo_demo`, and scaffold output. Reopen shared public surface only if the same pressure clearly repeats on another real default-facing dynamic-list surface. |
| Conversion-surface collapse (`into-element` vocabulary, raw landing taxonomy) | Not the main blocker on the canonical trio. | Clearly visible on helper/component/manual-assembly surfaces, including `async_playground_demo.rs`. | This already has a dedicated owner and a dedicated workstream. It should not be answered indirectly by widening action/local-state helpers. | Adjacent dedicated track, not a new action-first helper pass. | Route through `docs/workstreams/into-element-surface-fearless-refactor-v1/`. Keep the action-first lane focused on default-path ceremony only. |
| Advanced/runtime-owned manual-assembly friction | Not relevant to the canonical trio. | Visible in `async_playground_demo.rs` through `advanced::prelude::*` plus many explicit `.into_element(cx)` landings. | By definition this is not default-path evidence. These surfaces are allowed to expose more runtime-owned seams. | Explicitly excluded from default app-surface justification. | Handle on advanced/runtime tracks only. Do not use this evidence to justify widening the default app-facing API. |

---

## Current planning conclusion

As of 2026-03-16:

- no new shared public surface is justified immediately,
- tracked-value read density remains the strongest non-Todo-backed watch-list item,
- coordinated `locals_with((...)).on::<A>(...)` capture ceremony remains a second watch-list item,
- keyed-row payload mutation density is still primarily a canonical-trio productization problem,
- conversion-surface collapse remains important, but it belongs to the dedicated `into-element`
  track rather than to another generic action-first helper pass.

---

## Review checklist for future reopen decisions

Before reopening any candidate above, confirm all of the following:

1. the pressure survives current docs/productization/helper-adoption cleanup,
2. the pressure appears on the canonical trio and at least one additional non-Todo default-facing
   surface,
3. the proposed surface keeps action identity, invalidation ownership, and key/transaction context
   explicit,
4. the proposal improves the default product surface rather than only an advanced or
   manual-assembly lane.

If any of those checks fail, keep the change on docs, recipe, local-helper, or adjacent-track
ownership instead of widening shared public surface.
