# Action-First Authoring + View Runtime (Fearless Refactor v1) — Hard-Delete Gap Analysis

Last updated: 2026-03-08

This note answers a narrower question than the main v2 proposal:

> What still blocks the repo from hard-deleting the remaining compatibility/legacy-style authoring surfaces?

Short answer:

- **MVU itself is already hard-deleted in-tree.**
- **The repo is not yet ready to hard-delete every remaining compatibility surface around the new default path.**
- The remaining blockers are no longer architectural unknowns; they are now mostly **surface-policy decisions** plus a small number of **migration/gating tasks**.

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

## 1) App-entry closure surface still exists as a supported public path

Current state:

- `fret::App::{ui, ui_with_hooks}` still exist as public entry points.
- `view::<V>()` is now the documented default, but the closure entry path is still live and still used by some advanced demos.

Evidence anchors:

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/README.md`
- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-examples/src/embedded_viewport_demo.rs`

Why this blocks deletion:

- We have not yet made a repo-level decision whether `ui(...)` is:
  - a permanent secondary surface,
  - an advanced-only escape hatch, or
  - a true legacy surface to deprecate.

Required before hard delete:

- Recommended policy draft now lives in `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_POLICY_DECISION_DRAFT.md`.
- Adopt one explicit policy: either keep `ui(...)` as advanced-only bridge surface, or begin a deprecation plan toward removal.
- After that choice, migrate the remaining advanced demos and add the matching gate/deprecation window.
- The current caller-by-caller migration table now lives in `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md`.

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

Why this blocks deletion:

- The repo has not yet separated “advanced interop runner surface we keep on purpose” from “temporary compat path we intend to remove”.
- Hard-deleting it today would be a product decision, not just cleanup.

Required before hard delete:

- Classify the compat driver as either:
  - permanent advanced interop surface, or
  - quarantine/deprecate/remove candidate.
- If removal is desired, migrate the plot/chart demo family first or move the surface behind an explicit compat boundary.

---

## 3) `use_state::<T>()` still survives as a user-visible compatibility alias

Current state:

- `use_local*` is now the intended default teaching surface.
- `use_state::<T>()` still exists and still appears in a small number of starter/reference snippets.

Evidence anchors:

- `ecosystem/fret/src/view.rs`
- `apps/fret-cookbook/examples/hello.rs`
- `apps/fret-cookbook/examples/overlay_basics.rs`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`

Why this blocks deletion:

- We have not finalized whether `use_state` is:
  - a permanent shorthand for simple local slots,
  - a deprecated alias to be steered toward `use_local*`, or
  - a future API that should be repointed to a different semantics.

Required before hard delete:

- Decide whether `use_state` stays or becomes a deprecation target.
- If it becomes legacy, migrate the remaining teaching/reference surfaces and add a gate so new docs/examples stop introducing it.

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

Why this blocks deletion:

- As long as these contracts stay `CommandId`-first, we cannot honestly claim that the entire user-facing surface has converged on one typed-action mental model.
- This is especially important because v1 intentionally kept `ActionId == CommandId` to avoid schema churn, so the cleanup question is now a product-surface question rather than a runtime-mechanism question.

Required before hard delete:

- Decide whether `CommandId` remains a permanent mechanism-level contract for these widgets.
- If not, add typed-action adapters/replacements and move `CommandId` out of the default teaching surface.
- Only then consider deprecating the direct command-first widget APIs.

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

1. **Decide app-entry policy**
   - Is `App::ui(...)` advanced-but-supported, or on a path to deprecation?
2. **Decide compat-driver policy**
   - Keep as advanced interop, or quarantine/remove?
3. **Decide `use_state` fate**
   - Permanent shorthand, or deprecated alias in favor of `use_local*`?
4. **Decide command-contract policy**
   - Permanent mechanism-level `CommandId`, or migrate more widgets toward typed-action-first adapters?
5. **Then add targeted gates + deprecations**
   - only after the above four decisions are explicit.
6. **Only then hard-delete what is truly legacy**
   - keep advanced/interop surfaces if they are intentional product choices.

---

## Practical verdict

The repo is now much closer to a clean hard-delete phase than it was during the MVU transition, but it is **not** one final grep-away cleanup.

What remains is mostly:

- **one app-entry decision**,
- **one compat-runner decision**,
- **one local-state alias decision**,
- **one command-contract decision**,
- plus the follow-up gates/docs that make those decisions stick.
