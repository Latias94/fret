# P0 Immediate Parity Status - 2026-04-13

Status: Historical reference (partially superseded by later narrow P0 follow-ons)
Last updated: 2026-04-22

Status note (2026-04-22): this document remains useful as the original post-shortcut-batch P0
snapshot, but the current shipped P0 state now lives in later narrow follow-ons:
`docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`,
`docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`,
`docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`,
`docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`, and
`docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`. References below to the remaining P0
gap around collection/pane proof breadth, key-owner surface growth, or richer menu/tab policy
should therefore be read as the 2026-04-13 snapshot, not the current shipped verdict.

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

## Why this note exists

The active product-closure lane already owns the ordered P0/P1/P2/P3 story.
This note records the latest P0 evidence after the 2026-04-13 immediate-mode shortcut/parity
batch, so future work does not reopen generic helper growth based on stale assumptions.

This is intentionally a status note inside the existing umbrella lane, not a new broader
workstream.

## Assumptions-first read

### 1) The recent shortcut batch should be read as ecosystem-layer convenience closure

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `ecosystem/fret-ui-kit/src/imui/options.rs`
  - `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the repo would misclassify a policy-layer convenience pass as another runtime-surface reopen.

### 2) Focused item-local shortcut seams are no longer the main P0 blocker

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `ecosystem/fret-imui/src/tests/interaction_shortcuts.rs`
  - `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
  - `ecosystem/fret-imui/src/tests/models_controls.rs`
  - `ecosystem/fret-imui/src/tests/models_combo.rs`
  - `ecosystem/fret-imui/src/tests/popup_hover.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane would keep expanding shortcut-shaped helpers even though the remaining pressure has
    moved to proof breadth, key ownership, and product closure.

### 3) Further P0 work should split once it becomes implementation-heavy

- Evidence:
  - `DESIGN.md`
  - `TODO.md`
  - `MILESTONES.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - this folder would drift into a second generic `imui` backlog instead of staying an umbrella
    execution lane.

## Findings

### 1) Focused item-local shortcut coverage is materially better than the old P0 picture

The recent batch now covers focused activation shortcuts on these immediate surfaces:

- direct pressables: button, selectable, switch, checkbox, disclosure, tab item,
- popup/menu family: menu item, menu trigger, submenu trigger,
- combo family: combo trigger and model-backed combo trigger.

The important owner split stayed intact:

- the work landed in `fret-ui-kit::imui` and `fret-imui` tests,
- existing control-local handlers were reused where possible,
- and `crates/fret-ui` did not gain a new global shortcut owner model.

This should be read as "better immediate convenience breadth," not "runtime shortcut design is now
reopened."

### 2) Repeat semantics are now explicit instead of implicit folklore

The current tested rule is:

- `shortcut_repeat = false` means repeated keydown events do not retrigger activation,
- `shortcut_repeat = true` opts into repeated activation.

This is now locked by focused tests across direct pressables, popup items, menu/submenu triggers,
and combo/combo-model triggers.

### 3) The test language is converging on reusable shortcut helpers

`fret-imui` test helpers now provide a shared vocabulary for:

- control-modifier setup,
- repeated shortcut keydown,
- focus lookup by `test_id`,
- and advancing/running a frame around the interaction.

That matters because the remaining P0 discussion is now easier to review as behavior, not as a
large amount of hand-written harness boilerplate.

### 4) The remaining P0 gap has moved up one level

The current parity audit and test surface now point to a narrower remaining P0 backlog:

- broader first-party proof for multi-select collections,
- deeper child-region and pane composition proof,
- richer menu/tab policy depth,
- item-status lifecycle vocabulary in `ResponseExt`,
- and any eventual key-owner surface beyond focused item-local shortcuts.

The largest maturity gap is still not here.
The biggest remaining product gap is the combination of:

- P1 workbench shell closure,
- and P3 runner/backend multi-window hand-feel closure.

## Execution consequence

Use `imui-editor-grade-product-closure-v1` as the umbrella recorder.
Do not start an even larger parent workstream just to collect status.

From this note forward:

1. read focused shortcut seams as "recent P0 progress already recorded,"
2. keep `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md` as the detailed parity read,
3. prefer P1/P3 work when choosing the next product-closure slice,
4. and only start a new P0 follow-on if the next work is implementation-heavy around one narrow
   topic such as key ownership, item-status lifecycle, or collection/pane proof breadth.

## Suggested gate package for this status

- `cargo nextest run -p fret-imui`
- `git diff --check`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
