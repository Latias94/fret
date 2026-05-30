# IMUI Demo Metrics Debug Action Metadata v1 - Handoff

Status: Closed
Last updated: 2026-05-30

Current slice: lane closed on 2026-05-30 after DMDA-010, DMDA-020, and DMDA-030.

The prior `imui-demo-metrics-debug-devtools-v1` route productization lane is closed. This follow-on
keeps the route owner closed and adds a narrower action metadata owner for richer GUI execution
controls and future command-palette integration.

First-open commands:

```bash
cargo run -p fretboard-dev -- list tool-apps --json
cargo run -p fretboard-dev -- list tool-apps
```

Focused verification is recorded in `EVIDENCE_AND_GATES.md`.

Follow-ons:

1. Start a separate DevTools command-palette/action-execution follow-on if real execution controls
   are needed.
2. Keep the old route owner lane closed; do not move diagnostics UI into `fret-imui`.
