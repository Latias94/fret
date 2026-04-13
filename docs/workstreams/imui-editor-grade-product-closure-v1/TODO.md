# ImUi Editor-Grade Product Closure v1 - TODO

Status: active execution lane
Last updated: 2026-04-13

## Lane setup

- [x] Create the lane and record why the older `imui` closeout folders stay closed.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, and
  `docs/todo-tracker.md`.
- [x] Keep the lane narrow: start a dedicated follow-on once a phase becomes implementation-heavy.
      Result: `docs/workstreams/imui-response-status-lifecycle-v1/` now proves this rule for the
      implementation-heavy `ResponseExt` lifecycle vocabulary slice.

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
- [x] Record the post-shortcut-seam parity status inside the umbrella lane so focused item-local
      shortcuts are no longer treated as the primary P0 blocker.
      Result: `P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md` now records the 2026-04-13 shortcut batch,
      the repeat-semantic test floor, and the narrower remaining P0 backlog.

## P1 - Editor workbench shell closure

- [x] Build one reviewable proof matrix for workspace shell + docking + editor composites.
      Result: `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md` now freezes the current primary proof,
      supporting proofs, and reading order.
- [x] Decide which missing closure belongs in:
      - `ecosystem/fret-workspace`,
      - `ecosystem/fret-docking`,
      - `ecosystem/fret-ui-editor`,
      - or recipe crates.
      Result: `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md` now maps shell slots/tabstrip/command
      scope to `fret-workspace`, docking choreography to `fret-docking`, editor composites to
      `fret-ui-editor`, and scene-local center content to app/recipe ownership.
- [x] Keep shell-level missing pieces out of the generic `imui` backlog by default.
      Result: `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md` now freezes
      `workspace_shell_demo` / `editor_notes_demo` as the shell-first proof order and classifies
      `imui_editor_proof_demo` as supporting docking/editor evidence instead of the default shell
      backlog.
- [x] Promote at least one shell-level diagnostics smoke suite beyond tabstrip-only checks.
      Result: `P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md` now freezes
      `diag-hardening-smoke-workspace` as the promoted P1 shell smoke suite and requires the suite
      minimum to span tab close/reorder/split preview plus dirty-close prompt, Escape focus
      restore, and file-tree keep-alive.

## P2 - Unified diagnostics/devtools surface

- [x] Publish one first-open developer path for:
      inspect -> selector -> script -> bundle -> compare.
      Result: `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md` now freezes a CLI-first
      inspect/pick -> script -> bundle -> compare loop, and keeps DevTools GUI / MCP as thin
      consumers over the same artifacts root and compare semantics.
- [x] Decide what must stay in:
      - `apps/fret-devtools`,
      - `crates/fret-diag`,
      - `ecosystem/fret-bootstrap`,
      - and `apps/fret-devtools-mcp`.
      Result: `P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md` now freezes
      `fret-bootstrap` as the in-app runtime/export seam, `fret-diag` as the shared
      orchestration/artifact engine, `fret-devtools` as GUI UX over shared contracts, and
      `fret-devtools-mcp` as the headless automation/resource adapter.
- [x] Add one bounded devtools smoke package that validates the first-open path rather than only
      isolated tooling commands.
      Result: `P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`,
      `tools/diag_gate_imui_p2_devtools_first_open.py`, and
      `tools/diag-campaigns/devtools-first-open-smoke.json` now freeze one repo-owned gate that
      proves direct `diag run` -> named bundles -> latest resolution -> `diag compare`, plus the
      aggregate campaign root -> `diag summarize` -> `regression.summary.json` /
      `regression.index.json` -> `diag dashboard` handoff.
- [x] Stop forcing authors to discover the workflow by hopping across multiple diagnostics notes.
      Result: `P2_DISCOVERABILITY_ENTRY_2026-04-12.md` and `docs/diagnostics-first-open.md` now
      freeze one canonical first-open diagnostics entry, while the existing inspect, bundles/scripts,
      GUI dogfood, and diagnostics-v2 navigation notes are explicitly demoted to branch/reference
      roles instead of competing start pages.

## P3 - Multi-window hand-feel closure

- [x] Freeze the current runner/backend gap inventory into one short parity checklist for this lane.
      Result: `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md` now freezes hovered-window,
      peek-behind, transparent payload, and mixed-DPI follow-drag as the minimum P3 parity budget,
      and keeps the owner split pinned to `crates/fret-launch`, runner/backend integrations, and
      `ecosystem/fret-docking`.
- [x] Promote one bounded multi-window parity gate or diag suite that explicitly names:
      hovered window, peek-behind, transparent payload, and mixed-DPI follow-drag expectations.
      Result: `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md` and
      `tools/diag-campaigns/imui-p3-multiwindow-parity.json` now freeze one lane-owned bounded
      P3 package over four repo-owned scripts, without bloating `diag-hardening-smoke-docking`.
- [x] Keep `crates/fret-ui` contract growth out of runner-gap fixes unless ADR-backed evidence says
      the runtime contract is actually insufficient.
      Result: `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md` and
      `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md` now make the source-policy rejection
      explicit and tie the remaining proof surface to runner/backend-owned diagnostics.

## Closeout / follow-on management

- [ ] If P0 becomes mostly teaching-surface cleanup, split it into a narrow authoring-lane follow-on.
- [x] If further P0 work becomes mostly immediate convenience breadth
      (key ownership, item-status lifecycle, richer collection/pane proof), split a narrow follow-on
      instead of widening this umbrella folder.
      Result: `docs/workstreams/imui-response-status-lifecycle-v1/` now owns the narrow
      `ResponseExt` lifecycle vocabulary slice, while this umbrella keeps phase ordering and the
      remaining cross-phase backlog read.
- [x] If the remaining P0 pressure becomes helper-owned trigger response shape
      (menu/submenu/tab outward response) rather than public `ResponseExt` vocabulary, split that
      into its own narrow follow-on too.
      Result: `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/` now owns the
      helper-owned menu/submenu/tab trigger response-surface decision instead of reopening either
      the umbrella lane or the lifecycle lane.
- [x] If the helper-owned trigger response lane lands but leaves duplicate public naming behind,
      split a second narrow follow-on for canonicalization instead of rewriting the historical lane.
      Result: `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/` now owns the
      cleanup closeout that removes the duplicate alias layer after the response surface landed.
- [x] If P1 becomes mostly shell composition work, split it into a narrow workbench-shell follow-on.
      Result: `docs/workstreams/imui-workbench-shell-closure-v1/` now records the narrow P1 shell
      closure decision and already closes on a no-new-helper-yet verdict, leaving this umbrella
      focused on phase ordering and cross-phase status.
- [ ] If P2 becomes mostly tooling UX/productization, split it into a narrow devtools follow-on.
- [x] If P3 becomes mostly platform diagnostics and runner work, continue using the existing docking
      parity lane or start a narrower follow-on there instead of bloating this folder.
      Result: after the P1 shell lane closeout, the next active execution priority should continue
      in `docs/workstreams/docking-multiwindow-imgui-parity/`.
