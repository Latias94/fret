# Bootstrap Known Startup Failure Taxonomy v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this file now records the shipped gate set for the closed lane.

## Smallest current repro

Use this sequence before changing the shipped bootstrap known startup failure taxonomy:

```bash
cargo fmt
cargo nextest run -p fret-bootstrap
cargo nextest run -p fret --lib
cargo check -p fret-bootstrap --features diagnostics
python3 tools/check_layering.py
git diff --check
```

What this proves now:

- bootstrap startup failures still map into the known taxonomy,
- the `fret` facade still exposes the taxonomy bridge,
- diagnostics still compile with the unified bootstrap failure fields,
- and the refactor stays inside crate boundaries and clean diff hygiene.

## Current evidence set

- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/BASELINE_AUDIT_2026-04-09.md`
  freezes the baseline:
  - typed startup errors already existed,
  - but the taxonomy was fragmented across returned errors vs icon panic reports,
  - and diagnostics still logged icon-only field names.
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
  freezes the narrow contract:
  - taxonomy lives in `fret-bootstrap`,
  - `fret::Error` gets a bridge method instead of new root re-exports,
  - diagnostics switch to bootstrap-level field names,
  - no lifecycle redesign.
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M2_PROOF_SURFACE_2026-04-09.md`
  closes the proof surface on:
  - bootstrap mappings,
  - `fret` facade bridging,
  - diagnostics compile proof,
  - and source-budget compliance.
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  closes the lane on:
  - shipped taxonomy types and mappings,
  - unified diagnostics fields,
  - and green proof gates.

## Gate set

### Bootstrap taxonomy tests

```bash
cargo nextest run -p fret-bootstrap
```

### `fret` facade tests

```bash
cargo nextest run -p fret --lib
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

- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/DESIGN.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/TODO.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/MILESTONES.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret/src/lib.rs`
- `crates/fret-app/src/settings.rs`
- `crates/fret-app/src/keymap.rs`
- `crates/fret-app/src/menu_bar.rs`
- `crates/fret-assets/src/file_manifest.rs`
- `crates/fret-launch/src/assets.rs`
- `ecosystem/fret-icons/src/lib.rs`
