# Material 3 Tooltip Rich Parts Packet v1

Status: closed
Date: 2026-05-28

## Truth

- Rich tooltips should expose stable root, chrome, title, and supporting text automation surfaces.
- Plain tooltip selector behavior should remain stable.
- Tooltip overlay input policy remains owned by `fret-ui-kit`.

## Artifacts

- `ecosystem/fret-ui-material3/src/tooltip.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-tooltip-rich-parts-packet-v1/`

## Wiring

- `PlainTooltip` and `RichTooltip` share `tooltip_content_root` for tooltip semantics and
  `tooltip_policy_root` for delay/safe-hover/overlay behavior.
- Rich text mode derives:
  - `<base>.title` only when title content exists,
  - `<base>.supporting-text` for the supporting text node.

## Proof

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment rich_tooltip_opens_and_closes_on_hover_smoke tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes`

## Residual Risk

Rich tooltip action interactivity is split because the current tooltip overlay layer is intentionally
pointer transparent.
