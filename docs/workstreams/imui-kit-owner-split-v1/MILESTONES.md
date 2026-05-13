# ImUi Kit Owner Split v1 - Milestones

Status: closed
Last updated: 2026-05-13

## M0 - Baseline

Status: complete

Done when:

- the workstream exists with `WORKSTREAM.json`, `DESIGN.md`, `TODO.md`, `MILESTONES.md`, and
  `EVIDENCE_AND_GATES.md`;
- `M0_BASELINE_AUDIT_2026-05-13.md` records the source-size and owner-risk baseline;
- baseline docs/source gates pass.

Result (2026-05-13): baseline docs/source gates, focused `fret-ui-kit` IMUI nextest smoke,
`cargo check`, format check, and `git diff --check` passed.

## M1 - First Private Facade Owner Split

Status: complete

Done when:

- one coherent private implementation cluster is moved out of `facade_writer.rs`;
- public `ImUiFacade` method names and `fret::imui` re-export paths stay unchanged;
- focused `fret-ui-kit` IMUI smoke gates pass;
- the slice has a dated M-note.

Result (2026-05-13): `M1_BUTTON_ACTIONS_SLICE_2026-05-13.md` moved the
button/action/command-button inherent `ImUiFacade` wrappers into private
`facade_writer/button_actions.rs`, with focused `fret-ui-kit` check and nextest smoke passing.

## M2 - Response Status Path Verdict

Status: complete

Done when:

- the response/status transient path is audited across `facade_support.rs`,
  `interaction_runtime/*`, and `response/hover.rs`;
- either a private typed helper lands or a no-change verdict is recorded;
- no runtime contract or public response API is widened.

Result (2026-05-13): `M2_PRESSABLE_RESPONSE_ASSEMBLY_SLICE_2026-05-13.md` landed a shared
`interaction_runtime::populate_pressable_response(...)` helper so `active_trigger_behavior.rs`,
`item_behavior.rs`, and `slider_controls.rs` stop re-assembling the same pressable response core
by hand. Public IMUI names and `crates/fret-ui` runtime contracts stayed unchanged.

## M3 - Menu Facade Owner Split

Status: complete

Done when:

- the menu item / begin-menu inherent facade wrappers move behind a private owner module;
- public `ImUiFacade` method names and `fret::imui` paths stay unchanged;
- focused `fret-ui-kit` IMUI gates pass;
- the slice has a dated M-note.

Result (2026-05-13): `M3_MENU_ITEMS_FACADE_OWNER_SPLIT_2026-05-13.md` moved the menu item,
begin-menu, begin-submenu, and command-menu inherent `ImUiFacade` wrappers into private
`facade_writer/menu_items.rs`. `facade_writer.rs` dropped from 1687 to 1582 lines, and
`menu_items.rs` is 109 lines. Public IMUI names, `fret::imui` re-export paths, `fret-imui`, and
`crates/fret-ui` runtime contracts stayed unchanged.

## M4 - Selection / Combo Facade Owner Split

Status: complete

Done when:

- selectable, multi-selectable, and combo inherent facade wrappers move behind a private owner
  module;
- public `ImUiFacade` method names and `fret::imui` paths stay unchanged;
- focused `fret-ui-kit` IMUI gates pass;
- the slice has a dated M-note.

Result (2026-05-13): `M4_SELECTION_COMBO_FACADE_OWNER_SPLIT_2026-05-13.md` moved selectable,
multi-selectable, and combo inherent `ImUiFacade` wrappers into private
`facade_writer/selection_combo.rs`. `facade_writer.rs` dropped from 1582 to 1506 lines, and
`selection_combo.rs` is 80 lines. Public IMUI names, `fret::imui` re-export paths, `fret-imui`,
and `crates/fret-ui` runtime contracts stayed unchanged.

## M5 - Closeout Or Next Follow-On Split

Status: complete

Done when:

- this lane either closes with the structural hazard reduced, or names the next narrower follow-on;
- `WORKSTREAM.json`, `TODO.md`, and `EVIDENCE_AND_GATES.md` reflect the current state;
- closeout clearly states what remains out of scope.

Result (2026-05-13): `CLOSEOUT_AUDIT_2026-05-13.md` closes this lane. The next narrower follow-on
is `imui-facade-disclosure-owner-split-v1` for disclosure wrappers such as `collapsing_header` and
`tree_node`; additive widgets, docking, multi-window, debug draw feature growth, and runtime
contract changes remain out of scope.
