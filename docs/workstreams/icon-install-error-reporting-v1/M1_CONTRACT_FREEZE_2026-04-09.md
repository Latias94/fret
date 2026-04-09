# M1 Contract Freeze — 2026-04-09

Status: accepted decision

Related:

- `docs/workstreams/icon-install-error-reporting-v1/DESIGN.md`
- `docs/workstreams/icon-install-error-reporting-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-error-reporting-v1/TODO.md`
- `docs/workstreams/icon-install-error-reporting-v1/MILESTONES.md`
- `docs/workstreams/icon-install-error-reporting-v1/EVIDENCE_AND_GATES.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

## Decision

Freeze the reporting contract like this:

### 1) The shared report type lives in `fret-icons`

`fret-icons` owns:

- `IconInstallFailureKind`,
- `IconInstallFailureReport`,
- and the shared panic helpers used by explicit install seams.

Rationale:

- pack crates and generated templates already depend on `fret-icons`,
- bootstrap can depend downward on `fret-icons`,
- and this avoids introducing the wrong dependency edge.

### 2) Panic text remains human-readable

Explicit install failures should still panic with a readable string message.

Rationale:

- diagnostics is optional,
- the lane does not change return types,
- and a default panic hook must still show a useful message.

### 3) Structured reporting is scoped to the panic window

The shared report may be exposed to diagnostics only while the explicit install panic is in
flight.

Rationale:

- bootstrap diagnostics only needs to inspect it during panic handling,
- scoped visibility avoids stale cross-panic global state,
- and persistent startup error storage is a different lane.

### 4) Bootstrap diagnostics may log structured icon-install fields

When diagnostics is enabled and a known icon install failure panics, the bootstrap panic hook may
log:

- the install surface,
- the pack id when known,
- the failure kind,
- and structured details.

Rationale:

- this improves observability without changing lifecycle shape.

### 5) No broad lifecycle redesign in this lane

Do not change:

- `.setup(...)`,
- `init_app(...)`,
- `install_app(...)`,
- or builder/run return types

to carry icon install reporting in this lane.

Rationale:

- that is a broader integration decision than the reporting gap itself.

## Required proof for closeout

Close this lane only after the proof surface demonstrates:

1. shared helper/report primitives are real,
2. explicit install surfaces use them,
3. bootstrap diagnostics compiles against them,
4. ADR/alignment docs describe the shipped reporting contract.
