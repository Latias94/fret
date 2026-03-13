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
- [ ] Decide which existing standalone font docs remain background/audit-only after this workstream
  becomes the execution lane.
- [ ] Identify the ADR rows and alignment notes that must be updated when code lands:
  - [ ] ADR 0147
  - [ ] ADR 0257
  - [ ] ADR 0258
  - [ ] ADR 0259
  - [ ] `docs/adr/IMPLEMENTATION_ALIGNMENT.md` entries as needed

## M1 — Reset runner publication to one canonical operation

- [ ] Introduce one shared runner helper for publishing the settled renderer font environment.
- [ ] Remove startup/update call sequences that publish `TextFontStackKey` before locale/config are
  fully applied.
- [ ] Make desktop and web use the same publication ordering contract.
- [ ] Ensure the shared helper publishes:
  - [ ] `FontCatalog`
  - [ ] `FontCatalogMetadata`
  - [ ] `FontCatalogCache`
  - [ ] `TextFontStackKey`
  - [ ] rescan-status globals where relevant
- [ ] Delete obsolete helper paths once the canonical flow exists.

Regression gates:

- [ ] Add a targeted web startup test that fails if the published `TextFontStackKey` is stale
  relative to the renderer's current locale/configured environment.
- [ ] Add a desktop-side regression test for the same invariant on startup/update flows.

## M2 — Make bundled font profiles manifest-driven

- [ ] Extend `fret-fonts` from "bytes only" to "bytes + manifest/profile metadata".
- [ ] Define typed bundled roles:
  - [ ] `UiSans`
  - [ ] `UiSerif`
  - [ ] `UiMonospace`
  - [ ] `EmojiFallback`
  - [ ] `CjkFallback`
- [ ] Define bundled profile metadata:
  - [ ] profile name
  - [ ] provided roles
  - [ ] expected family names
  - [ ] determinism guarantees
- [ ] Move duplicated bundled-family knowledge out of:
  - [ ] `fret-runtime` curated bootstrap defaults
  - [ ] renderer fallback-policy hard-coded wasm assumptions
  - [ ] runner bootstrap special cases
- [ ] Decide whether the smallest web bootstrap profile guarantees `Serif` or explicitly does not.

Regression gates:

- [ ] Add manifest consistency tests in `fret-fonts`.
- [ ] Add a deterministic bundled-only test matrix covering:
  - [ ] `Ui`
  - [ ] `Monospace`
  - [ ] `Serif` (guaranteed or explicitly unsupported-by-profile)
  - [ ] CJK fallback
  - [ ] emoji fallback

## M3 — Make rescan apply diff-based

- [ ] Add a comparable fingerprint to the rescan result or a comparable renderer-owned environment
  snapshot.
- [ ] Make `apply_system_font_rescan_result(...)` return `false` on an effective no-op.
- [ ] Prevent no-op rescan apply from:
  - [ ] bumping `TextFontStackKey`
  - [ ] bumping font DB revision unnecessarily
  - [ ] clearing text layout caches
  - [ ] resetting glyph atlases
- [ ] Keep injected-font retention bounded and documented.

Regression gates:

- [ ] Add a unit/integration test for "no-op rescan does not change published font state".
- [ ] Add a diagnostics/perf check showing that no-op rescan apply does not trigger cache-reset
  churn.

## M4 — Tighten fallback-policy composition and diagnostics

- [ ] Keep one renderer-owned fallback-policy model as the only place where fallback composition is
  derived.
- [ ] Ensure bundled profiles participate in that model explicitly rather than through duplicated
  hard-coded family lists.
- [ ] Add a human-auditable bundled-profile / font-environment snapshot where useful.
- [ ] Confirm diagnostics bundles remain interpretable after the refactor:
  - [ ] font trace
  - [ ] fallback policy snapshot
  - [ ] registered-font-blob counters

Regression gates:

- [ ] Extend the current mixed-script conformance coverage with profile-aware expectations.
- [ ] Add at least one scripted diagnostics repro for bundled-only fallback behavior after the
  reset.

## Exit criteria for calling the workstream "closed enough"

- [ ] There is one canonical runner publication path for font state.
- [ ] Bundled font roles and profile guarantees live in one manifest-backed surface.
- [ ] No-op rescan apply does not churn renderer caches or published keys.
- [ ] Bundled-only determinism is explicit rather than inferred from scattered family-name lists.
- [ ] The remaining font docs can clearly distinguish:
  - [ ] architecture/contracts
  - [ ] execution tracker
  - [ ] audits/background rationale
