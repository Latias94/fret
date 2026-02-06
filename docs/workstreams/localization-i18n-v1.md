# Localization / i18n v1 (Ecosystem Plan)

Status: Draft (workstream notes only; ADRs remain the source of truth)

Related architecture references:

- `docs/adr/0066-fret-ui-runtime-contract-surface.md` (keep `fret-ui` mechanism-only)
- `docs/adr/0037-workspace-boundaries-and-components-repository.md` (core vs ecosystem split)
- `docs/adr/0014-settings-and-configuration-files.md` (file-based typed settings)
- `docs/repo-structure.md` (incubate policy-heavy crates in `ecosystem/`)

Tracking file:

- `docs/workstreams/localization-i18n-v1-todo.md`

## Why this workstream exists

Fret already has:

- locale-aware text shaping/fallback internals in the renderer,
- direction primitives (`LTR` / `RTL`) at component level,
- and strongly typed settings and command surfaces.

What is still missing is a coherent, reusable i18n story for UI strings and app-facing text that:

1. preserves core layering contracts,
2. works on desktop + wasm now,
3. and is mobile-ready without rewriting APIs.

Without this, localization drifts into ad-hoc string tables and per-crate conventions.

## Delivery assumption (greenfield)

This workstream assumes a **greenfield localization baseline**:

- there is no existing localization subsystem to preserve,
- no legacy catalog format to migrate,
- and no compatibility obligations for previous i18n APIs.

Therefore, v1 is planned as a **one-shot architecture-first implementation**, not a staged migration.

## Scope

In scope (v1):

- localized UI messages for commands, menus, dialogs, labels, and component copy,
- deterministic locale fallback behavior,
- app-owned locale selection + runtime locale switching,
- wasm-safe resource loading and mobile-ready resource abstractions,
- ergonomics for component authors (low boilerplate message lookup).

Out of scope (v1):

- changing `fret-ui` runtime contracts to embed localization policy,
- full date/number/currency formatting strategy across all locales,
- translation management workflow tooling (TMS sync, crowd platforms),
- OTA localization patching and signed catalog delivery.

## Design principles

1. **Mechanism vs policy**
   - `crates/fret-ui` stays mechanism-only.
   - i18n policy, catalogs, and message lookup ergonomics live in `ecosystem/`.

2. **App-owned locale state**
   - Locale selection belongs to app/runtime settings (`settings.json` layering).
   - UI reads locale through host/global state, not global mutable singletons hidden in components.

3. **Portable execution model**
   - No hard dependency on threads or blocking filesystem APIs.
   - Resource loading must map to desktop, wasm, and mobile asset models.

4. **Stable keys, replaceable backends**
   - Fret-owned message key types and lookup traits are backend-agnostic.
   - `fluent-rs` is the default backend, but not a leaked hard contract.

5. **Ergonomics first for authors**
   - Message lookup should be one-liner in component code.
   - Runtime errors for missing keys should be diagnosable and testable.

## Proposed layering (modular and extensible)

### Layer A: Backend-agnostic i18n contracts (`ecosystem/fret-i18n`)

Primary responsibilities:

- define `LocaleId`, `MessageKey`, argument value types,
- define lookup/fallback trait(s),
- define catalog source abstraction and hot-reload events,
- expose app/component ergonomics extensions.

This crate must remain free of:

- parser/runtime implementation details of any specific i18n engine,
- platform-specific IO dependencies.

### Layer B: Fluent backend adapter (`ecosystem/fret-i18n-fluent`)

Primary responsibilities:

- implement Layer A traits using `fluent-bundle` + `fluent-fallback`,
- parse/load `.ftl` resources,
- provide fallback chain resolution and diagnostics,
- support pseudo-localization mode for UI QA.

Notes:

- Start with `fluent-bundle` and `fluent-fallback` as the stable base.
- Avoid coupling to unfinished parts of `fluent-resmgr` behavior.

### Layer C: App and component integration

- app-level locale source: typed settings + runtime switching,
- component-level text lookup helpers in `fret-ui-kit` / `fret-ui-shadcn`,
- first-pass localized surfaces for command/menu labels and shadcn copy.

## Wasm and future mobile support requirements

### Wasm (required in v1)

- support fully embedded catalogs (no filesystem assumption),
- optional async fetch/update path behind feature flags,
- no blocking calls in lookup or reload paths,
- size-conscious defaults (catalog slicing by locale and feature flags).

### Mobile readiness (design now, implement incrementally)

- catalog source abstraction must support platform asset bundles,
- locale negotiation API must support platform-preferred locales,
- runtime switching must avoid desktop-only watcher assumptions,
- avoid APIs that force `std::fs` or desktop path semantics.

## Ergonomics goals (component authoring)

Target authoring shape (conceptual):

- `cx.t("menu.file")` for plain messages,
- `cx.t_args("files.count", args)` for parameterized messages,
- optional `Localized` wrappers for command/menu metadata surfaces.

Ergonomics constraints:

- no repeated manual locale plumbing in every component constructor,
- no stringly fallback logic scattered in UI code,
- clear missing-key behavior (debug warning + deterministic fallback text).

## One-shot delivery strategy

1. Lock i18n contracts and key conventions up front (ADR + workstream gates).
2. Implement `ecosystem/fret-i18n` + `ecosystem/fret-i18n-fluent` in the same delivery window.
3. Land app-level locale settings/switching and primary UI surfaces together:
   - command metadata (`title`, `description`, `category`),
   - workspace/menu bar labels,
   - shadcn/calendar and other user-visible component copy.
4. Add wasm validation and pseudo-localization regression gates before expanding scope.
5. Keep mobile-ready source abstractions in v1 APIs to avoid post-v1 contract rewrites.

## Testing and observability

Validation stack:

- unit tests (message resolution, fallback, missing keys, args),
- integration tests (runtime locale switch updates command/menu/component text),
- wasm smoke tests (embedded catalog + runtime lookup),
- pseudo-localization snapshots for truncation/overflow detection.

Diagnostics:

- structured diagnostics for missing key / missing locale / formatter errors,
- stable counters/logs for lookup fallback hits,
- keep selector/test_id-driven UI tests resilient to copy changes.

## Risks and mitigations

1. **Risk: policy leaks into runtime crates**
   - Mitigation: keep all i18n logic in `ecosystem/*`; ADR update required before any runtime surface expansion.

2. **Risk: inconsistent locale switching behavior across platforms**
   - Mitigation: capability-driven resource source abstraction and platform parity tests.

3. **Risk: wasm binary size growth**
   - Mitigation: feature-gated locale packs, lazy loading strategy, and catalog partitioning.

4. **Risk: adoption friction for component authors**
   - Mitigation: provide minimal helper APIs + implementation recipes + examples.

## Milestones and exit criteria

Milestones, owners, and evidence anchors are tracked in:

- `docs/workstreams/localization-i18n-v1-todo.md`

## Baseline settings shape (greenfield)

Current settings contract for locale selection is implemented in `SettingsFileV1`:

```json
{
  "settings_version": 1,
  "locale": {
    "primary": "en-US",
    "fallbacks": ["zh-CN"],
    "pseudo": false
  }
}
```

Notes:

- `primary` uses BCP-47 language tag strings (invalid values fall back to `en-US`).
- `fallbacks` are optional and deduplicated at runtime.
- `pseudo` enables pseudo-localization rendering mode for UI QA.

## Completed in this iteration

- Added `ecosystem/fret-i18n` contract crate with:
  - locale/key/args domain types,
  - lookup trait + diagnostics model,
  - runtime `I18nService` (locale chain + pseudo mode + `t`/`t_args`).
- Added `ecosystem/fret-i18n-fluent` backend adapter with deterministic locale fallback tests.
- Extended `crates/fret-app/src/settings.rs` with locale settings and `i18n_service()` resolver.
- Unified settings application via `apply_settings_globals` to keep globals consistent:
  - `SettingsFileV1`,
  - `DockingInteractionSettings`,
  - `I18nService`.
- Wired the same settings->global path in:
  - config hot reload (`crates/fret-app/src/config_watcher.rs`),
  - bootstrap (`ecosystem/fret-bootstrap/src/lib.rs`),
  - ui gallery setup/runtime settings write-back (`apps/fret-ui-gallery/src/driver.rs`).

Remaining for Phase 3+:

- runtime locale switch command flow,
- first localized user-visible surfaces (command/menu/shadcn copy),
- wasm/mobile-specific validation lanes.
