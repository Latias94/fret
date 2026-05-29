# IMUI Demo Metrics Debug Action Metadata v1 - Handoff

Status: Active
Last updated: 2026-05-30

Current slice: DMDA-020 completed on 2026-05-30. DMDA-030 is the next decision point.

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

1. Decide whether command-palette integration belongs here or should start a separate follow-on.
2. Keep the old route owner lane closed; do not move diagnostics UI into `fret-imui`.
