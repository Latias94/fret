# Fret Node Paint Root Frame Setup Adapter v1 - Milestones

Status: Active
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- Frame setup audit scope is explicit.
- Non-goals exclude static layer replay/store and cached/immediate passes.
- Gate set is recorded.

## M1 - Frame Setup Operation-Family Audit

Exit criteria:

- Frame setup operation families are listed with evidence anchors.
- First implementation candidate is selected, or a narrower follow-on is proposed.

Primary gates:

- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

## M2 - First Frame Seam Or Closeout

Exit criteria:

- The lane either ships a narrow frame seam or splits the next task explicitly.
- Evidence gates are fresh.
- `WORKSTREAM.json` status is updated.
