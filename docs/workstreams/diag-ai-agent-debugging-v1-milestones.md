# diag-ai-agent-debugging-v1 Milestones

Last updated: 2026-02-21

## M0: Baseline + budgets

Exit criteria:

- Hot spots are measured (top fields by bytes for representative bundles).
- Size budgets are agreed (AI packet default + max).
- A minimal packet contract draft exists (filenames + required fields).

## M1: Index shipping

Exit criteria:

- `bundle.index.json` v1 exists and is produced by tooling/runtime (where appropriate).
- `diag slice` prefers index when present.
- Index presence does not break v1/v2 consumers (additive only).

Current status:

- `bundle.index.json` v1 is produced by tooling (`fretboard diag index`).
- Consumers still need migration (`diag slice/query` fast-paths).

## M2: AI packet shipping

Exit criteria:

- A single command produces a bounded “AI packet” directory that is sufficient for common triage.
- CI artifacts can upload the AI packet by default (optionally alongside full bundles).

Current status:

- `fretboard diag ai-packet` exists (tooling-side) and exports a small directory for a bundle.

## M3: Manifest-first bundles (optional)

Exit criteria:

- A manifest-first layout is proven in at least one workflow (WS bundles or local dumps).
- Back-compat materialization to `bundle.json` is available and tested.
