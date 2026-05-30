# Material3 Field Logical Insets v1 TODO

Status: Closed
Last updated: 2026-05-30

## Tasks

- [x] M3-FIELD-001: Extend Material3 logical edge helpers for inline-start insets and margins.
  - Gate: `cargo nextest run -p fret-ui-material3 --lib foundation::logical_edges`.

- [x] M3-FIELD-002: Migrate TextField label/supporting text geometry to logical inline insets.
  - Gate: `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover text_field_rtl_label_and_supporting_text_use_logical_inline_insets`.

- [x] M3-FIELD-003: Migrate Select label/supporting text geometry to logical inline insets.
  - Gate: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_label_and_supporting_text_use_logical_inline_insets`.

- [x] M3-FIELD-004: Run quality gates and close the lane.
