# Icon Install Error Reporting v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this file now records the shipped gate set for the closed lane.

## Smallest current repro

Use this sequence before changing the shipped icon install reporting contract:

```bash
cargo nextest run -p fret-icons -p fret-icons-generator -p fret-bootstrap
cargo test -p fret-icons-lucide --features app-integration app_install_records_pack_metadata_and_freezes_registry
cargo test -p fret-icons-radix --features app-integration app_install_records_pack_metadata_and_freezes_registry
cargo check -p fret-bootstrap --features diagnostics
python3 tools/check_layering.py
git diff --check
```

What this proves now:

- the shared report/helper primitives behave correctly,
- first-party/generated install seams still freeze and record metadata correctly,
- bootstrap diagnostics still compiles with structured icon-install panic context,
- and the refactor stays inside crate boundaries and clean diff hygiene.

## Current evidence set

- `docs/workstreams/icon-install-error-reporting-v1/BASELINE_AUDIT_2026-04-09.md`
  freezes the baseline:
  - install semantics were already correct,
  - reporting was still ad-hoc,
  - and bootstrap diagnostics only saw generic panic text.
- `docs/workstreams/icon-install-error-reporting-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
  freezes the narrow reporting contract:
  - shared report type in `fret-icons`,
  - human-readable panic text,
  - scoped panic-time diagnostics visibility,
  - no lifecycle return-type changes.
- `docs/workstreams/icon-install-error-reporting-v1/M2_PROOF_SURFACE_2026-04-09.md`
  closes the proof surface on:
  - shared helper usage,
  - first-party/generated/bootstrap alignment,
  - and diagnostics-aware bootstrap compilation.
- `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  closes the lane on:
  - shipped reporting primitives,
  - updated ADR/alignment wording,
  - and green proof gates.

## Gate set

### Reporting-core tests

```bash
cargo nextest run -p fret-icons -p fret-icons-generator -p fret-bootstrap
```

### First-party app-install proofs

```bash
cargo test -p fret-icons-lucide --features app-integration app_install_records_pack_metadata_and_freezes_registry
cargo test -p fret-icons-radix --features app-integration app_install_records_pack_metadata_and_freezes_registry
```

### Diagnostics compile proof

```bash
cargo check -p fret-bootstrap --features diagnostics
```

### Boundary + diff hygiene

```bash
python3 tools/check_layering.py
git diff --check
```

## Evidence anchors

- `docs/workstreams/icon-install-error-reporting-v1/DESIGN.md`
- `docs/workstreams/icon-install-error-reporting-v1/TODO.md`
- `docs/workstreams/icon-install-error-reporting-v1/MILESTONES.md`
- `docs/workstreams/icon-install-error-reporting-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/icon-install-error-reporting-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-lucide/src/lib.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `ecosystem/fret-icons-radix/src/lib.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fret-icons-generator/src/lib.rs`
