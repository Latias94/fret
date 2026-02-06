# Localization / i18n v1 (Tracking)

Last updated: 2026-02-06

This file tracks concrete implementation work for:

- `docs/workstreams/localization-i18n-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

## Phase 0 - Foundation and inventory

- `[x]` Add the v1 workstream plan document.
  - Evidence: `docs/workstreams/localization-i18n-v1.md`
- `[x]` Add this milestone tracker document.
  - Evidence: `docs/workstreams/localization-i18n-v1-todo.md`
- `[ ]` Inventory user-visible strings in high-impact surfaces:
  - command metadata,
  - workspace/default menu labels,
  - `fret-ui-shadcn` component copy (calendar/date/time/form helpers).
- `[ ]` Produce baseline implementation map (`surface` -> `message key namespace`) with owners.

## Phase 1 - i18n core contract crate (`ecosystem/fret-i18n`)

Goal: lock a backend-agnostic, ergonomic API without changing runtime contracts.

- `[x]` Scaffold `ecosystem/fret-i18n` crate in workspace.
- `[x]` Define stable domain types:
  - `LocaleId`,
  - `MessageKey`,
  - message argument value enum.
- `[x]` Define lookup traits and fallback contract semantics.
- `[ ]` Define catalog source abstraction for:
  - embedded/static catalogs,
  - async catalogs,
  - hot-reload capable sources.
- `[x]` Define diagnostics model:
  - missing key,
  - missing locale,
  - formatting/runtime error.
- `[x]` Add unit tests for contract behavior and edge cases.

Exit criteria:

- clear API docs,
- no dependency on `fret-ui` internals,
- no direct dependency on platform-specific IO.

## Phase 2 - Fluent backend adapter (`ecosystem/fret-i18n-fluent`)

Goal: provide a production default backend using `fluent-rs` while keeping backend replaceable.

- `[x]` Scaffold `ecosystem/fret-i18n-fluent` crate.
- `[~]` Implement contract adapters on top of:
  - `fluent-bundle`,
  - `fluent-fallback`.
- `[x]` Implement `.ftl` resource loading for embedded and memory-backed sources.
- `[x]` Implement deterministic fallback chain behavior and tests.
- `[~]` Add pseudo-localization mode for UI QA and snapshot tests.
- `[ ]` Add docs/examples for key naming and resource layout conventions.

Exit criteria:

- adapter passes contract tests from Phase 1,
- missing-resource behavior is diagnosable and deterministic,
- wasm-compatible loading path exists (embedded-first).

## Phase 3 - App-level locale settings and switching

Goal: app-owned locale state and runtime switching through existing config layering.

- `[x]` Extend app settings model with locale fields:
  - primary locale,
  - fallback locales,
  - optional pseudo-locale mode.
- `[x]` Update layered config load/merge behavior accordingly.
- `[~]` Add runtime locale switch command flow and effect wiring.
- `[ ]` Add integration tests for:
  - settings load with defaults,
  - runtime switch updates visible text,
  - deterministic fallback resolution.

Targets:

- `crates/fret-app/src/settings.rs`
- `crates/fret-app/src/config_files.rs`
- app/driver integration points (workspace/demo shells)

Exit criteria:

- locale is app-owned and persisted via settings,
- no global mutable singleton hidden in component layer.

## Phase 4 - Surface implementation (ergonomics-first)

Goal: implement high-value surfaces directly on the new i18n contracts with minimal boilerplate.

- `[ ]` Command metadata localization implementation:
  - `title`, `description`, `category` use message keys or localized wrappers.
- `[ ]` Workspace/default menu label localization implementation.
- `[ ]` `fret-ui-shadcn` calendar locale model implementation using i18n provider.
- `[~]` Add helper APIs for UI authors:
  - simple `t(key)` lookup,
  - argument-aware lookup,
  - optional strongly typed key wrappers.

Progress notes (2026-02-06):

- Added `app.locale.switch_next` core command and command handling wiring in:
  - `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
  - `apps/fret-ui-gallery/src/driver.rs`
- Added first localized high-visibility surface:
  - core app command titles/categories now resolve via i18n catalog keys
  - localization is reapplied when locale changes
- Added tests for:
  - locale switch command rotation,
  - localized command metadata projection.
- `[ ]` Update at least one template/demo to showcase best-practice i18n usage.

Exit criteria:

- component authors can localize without manual locale threading noise,
- high-traffic user-visible strings no longer hardcoded.

## Delivery mode - one-shot greenfield implementation

- `[x]` Confirm this workstream is greenfield (no previous localization subsystem to migrate).
- `[~]` Keep all phases aligned to one delivery train (contract + backend + app integration + primary surfaces).
- `[x]` Avoid introducing temporary compatibility layers unless explicitly required by later product constraints.

## Phase 5 - wasm support hardening

Goal: ship a robust wasm path in v1.

- `[ ]` Provide embedded-catalog-only mode as the default wasm baseline.
- `[ ]` Add optional async catalog loading mode behind feature flags.
- `[ ]` Add wasm smoke tests covering:
  - lookup,
  - fallback,
  - runtime switch.
- `[ ]` Add size/perf budget checks for localization assets in wasm builds.

Exit criteria:

- no filesystem assumptions in wasm path,
- predictable startup behavior with embedded resources,
- documented trade-offs for async/lazy catalog loading.

## Phase 6 - mobile readiness (design gates + early implementation)

Goal: avoid desktop-only assumptions and keep the API portable to mobile runners.

- `[ ]` Define mobile asset-source contract requirements in the i18n source abstraction.
- `[ ]` Define locale negotiation inputs from platform-preferred locales.
- `[ ]` Add capability flags for resource source constraints where needed.
- `[ ]` Add a mobile-readiness checklist document section with explicit non-goals for v1.

Exit criteria:

- no API in i18n core requires desktop paths/watchers,
- extension points for mobile assets/locale APIs are explicit.

## Phase 7 - observability, diagnostics, and QA

Goal: make localization regressions easy to detect and triage.

- `[ ]` Add structured diagnostics counters for fallback/missing-key events.
- `[ ]` Add pseudo-locale screenshot/scripted QA lane for truncation/overflow checks.
- `[ ]` Add guidance for selector/test strategy resilient to localized copy changes.
- `[ ]` Add troubleshooting playbook section for localization issues.

Exit criteria:

- localization regressions are reproducible via diagnostics and scripts,
- core demos have at least one localized regression gate.

## Phase 8 - ADR and contract alignment

Goal: lock hard-to-change contract decisions before broad rollout.

- `[ ]` Draft ADR for i18n ecosystem contract surface and layering boundaries.
- `[ ]` Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` after implementation milestones land.
- `[ ]` Link accepted decisions from roadmap and docs index.

Exit criteria:

- contract-level decisions are auditable,
- implementation evidence anchors are maintained.

## Current evidence anchors (2026-02-06)

- `ecosystem/fret-i18n/src/lib.rs`
- `ecosystem/fret-i18n-fluent/src/lib.rs`
- `crates/fret-app/src/settings.rs`
- `crates/fret-app/src/config_files.rs`
- `crates/fret-app/src/config_watcher.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `apps/fret-ui-gallery/src/driver.rs`
