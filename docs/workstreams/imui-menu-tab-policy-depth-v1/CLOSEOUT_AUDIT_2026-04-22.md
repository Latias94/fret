# Closeout Audit — 2026-04-22

This audit records the final closeout read for `imui-menu-tab-policy-depth-v1`.

Goal:

- verify whether generic IMUI still owns an active menu/tab policy-depth implementation queue,
- separate the landed generic floor from pressures that no longer justify shared helper growth,
- and decide whether this lane should remain active or close on a no-new-generic-surface verdict.

## Audited evidence

Lane docs:

- `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/TODO.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/MILESTONES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_TAB_OWNER_VERDICT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_ACTIVE_MENUBAR_MNEMONIC_ROVING_OWNER_VERDICT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_REVERSE_DIRECTION_FOCUS_OWNER_VERDICT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_REVERSE_DIRECTION_FOCUS_HANDOFF_SLICE_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_SUBMENU_GRACE_CORRIDOR_PROOF_SLICE_2026-04-22.md`

Implementation / proof anchors:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/sub_trigger.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/sub.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/pointer_grace_intent.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`

Upstream comparison:

- `repo-ref/imgui/imgui_widgets.cpp`
- `repo-ref/imgui/imgui_demo.cpp`

Validation run used for closeout:

- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-menu-tab-policy-depth-v1/WORKSTREAM.json > /dev/null`
- `git diff --check`

## Findings

### 1. The generic IMUI menu floor is now complete enough for the contract this lane set out to own

Generic IMUI now has a coherent shipped floor for menu/submenu policy depth:

- top-level menubar hover-switch,
- keyboard-open on focused triggers,
- in-menu left/right top-level switching,
- reverse-direction same-frame focus handoff,
- submenu hover-open,
- sibling submenu hover-switch deferral inside the grace polygon,
- and safe-corridor close-timer cancellation while the pointer moves toward the child submenu.

This is no longer a primitive-only claim:

- focused `fret-imui` tests now lock the sibling-trigger path and the submenu-side void corridor
  path end-to-end,
- the IMUI helper stack no longer bypasses primitive submenu hover/grace ownership,
- and the current first-party showcase/proof surfaces now match the shipped helper contract.

Conclusion:

- there is no active generic IMUI submenu grace implementation gap left on this lane.

### 2. The strongest remaining non-generic pressures already have other outcomes

The broader pressures that looked adjacent at lane start are now already resolved elsewhere:

- editor-grade tab overflow / reorder / close stays in `fret-workspace`,
- outer-scope active-menubar mnemonic / roving posture stays shell-owned,
- reverse-direction top-level focus arbitration is now landed,
- and key-owner / shortcut-surface growth remains closed by separate verdict.

So the lane no longer has a mixed backlog of unfinished generic work.

Conclusion:

- keeping this lane active would no longer protect a real implementation queue; it would only keep
  historical questions artificially "open".

### 3. Dear ImGui no longer creates enough pressure for more generic submenu-intent growth here

The local `repo-ref/imgui` snapshot still shows the same essential submenu-intent outcomes in
`BeginMenuEx(...)`:

- defer switching while moving toward the child submenu,
- avoid premature close while moving through safe corridor space,
- and keep hover-open responsive.

Those outcomes are now already represented in Fret's generic IMUI floor at the level this repo
actually contracts and tests.

What remains after that is micro-tuning:

- different polygon slack,
- different timer constants,
- or future optional heuristics that do not yet have a first-party generic consumer asking for
  a new contract.

That is not enough evidence to widen `fret-ui-kit::imui` again.

Conclusion:

- the correct outcome is a no-new-generic-surface verdict, not another growth slice.

### 4. The next meaningful IMUI work should move to other owners or new narrower follow-ons

The refreshed parity audit now points elsewhere for the bigger remaining "Imgui-class" gaps:

- runner/backend multi-window closure,
- richer child-region depth,
- broader immediate collection breadth,
- or stronger first-party proof for other immediate convenience surfaces.

If a future product-surface repro demonstrates a submenu-intent gap that the current generic floor
cannot satisfy, that should start a new narrower follow-on with that exact repro and gate set.

Conclusion:

- this lane no longer owns the next best use of effort.

## Decision from this audit

Treat `imui-menu-tab-policy-depth-v1` as:

- closed for the generic menu/tab policy-depth goal,
- historical evidence for the admitted generic IMUI floor,
- and reopenable only through a narrower follow-on if fresh first-party evidence proves the current
  submenu floor is insufficient.

## Immediate execution consequence

From this point forward:

1. keep the current generic IMUI menu floor stable,
2. do not reopen this lane just to experiment with more submenu heuristics,
3. keep editor-grade tabs in `fret-workspace` and outer-scope menubar mnemonic posture in shell
   owners,
4. start a new narrower follow-on only if a fresh first-party repro proves the current submenu
   floor cannot satisfy a real generic consumer,
5. otherwise spend the next IMUI/editor-feel budget on runner/backend closure, child-region depth,
   collection breadth, or other separately justified proof surfaces.
