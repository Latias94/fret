# Icon Install Error Reporting v1

Status: Closed
Last updated: 2026-04-09

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `BASELINE_AUDIT_2026-04-09.md`
- `M1_CONTRACT_FREEZE_2026-04-09.md`
- `M2_PROOF_SURFACE_2026-04-09.md`
- `CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

Status note (2026-04-09): this lane is now closed on one narrow reporting contract:

- explicit icon install failures keep human-readable panic text,
- `fret-icons` exposes a known install-failure report during the panic window,
- and bootstrap diagnostics can log structured surface/pack/error details without changing setup
  return types.

Read the landed proof in `M2_PROOF_SURFACE_2026-04-09.md` and the final verdict in
`CLOSEOUT_AUDIT_2026-04-09.md`.

This lane is a narrow follow-on to the closed `icon-install-health-hardening-v1` lane.
It does not reopen failure semantics, helper fallback semantics, or the broader app/bootstrap
lifecycle shape.

It owns one narrower question:

> once explicit icon install surfaces are fail-fast, how should Fret report those failures in a
> durable and diagnostics-friendly way without redesigning `.setup(...)` / `init_app(...)` to
> return `Result`?

## Why this lane exists

The previous hardening lane closed the semantic split:

- explicit install seams fail fast,
- helper fallback remains best-effort and partial.

That left one smaller quality gap:

- pack crates and generator output still panic with ad-hoc strings,
- bootstrap diagnostics only sees a generic panic message,
- and there is no shared report shape for “known icon install failure”.

This lane exists to close that reporting gap while keeping the lifecycle contract unchanged.

## Assumptions-first baseline

### 1) Lane ownership

- Area: workstream ownership
- Assumption: this is a new narrow follow-on to the closed install-health lane rather than a
  reopening of that lane.
- Evidence:
  - `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  - `docs/roadmap.md`
  - `docs/workstreams/README.md`
- Confidence: Confident
- Consequence if wrong: we would blur shipped failure semantics with a fresh reporting concern.

### 2) The surrounding setup chain should remain non-fallible in this slice

- Area: lifecycle shape
- Assumption: the right fix is not a broad `Result` conversion of `.setup(...)` or
  `init_app(...)`.
- Evidence:
  - `ecosystem/fret-bootstrap/src/lib.rs`
  - `crates/fret-launch/src/runner/desktop/runner/run.rs`
  - `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- Confidence: Confident
- Consequence if wrong: the lane would under-scope a larger integration redesign.

### 3) Pack crates need a common reporting primitive below bootstrap

- Area: layering
- Assumption: the known failure report must live in `fret-icons` rather than `fret-bootstrap`
  because pack crates and generated templates already depend on `fret-icons` but should not depend
  on bootstrap.
- Evidence:
  - `ecosystem/fret-icons-lucide/src/app.rs`
  - `ecosystem/fret-icons-radix/src/app.rs`
  - `crates/fret-icons-generator/src/templates.rs`
  - `ecosystem/fret-bootstrap/Cargo.toml`
- Confidence: Confident
- Consequence if wrong: we would introduce the wrong dependency direction.

### 4) Panic text still needs to stay human-readable

- Area: user-facing failure output
- Assumption: the reporting surface should not switch explicit install failures to opaque
  non-string panic payloads because default panic hooks outside diagnostics would degrade.
- Evidence:
  - `ecosystem/fret-bootstrap/src/lib.rs`
  - Rust default panic-hook behavior implied by current string-only message extraction in
    `init_panic_hook()`
- Confidence: Likely
- Consequence if wrong: non-diagnostics runs could regress to low-signal panic output.

### 5) Diagnostics only need read access during the panic window

- Area: reporting mechanism
- Assumption: a scoped panic-time report is enough for this lane; persistent startup error UI or
  bundle capture is a later follow-on.
- Evidence:
  - `ecosystem/fret-bootstrap/src/lib.rs`
  - `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- Confidence: Likely
- Consequence if wrong: this lane would close too early on a partial observability surface.

## In scope

- Standardize a known icon-install failure report shape.
- Route explicit install seams through shared panic helpers instead of ad-hoc panic formatting.
- Let bootstrap diagnostics log structured fields when one of those known failures occurs.
- Keep panic text human-readable in non-diagnostics runs.

## Out of scope

- Changing `.setup(...)`, `init_app(...)`, or builder lifecycle methods to return `Result`.
- UI-level startup error modals or richer app-facing recovery UX.
- Diagnostics bundle persistence for startup failures outside the panic-hook path.
- Reopening helper fallback or install-health semantics themselves.

## Owning layers

- `ecosystem/fret-icons`
  - known report type and shared panic helpers
- `ecosystem/fret-bootstrap`
  - diagnostics-aware panic-hook logging
- first-party / generated pack install seams
  - `ecosystem/fret-icons-lucide`
  - `ecosystem/fret-icons-radix`
  - `crates/fret-icons-generator`

## Target shipped state

When this lane is done, the following must be true:

1. explicit install seams no longer hand-roll panic strings independently;
2. known icon install failures share one report type in `fret-icons`;
3. that report is available to diagnostics during the panic window without leaving stale global
   state behind;
4. bootstrap diagnostics can log structured icon-install context;
5. ordinary panic output remains human-readable;
6. ADR/alignment docs state the reporting contract explicitly.
