# Resource Loading Fearless Refactor v1 — Current Incorrect Logic Audit

Status: Active debt map

Purpose:

- record the current wrong logic explicitly,
- stop the repo from drifting back toward path-first authoring,
- define when the remaining compatibility seams may be deleted.

## 1. Path-first image/SVG helpers are the wrong default story

Evidence:

- `ecosystem/fret-ui-assets/src/image_source.rs`
- `ecosystem/fret-ui-assets/src/svg_file.rs`
- `docs/adr/0317-portable-asset-locator-and-resolver-contract-v1.md`

What exists today:

- `ImageSource::from_file_path(...)` / `ImageSource::from_path(...)`
- `SvgFileSource::from_file_path(...)` / `SvgFileSource::from_path(...)`
- native-only `AssetLocator::file(...)` bridging for development flows

Why this is wrong as the main story:

- wasm/mobile cannot truthfully promise host file paths as the ordinary app-facing contract.
- packaged builds need logical identity (`bundle + key + revision`), not unstable host paths.
- path-first helpers bypass the main authoring vocabulary that the runtime can diagnose,
  invalidate, and map across platforms.
- reusable ecosystem crates should not teach app authors repo-relative or machine-local paths.

Correct direction:

- widget/component code should stay locator-first (`AssetLocator::bundle(...)`,
  `AssetRequest`, resolver-backed UI helpers).
- raw file paths remain a native/dev-only compatibility seam for local manifests, hot reload,
  and explicit platform API handoff.

Current deletion posture:

- keep the path constructors deprecated until first-party docs/examples no longer teach them,
  locator-first helpers cover the same development workflows, and first-party callers no longer
  need them on the default authoring lane.

## 2. Former partial `install(...)` semantics were misleading and are now deleted

Evidence:

- `ecosystem/fret-ui-assets/src/app.rs`
- `ecosystem/fret-ui-assets/src/advanced.rs`
- `docs/crate-usage-guide.md`

What exists today:

- explicit `configure_caches(...)` / `configure_caches_with_budgets(...)`
- explicit `configure_caches_with_ui_services(...)` /
  `configure_caches_with_ui_services_and_budgets(...)`

Why the old shape was wrong:

- the old `install*` names implied a fully wired subsystem.
- these cache setup functions only create caches and apply budgets; they do not drive
  `UiAssets::handle_event(...)`.
- partial setup hidden behind install-like naming made image/SVG readiness bugs look like runtime
  flakiness instead of an incomplete startup contract.

Correct direction:

- keep only the honest names (`configure_caches*`) on the app/advanced surfaces.
- any future fully wired startup story should live behind a higher-level bootstrap/bundle surface
  that also documents event-driving responsibilities.

Current deletion posture:

- deleted; no deprecated `install*` compatibility aliases remain on `fret-ui-assets`.

## 3. Font baseline behavior is improved but not fully closed

Evidence:

- `crates/fret-launch/src/runner/font_catalog.rs`
- `crates/fret-runtime/src/font_catalog.rs`
- `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`

What exists today:

- the framework now publishes `BundledFontBaselineSnapshot`.
- web and the current native winit path install the same bundled default profile and publish the
  same baseline snapshot shape.
- `FontFamilyDefaultsPolicy::None` keeps system-font augmentation additive instead of redefining
  the bundled baseline identity.

Why this is still not fully correct:

- the work is only partially aligned across all targets.
- mobile/package-specific startup evidence is still incomplete.
- fonts, images, and SVG text are not yet fully closed under one end-to-end asset publication story.

Correct direction:

- every platform should publish a framework-owned bundled baseline before first-frame text work.
- system font discovery stays a capability layer on top, not the source of baseline identity.
- font assets should participate in the same truthful resource-loading contract used by the rest of
  the runtime.
- SVG text should stay on the truthful "rejected until shared" baseline until the staged bridge in
  `SVG_TEXT_FONT_ENVIRONMENT_PLAN.md` lands.

Current deletion posture:

- do not close this audit item until `RESLOAD-font-300` is complete and the remaining
  `RESLOAD-font-310` / `RESLOAD-font-320` gaps are either implemented or explicitly retired.

## 4. Removal checklist for deprecated compatibility seams

The remaining deprecated path helpers may be deleted only after all of the following are true:

1. first-party docs, gallery snippets, cookbook examples, and scaffolds teach locator-first asset
   authoring and explicit cache/configuration semantics first;
2. app/bootstrap entry surfaces can cover the common development and packaged workflows without
   requiring ordinary app code to drop to raw file paths;
3. ecosystem libraries can publish package-owned bundle assets and icon packs through one named
   installer/bundle surface;
4. desktop/web/mobile all publish the same conceptual bundled font baseline before first-frame
   text work;
5. the remaining path-first compatibility shims are unused by first-party teaching surfaces and
   can be removed without reintroducing a platform lie.
