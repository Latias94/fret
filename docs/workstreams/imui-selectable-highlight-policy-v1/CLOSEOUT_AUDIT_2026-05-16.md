# ImUi Selectable Highlight Policy Closeout Audit - 2026-05-16

Status: closed

## Shipped Outcome

`fret-ui-kit::imui` now exposes a small selectable highlight policy:

- `SelectableOptions::highlighted` renders an enabled unselected row with hover-style emphasis.
- Highlighted does not change `selected`, accessibility selected semantics, focusability, click
  response, or popup close behavior.
- Selected rows keep selected styling even when highlighted.
- Disabled highlighted rows remain muted and do not gain hover-style background.
- The input-text picker recipe now uses `selected: checked` and `highlighted: active`, so keyboard
  navigation no longer reports the active candidate as selected.

This keeps the Dear ImGui parity point in the policy/component layer and leaves `fret-imui` and
runtime contracts unchanged.

## Closed Scope

This lane intentionally did not add:

- a selectable flags enum mirror,
- span-all-columns or overlap behavior,
- select-on-nav behavior,
- list-box chrome,
- multi-select request/IO vocabulary.

Those remain proof-led follow-ons if a concrete surface repeats the same behavior tax.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_selectable_smoke.rs`
- `docs/workstreams/imui-selectable-highlight-policy-v1/EVIDENCE_AND_GATES.md`

## Gate Result

See `EVIDENCE_AND_GATES.md` for the canonical command set and local run results.
