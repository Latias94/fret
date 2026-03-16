# ADR 0317: Portable Asset Locator and Resolver Contract v1

## Upstream references (non-normative)

This document references public upstream docs for cross-platform asset packaging and lookup:

- Flutter assets and package assets: https://docs.flutter.dev/ui/assets/assets-and-images
- Compose Multiplatform resources: https://www.jetbrains.com/help/kotlin-multiplatform-dev/compose-multiplatform-resources-usage.html
- Swift Package resource bundles: https://developer.apple.com/documentation/xcode/bundling-resources-with-a-swift-package
- Android app resources overview: https://developer.android.com/guide/topics/resources/providing-resources

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Accepted

## Context

Fret currently has multiple partially overlapping resource stories:

- images through `ecosystem/fret-ui-assets`,
- SVG bytes through `crates/fret-ui` plus file helpers in `ecosystem/fret-ui-assets`,
- font bootstrap/publication through `crates/fret-launch`, `crates/fret-fonts`, and renderer code,
- icon semantics through `IconRegistry` and ADR 0065.

That split makes it too easy for desktop-first development shortcuts to become accidental
cross-platform contracts. The biggest drift is path-first authoring:

- widget-facing examples still expose filesystem paths,
- native/package-dev helpers are easy to discover,
- but wasm/mobile cannot truthfully promise raw host file paths as the default app story.

At the same time, Fret already has the first slices of a general asset contract:

- `crates/fret-assets` defines locator vocabulary, bundle identity, revisions, capabilities, and
  resolver traits,
- `crates/fret-runtime` hosts an ordered resolver stack,
- `ecosystem/fret` exposes builder/setup-facing asset registration surfaces,
- `fretboard` can generate builder-mountable and installer-mountable asset modules.

This ADR locks the portable v1 contract before more examples, ecosystem crates, or platform ports
grow around the wrong seams.

## Goals

1. Make logical bundle/key identity the default portable asset story.
2. Keep `file` and `url` truthful escape hatches instead of the golden path.
3. Lock ordered resolver precedence so builder mounts, static entries, and custom resolvers obey
   one override model.
4. Define revision and diagnostics expectations shared by images, SVGs, fonts, and future generic
   shipped bytes.
5. Give app authors and ecosystem authors one stable startup story that works across desktop, web,
   and future mobile targets.

## Non-goals

- Defining the final font baseline or font fallback policy. Those remain tracked by the font ADRs
  and the resource-loading workstream.
- Defining the final web hashed-output pipeline or mobile native resource mapping details.
- Replacing `IconRegistry` as the component-facing icon semantic contract from ADR 0065.
- Freezing every future loader/backend implementation detail.

## Decision

### 1) The default portable identity is logical `bundle + key`, not raw path or URL

The default author-facing and ecosystem-facing asset identity in Fret is:

- `AssetBundleId`
- `AssetKey`
- `AssetLocator::bundle(bundle, key)`

This is the identity that examples, generated asset modules, cookbook guidance, and reusable crate
contracts should teach first.

Rationale:

- the same logical asset must survive dev checkout paths, packaged desktop bundles, web outputs,
  and mobile app bundles,
- widget code must not encode packaging layout,
- and logical ids compose cleanly across app-owned and package-owned assets.

### 2) Locator kinds are explicit, but they are not equal in portability

The v1 locator vocabulary is:

- `Memory`
- `Embedded`
- `BundleAsset`
- `File`
- `Url`

Normative meaning:

- `BundleAsset`: portable published lookup identity; this is the default story for app-owned and
  package-owned shipped assets.
- `Embedded`: stable owner-scoped bytes for lower-level/runtime-facing cases where bytes are
  published directly by a package or generated module without first defining a public logical
  bundle story.
- `Memory`: ephemeral runtime-owned bytes (for example diagnostics captures or generated transient
  data) that still benefit from the same resolver/revision vocabulary.
- `File`: native/dev escape hatch; capability-gated and never the primary authoring contract.
- `Url`: explicit remote/deferred escape hatch; capability-gated and never the primary authoring
  contract.

Portable guidance:

- app code and reusable ecosystem crates should prefer `BundleAsset`,
- `Embedded` is allowed but is not the primary cross-package teaching surface,
- `File` and `Url` may exist in the API, but first-party docs/examples must label them as explicit
  escape hatches.

### 3) Bundle identity is namespaced by ownership

The authoritative v1 ownership model is:

- `AssetBundleId::app(...)` for app-owned assets,
- `AssetBundleId::package(...)` for reusable crate/package-owned assets.

Consequences:

- reusable libraries do not publish collision-prone global string soup,
- apps can reason about “who owns these bytes?” without reading implementation details,
- generated tooling can infer sensible defaults for app vs package resources.

Opaque legacy bundle strings may continue to exist temporarily for migration, but they are not the
recommended authoring surface and should not be expanded as the new default story.

### 4) Capability reporting must stay truthful

All hosts/resolvers that participate in the general asset contract must report truthful
`AssetCapabilities`.

The v1 capability axes are:

- `memory`
- `embedded`
- `bundle_asset`
- `file`
- `url`
- `file_watch`
- `system_font_scan`

Rules:

- `file` and `url` support must be treated as optional capability bits, not universal framework
  guarantees,
- platform-facing docs and diagnostics must remain honest when a locator kind is not available,
- `file_watch` and `system_font_scan` are augmentation capabilities, not prerequisites for the
  portable story,
- `crates/fret-ui` and other mechanism crates must not require host filesystem access to function.

### 5) Host resolver precedence is ordered and later registration overrides earlier registration

The host-level asset service is one ordered resolver stack shared by:

- primary resolver installation,
- additional resolver registration,
- bundle entry registration,
- embedded entry registration.

Normative precedence rule:

- later registration wins over earlier registration for the same logical locator,
- static entry layers and custom resolver layers participate in the same override model,
- replacing the primary resolver preserves its existing stack slot instead of silently jumping to
  the front of the stack.

This rule applies equally on:

- the runtime host path,
- the `fret` builder/startup path,
- and generated asset module installation.

### 6) Revisions are resolver-owned change tokens and must drive invalidation

Every successful resolution returns `ResolvedAssetBytes` with:

- the resolved locator,
- an `AssetRevision`,
- optional media type,
- and the resolved bytes.

`AssetRevision` is a resolver-owned change token. It must change whenever consumer-visible asset
content changes in a way that should invalidate caches or re-trigger decode/use paths.

Examples:

- generated bundled or embedded assets should use stable content-derived revisions,
- file-backed manifests may derive revisions from the current file contents,
- future hot-reload/watchers must publish new revisions when refreshed bytes differ,
- manifest regeneration or packaging changes that alter decode-relevant bytes/metadata must not
  silently reuse stale revisions.

### 7) Runtime diagnostics and startup/build diagnostics are distinct contracts

Runtime asset resolution must use the shared `AssetLoadError` surface.

The v1 meaning is:

- `ResolverUnavailable`: no host resolver service is installed.
- `UnsupportedLocatorKind`: no participating resolver stack can support the requested locator kind
  on this host.
- `NotFound`: the locator kind is supported, but the requested logical asset is absent.
- `StaleManifestMapping`: a file-backed manifest still claims the logical bundle/key mapping
  exists, but the mapped host file path is gone. This must not be collapsed into `NotFound`
  because the runtime did resolve the logical mapping and found a stale development/package-dev
  manifest contract instead of a truly missing logical asset.
- `AccessDenied`: the host or platform denied access.
- `Message`: bounded escape hatch for resolver-specific failures until richer typed diagnostics are
  added.

Startup/build-time artifact errors (for example manifest parse/validation/read/write failures) are
not the same contract. Those belong to dedicated artifact/tooling error surfaces such as
`AssetManifestLoadError`.

This separation keeps app/runtime diagnostics honest:

- “the manifest artifact is invalid” is not the same as “the runtime asset locator was missing”,
- and packaged startup validation should fail early instead of leaking vague runtime `NotFound`
  errors later.

### 8) App startup must have both a builder lane and an installer lane

Fret supports two first-class startup integration lanes:

- builder lane: `FretApp` / `UiAppBuilder` asset mount methods,
- app setup lane: named installers/bundles that implement `InstallIntoApp`.

Rules:

- generated asset modules should expose both a builder-facing mount (`mount(builder)`) and an
  app-facing installer/registration surface (`Bundle`, `install(app)`, or `register(app)`),
- reusable crates should expose named installer/bundle values instead of asking applications to
  replay the crate's internal low-level asset registration steps manually,
- app code should not need to know whether a dependency internally uses icon registries, package
  bundles, or both.

This keeps the user-facing story aligned with other mature UI ecosystems:

- package-owned resources remain packaged with the dependency,
- the app composes installers/bundles,
- and widget code still consumes stable logical ids.

### 9) Images, SVGs, and fonts share the identity model; `fret-ui` still consumes stable handles

Images, SVGs, fonts, and future generic bytes should share the same conceptual identity model and
resolution vocabulary.

However, `crates/fret-ui` remains a mechanism layer:

- it consumes stable handles, resolved bytes adapters, or published font-environment state,
- it does not become a packaging/path-manifest policy crate,
- it does not teach repo-relative paths as a component contract.

`IconRegistry` remains the component-facing semantic icon vocabulary per ADR 0065. The relationship
is hybrid, not duplicated:

- icon semantics resolve through `IconId`,
- non-icon shipped bytes resolve through the general asset contract,
- reusable crates may compose both behind one installer/bundle.

### 10) Deprecation and migration should be staged, but the teaching surface changes immediately

Legacy path-first helpers may remain temporarily as compatibility shims while the refactor is still
in flight.

But immediately:

- first-party docs/examples must stop teaching repo-relative paths as the default story,
- file-path helpers must be labeled as native/dev escape hatches,
- new reusable crate guidance must prefer package bundle ids plus named installer/bundle surfaces.

Deletion timing can follow after the builder/setup/package story is fully closed.

## Consequences

### Positive

- Desktop, web, and mobile can share one truthful default asset identity story.
- App authors get one stable composition model for first-party and ecosystem assets.
- Ecosystem crates can ship resources without leaking packaging internals into widget code.
- Resolver ordering becomes deterministic and reviewable.
- Images, SVGs, and fonts can converge on one revision/diagnostics vocabulary.

### Costs and risks

- More tooling work is required for packaged web/mobile output mapping and capability reporting.
- Remaining file-path examples and helpers now have explicit migration pressure.
- Fonts and SVG text still need follow-up work to fully align with this contract.

## Alternatives considered

### A) Keep raw file paths as the primary app-facing story

Rejected:

- it is false on wasm/mobile,
- it bakes packaging layout into widget code,
- and it does not scale to reusable ecosystem crates cleanly.

### B) Keep a separate loading contract per asset class

Rejected:

- it guarantees drift between images, SVGs, and fonts,
- complicates diagnostics and invalidation,
- and makes future tooling harder to standardize.

### C) Collapse icon semantics directly into generic asset bundle keys

Rejected for v1:

- component code still benefits from semantic `IconId` indirection,
- vendor icon packs need namespaced semantic behavior that is already covered by ADR 0065,
- and the app-facing contract only needs the two systems to compose cleanly, not to become one type.

## Implementation guidance (non-normative)

Primary evidence anchors for the current landed slice:

- `crates/fret-assets/src/lib.rs`
- `crates/fret-assets/src/file_manifest.rs`
- `crates/fret-runtime/src/asset_resolver.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/app_entry.rs`
- `apps/fretboard/src/assets.rs`
- `ecosystem/fret-ui-assets/src/asset_resolver.rs`
- `ecosystem/fret-icons/tests/semantic_alias_conflicts.rs`

Related contracts and workstreams:

- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`
- `docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`
- `docs/adr/0259-system-font-rescan-and-injected-font-retention-v1.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/TODO.md`
