# M2 Proof Surface — 2026-04-09

Status: accepted proof

Related:

- `docs/workstreams/icon-install-health-hardening-v1/DESIGN.md`
- `docs/workstreams/icon-install-health-hardening-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/TODO.md`
- `docs/workstreams/icon-install-health-hardening-v1/MILESTONES.md`
- `docs/workstreams/icon-install-health-hardening-v1/EVIDENCE_AND_GATES.md`
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

## Purpose

Freeze the first complete proof slice for the install-health hardening contract. This note records
the smallest landed evidence that:

- explicit install seams now fail fast,
- metadata conflicts are explicit contract errors,
- helper fallback keeps the valid subset,
- and generated/first-party/bootstrap surfaces all teach the same semantics.

## What shipped in the proof

### 1) Installed-pack metadata conflicts are now explicit

`InstalledIconPacks::record(...)` now returns:

- `Ok(true)` for the first insert,
- `Ok(false)` for the same metadata repeated,
- and an explicit error for conflicting metadata under the same `pack_id`.

This makes installed-pack provenance safe for future consumers instead of relying on debug-only
assertions.

### 2) Explicit install seams now fail fast

These surfaces now treat broken registry state as install-time contract failure:

- `BootstrapBuilder::register_icon_pack_contract(...)`,
- bootstrap's raw `register_icon_pack(...)`,
- first-party `crate::app::install(...)`,
- and generated `crate::app::install(...)`.

The proof keeps the surrounding lifecycle non-fallible while preventing misleading “successful”
install state.

### 3) Helper fallback now preserves the valid subset

`freeze_or_default_with_context(...)` no longer swaps the whole registry for an empty default when
some unrelated entries fail to resolve.

Instead it:

- freezes every valid icon it can,
- emits diagnostics for errors,
- and leaves helper-owned callers with a usable partial snapshot.

### 4) Tests/source-policy now lock the split

The proof adds regression coverage for:

- metadata-conflict rejection,
- partial helper fallback,
- preload behavior with mixed valid/invalid icons,
- bootstrap panic behavior,
- first-party app install surface shape,
- and generated pack install template shape.

## Gates executed on 2026-04-09

```bash
cargo nextest run -p fret-icons -p fret-ui-kit -p fret-bootstrap -p fret-icons-lucide -p fret-icons-radix -p fret-icons-generator -p fretboard
python3 tools/check_layering.py
git diff --check
```

Observed result:

- `cargo nextest ...`: `627 tests run: 627 passed`
- `python3 tools/check_layering.py`: passed
- `git diff --check`: passed

## Evidence anchors from the proof

- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
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

1. explicit install seams are now strict;
2. helper fallback is now partial rather than destructive;
3. metadata conflict is now a real contract error;
4. first-party/generated/bootstrap surfaces all align on the same install-health semantics.
