# Icon Install Error Reporting v1 — Milestones

Status: Closed
Last updated: 2026-04-09

## M0 — Baseline and scope freeze

Exit criteria:

- The reporting problem is explicitly separated from the closed install-health lane.
- Current panic/reporting behavior is audited across pack installs, generator output, and
  bootstrap diagnostics.
- The lane's non-goals are explicit.

Primary evidence:

- `docs/workstreams/icon-install-error-reporting-v1/DESIGN.md`
- `docs/workstreams/icon-install-error-reporting-v1/TODO.md`
- `docs/workstreams/icon-install-error-reporting-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Current status:

- Opened and closed on 2026-04-09 as a narrow follow-on to the install-health lane.
- M0 baseline freeze closed on 2026-04-09.

## M1 — Reporting contract freeze

Exit criteria:

- The shared report home is explicit.
- The human-readable panic requirement is explicit.
- The diagnostics visibility window is explicit.

Primary evidence:

- `docs/workstreams/icon-install-error-reporting-v1/DESIGN.md`
- `docs/workstreams/icon-install-error-reporting-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`

Current status:

- M1 contract freeze closed on 2026-04-09.

## M2 — Proof surface

Exit criteria:

- Shared report and helper primitives are real.
- First-party and generated install seams use the shared helpers.
- Bootstrap diagnostics compiles against the structured path.
- Tests/source-policy lock the helper usage.

Primary gates:

- `cargo nextest run -p fret-icons -p fret-icons-generator -p fret-bootstrap`
- `cargo test -p fret-icons-lucide --features app-integration app_install_records_pack_metadata_and_freezes_registry`
- `cargo test -p fret-icons-radix --features app-integration app_install_records_pack_metadata_and_freezes_registry`
- `cargo check -p fret-bootstrap --features diagnostics`

Current status:

- M2 proof surface closed on 2026-04-09.
- See `docs/workstreams/icon-install-error-reporting-v1/M2_PROOF_SURFACE_2026-04-09.md`.

## M3 — Docs and closeout

Exit criteria:

- ADR/alignment docs match the shipped reporting contract.
- The gate set is explicit.
- The lane closes explicitly.

Primary gates:

- `cargo nextest run -p fret-icons -p fret-icons-generator -p fret-bootstrap`
- `cargo test -p fret-icons-lucide --features app-integration app_install_records_pack_metadata_and_freezes_registry`
- `cargo test -p fret-icons-radix --features app-integration app_install_records_pack_metadata_and_freezes_registry`
- `cargo check -p fret-bootstrap --features diagnostics`
- `python3 tools/check_layering.py`
- `git diff --check`

Current status:

- M3 docs and closeout closed on 2026-04-09.
- The lane is now closed on
  `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
