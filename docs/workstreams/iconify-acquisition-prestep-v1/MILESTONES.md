# Iconify Acquisition Pre-step v1 — Milestones

Status: Closed
Last updated: 2026-04-09

## M0 — Scope and evidence freeze

Exit criteria:

- The acquisition problem is explicitly separated from the closed generator lane.
- The current generator input boundary and optional acquisition references are audited.
- The lane's non-goals are explicit.

Primary evidence:

- `docs/workstreams/iconify-acquisition-prestep-v1/DESIGN.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/TODO.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Current status:

- Opened on 2026-04-09 as a narrow follow-on to the closed generator lane.
- M0 evidence freeze closed on 2026-04-09 with a baseline audit of the current generator/input
  boundary, multicolor posture, and third-party extensibility surface.
- The next active work is M1 acquisition contract freeze.

## M1 — Acquisition contract freeze

Exit criteria:

- The pre-step artifact boundary is explicit.
- Ownership of the public acquisition surface is explicit.
- Pinning/provenance expectations are explicit.
- The first proof target is explicit.

Primary evidence:

- `docs/workstreams/iconify-acquisition-prestep-v1/DESIGN.md`
- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-import-pack-generator-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `repo-ref/dioxus-iconify/README.md`
- `repo-ref/dioxus-iconify/src/api.rs`
- `docs/workstreams/iconify-acquisition-prestep-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

## M2 — Proof surface

Exit criteria:

- One smallest acquisition proof lands.
- The proof emits pinned local artifact(s) rather than hidden transient state.
- The acquired artifact flows into the existing generator path without manual cleanup.

Primary gates:

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo check -p fretboard --quiet`

Current status:

- M1 contract freeze closed on 2026-04-09.
- M2 proof surface closed on 2026-04-09.

## M3 — Docs and closeout

Exit criteria:

- The acquisition proof leaves one deterministic gate.
- User-facing docs teach only the shipped acquisition contract.
- The lane closes explicitly or splits a narrower follow-on.

Primary gates:

- `cargo nextest run -p fret-icons-generator -p fretboard`
- `cargo check -p fretboard --quiet`

Current status:

- M3 docs and closeout closed on 2026-04-09.
- The lane is now closed.
