# Baseline Audit — 2026-04-09

Lane: `bootstrap-known-startup-failure-taxonomy-v1`
Status: Closed historical audit

## Question

After the closed icon-reporting lane, what startup/install failures are still “known” but not
recoverable through one bootstrap-level taxonomy?

## Findings

### 1) `BootstrapError` already names the major returned startup categories

Observed in `ecosystem/fret-bootstrap/src/lib.rs`:

- `Settings`
- `Keymap`
- `MenuBar`
- `AssetManifest`
- `AssetStartup`

Conclusion:

- the returned startup surface already had typed categories,
- but there was still no shared `known_failure_report()` bridge to normalize them.

### 2) Bootstrap diagnostics still logged an icon-only schema

Observed in `ecosystem/fret-bootstrap/src/lib.rs`:

- `init_panic_hook()` read
  `fret_icons::current_icon_install_failure_report_for_diagnostics()`,
- but it logged `icon_install_surface`, `icon_install_pack_id`, and
  `icon_install_failure_kind` directly.

Conclusion:

- diagnostics only recognized the icon panic path,
- and the field vocabulary no longer matched the broader bootstrap lifecycle question.

### 3) `fret::Error` split asset failures away from `BootstrapError`

Observed in `ecosystem/fret/src/lib.rs`:

- `Error::Bootstrap(BootstrapError)`
- `Error::AssetManifest(AssetManifestError)`
- `Error::AssetStartup(AssetStartupPlanError)`

Conclusion:

- even if `BootstrapError` grew a taxonomy helper, the app-facing facade still needed one bridge
  method to recover the same known report after asset failures were split into separate public
  variants.

### 4) The icon-reporting lane already ruled out broad lifecycle redesign

Observed in:

- `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/roadmap.md`

Conclusion:

- the right continuation is a narrow taxonomy follow-on,
- not a return-type redesign of `.setup(...)`, `init_app(...)`, or explicit icon install seams.

## Baseline verdict

Open a new narrow follow-on that:

- lives in `fret-bootstrap`,
- maps returned startup errors and icon panic reports into one known bootstrap report,
- adds one `fret::Error` bridge method,
- and keeps the root `fret` direct re-export budget unchanged.
