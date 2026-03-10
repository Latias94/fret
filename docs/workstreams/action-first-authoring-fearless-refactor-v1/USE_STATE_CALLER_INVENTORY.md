# Action-First Authoring + View Runtime (Fearless Refactor v1) — `use_state` Caller Inventory

Last updated: 2026-03-09

Related:

- Teaching-surface local-state inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`
- Hard-delete gap analysis: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`
- Hard-delete execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`

This note answers one narrow question for Stage 4 of the hard-delete sequence:

> Where does `ViewCx::use_state::<T>()` still appear today, and do those appearances look like
> real default-path teaching surface, advanced reference, or purely contract-level documentation?

---

## Summary

Current in-tree status (2026-03-09):

- **0 direct runtime/teaching-surface call sites** still call `cx.use_state::<T>()`.
- The remaining in-tree `use_state` presence is now limited to:
  1. runtime/API substrate code,
  2. migration/contract documentation that intentionally explains the explicit raw-model seam.
- `use_state` also remains present in architectural and migration docs because it is still a real
  public API and the implementation substrate under `use_local*`.

Practical interpretation:

- `use_state` is no longer the intended default local-state story.
- But it is also **not** ready for hard delete or even immediate code-level deprecation.

---

## Runtime / teaching-surface status

There are currently **no** direct runtime or teaching-surface callers outside the runtime itself.

Reading:

- The previous remaining cookbook/reference examples have now been migrated.
- This means `use_state` no longer leaks into example/template/gallery code as a default-facing
  local-state hook.

---

### Group A — Recently migrated starter surfaces

| File | Migration update |
| --- | --- |
| `apps/fret-cookbook/examples/hello.rs` | migrated on 2026-03-09 to `use_local::<u32>()` + `on_action_notify_local_update::<act::Click, u32>(...)` |
| `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs` | migrated on 2026-03-09 to `use_local::<u32>()` for the local counter while keeping the injected `last_action: Model<Arc<str>>` as intentional explicit shared state |
| `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs`) | migrated on 2026-03-09 to emit `use_local::<u32>()` + `on_action_notify_local_update::<act::Click, u32>(...)` |
| `apps/fret-cookbook/examples/overlay_basics.rs` | migrated on 2026-03-09 to `use_local::<bool>()` / `use_local::<u32>()` while bridging model-only dialog contracts with `LocalState::clone_model()` |
| `apps/fret-cookbook/examples/imui_action_basics.rs` | migrated on 2026-03-09 to `use_local::<u32>()` + `on_action_notify_local_update::<act::Inc, u32>(...)` |

Reading:

- These were the highest-priority starter/reference leaks.
- Their migration means `use_state` is no longer part of the current first-contact or
  reference-surface default path.

---

### Group B — API definition / substrate

| File | Role |
| --- | --- |
| `ecosystem/fret/src/view.rs` | defines `use_state`, `use_state_keyed`, and currently implements `use_local_with(...)` by delegating to `use_state_with(...)` |

Reading:

- This matters because `use_state` is not just a leftover tutorial alias; it is still part of the
  current public API and implementation layering.

---

## Documentation references

These references are not all “teaching bugs”; some are legitimate contract notes that must remain
as long as the API exists.

### Contract / architecture docs

- `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/EVIDENCE_AND_GATES.md`

Why they exist:

- they describe the current accepted v1 contract,
- and they explicitly note that `use_state` is still model-backed.

### Migration / transition docs

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`

Why they exist:

- these notes are intentionally allowed to mention `use_state` because they explain migration and
  post-v1 cleanup pressure.

### Default-facing narrative docs that should trend away from it

- `docs/examples/todo-app-golden-path.md`

Reading:

- This doc has now been updated to list `use_local*` as the default local-state hook family.

---

## Recommended interpretation

The current caller set now suggests a clearer split:

1. `use_state` is **not** the intended default teaching surface anymore.
2. `use_state` is still a **real explicit raw-model seam** because it remains public API and
   implementation substrate.
3. The repo is **not** ready to deprecate/delete it until:
   - the explicit raw-model role is either affirmed or deprecated as policy,
   - default-facing docs/templates stay locked to `use_local*`,
   - and a narrow reintroduction gate exists if the repo decides the first-contact path must stay
     permanently free of `use_state`.

---

## Recommended next step

The narrow next move should be:

1. keep the policy as “explicit raw-model hook, non-default”,
2. keep starter/reference surfaces on `use_local*`,
3. keep the new narrow docs/template/source gate in place for the approved first-contact surfaces,
4. then revisit whether `use_state` should remain permanent or later become a deprecated alias.

At this point `use_state` should be treated as:

- **non-default**, but
- **not yet deprecation-ready**.
