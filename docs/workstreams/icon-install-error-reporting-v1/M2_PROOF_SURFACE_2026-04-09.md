# M2 Proof Surface — 2026-04-09

Status: accepted proof

Related:

- `docs/workstreams/icon-install-error-reporting-v1/DESIGN.md`
- `docs/workstreams/icon-install-error-reporting-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/TODO.md`
- `docs/workstreams/icon-install-error-reporting-v1/MILESTONES.md`
- `docs/workstreams/icon-install-error-reporting-v1/EVIDENCE_AND_GATES.md`
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

## Purpose

Freeze the first complete proof slice for the icon install reporting contract. This note records
the smallest landed evidence that:

- known icon install failures now share one report/helper vocabulary,
- first-party/generated install seams route through it,
- bootstrap diagnostics can compile with structured panic context,
- and panic text stays human-readable.

## What shipped in the proof

### 1) `fret-icons` now owns a known install-failure report

The shared icon layer now defines:

- `IconInstallFailureKind`,
- `IconInstallFailureReport`,
- shared panic helpers for freeze failure and metadata conflict,
- and a diagnostics-only accessor for the current scoped report.

### 2) Structured report visibility is panic-scoped

The report is only exposed during the panic window:

- install helpers register the scoped report,
- panic with human-readable string text,
- and the scope clears automatically during unwind.

This keeps diagnostics observability without leaving stale process-wide state behind.

### 3) Explicit install seams now use the shared helpers

The proof routes these surfaces through the new helper path:

- `fret-bootstrap` explicit pack registration,
- Lucide app install,
- Radix app install,
- generated pack app install template.

### 4) Bootstrap diagnostics can now log structured icon-install context

The diagnostics panic hook now recognizes known icon install failures and can log:

- the install surface,
- pack id when known,
- the failure kind,
- and structured detail lines,

while preserving the existing generic panic logging path for unrelated panics.

## Gates executed on 2026-04-09

```bash
cargo nextest run -p fret-icons -p fret-icons-generator -p fret-bootstrap
cargo test -p fret-icons-lucide --features app-integration app_install_records_pack_metadata_and_freezes_registry
cargo test -p fret-icons-radix --features app-integration app_install_records_pack_metadata_and_freezes_registry
cargo check -p fret-bootstrap --features diagnostics
```

Observed result:

- `cargo nextest ...`: `42 tests run: 42 passed`
- `cargo test -p fret-icons-lucide --features app-integration ...`: `1 passed`
- `cargo test -p fret-icons-radix --features app-integration ...`: `1 passed`
- `cargo check -p fret-bootstrap --features diagnostics`: passed

## Evidence anchors from the proof

- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-lucide/src/lib.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `ecosystem/fret-icons-radix/src/lib.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `crates/fret-icons-generator/src/lib.rs`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

## M2 verdict

Treat M2 as closed on these points:

1. known icon install failures now share one reporting primitive;
2. explicit install seams use that primitive instead of hand-rolled panic text;
3. bootstrap diagnostics can observe structured icon-install panic context;
4. the lifecycle contract remains unchanged.
