# M1 Text Chrome Stability - 2026-04-28

## Slice

This slice removes the remaining shadcn input chrome dependency from IMUI text controls.

## Decision

Keep shadcn `Input` and `Textarea` recipes aligned with upstream new-york-v4 focus-ring behavior,
but give IMUI text controls their own compact field chrome:

- padding: 8px horizontal, 3px vertical,
- border: 1px,
- radius: shared IMUI `CONTROL_RADIUS`,
- focus: border color only,
- `focus_ring: None`.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input.tsx`
- `repo-ref/ui/apps/v4/registry/new-york-v4/ui/textarea.tsx`
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`

## Gates

Verified focused gates:

```bash
cargo nextest run -p fret-ui-kit --features imui compact_imui_chrome_without_focus_ring --no-fail-fast
cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --no-fail-fast
cargo check -p fret-ui-kit --features imui --jobs 2
```
