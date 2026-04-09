# M1 Contract Freeze — 2026-04-09

Lane: `bootstrap-known-startup-failure-taxonomy-v1`
Status: Closed decision record

## Decision

Freeze the lane on the following contract:

1. `fret-bootstrap` owns the broader known startup failure taxonomy:
   - `BootstrapKnownFailureStage`
   - `BootstrapKnownFailureKind`
   - `BootstrapKnownFailureReport`
2. Returned startup failures map into that taxonomy through:
   - `BootstrapKnownFailureReport::from_bootstrap_error(...)`
   - `BootstrapKnownFailureReport::from_asset_manifest_error(...)`
   - `BootstrapKnownFailureReport::from_asset_startup_error(...)`
   - `BootstrapError::known_failure_report()`
3. Panic-only explicit icon install failures map into the same taxonomy through
   `BootstrapKnownFailureReport::from_icon_install_failure(...)`.
4. `fret::Error` exposes one app-facing bridge method
   `known_bootstrap_failure_report()` that recovers the taxonomy even when asset failures are split
   into separate public variants.
5. Bootstrap diagnostics switch to `bootstrap_failure_*` fields while keeping ordinary panic text
   human-readable.
6. The root `fret` direct re-export budget stays unchanged; the bridge method may return a
   `fret_bootstrap::...` type, but the facade does not grow new direct root re-exports for this
   lane.

## Why this is the right boundary

- `fret-bootstrap` already depends on the returned startup error sources and on `fret-icons`.
- `fret-icons` should not grow unrelated settings/keymap/menu/assets concerns.
- `fret` needs one bridge because its public `Error` splits asset failures away from
  `BootstrapError`.
- the root `fret` surface has explicit policy tests that keep direct re-exports tightly curated.

## Rejected alternatives

### 1) Broader `Result` plumbing

Rejected because:

- it reopens a closed lifecycle question,
- it widens scope from taxonomy/reporting into bootstrap execution semantics,
- and the closed icon-reporting lane already ruled it out.

### 2) Move the broader taxonomy into `fret-icons`

Rejected because:

- `fret-icons` should remain icon-focused,
- returned settings/keymap/menu/assets startup failures are not icon-layer concerns,
- and the lifecycle seam already lives above icons.

### 3) Add new root-level `fret` direct re-exports

Rejected because:

- the root surface has a deliberately closed direct re-export budget,
- and the public bridge method already gives app authors one recoverable entrypoint without
  widening that budget.
