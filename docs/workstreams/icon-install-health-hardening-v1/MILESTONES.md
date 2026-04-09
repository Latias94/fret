# Icon Install Health Hardening v1 — Milestones

Status: Closed
Last updated: 2026-04-09

## M0 — Baseline and problem freeze

Exit criteria:

- The install/freeze problem is explicitly separated from the closed icon contract lane.
- Explicit install seams, helper fallback, and metadata recording behavior are audited.
- The lane's non-goals are explicit.

Primary evidence:

- `docs/workstreams/icon-install-health-hardening-v1/DESIGN.md`
- `docs/workstreams/icon-install-health-hardening-v1/TODO.md`
- `docs/workstreams/icon-install-health-hardening-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Current status:

- Opened on 2026-04-09 as a narrow follow-on to the closed icon contract lane.
- M0 baseline freeze closed on 2026-04-09.
- The next active work was M1 contract freeze.

## M1 — Contract freeze

Exit criteria:

- Fail-fast explicit install seams are named explicitly.
- Best-effort runtime helper semantics are named explicitly.
- The metadata-conflict rule is explicit.

Primary evidence:

- `docs/workstreams/icon-install-health-hardening-v1/DESIGN.md`
- `docs/workstreams/icon-install-health-hardening-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`

Current status:

- M1 contract freeze closed on 2026-04-09.
- The next active work was M2 proof surface.

## M2 — Proof surface

Exit criteria:

- Explicit install seams fail fast on bad registry state.
- Metadata conflicts fail fast.
- Best-effort helper paths preserve the valid subset.
- Generated pack output and first-party pack install seams prove the same semantics.

Primary gates:

- `cargo nextest run -p fret-icons -p fret-ui-kit -p fret-bootstrap -p fret-icons-lucide -p fret-icons-radix -p fret-icons-generator -p fretboard`

Current status:

- M2 proof surface closed on 2026-04-09.
- See `docs/workstreams/icon-install-health-hardening-v1/M2_PROOF_SURFACE_2026-04-09.md`.

## M3 — Gates and closeout

Exit criteria:

- The gate set is green.
- ADR and alignment notes match the shipped semantics.
- The lane closes explicitly or splits a narrower follow-on.

Primary gates:

- `cargo nextest run -p fret-icons -p fret-ui-kit -p fret-bootstrap -p fret-icons-lucide -p fret-icons-radix -p fret-icons-generator -p fretboard`
- `python3 tools/check_layering.py`
- `git diff --check`

Current status:

- M3 gates and closeout closed on 2026-04-09.
- The lane is now closed on
  `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
