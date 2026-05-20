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
| Code text | `text_code_block(...)`, `text_code_wrap(...)`, and `text_code_label(...)` | Code blocks are monospace and horizontal-scroll-friendly; inline code may wrap by grapheme; code labels are single-line, shrinkable, and ellipsized. | Docs code blocks, inline code prose, fixed-height package/env/dependency identifiers. |
| Table cell text | `text_table_cell(...)` | Single line, shrinkable, `min-width: 0`, ellipsis. | Header/body table cells unless a future multi-line row-height policy is explicit. |

These five names are the minimum stable vocabulary for future resize triage. If a new component text
case does not fit one of them, the first question is whether it is a derived role, not whether the
component should construct `TextProps` locally.

## Current Derived Roles

- `text_compact_paragraph(...)`: dense wrapping paragraph for editor/IMUI panels. It may grow
  height and must not be used inside fixed-height control chrome unless that parent measures
  multi-line height. `text_compact_paragraph_inherited(...)` keeps the same wrapping/fill-width
  layout contract for component-owned description slots that already install their own inherited
  typography.
- `text_compact_paragraph_line_clamp(...)`: dense wrapping paragraph for list/card descriptions
  with a fixed maximum line count. It is still paragraph-family text, but the helper owns the
  `max-height + ellipsis` clamp contract so snippets/components do not hand-roll local
  `TextProps`.
- `text_list_row_label(...)` and `text_list_row_label_attributed(...)`: dense command/list/tree row
  labels. They are not button labels; they fill row width, shrink to zero, and ellipsize to keep
  row height stable. The attributed variant exists for row labels that need per-span decoration
  such as strikethrough without re-owning local row text layout policy.
- `text_menu_group_label(...)`: muted `text-xs` group headings inside menu/select/listbox surfaces.
  They are non-interactive labels, not readouts; they fill the row width, shrink to zero, and
  ellipsize so fixed menu rows do not grow under resize.
- `text_status_message(...)`: muted `text-sm` non-interactive empty/loading/status messages inside
  command/listbox/status surfaces. These messages are not group labels and not compact readouts;
  they stay single-line, shrinkable, and ellipsized when mounted in fixed command/list rows.
- `text_control_readout_tabular(...)`, `text_control_readout_tabular_emphasis(...)`, and
  `text_control_readout_compact_tabular_emphasis(...)`: numeric control-readout variants for
  counters, page summaries, fixed badges, and dense dashboard values. They remain in the
  control-readout family, add inherited `tnum` OpenType features, and keep the same single-line
  resize contract; the emphasis variants add medium weight, while the compact emphasis variant
  keeps `text-xs` sizing for badge-like slots.
- `text_button_label_fill(...)` and `text_button_label_compact_fill(...)`: button-label variants
  for button-like rows whose label owns the remaining inline space between icons/actions. They
  remain in the button-label family, add fill/grow/basis-zero layout, and keep single-line
  ellipsis; the compact variant owns `text-xs font-medium` for small trigger rows.
- Component-local refinements may layer font features, variable font axes, or explicit weight
  overrides onto button-label roles through inherited text refinement. That keeps the role-owned
  no-wrap/ellipsis layout intact instead of forcing components back to leaf-local `TextStyle`
  builders.
- Calendar-like fixed button cells may consume the button-label/readout role families with
  component-local inherited refinements for normal weight and center alignment. The role still owns
  no-wrap, shrink, min-width-zero, and ellipsis; the calendar recipe owns cell chrome, selection
  foreground, and date/range semantics.
- `text_control_label(...)`: fill-width checkbox/radio/switch/combo/slider label text. It keeps
  fixed control chrome single-line under resize.
- `text_section_chrome_label(...)`, `text_chrome_title(...)`, and `text_chrome_glyph(...)`: section,
  title-bar, and glyph chrome roles. `text_chrome_title(...)` owns medium emphasis plus fill/grow
  title-bar layout; all three keep fixed chrome rows single-line, with ellipsis or clip according
  to the slot.
- `text_code_label(...)`: code-text derivative for fixed-height identifier slots such as package
  names, env keys, and dependency rows. `text_code_label_emphasis(...)` keeps the same single-line
  resize contract for primary identifier slots that need medium emphasis, such as package names or
  target versions. These are intentionally not substitutes for wrapping inline prose code.
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
3. If the text is code-like, choose among `text_code_block(...)` for scrollable block code,
   `text_code_wrap(...)` for inline prose code that can wrap long identifiers, and
   `text_code_label(...)` for fixed-height identifier slots that must truncate under resize.
4. New direct `TextProps` / `StyledTextProps` construction under `fret-ui-kit::imui`, editor
   controls, or first-party proof apps is a contract change. It must either move into a role helper
   or update the source gate with a documented owner and proof. Builder-style `ui::raw_text(...)`
   and `ui::text_block(...)` count as direct text policy too, unless the surface is an explicit
   text/rendering capability probe.
5. Component recipes may apply default typography/layout to bare text children, but they must not
   overwrite a caller-supplied role child that already carries inherited text-role metadata. A
   recipe wrapper can add container layout or hover decoration, but role-owned style, wrap, and
   overflow remain the role's contract unless the recipe documents a stronger slot policy.
6. Do not add a public `TextRole` enum until at least two consumers need a data-driven role value.
   The current API remains helper-based to avoid freezing unnecessary public surface.
7. Remaining bare text in first-party proof apps is allowed only when the surface is itself testing
   text/input/rendering behavior or intentionally carries a visual display payload that does not
   fit the compact role vocabulary yet. Current allowed residuals are `components_gallery` text
   smoke/font override probes, `ime_smoke_demo` IME behavior instructions/status, text/CJK/emoji
   conformance probes, rendering-effect overlay probes, `hello_counter_demo`'s large numeric
   display, and `hello_world_compare_demo`'s GPUI/Fret comparison title payload. Do not migrate
   those mechanically into compact chrome roles; add a new role only when a non-proof surface
   repeats the need.

## Gates

- Shared role behavior:
  - `cargo nextest run -p fret-ui-kit --features imui --lib control_readout_text_uses_muted_compact_single_line_truncation control_readout_tabular_text_uses_muted_single_line_truncation control_readout_tabular_emphasis_text_uses_medium_single_line_truncation button_label_text_uses_medium_single_line_truncation prose_variants_and_code_wrap_install_semantic_inherited_overrides table_cell_text_uses_compact_single_line_truncation attributed_list_row_label_text_uses_fill_width_single_line_truncation --no-fail-fast`
  - `cargo nextest run -p fret-ui-kit --features imui --lib menu_group_label_text_uses_muted_xs_single_line_truncation --no-fail-fast`
  - `cargo nextest run -p fret-ui-kit --features imui --lib status_message_text_uses_muted_sm_single_line_truncation --no-fail-fast`
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
