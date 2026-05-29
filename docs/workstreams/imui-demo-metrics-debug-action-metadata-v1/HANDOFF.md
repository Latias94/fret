# IMUI Demo Metrics Debug Action Metadata v1 - Handoff

Status: Active
Last updated: 2026-05-30

Current slice: DMDA-010 completed on 2026-05-30. DMDA-020 is the next candidate slice.

The prior `imui-demo-metrics-debug-devtools-v1` route productization lane is closed. This follow-on
keeps the route owner closed and adds a narrower action metadata owner for richer GUI execution
controls and future command-palette integration.

First-open commands:

```bash
cargo run -p fretboard-dev -- list tool-apps --json
cargo run -p fretboard-dev -- list tool-apps
```

Focused verification is recorded in `EVIDENCE_AND_GATES.md`.

Next expected work:

1. Decide whether DMDA-020 should add a DevTools-side disabled/runnable action row using
   `requires_bundle`, or whether broader command-palette integration needs a separate lane.
2. Keep the old route owner lane closed; do not move diagnostics UI into `fret-imui`.
