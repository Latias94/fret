# ImUi Facade Disclosure Owner Split v1 - M0 Baseline Audit

Status: baseline captured
Date: 2026-05-13

## Baseline

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1506 lines before M1
- The disclosure wrapper cluster still lived in `facade_writer.rs`.
- The target methods were `collapsing_header`, `collapsing_header_with_options`, `tree_node`, and
  `tree_node_with_options`.

## Guardrails

- No public method renames.
- No `fret::imui` path changes.
- No `fret-imui` dependency or public surface changes.
- No `crates/fret-ui` runtime contract changes.
