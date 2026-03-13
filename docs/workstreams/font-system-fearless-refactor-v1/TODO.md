# Font System (Fearless Refactor v1) — TODO

Status: active execution tracker; keep this updated as deletions and replacements land

Workstream design:

- `docs/workstreams/font-system-fearless-refactor-v1/DESIGN.md`

Background inputs:

- `docs/workstreams/standalone/font-system-v1.md`
- `docs/audits/font-system-parley-zed-xilem-2026-02.md`

## M0 — Workstream setup and contract lock

- [x] Create a dedicated fearless-refactor workstream directory for the font system.
- [x] Add the three core planning docs:
  - [x] `DESIGN.md`
  - [x] `TODO.md`
  - [x] `MILESTONES.md`
- [x] Cross-link the workstream from `docs/README.md`.
- [x] Decide which existing standalone font docs remain background/audit-only after this workstream
  becomes the execution lane.
- [x] Identify the ADR rows and alignment notes that must be updated when code lands:
  - [x] ADR 0147
  - [x] ADR 0257
  - [x] ADR 0258
  - [x] ADR 0259
  - [x] `docs/adr/IMPLEMENTATION_ALIGNMENT.md` entries as needed

## M1 — Reset runner publication to one canonical operation

- [x] Introduce one shared runner helper for publishing the settled renderer font environment.
- [x] Remove startup/update call sequences that publish `TextFontStackKey` before locale/config are
  fully applied.
- [x] Make desktop and web use the same publication ordering contract.
- [ ] Ensure the shared helper publishes:
  - [x] `FontCatalog`
  - [x] `FontCatalogMetadata`
  - [x] `FontCatalogCache`
  - [x] `TextFontStackKey`
  - [x] rescan-status globals where relevant
- [x] Delete obsolete helper paths once the canonical flow exists.

Regression gates:

- [x] Add helper-level regression tests for publication ordering and empty-entry startup
  preservation.
- [x] Add a targeted web startup test that fails if the published `TextFontStackKey` is stale
  relative to the renderer's current locale/configured environment.
- [x] Add a desktop-side regression test for the same invariant on startup/update flows.

## M2 — Make bundled font profiles manifest-driven

- [x] Extend `fret-fonts` from "bytes only" to "bytes + manifest/profile metadata".
- [x] Define typed bundled roles:
  - [x] `UiSans`
  - [x] `UiSerif`
  - [x] `UiMonospace`
  - [x] `EmojiFallback`
  - [x] `CjkFallback`
- [x] Define bundled profile metadata:
  - [x] profile name
  - [x] provided roles
  - [x] expected family names
  - [x] determinism guarantees
- [x] Move duplicated bundled-family knowledge out of:
  - [x] `fret-runtime` curated bootstrap defaults
  - [x] renderer fallback-policy hard-coded wasm assumptions
  - [x] runner bootstrap special cases
- [x] Decide whether the smallest web bootstrap profile guarantees `Serif` or explicitly does not.

Regression gates:

- [x] Add manifest consistency tests in `fret-fonts`.
- [x] Add a deterministic bundled-only test matrix covering:
  - [x] `Ui`
  - [x] `Monospace`
  - [x] `Serif` (guaranteed or explicitly unsupported-by-profile)
  - [x] CJK fallback
  - [x] emoji fallback

## M3 — Make rescan apply diff-based

- [x] Add a comparable fingerprint to the rescan result or a comparable renderer-owned environment
  snapshot.
- [x] Make `apply_system_font_rescan_result(...)` return `false` on an effective no-op.
- [x] Prevent no-op rescan apply from:
  - [x] bumping `TextFontStackKey`
  - [x] bumping font DB revision unnecessarily
  - [x] clearing text layout caches
  - [x] resetting glyph atlases
- [x] Keep injected-font retention bounded and documented.

Regression gates:

- [x] Add a unit/integration test for "no-op rescan does not change published font state".
- [x] Add a diagnostics/perf check showing that no-op rescan apply does not trigger cache-reset
  churn.

## M4 — Tighten fallback-policy composition and diagnostics

- [x] Keep one renderer-owned fallback-policy model as the only place where fallback composition is
  derived.
- [x] Ensure bundled profiles participate in that model explicitly rather than through duplicated
  hard-coded family lists.
- [x] Add a human-auditable bundled-profile / font-environment snapshot where useful.
- [x] Confirm diagnostics bundles remain interpretable after the refactor:
  - [x] font trace
  - [x] fallback policy snapshot
  - [x] registered-font-blob counters

Regression gates:

- [x] Extend the current mixed-script conformance coverage with profile-aware expectations.
- [x] Add at least one scripted diagnostics repro for bundled-only fallback behavior after the
  reset.
- [x] Add a native system-font mixed-script locale-switch conformance gate that proves
  `fallback_policy_key`, traced `locale_bcp47`, and zero-missing-glyph evidence move together on
  the platform-default/system-fallback lane rather than the curated common-fallback lane.

## Exit criteria for calling the workstream "closed enough"

- [x] There is one canonical runner publication path for font state.
- [x] Bundled font roles and profile guarantees live in one manifest-backed surface.
- [x] No-op rescan apply does not churn renderer caches or published keys.
- [x] Bundled-only determinism is explicit rather than inferred from scattered family-name lists.
- [x] The remaining font docs can clearly distinguish:
  - [x] architecture/contracts
  - [x] execution tracker
  - [x] audits/background rationale
