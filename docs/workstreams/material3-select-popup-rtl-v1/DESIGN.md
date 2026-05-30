# Material3 Select Popup RTL v1

Status: Closed
Last updated: 2026-05-30

## Problem

The previous field logical inset lane aligned Select trigger label and supporting text, but the
popup/listbox path still needs independent RTL proof. Popper `Start` alignment must mean logical
inline-start, and listbox option rows must place leading/trailing visual slots according to layout
direction.

## Scope

- Resolve Select popup placement from the Material layout direction.
- Keep Select listbox text and option row visual slots under the same resolved direction.
- Add stable option label/supporting text test ids only where needed for diagnostics.
- Add focused RTL tests for popup anchoring and option row leading/trailing placement.

## Non-Goals

- Do not refactor Select state ownership, typeahead, focus restore, or overlay dismissal.
- Do not change trigger input-row icon order in this lane.
- Do not migrate ExposedDropdown or Autocomplete here.
- Do not promote Material-specific helpers into shadcn or core crates.

## Source Ordering

- Material spec: logical field/listbox direction intent.
- Compose Material3: toolkit-style direction resolution and list item slot expectations.
- MUI: web Select/Menu composition sanity check.
- Fret-side exemplar: shadcn Select and `fret-ui-kit` popper/listbox direction patterns.

## Parity Proof Note

- Truth: In RTL, `SelectMenuAlign::Start` anchors the popup's physical right edge to the trigger's
  physical right edge when the menu is wider than the trigger.
- Truth: In RTL, a listbox option's leading icon appears on the physical right and trailing icon on
  the physical left.
- Artifacts: Select recipe direction wiring, stable option part ids, focused diagnostics tests.
- Wiring: The resolved Material layout direction drives popper placement and listbox row assembly.
- Proof: RTL popup/listbox tests fail if Select falls back to LTR physical left/right behavior.
- Residual risk: trigger input-row icon order and field-family popup reuse stay as follow-ons.
