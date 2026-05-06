# M0 Baseline Audit - 2026-05-06

Status: baseline recorded; first implementation slice is command-model owner split.

## Source Snapshot

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs` is the current owner for debug draw
  options, responses, command recording, channel split/merge, command summaries, path building,
  canvas painting, image/SVG helpers, mesh helpers, geometry helpers, and private tests.
- The file is about 139 KB / 4519 lines before this lane's first implementation slice.
- The focused compile-smoke surface already exists in
  `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`.

## Why This Split Comes First

The previous gap-closure closeout named this file as the highest split candidate and explicitly
deferred implementation to a narrower follow-on. The current pressure is not a missing public
helper; it is the cost of safely reviewing future debug draw parity work in a file that mixes too
many independent owners.

## First Slice Decision

Start with `debug_draw_controls/commands.rs`.

This is the lowest-risk owner boundary because the command-model cluster is already internally
cohesive:

- public command kind and summary vocabulary,
- private recorded command enum,
- command-to-summary conversion,
- aggregate list summary accounting.

The parent module can keep public draw-list methods and painter/path helpers unchanged while using
the extracted command model through `pub(super)` visibility.

## Explicitly Deferred

- Paint helper split.
- Path sampling split.
- Private test module split.
- Any new Dear ImGui draw-list capability such as callbacks, raw buffers, draw command user data,
  renderer draw-call attribution, or per-geometry hit testing.

## Gate Set

Use the lane gates in `EVIDENCE_AND_GATES.md`, with this focused command first:

```bash
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
```
