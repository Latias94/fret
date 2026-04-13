# ImUi Editor-Grade Product Closure v1 - Milestones

Status: active execution lane
Last updated: 2026-04-13

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why a new follow-on is warranted instead of reopening older `imui`
  closeout lanes,
- the remaining maturity gap is split into P0/P1/P2/P3,
- and each phase names its current proof family.

Primary evidence:

- `M0_BASELINE_AUDIT_2026-04-12.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`

Current status:

- Closed on 2026-04-12 via `M0_BASELINE_AUDIT_2026-04-12.md`.

## M1 - P0 default authoring lane closure

Exit criteria:

- one first-party default immediate authoring path is named and taught consistently,
- the path explains stable identity, layout defaults, and focus/hover expectations without
  depending on runtime widening,
- and the minimum proof budget for future `fret-ui-kit::imui` helper widening is explicitly frozen
  as the golden pair rather than inferred from ad hoc example pressure.

Primary evidence:

- `DESIGN.md`
- `P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
- `P0_FOOTGUN_AUDIT_2026-04-12.md`
- `P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `P0_ROOT_HOSTING_RULE_2026-04-12.md`
- `P0_STABLE_IDENTITY_RULE_2026-04-12.md`
- `P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

Current status:

- In progress.
- The generic/editor golden pair is now frozen as:
  `apps/fret-cookbook/examples/imui_action_basics.rs` +
  `apps/fret-examples/src/imui_editor_proof_demo.rs`.
- Future `fret-ui-kit::imui` public helper widening now has an explicit minimum proof budget:
  it must name two real first-party proof surfaces, and the current budget floor is the frozen
  golden pair rather than any single reference demo.
- The first-open mounting rule is now explicit:
  nested layout host -> `fret_imui::imui(cx, ...)`,
  root/non-layout parent -> `fret_imui::imui_vstack(cx.elements(), ...)`.
- The first-open stable-identity rule is now explicit:
  `ui.for_each_unkeyed(...)` is only for static/order-stable lists, while dynamic collections
  should default to `ui.for_each_keyed(...)` or `ui.id(key, ...)`.
- The current footgun audit concludes that documentation and proof-selection dominate; the only
  credible helper-shape candidate is a narrow app-lane root-host helper.
- The demote/delete plan is now frozen:
  `imui_hello_demo` is smoke/reference, public docs name the golden pair explicitly, and the
  source-policy gates distinguish golden/reference/compatibility roles.
- A 2026-04-13 status pass now records that focused item-local shortcut seams materially improved
  across direct pressables, popup/menu triggers, and combo/combo-model triggers without widening
  the runtime's global shortcut owner model.
- The tested shortcut repeat rule is now explicit at the immediate layer:
  default key repeat does not retrigger activation, and `shortcut_repeat=true` is the opt-in seam.
- The remaining credible P0 backlog is now narrower:
  broader multi-select proof, deeper child-region/menu/tab policy, item-status lifecycle
  vocabulary, and any eventual key-owner surface.
- Execution consequence:
  keep this lane as the umbrella recorder, but split any future implementation-heavy P0 work into a
  narrow follow-on instead of turning M1 back into a generic helper-growth backlog.

## M2 - P1 editor workbench shell closure

Exit criteria:

- the repo can point to one coherent editor/workbench proof rather than isolated shell examples,
- workspace shell, docking, and editor composites are mapped to explicit owners,
- and the default proof set is reviewable as one system.

Primary evidence:

- `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`

Current status:

- In progress.
- The primary P1 workbench-shell proof is now frozen as
  `apps/fret-examples/src/workspace_shell_demo.rs`.
- The minimal shell-mounted editor-rail proof is now frozen as
  `apps/fret-examples/src/editor_notes_demo.rs`.
- `apps/fret-examples/src/imui_editor_proof_demo.rs` remains supporting docking/editor evidence,
  but is no longer the default workbench-shell reading order.
- The current owner split is now explicit:
  `fret-workspace` for shell slots/tabstrip/command scope,
  `fret-docking` for dock choreography,
  `fret-ui-editor` for editor composites,
  and app/example ownership for scene-local center content.
- The promoted P1 shell diagnostics floor is now explicit:
  `diag-hardening-smoke-workspace` remains the launched shell suite, and its frozen minimum now
  spans tab close/reorder/split preview, dirty-close prompt, Escape focus restore, and file-tree
  keep-alive.
- P1 is now split for implementation-heavy work:
  `docs/workstreams/imui-workbench-shell-closure-v1/` owns the active default-workbench shell
  closure follow-on, while this umbrella keeps the phase ordering and status record.
  Update on 2026-04-13:
  that follow-on already closed on a no-new-helper-yet verdict for promoted first-party shell
  helpers, so future active execution should no longer treat P1 shell composition as the default
  open lane.

## M3 - P2 unified diagnostics/devtools surface

Exit criteria:

- one first-open developer loop ties together inspect, selectors, scripts, bundles, and compare,
- GUI, CLI, and MCP are explicitly shown as consumers of the same artifact contract,
- and the lane names one bounded devtools smoke/gate package.

Primary evidence:

- `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
- `P2_DISCOVERABILITY_ENTRY_2026-04-12.md`
- `docs/diagnostics-first-open.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `apps/fret-devtools/src/main.rs`
- `apps/fret-devtools-mcp/src/main.rs`

Current status:

- In progress.
- The first-open P2 developer path is now explicit:
  inspect/pick -> script -> bundle -> compare starts from the CLI-compatible diagnostics contract,
  while DevTools GUI and MCP stay thin consumers of the same artifacts root.
- The compare story is now explicit:
  direct bundle/session diff uses `diag compare`, while aggregate run-set comparison flows through
  `diag summarize` plus shared `regression.summary.json` / `regression.index.json` consumers.
- The owner split is now explicit:
  `fret-bootstrap` owns runtime capture/export, `fret-diag` owns shared orchestration and artifact
  projections, `fret-devtools` owns GUI UX over those contracts, and `fret-devtools-mcp` owns the
  headless automation/resource adapter over the same contracts.
- The bounded devtools smoke package is now explicit:
  `python3 tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke`
  now freezes one repo-owned gate over `tools/diag-scripts/tooling/todo/todo-baseline.json` and
  `tools/diag-campaigns/devtools-first-open-smoke.json`.
- The direct half of that gate now proves:
  real `diag run`, named bundle emission, latest-bundle resolution through
  `script.result.json:last_bundle_dir`, and direct `diag compare` over a state-changing pair.
- The aggregate half of that gate now proves:
  one campaign root with `campaign.manifest.json`, explicit root `diag summarize`,
  `regression.summary.json`, `regression.index.json`, and successful `diag dashboard` projection.
- The discoverability entry is now explicit:
  `docs/diagnostics-first-open.md` is the canonical first-open diagnostics workflow, and the
  inspect, bundles/scripts, GUI dogfood, and diagnostics-v2 navigation docs are now explicit
  branch/reference notes instead of competing start pages.
- P2 is now closed for this lane.
  Future diagnostics/devtools work that becomes implementation-heavy should move into a narrower
  devtools follow-on instead of widening this folder.

## M4 - P3 multi-window hand-feel closure

Exit criteria:

- hovered-window, peek-behind, transparent payload, and mixed-DPI follow-drag responsibilities stay
  runner-owned,
- the current parity matrix and proof package are concise enough to reopen quickly,
- and no `imui` helper growth is used as a workaround for runner/backend gaps.

Primary evidence:

- `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Current status:

- In progress.
- The first P3 checklist is now explicit:
  `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md` freezes hovered-window, peek-behind,
  transparent payload, and mixed-DPI follow-drag as the runner-owned parity budget for this lane.
- The owner split is now explicit:
  `crates/fret-launch`, runner/backend integrations, and `ecosystem/fret-docking` stay the default
  owners, while `crates/fret-ui` and generic `imui` helpers remain out of scope unless stronger
  evidence reopens them.
- The bounded parity package is now explicit:
  `tools/diag-campaigns/imui-p3-multiwindow-parity.json` now binds four repo-owned scripts into one
  lane-owned package over `docking_arbitration_demo`, and keeps `diag-hardening-smoke-docking`
  small instead of overloading it with all P3 stress coverage.
- P3 is now the next active execution priority after the P1 shell closeout.
  Continue multi-window implementation-heavy work in the existing docking parity lane or a narrower
  runner follow-on instead of widening this folder.

## M5 - Narrow follow-ons or closeout

Exit criteria:

- the lane either closes with explicit owner splits and reference links,
- or splits into narrower follow-ons for the implementation-heavy phases that genuinely need their
  own execution folders.

Primary evidence:

- `WORKSTREAM.json`
- `TODO.md`
- phase-specific follow-on lanes created after M1/M2/M3/M4 evidence is strong enough

Current status:

- Not started.
