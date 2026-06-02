# Material3 Logical Edge Layout v1 TODO

Status: Closed
Last updated: 2026-05-30

## Tasks

- [x] M3-EDGE-001: Add Material3 logical edge helper functions.
  - Scope: `ecosystem/fret-ui-material3/src/foundation/logical_edges.rs`.
  - Gate: `cargo nextest run -p fret-ui-material3 --lib foundation::logical_edges`.

- [x] M3-EDGE-002: Migrate FilterChip and InputChip content padding/trailing action insets.
  - Scope: `ecosystem/fret-ui-material3/src/filter_chip.rs`,
    `ecosystem/fret-ui-material3/src/input_chip.rs`.
  - Gate: `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state rtl_filter_and_input_chips_mirror_inline_content_edges`.

- [x] M3-EDGE-003: Run targeted gates and close the lane with residual follow-ons.
