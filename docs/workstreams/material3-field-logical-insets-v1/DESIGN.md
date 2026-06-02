# Material3 Field Logical Insets v1

Status: Closed
Last updated: 2026-05-30

## Problem

TextField and Select resolve visual field geometry with Material-specific helpers, but their floating
label and supporting text placement still used physical `left` / `right` insets and margins. Under
RTL, inline-start should map to the physical right edge and inline-end should map to the physical
left edge.

## Scope

- Extend the Material3 logical edge helper with inline-start inset and inline-start/end margins.
- Migrate TextField floating label and supporting text.
- Migrate Select trigger label and supporting text.
- Add focused RTL geometry tests for TextField and Select.

## Non-Goals

- Do not migrate full input-row icon ordering in this lane.
- Do not change Select popup/listbox RTL behavior.
- Do not alter field-family state ownership, floating-label timing, or overlay choreography.

## Source Ordering

- Material spec: logical field geometry intent.
- Compose Material3: field-family toolkit behavior and composition-local direction expectations.
- MUI: web field composition sanity check.
- Fret-side truth: existing Material3 field helpers and diagnostics tests.

## Parity Proof Note

- Truth: In RTL, field label/supporting text inline-start offsets are measured from the physical
  right edge, while inline-end guard space is on the physical left edge.
- Artifacts: logical edge helper extension, TextField/Select migration, focused diagnostics tests.
- Wiring: TextField/Select resolve Material layout direction and pass it into label/supporting text
  geometry helpers.
- Proof: RTL TextField and Select tests assert label/supporting text are closer to the physical
  right edge than the left edge.
- Residual risk: input icon row order and Select popup/listbox RTL remain separate follow-ons.
