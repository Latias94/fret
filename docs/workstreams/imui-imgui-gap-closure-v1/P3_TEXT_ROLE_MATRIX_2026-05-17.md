---
title: P3 Text Role Matrix
status: contract slice; no new runtime API
date: 2026-05-17
scope: imui, editor, text, resize
---

# P3 Text Role Matrix

This note freezes the current text-semantics answer for the resize class of bugs where text wraps
inside fixed-height editor rows, then paints past the row bottom. The fix direction is not to add a
new `TextProps` helper for every component. The fix direction is a small role vocabulary with
explicit wrapping and overflow contracts, plus source gates that keep controls from constructing
ad-hoc text policy.

## Ownership

- `fret-ui` remains the mechanism layer: text layout, wrapping, overflow, ink, measurement, and
  rendering primitives.
- `fret-ui-kit::declarative::text` owns shared app/component text roles that are not editor-specific.
- `fret-ui-kit::imui` consumes those roles for immediate-mode control surfaces.
- `fret-ui-editor` owns editor-density and editor-chrome text roles in its primitive layer.
- `fret-imui` stays policy-light and does not gain a mutable text/style stack.

## Stable Base Roles

| Role | Shared helper | Default resize contract | Primary use |
| --- | --- | --- | --- |
| Control readout | `text_control_readout(...)` | Single line, shrinkable, `min-width: 0`, muted, ellipsis. | Toolbar/status/value readouts, menu shortcuts, compact auxiliary values. |
| Button label | `text_button_label(...)` | Single line, shrinkable, medium weight, ellipsis. | Buttons and button-like triggers. |
| Paragraph | `text_paragraph(...)` / `text_paragraph_break_words(...)` | Wrapping body copy; parent layout must account for multi-line height. | Prose and explanatory copy outside fixed control rows. |
| Code text | `text_code_block(...)` plus `text_code_wrap(...)` for inline wrapping code | Code blocks are monospace and horizontal-scroll-friendly; inline code may wrap by grapheme. | Docs code blocks, code-like readouts, identifiers. |
| Table cell text | `text_table_cell(...)` | Single line, shrinkable, `min-width: 0`, ellipsis. | Header/body table cells unless a future multi-line row-height policy is explicit. |

These five names are the minimum stable vocabulary for future resize triage. If a new component text
case does not fit one of them, the first question is whether it is a derived role, not whether the
component should construct `TextProps` locally.

## Current Derived Roles

- `text_compact_paragraph(...)`: dense wrapping paragraph for editor/IMUI panels. It may grow
  height and must not be used inside fixed-height control chrome unless that parent measures
  multi-line height.
- `text_list_row_label(...)`: dense command/list/tree row labels. It is not a button label; it fills
  row width, shrinks to zero, and ellipsizes to keep row height stable.
- `text_control_label(...)`: fill-width checkbox/radio/switch/combo/slider label text. It keeps
  fixed control chrome single-line under resize.
- `text_section_chrome_label(...)`, `text_chrome_title(...)`, and `text_chrome_glyph(...)`: section,
  title-bar, and glyph chrome roles. They keep fixed chrome rows single-line, with ellipsis or clip
  according to the slot.
- Editor readout primitives in `fret-ui-editor/src/primitives/readout.rs`: editor-specific status
  badges, inline errors, validation messages, section labels, preview captions, tooltip readouts,
  inspector panel titles, property-row labels, and property chrome. Direct editor `TextProps`
  construction stays allowlisted to primitive owners.
- Editor popup/list primitives in `fret-ui-editor/src/primitives/popup_list.rs`: editor assist and
  popup row text roles. They remain editor-layer policy, not `fret-imui` runtime behavior.

## Triage Rules

1. If the text is a control label, readout, trigger label, list row, table cell, title, or glyph, it
   must not wrap by default. Use the matching single-line role and make the parent allow shrinkage
   with `min-width: 0` when the row is flexed.
2. If the text is paragraph, validation prose, or explanatory body copy, wrapping is allowed, but the
   parent must measure/grow for multiple lines. Painting a second line past a fixed row bottom is a
   layout bug, not an acceptable text role outcome.
3. If the text is code-like, choose between `text_code_block(...)` for scrollable block code and
   `text_code_wrap(...)` for inline code that can wrap long identifiers.
4. New direct `TextProps` construction under `fret-ui-kit::imui` or editor controls is a contract
   change. It must either move into a role helper or update the source gate with a documented owner
   and proof.
5. Do not add a public `TextRole` enum until at least two consumers need a data-driven role value.
   The current API remains helper-based to avoid freezing unnecessary public surface.
6. Remaining bare text in first-party proof apps is allowed only when the surface is itself testing
   text/input rendering behavior. Current allowed residuals are `components_gallery` text
   smoke/font override probes and `ime_smoke_demo` IME behavior instructions/status. Do not migrate
   those mechanically into chrome roles; add a new role only when a non-proof surface repeats the
   need.

## Gates

- Shared role behavior:
  - `cargo nextest run -p fret-ui-kit --features imui --lib control_readout_text_uses_muted_compact_single_line_truncation button_label_text_uses_medium_single_line_truncation prose_variants_and_code_wrap_install_semantic_inherited_overrides table_cell_text_uses_compact_single_line_truncation --no-fail-fast`
  - `cargo nextest run -p fret-ui-kit --features imui --lib base_single_line_text_roles_stay_single_line_under_narrow_layout paragraph_text_role_measures_multiple_lines_under_narrow_layout --no-fail-fast`
- IMUI consumers:
  - `cargo nextest run -p fret-ui-kit --features imui --lib imui_text_item_is_single_line_and_shrinkable imui_text_wrapped_is_explicit_wrapping_text compact_paragraph_text_uses_wrapping_fill_width_layout menu_item_shortcut_text_uses_shared_control_readout_role menu_item_label_text_uses_shared_list_row_text_role control_label_text_uses_fill_width_single_line_truncation --no-fail-fast`
- Editor consumers:
  - `cargo nextest run -p fret-ui-editor editor_input_value_text_is_single_line_and_shrinkable editor_inline_error_text_is_single_line_and_shrinkable editor_validation_message_text_wraps_and_shrinks popup_list_row_text_is_single_line_and_shrinkable editor_property_group_header_text_is_single_line_and_shrinkable editor_property_row_label_text_is_single_line_and_shrinkable editor_inspector_panel_title_text_is_single_line_and_shrinkable inspector_panel_title_stays_single_line_when_header_is_narrow --no-fail-fast`
- Source contract:
  - `python tools/gate_imui_workstream_source.py`
- First-party residual proof surface:
  - `cargo nextest run -p fret-examples --test text_role_residual_surface remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast`
- Layout-container regression:
  - `cargo nextest run -p fret-ui-editor row_label_slot_keeps_fixed_line_box_when_label_text_wraps_under_narrow_layout --no-fail-fast`
  - `cargo nextest run -p fret-ui-editor row_value_slot_keeps_overflow_visible_for_wrapping_value_children row_value_slot_grows_to_wrapping_value_text_under_narrow_layout property_grid_keeps_rows_separated_when_value_text_wraps_under_narrow_layout --no-fail-fast`

## Result

This is a contract/guard slice only. It does not claim that every text surface in the repository has
been migrated. It makes future resize fixes cheaper to classify: either a surface maps to one of the
stable roles above, or the work must explain why a new derived role is needed and add a focused gate.
The first follow-up container fix is `PropertyRow`: fixed chrome slots still clip, but value slots
must not clip explicit wrapping validation/prose children. The layout-level regression verifies that
narrow wrapping validation text grows the value slot and row bounds instead of painting past the row
bottom. The second regression covers `PropertyGrid` as the realistic inspector composition: mixed
single-line and wrapping rows stay vertically separated when the validation row grows.
The label-side follow-up adds the opposite guardrail: fixed property-row label chrome keeps a
single row-height line box even if a caller accidentally supplies bare/default wrapping text.
