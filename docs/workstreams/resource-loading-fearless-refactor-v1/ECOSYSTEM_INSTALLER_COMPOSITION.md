# Ecosystem Installer Composition for Package Assets and Icon Packs

Status: Current first-party guidance (2026-03-15)

## Problem this note answers

When an app depends on a reusable ecosystem crate that itself depends on an icon pack or ships
images/SVG/fonts, who owns resource registration?

The short answer is:

- the ecosystem crate owns its internal resource registration,
- the app composes one installer/bundle value,
- widget code still consumes semantic `IconId` values or logical `AssetLocator::bundle(...)`
  lookups instead of replaying low-level mounts manually.

## The ownership split

### Package-owned shipped bytes

Reusable crates should publish shipped images/SVGs/fonts under `AssetBundleId::package(...)`.

That keeps resource identity stable across:

- desktop native packaging,
- web/WASM packaging,
- future mobile packaging,
- and in-tree generated/module-based startup flows.

The app should not copy those bytes into its own app bundle namespace unless it intentionally wants
to fork or override them.

### Icon semantics

Icons remain on `IconRegistry` / `IconId` as the component-facing semantic contract.

That means:

- vendor ids stay namespaced (`lucide.*`, `radix.*`, `my-kit.*`),
- semantic `ui.*` aliases are a separate policy layer,
- non-icon assets still go through the general asset resolver contract.

This is a hybrid model, not a duplication bug:

- icons use semantic ids for component authoring,
- package-owned shipped bytes use bundle/key lookup for general resource loading.

## The installer pattern

Reusable ecosystem crates should expose one app-facing installer surface that hides their internal
composition.

Typical shapes:

- `pub fn install(app: &mut App)`
- `pub struct Bundle; impl InstallIntoApp for Bundle`
- a named bundle type that composes multiple sub-installers internally

That installer may do all of the following:

- install one or more icon packs into `IconRegistry`,
- register package-owned bundle assets,
- install component globals/services,
- install shadcn/material recipe glue.

App code should usually do one of these:

```rust,ignore
FretApp::new("my-app")
    .setup(my_ecosystem::app::install)
```

or

```rust,ignore
FretApp::new("my-app")
    .setup((my_ecosystem::Bundle, my_other_ecosystem::Bundle))
```

The app should not usually do this:

- call `register_bundle_entries(...)` for the dependency's internal package assets by hand,
- re-register the dependency's icon pack manually,
- depend on the dependency's repo-relative file layout,
- or rewrite widget code to use raw filesystem paths.

## Conflict policy

### Package asset conflicts

Package assets use the general asset resolver precedence from ADR 0317:

- later registration wins for the same logical locator,
- builder and runtime registration order are preserved,
- package-owned bundles should therefore use stable package names to avoid accidental collisions.

### Icon conflicts

Icon conflict policy is intentionally different:

- vendor ids are namespaced, so `lucide.search` and `radix.magnifying-glass` do not collide,
- semantic `ui.*` aliases use `IconRegistry::alias_if_missing(...)` in first-party packs,
- so the default semantic policy is first-writer-wins,
- app/bootstrap code can still explicitly override later with `IconRegistry::alias(...)`.

Consequences:

- two dependency bundles can both install their own vendor icons safely,
- the first dependency that claims `ui.search` keeps that semantic default,
- the app can intentionally replace `ui.search` afterwards without replaying the dependency's asset
  mounts.

## Recommended publication strategy for ecosystem authors

1. Keep general shipped bytes under `AssetBundleId::package(...)`.
2. Keep icon semantics on `IconRegistry` / `IconId`.
3. Publish one installer/bundle surface that composes both.
4. Treat direct low-level registration APIs as crate-internal wiring, not the default app-facing
   teaching surface.

### Generated module vs higher-level installer

Use the generated `--surface fret` asset module directly when your crate only needs to publish
shipped bytes.

Typical examples:

- app-owned assets mounted through `generated_assets::mount(builder)?`,
- a reusable asset-only crate that can expose the generated `Bundle` / `install(app)` surface as-is,
- one package bundle with no additional icon packs, commands, theme wiring, or runtime globals.

The generated module can also publish first-party startup helpers such as
`preferred_startup_plan()` / `preferred_startup_mode()` when tooling wants to own the development
vs packaged switch directly.
Direct bootstrap apps can compose the same policy with
`fret_bootstrap::assets::{AssetStartupPlan, AssetStartupMode}` and
`BootstrapBuilder::with_asset_startup(...)` when they are not on the `fret` facade.

Wrap that generated module in a hand-written higher-level installer/bundle surface when the crate
also composes other app-facing responsibilities.

Typical examples:

- install one or more icon packs,
- compose multiple generated asset modules,
- install commands, settings, theme/bootstrap defaults, or other globals/services,
- intentionally publish one named dependency bundle so apps do not have to know your internal
  installer graph.

Rule of thumb:

- generated modules own low-level byte publication,
- hand-written installers own dependency composition and app-facing naming.

### `BundleAsset` vs `Embedded`

Prefer `BundleAsset` when the bytes are part of the crate's logical public lookup story.

That includes:

- images/SVGs/fonts that widgets resolve through `AssetLocator::bundle(...)`,
- assets that docs/examples should reference by stable logical key,
- resources that may need app overrides or resolver-order composition across crates.

Use `Embedded` when the bytes are lower-level owner-scoped publication and are not the public
cross-package lookup contract.

That includes:

- private generated bytes used by one crate's runtime/bootstrap internals,
- lower-level/package-local data publication where no stable bundle/key contract is being promised,
- cases where consumers should not depend on the asset identity directly.

If you are unsure, choose `BundleAsset`. It is the default teaching surface for app code and
reusable ecosystem crates.

## First-party evidence

- `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
- `apps/fret-cookbook/examples/app_owned_bundle_assets_basics.rs`
- `ecosystem/fret/src/integration.rs`
  - `ecosystem_bundle_installer_can_publish_package_assets_and_icons`
  - `app_follow_up_installer_can_override_semantic_icon_without_replaying_bundle_mounts`
- `ecosystem/fret-icons/tests/semantic_alias_conflicts.rs`
- `docs/adr/0317-portable-asset-locator-and-resolver-contract-v1.md`
