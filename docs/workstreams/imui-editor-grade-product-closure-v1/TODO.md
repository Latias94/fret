# ImUi Editor-Grade Product Closure v1 - TODO

Status: active execution lane
Last updated: 2026-04-12

## Lane setup

- [x] Create the lane and record why the older `imui` closeout folders stay closed.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, and
  `docs/todo-tracker.md`.
- [ ] Keep the lane narrow: start a dedicated follow-on once a phase becomes implementation-heavy.

## P0 - Default authoring lane closure

- [x] Inventory the current first-party teaching surfaces that imply the default immediate path.
      Result: `P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md` with a bounded
      golden/reference/historical table.
- [x] Pick the smallest second proof surface beyond `apps/fret-examples/src/imui_editor_proof_demo.rs`
      that should teach the golden path.
      Result: `apps/fret-cookbook/examples/imui_action_basics.rs`.
- [x] Audit the remaining immediate authoring footguns and separate:
      - documentation/teaching issues,
      - proof-surface selection issues,
      - and genuinely missing helper surface.
      Result: `P0_FOOTGUN_AUDIT_2026-04-12.md`.
- [x] Freeze a demote/delete plan for first-party docs/examples that still imply the wrong layer.
      Result: `P0_DEMOTE_DELETE_PLAN_2026-04-12.md`, public docs/gates now route immediate-mode
      readers through the golden pair and demote `imui_hello_demo` to smoke/reference.
- [x] Freeze the proof budget rule for future `fret-ui-kit::imui` public helper widening.
      Result: `P0_PROOF_BUDGET_RULE_2026-04-12.md` now requires at least two real first-party proof
      surfaces, freezes the current minimum budget as `imui_action_basics` +
      `imui_editor_proof_demo`, and rejects reference/compatibility-only surfaces as sole
      justification.
- [x] Publish the first-open root-host teaching rule for `imui(...)` vs `imui_vstack(...)`.
      Result: `P0_ROOT_HOSTING_RULE_2026-04-12.md` and `docs/examples/README.md` now explain the
      nested-layout-host shape versus the explicit root-host bridge, without reopening helper
      growth.
- [x] Publish the first-open stable-identity rule for static vs dynamic IMUI collections.
      Result: `P0_STABLE_IDENTITY_RULE_2026-04-12.md` and `docs/examples/README.md` now explain
      when `ui.for_each_unkeyed(...)` is acceptable versus when `ui.for_each_keyed(...)` /
      `ui.id(key, ...)` is the default posture.

## P1 - Editor workbench shell closure

- [ ] Build one reviewable proof matrix for workspace shell + docking + editor composites.
- [ ] Decide which missing closure belongs in:
      - `ecosystem/fret-workspace`,
      - `ecosystem/fret-docking`,
      - `ecosystem/fret-ui-editor`,
      - or recipe crates.
- [ ] Keep shell-level missing pieces out of the generic `imui` backlog by default.
- [ ] Promote at least one shell-level diagnostics smoke suite beyond tabstrip-only checks.

## P2 - Unified diagnostics/devtools surface

- [ ] Publish one first-open developer path for:
      inspect -> selector -> script -> bundle -> compare.
- [ ] Decide what must stay in:
      - `apps/fret-devtools`,
      - `crates/fret-diag`,
      - `ecosystem/fret-bootstrap`,
      - and `apps/fret-devtools-mcp`.
- [ ] Add one bounded devtools smoke package that validates the first-open path rather than only
      isolated tooling commands.
- [ ] Stop forcing authors to discover the workflow by hopping across multiple diagnostics notes.

## P3 - Multi-window hand-feel closure

- [ ] Freeze the current runner/backend gap inventory into one short parity checklist for this lane.
- [ ] Promote one bounded multi-window parity gate or diag suite that explicitly names:
      hovered window, peek-behind, transparent payload, and mixed-DPI follow-drag expectations.
- [ ] Keep `crates/fret-ui` contract growth out of runner-gap fixes unless ADR-backed evidence says
      the runtime contract is actually insufficient.

## Closeout / follow-on management

- [ ] If P0 becomes mostly teaching-surface cleanup, split it into a narrow authoring-lane follow-on.
- [ ] If P1 becomes mostly shell composition work, split it into a narrow workbench-shell follow-on.
- [ ] If P2 becomes mostly tooling UX/productization, split it into a narrow devtools follow-on.
- [ ] If P3 becomes mostly platform diagnostics and runner work, continue using the existing docking
      parity lane or start a narrower follow-on there instead of bloating this folder.
