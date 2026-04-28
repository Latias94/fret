# ImUi Editor-Grade Product Closure v1 - TODO

Status: maintenance umbrella lane
Last updated: 2026-04-28

Status note (2026-04-22): keep phase ordering and follow-on decisions here. Do not resume
implementation-heavy work in this folder while the closed child-region depth closeout record lives
in `docs/workstreams/imui-child-region-depth-v1/` and the remaining P3 execution continues in
`docs/workstreams/docking-multiwindow-imgui-parity/`.

## Lane setup

- [x] Create the lane and record why the older `imui` closeout folders stay closed.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, and
  `docs/todo-tracker.md`.
- [x] Keep the lane narrow: start a dedicated follow-on once a phase becomes implementation-heavy.
      Result: `docs/workstreams/imui-response-status-lifecycle-v1/` now proves this rule for the
      implementation-heavy `ResponseExt` lifecycle vocabulary slice.
- [x] Demote this folder from active execution to umbrella maintenance once the implementation-heavy
      phases moved into narrower lanes.
      Result: this folder now records phase ordering and cross-phase status, while the narrow P0/P1
      closeout records stay closed and the remaining active P3 execution continues in
      `docs/workstreams/docking-multiwindow-imgui-parity/`.
- [x] Close the second proof-surface follow-on without widening shared IMUI helpers.
      Result: `docs/workstreams/imui-collection-second-proof-surface-v1/` now records the closed
      second-proof-surface follow-on, lands the `Scene collection` left-rail surface in
      `editor_notes_demo.rs`, and records that it does not yet prove that both collection proof surfaces
      need the same shared helper.

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
- [x] Publish the first-open mounting rule for safe-default `imui(...)` versus explicit
      `imui_raw(...)`.
      Result: `P0_ROOT_HOSTING_RULE_2026-04-12.md` and `docs/examples/README.md` now explain the
      safe default for root/non-layout parents versus the advanced explicit-layout seam, without
      reopening helper growth.
- [x] Publish the first-open stable-identity rule for static vs dynamic IMUI collections.
      Result: `P0_STABLE_IDENTITY_RULE_2026-04-12.md` and `docs/examples/README.md` now explain
      when `ui.for_each_unkeyed(...)` is acceptable versus when `ui.for_each_keyed(...)` /
      `ui.id(key, ...)` is the default posture.
- [x] Record the post-shortcut-seam parity status inside the umbrella lane so focused item-local
      shortcuts are no longer treated as the primary P0 blocker.
      Result: `P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md` now records the 2026-04-13 shortcut batch,
      the repeat-semantic test floor, and the narrower remaining P0 backlog.
- [x] Promote a launched first-open authoring proof for the generic/default IMUI path.
      Result: `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json`
      proves command palette, declarative, GenUI, and IMUI triggers all dispatch the same typed
      action into one view-local state path; `tools/diag_gate_action_first_authoring_v1.py --only
      cookbook-imui-action-basics-cross-frontend` runs that proof without the broader action-first
      gate set.

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

- [x] Keep pure teaching-surface cleanup out of this umbrella unless it becomes the dominant
      remaining P0 pressure.
      Result: the remaining P0 backlog no longer reads as teaching-surface cleanup first, so no
      dedicated authoring-lane follow-on is warranted yet.
- [x] If further P0 work becomes mostly immediate convenience breadth
      (key ownership, item-status lifecycle, richer collection/pane proof), split a narrow follow-on
      instead of widening this umbrella folder.
      Result: `docs/workstreams/imui-response-status-lifecycle-v1/` now owns the narrow
      `ResponseExt` lifecycle vocabulary slice,
      `docs/workstreams/imui-key-owner-surface-v1/` now records the closed key-owner /
      item-local shortcut ownership follow-on with
      `M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md` plus
      `CLOSEOUT_AUDIT_2026-04-21.md`, so the current helper-local
      `activate_shortcut` + command-metadata seams remain the shipped answer until stronger
      first-party proof warrants a different narrow lane, and
      `docs/workstreams/imui-collection-pane-proof-v1/` now records the closed collection-first /
      pane-first proof pair with a no-helper-widening verdict, while this umbrella keeps phase
      ordering and the remaining cross-phase backlog read.
- [x] If further P0/P1 pressure becomes mostly shared IMUI control affordance and compact field
      behavior, split a narrow control-surface follow-on instead of turning showcase cleanup into
      the umbrella lane's implementation log.
      Result: `docs/workstreams/imui-control-chrome-fearless-refactor-v1/` now owns the shared
      `fret-ui-kit::imui` control-chrome rewrite for button/switch/slider/combo/input defaults,
      while this umbrella keeps the higher-level product-closure ordering.
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
- [x] If the remaining P0 pressure becomes broader menu/submenu/tab policy depth instead of
      helper-owned outward response shape, split another narrow follow-on instead of reopening the
      closed response lanes.
      Result: `docs/workstreams/imui-menu-tab-policy-depth-v1/` now owns the current hover-switch /
      submenu grace / tab ownership audit, keeping response-surface naming, key ownership,
      collection breadth, shell helpers, and runtime widening in their separate lanes.
- [x] If the remaining P1 pressure becomes `BeginChild()`-scale child-region depth instead of
      proof breadth, split another narrow follow-on instead of reopening the closed
      collection/pane lane.
      Result: `docs/workstreams/imui-child-region-depth-v1/` now records the closed child-region
      depth verdict: the bounded `ChildRegionChrome::{Framed, Bare}` slice is landed, while
      pane-proof breadth, shell-helper promotion, menu/tab policy, and runtime widening remain in
      their separate lanes.
- [x] If the remaining collection depth becomes narrower background marquee / box-select proof
      rather than generic helper widening, split another narrow follow-on and keep it app-owned
      until the frozen proof budget is satisfied.
      Result: `docs/workstreams/imui-collection-box-select-v1/` now records the closed
      background-only box-select slice in `imui_editor_proof_demo`, keeps lasso /
      keyboard-owner depth and shared helper growth out of generic `fret-ui-kit::imui`, and
      leaves broader collection depth to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned keyboard-owner proof rather than
      a reopened generic key-owner or helper-widening question, split another narrow follow-on and
      keep it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-keyboard-owner-v1/` now records the closed
      app-owned collection keyboard-owner slice in `imui_editor_proof_demo`, keeps the generic
      key-owner verdict closed, and leaves lasso / action semantics / shared helper growth to
      future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned delete-selected semantics rather
      than broader collection command breadth or helper growth, split another narrow follow-on and
      keep it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-delete-action-v1/` now records the closed
      app-owned collection delete-selected slice in `imui_editor_proof_demo`, keeps select-all /
      rename / context-menu breadth and shared helper growth out of generic `fret-ui-kit::imui`,
      and leaves lasso / second-proof-surface questions to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned context-menu quick actions
      rather than broader collection command breadth or helper growth, split another narrow
      follow-on and keep it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-context-menu-v1/` now records the closed
      app-owned collection context-menu slice in `imui_editor_proof_demo`, keeps select-all /
      rename / broader command breadth and shared helper growth out of generic `fret-ui-kit::imui`,
      and leaves lasso / second-proof-surface questions to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned zoom/layout depth rather than
      broader collection command breadth or helper growth, split another narrow follow-on and keep
      it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-zoom-v1/` now records the closed app-owned collection zoom/layout slice in `imui_editor_proof_demo`, replaces the frozen column count with viewport-plus-zoom-derived layout metrics, keeps select-all / rename / second-proof-surface / shared helper growth out of generic `fret-ui-kit::imui`, and leaves broader collection product depth to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned select-all breadth rather than
      broader collection command breadth or helper growth, split another narrow follow-on and keep
      it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-select-all-v1/` now records the closed app-owned collection select-all slice in `imui_editor_proof_demo`, routes Primary+A through the existing collection-scope owner, keeps rename / second-proof-surface / shared helper growth out of generic `fret-ui-kit::imui`, and leaves broader collection product depth to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned rename breadth rather than
      broader collection command breadth or helper growth, split another narrow follow-on and keep
      it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-rename-v1/` now records the closed app-owned collection rename slice in `imui_editor_proof_demo`, routes F2 plus the existing context-menu entry through one rename modal, keeps second-proof-surface / shared helper growth out of generic `fret-ui-kit::imui`, and leaves broader collection product depth to future narrower follow-ons.
- [x] If the remaining collection depth becomes narrower app-owned inline rename posture rather than
      broader collection command breadth or helper growth, split another narrow follow-on and keep
      it local to the existing proof surface.
      Result: `docs/workstreams/imui-collection-inline-rename-v1/` now records the closed app-owned collection inline rename slice in `imui_editor_proof_demo`, routes F2 plus the existing context-menu entry through one inline editor mounted inside the active asset tile, keeps second-proof-surface / shared helper growth out of generic `fret-ui-kit::imui`, and leaves broader collection product depth to future narrower follow-ons.
- [x] If the collection-first proof starts carrying too much app-owned implementation in one host file,
      split a narrow demo-local modularization follow-on before arguing for shared helpers from
      maintenance pressure alone.
      Result: `docs/workstreams/imui-editor-proof-collection-modularization-v1/` now records the closed demo-local collection module slice in `imui_editor_proof_demo`, moves collection assets/models/render/unit tests into `collection.rs`, keeps the host on `mod collection;` plus one render call and drag-asset delegation, and reset the default next non-multi-window priority to broader app-owned command-package breadth before that command-package lane later closed.
- [x] After the inline rename closeout lands, refresh the next non-multi-window IMUI follow-on
      order instead of reopening older collection, key-owner, or generic helper lanes by habit.
      Result: `P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md` now freezes the current order as
      closed app-owned collection command-package breadth first, second proof-surface promotion
      next, and only later any reconsideration of shared helper growth, while child-region resize,
      submenu-intent tuning, key-owner reopening, and generic helper widening stay explicitly deferred.
- [x] Start the broader app-owned collection command-package lane locally on the same proof surface
      instead of inventing another generic helper or reopening the structural modularization folder.
      Result: `docs/workstreams/imui-collection-command-package-v1/` now records the closed
      command-package lane, lands duplicate-selected plus explicit rename-trigger slices in
      `imui_editor_proof_demo/collection.rs`, keeps those routes app-owned on the existing
      keyboard/button/context-menu owner paths, rejects a third command verb in this folder, and
      moves the next non-multi-window priority to a second proof surface.
- [x] After the command-package closeout lands, start and close the second proof-surface follow-on instead of
      reopening the closed package or widening shared helpers from one proof.
      Result: `docs/workstreams/imui-collection-second-proof-surface-v1/` now records the closed
      follow-on, names `editor_notes_demo.rs` as the primary shell-mounted candidate, keeps
      `workspace_shell_demo.rs` as supporting evidence, lands the `Scene collection` left-rail
      surface in `editor_notes_demo.rs`, and closes on a no-helper-widening verdict because the two
      collection proof surfaces do not yet need the same shared helper.
- [x] If P1 becomes mostly shell composition work, split it into a narrow workbench-shell follow-on.
      Result: `docs/workstreams/imui-workbench-shell-closure-v1/` now records the narrow P1 shell
      closure decision and already closes on a no-new-helper-yet verdict, leaving this umbrella
      focused on phase ordering and cross-phase status.
- [x] Keep future diagnostics/devtools productization out of this umbrella unless fresh P2 pressure
      becomes implementation-heavy again.
      Result: P2 is closed in this lane; any future tooling UX/productization should start as a
      narrow devtools follow-on instead of widening this folder.
- [x] Record the identity-warning diagnostics/browser chain as a closed P2 evidence branch.
      Result: the structured identity diagnostics, browser query model, offline HTML sidecar,
      structural smoke gate, and committed sample bundle all live in narrow closed follow-ons, while
      this umbrella records them as part of the first-open diagnostics path.
- [x] If P3 becomes mostly platform diagnostics and runner work, continue using the existing docking
      parity lane or start a narrower follow-on there instead of bloating this folder.
      Result: after the P1 shell closeout and the umbrella maintenance refresh, the active
      execution priority continues in `docs/workstreams/docking-multiwindow-imgui-parity/`, with
      `WORKSTREAM.json` and `M0_BASELINE_AUDIT_2026-04-13.md` as the first-open resume surface.
