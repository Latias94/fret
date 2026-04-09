# Baseline Audit — 2026-04-09

Status: accepted baseline

Related:

- `docs/workstreams/icon-install-error-reporting-v1/DESIGN.md`
- `docs/workstreams/icon-install-error-reporting-v1/TODO.md`
- `docs/workstreams/icon-install-error-reporting-v1/MILESTONES.md`
- `docs/workstreams/icon-install-error-reporting-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`

## Purpose

Freeze the assumptions-first baseline for the narrow reporting follow-on before changing code.

This note records what the repo already settles:

- explicit install failure semantics are already correct,
- the remaining problem is how known failures are reported,
- and the dependency direction constrains where the shared reporting primitive can live.

## Baseline findings

### 1) Failure semantics are already closed

The previous lane already established the semantic split:

- explicit install seams fail fast,
- helper fallback remains best-effort and partial.

Evidence:

- `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`

Consequence:

- this lane should not reopen those semantics, only their reporting shape.

### 2) Reporting is still ad-hoc at install call sites

First-party and generated pack installers still hand-roll panic strings locally.

Evidence:

- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `crates/fret-icons-generator/src/templates.rs`

Consequence:

- the repo has no shared “known icon install failure” vocabulary yet.

### 3) Bootstrap diagnostics only sees generic panic text

The diagnostics panic hook currently extracts string payload + location + backtrace, but it has no
structured icon-install context to log.

Evidence:

- `ecosystem/fret-bootstrap/src/lib.rs`

Consequence:

- diagnostics can report that a panic happened, but not which explicit icon install surface failed
  in a structured way.

### 4) The shared report home cannot be bootstrap

Pack crates and generated templates already depend on `fret-icons` and should not grow a bootstrap
dependency edge.

Evidence:

- `ecosystem/fret-icons-lucide/src/app.rs`
- `ecosystem/fret-icons-radix/src/app.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `ecosystem/fret-bootstrap/Cargo.toml`

Consequence:

- the shared reporting primitive has to live in `fret-icons` or an even lower layer, not in
  bootstrap.

### 5) String panic output still matters

Any new reporting mechanism must preserve useful default panic output for runs that do not install
bootstrap diagnostics.

Evidence:

- current install seams panic with strings,
- bootstrap diagnostics currently extracts string payloads,
- the lane is explicitly not changing lifecycle return types.

Consequence:

- typed-only panic payloads would likely be the wrong first step.

## Baseline decision

Treat this lane as a narrow reporting contract:

1. shared report type in `fret-icons`,
2. human-readable panic text preserved,
3. diagnostics-only structured visibility during the panic window,
4. no setup/bootstrap return-type redesign.
