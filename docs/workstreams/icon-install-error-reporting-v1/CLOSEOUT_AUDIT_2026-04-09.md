# Closeout Audit — 2026-04-09

Status: closed closeout record

Related:

- `docs/workstreams/icon-install-error-reporting-v1/DESIGN.md`
- `docs/workstreams/icon-install-error-reporting-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/TODO.md`
- `docs/workstreams/icon-install-error-reporting-v1/MILESTONES.md`
- `docs/workstreams/icon-install-error-reporting-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `crates/fret-icons-generator/src/templates.rs`

## Verdict

This lane is now closed.

It successfully landed the narrow follow-on that the install-health lane left open:

- a shared icon-install failure report vocabulary,
- shared panic helpers for explicit install seams,
- diagnostics-aware structured logging in the bootstrap panic hook,
- and retained human-readable panic output without changing lifecycle return types.

## What shipped

### 1) Known icon install failures now share one contract

The repo no longer treats explicit icon install reporting as a set of unrelated panic strings.

Instead, `fret-icons` now owns the shared vocabulary for:

- registry-freeze failure,
- installed-pack metadata conflict,
- install surface identity,
- and structured detail lines.

### 2) Structured reporting did not require lifecycle redesign

This lane kept the hard-won boundary from the previous lane:

- explicit install seams still fail fast,
- but the surrounding `.setup(...)` / `init_app(...)` shape remains unchanged.

That means reporting improved without reopening a much larger return-type redesign.

### 3) Diagnostics can now report more than just “panic happened”

Bootstrap diagnostics can now log known icon-install context in a structured way while still
falling back to the generic panic path for unrelated failures.

This is a real observability improvement, but it remains deliberately narrow:

- panic-time structured logging,
- not persistent startup error UI,
- not bundle persistence,
- and not recovery semantics.

### 4) The lane stayed in the correct layer

The shared reporting primitive lives in `fret-icons`, not in bootstrap.

That preserves dependency direction:

- pack crates and generated templates depend downward on the shared icon layer,
- bootstrap consumes the same report for diagnostics,
- and no bootstrap dependency leaks into pack crates.

## Gates that define the shipped surface

- `cargo nextest run -p fret-icons -p fret-icons-generator -p fret-bootstrap`
- `cargo test -p fret-icons-lucide --features app-integration app_install_records_pack_metadata_and_freezes_registry`
- `cargo test -p fret-icons-radix --features app-integration app_install_records_pack_metadata_and_freezes_registry`
- `cargo check -p fret-bootstrap --features diagnostics`
- `python3 tools/check_layering.py`
- `git diff --check`

## Follow-on policy

Do not reopen this lane for:

- broad `Result` plumbing across setup/bootstrap lifecycle,
- startup error modals or recovery UX,
- or diagnostics bundle persistence for startup failures.

If future work is needed, open a narrower follow-on such as:

1. a dedicated startup error presentation surface for apps,
2. diagnostics bundle capture for known startup/install failures,
3. or a broader bootstrap error taxonomy that goes beyond icon install.
