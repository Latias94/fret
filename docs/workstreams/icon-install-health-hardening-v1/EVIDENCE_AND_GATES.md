# Icon Install Health Hardening v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this file now records the shipped gate set for the closed lane.

## Smallest current repro

Use this sequence before changing icon install/freeze semantics again:

```bash
cargo nextest run -p fret-icons -p fret-ui-kit -p fret-bootstrap -p fret-icons-lucide -p fret-icons-radix -p fret-icons-generator -p fretboard
python3 tools/check_layering.py
git diff --check
```

What this proves now:

- registry freeze behavior and metadata recording are covered,
- helper preload/fallback keeps valid icons when unrelated entries are broken,
- bootstrap and first-party pack install seams stay on the strict contract,
- generated pack output teaches the same contract,
- and the refactor stays inside crate boundaries.

## Current evidence set

- `docs/workstreams/icon-install-health-hardening-v1/BASELINE_AUDIT_2026-04-09.md`
  freezes the baseline:
  - explicit install seams still mixed strict vs best-effort behavior,
  - metadata conflict handling was not yet a release-visible contract failure,
  - and helper fallback could discard usable icons.
- `docs/workstreams/icon-install-health-hardening-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
  freezes the narrow contract split:
  - fail-fast explicit install seams,
  - best-effort partial helper fallback,
  - no broad fallible setup redesign.
- `docs/workstreams/icon-install-health-hardening-v1/M2_PROOF_SURFACE_2026-04-09.md`
  closes the proof surface on:
  - strict first-party/generated/bootstrap install behavior,
  - partial helper fallback,
  - and aligned tests/source-policy coverage.
- `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  closes the lane on:
  - fail-fast explicit install seams,
  - metadata-conflict rejection,
  - partial helper fallback,
  - and green gate evidence.
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  keeps the broader icon contract lane closed.
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  keeps presentation-default policy out of this install-health follow-on.

## Active gate set

### Cross-crate install hardening

```bash
cargo nextest run -p fret-icons -p fret-ui-kit -p fret-bootstrap -p fret-icons-lucide -p fret-icons-radix -p fret-icons-generator -p fretboard
```

### Layering

```bash
python3 tools/check_layering.py
```

### Diff hygiene

```bash
git diff --check
```

## Evidence anchors

- `docs/workstreams/icon-install-health-hardening-v1/DESIGN.md`
- `docs/workstreams/icon-install-health-hardening-v1/TODO.md`
- `docs/workstreams/icon-install-health-hardening-v1/MILESTONES.md`
- `docs/workstreams/icon-install-health-hardening-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/icon-install-health-hardening-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-lucide/src/lib.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `ecosystem/fret-icons-radix/src/lib.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fret-icons-generator/src/lib.rs`
