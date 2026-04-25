# ImUi Edit Lifecycle Hardening v1 - M2 ImUi Input Stability Slice - 2026-04-25

## Decision

`input_text_model_with_options` is the public IMUI single-line text entry helper. It must behave
like a compact editor control, not like a raw mechanism-level text input whose measured height can
change with focus, font fallback, or text metrics. The policy belongs in `fret-ui-kit::imui`, while
`crates/fret-ui` continues to expose only the text-input mechanism.

## Shipped Invariant

- IMUI single-line text inputs now use the shared IMUI field height
  (`control_chrome::FIELD_MIN_HEIGHT`) as `height`, `min_height`, and `max_height`.
- IMUI single-line text inputs now opt into the ecosystem control text style helper, giving them a
  fixed control line box.
- Existing IMUI text lifecycle semantics remain unchanged.
- No public option or runtime contract was widened.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`

Verified on 2026-04-25:

```bash
cargo fmt --package fret-ui-kit --package fret-imui --check
cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --jobs 2
cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges --jobs 2
cargo check -p fret-ui-kit --features imui --jobs 2
```

## Residual Risk

This slice intentionally covers single-line IMUI text input. Multiline textarea remains a separate
policy surface because it can validly grow by content and already exposes `min_height` plus the
`stable_line_boxes` option.
