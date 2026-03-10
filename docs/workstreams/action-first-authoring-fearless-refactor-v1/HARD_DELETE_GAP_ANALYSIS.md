# Action-First Authoring + View Runtime (Fearless Refactor v1) — Hard-Delete Gap Analysis

Last updated: 2026-03-09

Related execution sequence:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`

This note answers a narrower question than the main v2 proposal:

> What still blocks the repo from hard-deleting the remaining compatibility/legacy-style authoring surfaces?

Short answer:

- **MVU itself is already hard-deleted in-tree.**
- **The repo is not yet ready to hard-delete every remaining compatibility surface around the new default path.**
- The remaining blockers are no longer architectural unknowns; they are now mostly **surface-policy decisions** plus a small number of **migration/gating tasks**.
- The concrete execution order for those decisions now lives in
  `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`.
- A compressed “what is actually next” read now also lives in
  `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`.

---

## What is already closed

These items are no longer the reason cleanup is blocked:

- In-tree MVU/message-router surfaces are gone.
- Templates and primary onboarding docs now teach the action-first/view-runtime path.
- The default starter keyed-list path is aligned across cookbook comparison docs, `todo_demo`, and the `fretboard` simple-todo scaffold.
- Historical MVU planning docs are now marked as historical/superseded where appropriate.

Reference anchors:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`

---

## Remaining blockers before broader hard deletes

Current prioritization (2026-03-09):

- `App::ui*` is no longer a blocker; the pre-release hard delete has already landed.
- `run_native_with_compat_driver(...)` and `use_state::<T>()` are both currently better framed as
  intentional advanced/non-default seams than as near-term hard-delete targets.
- The command-first widget family is no longer the repo’s default next broad implementation pass.
  The obvious app-facing/internal residue is now much smaller: the curated post-v1 follow-up
  aligned workspace tab-strip overflow and GenUI shadcn overlay menu rows to `action(...)`, and
  the remaining visible cases are mostly intentional retained surfaces now recorded in
  `COMMAND_FIRST_INTENTIONAL_SURFACES.md`.

## 1) App-entry closure surface is now closed on the public facade

Current state:

- `fret::App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` no longer exist on the public facade.
- `view::<V>()` / `view_with_hooks::<V>(...)` are the only documented app-entry paths on `fret`.
- README/rustdoc policy is locked by an in-crate test plus `tools/gate_fret_builder_only_surface.py`.

Evidence anchors:

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/README.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md`

Why this no longer blocks cleanup:

- no in-tree callers remained before the patch,
- the surface had not shipped in a published `fret` release,
- and the repo chose to eliminate the split mental model before public release.

Required after hard delete:

- preserve the docs/test gate so `view::<V>()` remains the only default teaching path,
- treat any future restoration of closure-root app entry as a new product decision.

---

## 2) `run_native_with_compat_driver(...)` is still active in real demos

Current state:

- The compat driver path is still public in `fret`.
- It is still used by a noticeable cluster of plotting/interop-style demos.

Evidence anchors:

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/README.md`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/area_demo.rs`
- `apps/fret-examples/src/plot3d_demo.rs`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`

Why this blocks deletion:

- The repo has not yet separated “advanced interop runner surface we keep on purpose” from “temporary compat path we intend to remove”.
- Hard-deleting it today would be a product decision, not just cleanup.
- The current caller inventory now shows three real in-tree families (plot/chart demos, low-level renderer/asset demos, advanced shell demos), which strengthens the case that this is not a trivial leftover.

Required before hard delete:

- Classify the compat driver as either:
  - permanent advanced interop surface, or
  - quarantine/deprecate/remove candidate.
- If removal is desired, migrate the plot/chart demo family first or move the surface behind an explicit compat boundary.

Progress update (as of 2026-03-09):

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
  now recommends keeping `run_native_with_compat_driver(...)` for now as an advanced low-level
  interop seam and deferring hard delete until a clearer quarantine/replacement plan exists.
- `ecosystem/fret/README.md` and `ecosystem/fret/src/lib.rs` now match that policy wording, so the
  remaining compat-driver work is product-surface policy / possible future quarantine rather than
  basic docs cleanup.

---

## 3) `use_state::<T>()` still survives as a user-visible compatibility alias

Current state:

- `use_local*` is now the intended default teaching surface.
- `use_state::<T>()` still exists as public API, but direct runtime/teaching-surface callers have now been migrated off it.

Evidence anchors:

- `ecosystem/fret/src/view.rs`
- `apps/fret-cookbook/examples/hello.rs`
- `apps/fret-cookbook/examples/overlay_basics.rs`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`

Why this blocks deletion:

- We have not finalized whether `use_state` is:
  - a permanent shorthand for simple local slots,
  - a deprecated alias to be steered toward `use_local*`, or
  - a future API that should be repointed to a different semantics.
- The current caller inventory now shows that the remaining pressure is no longer starter/reference
  leakage; it is now almost entirely policy/facade clarity around a still-public raw-model seam.

Required before hard delete:

- Decide whether `use_state` stays or becomes a deprecation target.
- If it becomes legacy, add a narrow gate so new docs/examples/templates stop reintroducing it.

Progress update (as of 2026-03-09):

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
  now recommends keeping `use_state` for now as an explicit raw-model hook, while treating
  `use_local*` as the only default local-state teaching path.
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
  now records that the starter/reference callers have moved to `use_local*`, leaving `use_state`
  present only in the runtime/API substrate plus intentional migration/contract docs.
- `tools/gate_no_use_state_in_default_teaching_surfaces.py` now guards the approved
  first-contact/reference files, and scaffold template output remains covered by unit assertions.

---

## 4) Several component contracts are still `CommandId`-first, not action-first

Current state:

- The repo-level teaching surface is action-first.
- But multiple menu/command-oriented widgets still expose `CommandId`-centric public APIs.

Evidence anchors:

- `ecosystem/fret-ui-shadcn/src/command.rs`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/menubar.rs`
- `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
- `ecosystem/fret-ui-material3/src/snackbar.rs`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`

Why this blocks deletion:

- As long as these contracts stay `CommandId`-first, we cannot honestly claim that the entire user-facing surface has converged on one typed-action mental model.
- This is especially important because v1 intentionally kept `ActionId == CommandId` to avoid schema churn, so the cleanup question is now a product-surface question rather than a runtime-mechanism question.

Required before hard delete:

- Decide whether `CommandId` remains a permanent mechanism-level contract for these widgets.
- If not, add typed-action adapters/replacements and move `CommandId` out of the default teaching surface.
- Only then consider deprecating the direct command-first widget APIs.

Progress update (as of 2026-03-09):

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`
  now scopes this blocker into three buckets:
  - already-aligned dual-surface widgets (`Button`, `CommandItem`),
  - menu-family alias candidates (`ContextMenu*`, `Menubar*`),
  - and lower-risk app-facing alias candidates (`NavigationMenu*`, `BreadcrumbItem`, Material `Snackbar`).
- The first low-risk alias pass has now started: `BreadcrumbItem::action(...)`,
  `NavigationMenuLink::action(...)`, and `NavigationMenuItem::action(...)` are landed, while the
  heavier menu-family item APIs remain the main unfinished command-first surface.
- This means the blocker is now implementation-scoped rather than an unbounded policy blob.

---

## 5) Cleanup gates still need one more pass beyond MVU

Current state:

- We already gate against MVU reintroduction.
- We do **not** yet gate the remaining non-MVU compatibility surfaces with the same precision.

Examples:

- no dedicated gate against reintroducing `use_state` into first-contact docs/templates,
- no gate that distinguishes app-entry `.ui(...)` from the default `view::<V>()` recommendation,
- no explicit gate that keeps command-first widget contracts out of onboarding/docs unless intentionally advanced.

Why this blocks deletion:

- Without gates, deletions/deprecations will drift or be partially reverted by future examples/docs churn.

Required before hard delete:

- Add narrow gates only after the policy decisions above are made.
- Prefer “docs/examples default-path gate” over global code bans when a surface is still intentionally allowed in advanced code.

---

## Non-blockers (do not confuse these with cleanup debt)

These surfaces may still be noisy, but they are **not** the same as the remaining legacy cleanup blockers:

- Builder/patch `.ui()` on composed widgets is not the same thing as app-entry `.ui(...)`.
- `ui::children!` may remain as a compatibility/escape-hatch composition tool even if builder-first becomes the teaching default.
- Historical MVU migration docs should stay; they are archive/mapping material, not default authoring surfaces.

---

## Recommended hard-delete sequence from here

1. **Decide compat-driver policy**
   - Keep as advanced interop, or quarantine/remove?
2. **Decide `use_state` fate**
   - Permanent shorthand, or deprecated alias in favor of `use_local*`?
3. **Decide command-contract policy**
   - Permanent mechanism-level `CommandId`, or migrate more widgets toward typed-action-first adapters?
4. **Then add targeted gates + deprecations**
   - only after the above four decisions are explicit.
5. **Only then hard-delete what is truly legacy**
   - keep advanced/interop surfaces if they are intentional product choices.

---

## Practical verdict

The repo is now much closer to a clean hard-delete phase than it was during the MVU transition, but it is **not** one final grep-away cleanup.

What remains is mostly:

- **one closed app-entry lane that now serves as precedent**,
- **one compat-runner decision**,
- **one local-state alias decision**,
- **one command-contract decision**,
- plus the follow-up gates/docs that make those decisions stick.
