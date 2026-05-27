# Milestones

## M1 - Source Classification

Closed on 2026-05-28.

- Compose proves TimePicker display selectors are selectable radio-like controls.
- Compose proves dial values use hour/minute spoken content descriptions.
- Compose exposes `Select AM or PM` for the period toggle group.

## M2 - Recipe Alignment

Closed on 2026-05-28.

- Fret TimePicker display selectors use `SemanticsRole::RadioButton`.
- Fret TimePicker selector values and dial labels use Compose-aligned English strings.
- Docked dial/input period groups expose stable parent ids and semantic group labels.

## M3 - Gates And Packet Closeout

Closed on 2026-05-28.

- Focused automation test covers the semantics contract.
- Matrix residual risk was narrowed to localization/string registry work instead of a vague
  live-region follow-on; the registry work is closed by
  `material3-time-picker-string-registry-packet-v1`.
