# Bootstrap Known Startup Failure Taxonomy v1

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
- `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

Status note (2026-04-09): this lane is now closed on one narrow bootstrap contract:

- returned bootstrap/startup failures and panic-only explicit icon install failures map into one
  `BootstrapKnownFailureReport`,
- `fret::Error` exposes one public bridge method for that taxonomy,
- bootstrap diagnostics log `bootstrap_failure_*` fields instead of an icon-only schema,
- and the surrounding lifecycle shape stays unchanged.

Read the landed proof in `M2_PROOF_SURFACE_2026-04-09.md` and the final verdict in
`CLOSEOUT_AUDIT_2026-04-09.md`.

This lane is a narrow follow-on to the closed `icon-install-error-reporting-v1` lane.
It does not reopen broad `Result` plumbing, startup recovery UI, or pack-crate dependency
direction.

It owns one narrower question:

> once known icon install failures already have a structured panic-time report, how should the
> wider bootstrap surface expose one durable taxonomy for both returned startup errors and
> panic-only explicit install failures without widening the root `fret` surface or redesigning the
> lifecycle?

## Why this lane exists

The closed icon-reporting lane solved only part of the startup story:

- explicit icon install failures had a known panic-time report,
- but returned startup failures from settings/keymap/menu/assets still surfaced as unrelated error
  types,
- and `fret::Error` split some bootstrap asset failures away from `BootstrapError`, which meant
  there was still no single app-facing introspection point for “known startup/install failure”.

This lane exists to close that taxonomy gap while preserving the already-closed lifecycle and
layering decisions.

## Assumptions-first baseline

### 1) Lane ownership

- Area: workstream ownership
- Assumption: this is a new narrow follow-on rather than a reopening of the closed
  icon-install-reporting lane.
- Evidence:
  - `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  - `docs/roadmap.md`
  - `docs/workstreams/README.md`
- Confidence: Confident
- Consequence if wrong: we would blur a closed icon-only reporting decision with a wider bootstrap
  lifecycle concern.

### 2) The taxonomy belongs in `fret-bootstrap`

- Area: layering
- Assumption: the right home is `fret-bootstrap`, because it already owns the lifecycle seam and
  already depends on settings/keymap/menu/assets/icons, while pack crates must not depend back on
  bootstrap.
- Evidence:
  - `ecosystem/fret-bootstrap/Cargo.toml`
  - `ecosystem/fret-bootstrap/src/lib.rs`
  - `ecosystem/fret/src/lib.rs`
  - `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- Confidence: Confident
- Consequence if wrong: we would either widen `fret-icons` into unrelated startup concerns or
  introduce reverse dependencies into pack crates.

### 3) Lifecycle return types remain out of scope

- Area: lifecycle shape
- Assumption: the right fix is not a broad `Result` conversion of `.setup(...)`, `init_app(...)`,
  or explicit icon install seams.
- Evidence:
  - `ecosystem/fret-bootstrap/src/lib.rs`
  - `crates/fret-launch/src/runner/desktop/runner/run.rs`
  - `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- Confidence: Confident
- Consequence if wrong: this lane would under-scope a broader bootstrap redesign.

### 4) The `fret` facade needs one bridge method, not a broader root export budget

- Area: public surface policy
- Assumption: the app-facing bridge should be `fret::Error::known_bootstrap_failure_report()`,
  while the root `fret` direct re-export budget stays closed.
- Evidence:
  - `ecosystem/fret/src/lib.rs`
  - `ecosystem/fret/src/lib.rs` (`authoring_surface_policy_tests`)
  - `docs/roadmap.md`
- Confidence: Likely
- Consequence if wrong: we would either keep app authors without a unified bridge or accidentally
  grow the curated root surface.

### 5) Diagnostics should emit bootstrap-level fields, not icon-only field names

- Area: diagnostics schema
- Assumption: once the taxonomy is broader than icon install, panic-hook logs should switch from
  `icon_install_*` fields to `bootstrap_failure_*`.
- Evidence:
  - `ecosystem/fret-bootstrap/src/lib.rs`
  - `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- Confidence: Likely
- Consequence if wrong: diagnostics would keep an icon-shaped schema even after bootstrap-level
  failure coverage exists.

## In scope

- Define a bootstrap-level known failure taxonomy with stable stage/kind/report primitives.
- Map returned `BootstrapError` variants into that taxonomy.
- Map `fret::Error` asset-split variants back into the same taxonomy.
- Map the existing icon panic-time report into the bootstrap taxonomy.
- Emit unified diagnostics panic fields for known bootstrap failures.

## Out of scope

- Converting bootstrap/setup/install surfaces to `Result`-based lifecycles.
- Startup recovery UI or richer app-facing remediation flows.
- Persistent diagnostics bundle capture for startup failures outside the panic-hook path.
- Adding a bootstrap dependency edge into pack crates.
- Growing `fret` root direct re-exports beyond the existing curated budget.

## Owning layers

- `ecosystem/fret-bootstrap`
  - owns the taxonomy types, returned-error mappings, and panic-hook logging
- `ecosystem/fret`
  - owns the app-facing facade bridge for the taxonomy
- supporting error sources
  - `crates/fret-app`
  - `crates/fret-assets`
  - `crates/fret-launch`
  - `ecosystem/fret-icons`

## Target shipped state

When this lane is done, the following must be true:

1. returned settings/keymap/menu/assets startup failures map into one known bootstrap report;
2. panic-only explicit icon install failures map into that same bootstrap report shape;
3. `fret::Error` exposes one method to recover the taxonomy even when asset failures are split into
   separate public variants;
4. bootstrap diagnostics log one bootstrap-level field family;
5. ordinary panic text remains human-readable;
6. the `fret` root direct re-export budget stays unchanged.
