# Default App Productization Fearless Refactor v1 — TODO

This file is the execution checklist for `DESIGN.md`.

## M0 — Baseline and inherited-decision freeze

- [x] Add the workstream folder with:
  - [x] `DESIGN.md`
  - [x] `TODO.md`
  - [x] `MILESTONES.md`
  - [x] `DRIFT_AUDIT_2026-04-02.md`
  - [x] `RECIPE_PROMOTION_AUDIT_2026-04-02.md`
- [x] Freeze the inherited decisions this lane must not silently reopen:
  - [x] default local-state path stays on `LocalState<T>` / `use_local*`
  - [x] grouped local bundles currently teach `*Locals::new(cx)`
  - [x] `todo` remains the third rung
  - [x] no universal `AppShell`
- [x] Record the initial drift read:
  - [x] docs/golden-path surfaces
  - [x] cookbook/demo/template surfaces
  - [x] current recipe-pressure inventory
- [x] Freeze the ADR posture for lane startup:
  - [x] no new ADR is required to start
  - [x] ADR follow-up is required only on real contract motion

## M1 — Blessed-path convergence

- [x] Audit the grouped-local construction drift across:
  - [x] `ecosystem/fret/README.md`
  - [x] `docs/examples/todo-app-golden-path.md`
  - [x] `apps/fret-cookbook/examples/simple_todo.rs`
  - [x] `apps/fret-examples/src/todo_demo.rs`
  - [x] `apps/fretboard/src/scaffold/templates.rs`
- [x] Decide that the current inherited `*Locals::new(cx)` target still stands unchanged.
- [x] Record that no ADR follow-up is needed for M1 because the blessed target did not change.
- [x] Align the default ladder on one grouped-local construction story.
- [x] Add or refresh source-policy gates that keep the chosen blessed path stable.

M1 evidence:

- Live default-path examples now converge on grouped locals via `*Locals::new(cx)` and `cx.state().local*`.
- Source-policy gates were refreshed in `apps/fret-examples/src/lib.rs`.
- Scaffold template gates were refreshed in `apps/fretboard/src/scaffold/templates.rs`.

## M2 — Rich-template productization

- [x] Inventory which concepts in the richer `todo` starter are actually required for the third rung.
- [x] Classify each richer-template concept as:
  - [x] product baseline,
  - [x] optional richer example material,
  - [x] framework-showcase drift
- [x] Slim the starter so first open feels like a product starting point.
- [x] Keep the ladder explicit:
  - [x] `hello` = smallest surface,
  - [x] `simple-todo` = first real local-state + typed-action app,
  - [x] `todo` = richer product baseline, not the feature wall
- [x] Update generated template README/help text if the rung story changes.

M2 evidence:

- Rich-template concept classification and landed cuts are recorded in `RICH_TEMPLATE_PRODUCTIZATION_AUDIT_2026-04-02.md`.
- The richer scaffold now keeps selector/query slices visible but secondary, removes in-card command palette chrome, and adds "first cuts" guidance in the generated README.
- Ladder wording was refreshed across `docs/examples/README.md`, `docs/examples/todo-app-golden-path.md`, and `ecosystem/fret/README.md`.

## M3 — Recipe promotion decision

- [x] Audit repeated app-level helper pressure on the default app lane.
- [x] Use the current compare set first:
  - [x] `apps/fret-examples/src/todo_demo.rs`
  - [x] richer scaffold template in `apps/fretboard/src/scaffold/templates.rs`
  - [x] cookbook/default app-owned scaffolds where relevant
- [x] Classify candidate helpers such as:
  - [x] responsive centered page wrapper
  - [x] todo/card header recipe
  - [x] hover-reveal destructive action row
- [x] Keep page-shell-shaped helpers app-owned unless a new shell audit proves aligned consumers.
- [x] Apply the promotion gate and reject promotion where:
  - [x] fewer than three aligned first-party consumers exist,
  - [x] the behavior remains Todo-specific or policy-divergent,
  - [x] no explicit shared owner is justified,
  - [x] the surface would otherwise drift toward the default `fret` root or prelude
- [x] Record the keep-local vs promote verdict in a companion audit note.

M3 evidence:

- `RECIPE_PROMOTION_AUDIT_2026-04-02.md` now closes on a keep-local verdict for all current candidates.
- `todo_demo` now makes that verdict concrete with file-local app-owned helpers:
  - `todo_page(...)`
  - `todo_card_section(...)`
  - `todo_card_footer_section(...)`
  - `subtle_destructive_button_style(...)`
- No shared recipe owner was introduced.

## M4 — Teaching-surface alignment

- [x] Update docs/examples/templates together for each landed slice.
- [x] Keep `docs/README.md` pointing at the active lane.
- [x] Update app-facing docs if the default ladder wording changes:
  - [x] `docs/examples/README.md`
  - [x] `docs/crate-usage-guide.md`
  - [x] `ecosystem/fret/README.md`
- [x] Keep cookbook, demo, and scaffold surfaces teaching the same first-contact story.
- [x] Remove displaced wording once the replacement is proven.

M4 evidence:

- The ingress docs now repeat the same ladder story:
  - `hello` = smallest starter,
  - `simple-todo` = first real local-state + typed-action app,
  - `todo` = richer third-rung product baseline with deletable selector/query seams.
- The wording was aligned across:
  - `docs/README.md`
  - `docs/examples/README.md`
  - `docs/examples/todo-app-golden-path.md`
  - `docs/first-hour.md`
  - `docs/crate-usage-guide.md`
  - `ecosystem/fret/README.md`
- Generated template README guidance remains aligned with the docs ingress story.

## M5 — Gates and evidence

- [x] Keep or add source-policy gates for the canonical default ladder.
- [x] Keep or add scaffold-template tests that lock the chosen blessed path.
- [x] Add a small proof artifact for resize/layout-sensitive `todo_demo` behavior if the lane edits it again.
- [x] Leave one evidence note per major slice so future reopen attempts have concrete anchors.

M5 evidence:

- Resize/layout proof artifacts are now promoted and documented in
  `RESIZE_LAYOUT_PROOF_2026-04-02.md`.
- The resize proof scripts now capture `layout.taffy.v1.json` sidecars in addition to bundles and
  screenshots:
  - `tools/diag-scripts/tooling/todo/todo-resize-roundtrip-immediate-layout.json`
  - `tools/diag-scripts/tooling/todo/todo-resize-roundtrip-footer-within-window.json`
- `todo_demo` now keeps row-local builder work inside `ui::for_each_keyed_with_cx(...)`, and the
  default `cx.state().local*` diagnostics now preserve app callsites via `ecosystem/fret/src/view.rs`.
- Source-policy/template gates now also lock page-shell ownership on the default ladder:
  - demos/templates keep local `todo_page(...)` helpers,
  - cookbook `centered_page_*` stays cookbook-owned instead of leaking into generated app shells.

## Notes

- This lane is release-facing productization, not framework contract redesign.
- Todo pressure may justify audits and recipe classification, but not automatic public API growth.
- Residual app-facing render-interface gaps surfaced by `todo_demo` now belong to
  `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/` rather than
  reopening this lane.
- If a decision would change the default public contract, stop and escalate to an ADR-backed slice.
- Residual follow-up: the grouped-local `*Locals::new(cx)` path still triggers a per-frame
  repeated-call warning during launched diagnostics runs. The proof note records this as an
  authoring-diagnostics gap rather than a resize correctness failure.
