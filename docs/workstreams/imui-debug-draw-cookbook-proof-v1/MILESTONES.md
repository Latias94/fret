# ImUi Debug Draw Cookbook Proof v1 Milestones

Status: Closed.

## M0 - Public Cookbook Example

Exit criteria:

- The example compiles through `fret::imui::{prelude::*, kit::*}`.
- It demonstrates clip stack, channel split/merge, multi-color rects, triangle mesh helpers, and
  metadata summaries without touching internal crates.

Result: Complete.

## M1 - Discoverability

Exit criteria:

- `fretboard-dev` auto-enables `cookbook-imui` for `imui_debug_draw_basics`.
- `apps/fret-cookbook/README.md`, `apps/fret-cookbook/EXAMPLES.md`, and
  `docs/examples/README.md` list the example.

Result: Complete.

## M2 - Evidence

Exit criteria:

- Focused build/test gates cover the example and authoring markers.
- Workstream and audit indexes record that debug-draw is no longer a hidden API-only feature.

Result: Complete.
