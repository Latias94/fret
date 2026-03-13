# Font System (Fearless Refactor v1)

Status: active execution lane for the pre-release font-system reset

This workstream turns the existing font audit and roadmap into a **hard-reset execution plan**.

It is intentionally a **fearless refactor**:

- Fret is still pre-release.
- We do not need compatibility shims for mistaken public-looking seams.
- If a font/bootstrap/publication surface is structurally wrong, we should replace or delete it,
  not preserve it.

This document is **not** an ADR. If this workstream changes hard-to-change contracts, update the
relevant ADRs and alignment notes separately.

Primary inputs:

- `docs/workstreams/standalone/font-system-v1.md`
- `docs/audits/font-system-parley-zed-xilem-2026-02.md`
- `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`
- `docs/adr/0257-font-selection-fallback-and-variable-font-instances-v1.md`
- `docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`
- `docs/adr/0259-system-font-rescan-and-injected-font-retention-v1.md`

## Problem Statement

### From an app author's perspective

Fret already exposes enough knobs to make text correct, but the current author-facing outcome is
still too implicit:

- bundled font roles are not described as a stable product surface,
- generic-family behavior in bundled-only environments is only partially deterministic,
- settings/catalog behavior is split across multiple globals without one obvious "font environment"
  mental model.

### From a maintainer's perspective

The current architecture direction is mostly correct, but the implementation still has drift risk:

- runner publication order is not consistently atomic,
- bundled font family names are duplicated across crates,
- "catalog refresh happened" and "effective font environment changed" are still too easy to
  conflate,
- the rescan pipeline does not yet make no-op apply paths cheap enough.

### From a diagnostics and performance perspective

Text caches and font caches are now important enough that font-state churn must be treated as a
first-class architecture topic, not a local implementation detail.

If font-state publication or rescan semantics are vague, the framework pays for it in:

- unnecessary cache resets,
- avoidable relayout and glyph atlas churn,
- harder-to-interpret diagnostics bundles,
- platform-specific drift that only appears late in editor-grade scenarios.

## Key Findings From The Current Audit

### 1) Runner publication is not yet modeled as one atomic font-environment update

The current direction says the renderer owns `TextFontStackKey`, locale-aware fallback behavior,
and the effective fallback policy fingerprint. That is correct.

However, runner wiring still exposes multiple update steps that can drift in order. The clearest
example today is the Web startup path: the runner can publish `TextFontStackKey` before locale
application has converged, even though locale is part of the effective font-selection environment.

The problem is not "web only"; the deeper issue is that the framework still lacks one canonical
"publish renderer font environment to runtime globals" operation.

### 2) Bundled font roles are underspecified in code

`fret-fonts` provides useful bytes, but not a typed manifest that answers:

- which roles the bundle provides,
- which family names are expected to resolve from those bytes,
- whether a profile guarantees deterministic `Ui` / `Serif` / `Monospace`,
- which fallback tiers are present (CJK, emoji, other future script bundles).

As a result, multiple crates currently hard-code family names and curated defaults independently.

### 3) Rescan/apply semantics still invalidate too broadly

The current desktop rescan architecture is already much better than a synchronous UI-thread scan,
but the apply side still behaves too much like "rescan happened" instead of
"effective font environment changed".

For a framework with renderer text caches, no-op rescan application must be cheap and explicit.

### 4) Deterministic bundled-only behavior is not yet defined as a full product surface

We have the building blocks for deterministic bundled-only text:

- bundled bootstrap fonts,
- common fallback injection,
- system-font disable knob,
- deterministic tests and diag predicates.

What is still missing is a stronger statement of **what a bundled profile guarantees**.

In particular, generic-family semantics in bundled-only mode should be explicit rather than
accidentally inherited from ad-hoc curated defaults.

## Guiding Constraints

- Keep `fret-core::FontId` semantic and portable.
- Keep Parley/fontique as the single source of truth for family resolution and fallback selection.
- Keep the renderer as the owner of:
  - font selection,
  - fallback policy composition,
  - cache invalidation keys,
  - shaping/rasterization instance identity.
- Keep the runner as the owner of:
  - publication into globals,
  - async rescan orchestration,
  - user/app configuration plumbing.
- Make bundled-only environments a first-class, deterministic operating mode.
- Prefer deletion over compatibility when an existing surface is structurally wrong.

## Non-goals

- Replacing Parley/fontique with a different text backend in this workstream.
- Designing a full end-user variable-font axis UI.
- Adding every future script bundle now.
- Preserving the current helper split if a smaller and clearer contract is available.

## Target Shape

The reset should converge on a smaller set of explicit, auditable architecture pieces.

### 1) Introduce a single renderer-to-runtime font publication contract

We should have one canonical helper that publishes the **effective font environment** in an
explicit order.

Target flow:

1. Apply renderer-owned inputs:
   - font family config,
   - locale,
   - injected fonts or rescan result.
2. Let the renderer settle its derived environment:
   - `TextFontStackKey`,
   - fallback-policy fingerprint,
   - font DB revision.
3. Enumerate and publish runtime snapshots:
   - `FontCatalog`,
   - `FontCatalogMetadata`,
   - `FontCatalogCache`,
   - `TextFontStackKey`,
   - rescan state globals where relevant.

Rule:

- The runner must never publish a stale `TextFontStackKey` relative to the renderer's already
  applied locale/configuration state.

This likely means replacing the current scattered "set config / set locale / refresh catalog / set
key" call pattern with one shared helper and deleting the ad-hoc variants.

### 2) Make bundled font profiles manifest-driven

`fret-fonts` should stop being "only a byte bag" and become a bundle manifest surface.

Target concepts:

- `BundledFontRole`
  - `UiSans`
  - `UiSerif`
  - `UiMonospace`
  - `EmojiFallback`
  - `CjkFallback`
- `BundledFontFaceSpec`
  - family name,
  - role(s),
  - style/weight hints if needed,
  - payload reference
- `BundledFontProfile`
  - profile name,
  - provided roles,
  - expected family names,
  - determinism guarantees

This does **not** mean the runtime or UI should resolve fonts from the manifest directly. The
renderer still resolves actual availability from real bytes. The manifest exists to eliminate
cross-crate hard-coded family-name duplication and to make bundled-profile guarantees auditable.

### 3) Define bundled-only generic-family guarantees explicitly

We should stop treating generic-family behavior in bundled-only mode as an accident of curated
family lists.

Target rule:

- a bundled font profile explicitly declares which generic families it guarantees,
- runner bootstrap only seeds those generics from the profile manifest,
- if a generic is not guaranteed by the profile, the framework does not pretend otherwise.

This allows us to support multiple profiles without confusion:

- small web bootstrap profile,
- richer editor-grade profile,
- diagnostics/conformance profile.

The important part is not "one profile for everything"; it is that each profile has a typed,
auditable contract.

### 4) Make rescan application diff-based

The rescan pipeline should only trigger renderer-wide invalidation when the **effective** font
environment changed.

Target behavior:

- background rescan computes a comparable fingerprint of the resulting collection/environment,
- apply returns `false` for an effective no-op,
- no-op apply does not reset text caches or bump `TextFontStackKey`,
- injected-font retention remains bounded, but its consequences become diagnosable.

This keeps the current async architecture while making it cheaper and more predictable.

### 5) Keep fallback policy composition explicit and inspectable

The current direction is already correct:

- requested family/generic,
- script + locale fallback,
- curated/common fallback overrides,
- system-font availability,
- bundled-profile guarantees.

This workstream should turn that direction into a smaller and clearer implementation model:

- one renderer-owned fallback-policy object,
- one publication helper,
- one bundled-profile manifest source.

If a piece of fallback behavior is not explainable through those inputs, it is in the wrong place.

## Seams To Delete Or Replace

- Scattered runner update sequences that publish font globals in partially-settled states.
- Duplicated bundled family-name knowledge in `fret-runtime`, `fret-render-text`, and runner code.
- Any path where "font catalog refreshed" automatically implies "text caches must be reset".
- Any bundled-only behavior that silently depends on host-installed fonts or undocumented generic
  fallbacks.

## Recommended Execution Order

### Phase A — Contract consolidation

- Add this workstream and treat `standalone/font-system-v1.md` as background/audit input.
- Decide the canonical shared publication helper.
- Decide the bundled manifest vocabulary and profile names.

### Phase B — Runner publication reset

- Refactor desktop and web runners to use the shared publication helper.
- Remove stale ordering assumptions.
- Add explicit regression coverage for startup and post-update publication ordering.

### Phase C — Bundled profile reset

- Move bundled family-role knowledge into `fret-fonts`.
- Stop seeding family defaults from scattered hard-coded lists when a manifest-backed profile exists.
- Decide which bundled-only profiles guarantee which generics.

### Phase D — Rescan and invalidation closure

- Make rescan apply diff-based.
- Add no-op rescan tests and diagnostics coverage.
- Ensure no-op apply does not churn `TextFontStackKey`.

### Phase E — Close the diagnostics loop

- Keep `RendererTextFontTraceSnapshot` and fallback-policy snapshots aligned with the new contract.
- Add a human-auditable bundled-profile / font-environment view where needed.

## Expected Outcome

When this workstream is done, Fret should have a font system that is:

- architecturally simpler,
- easier to reason about during refactors,
- more deterministic on bundled-only targets,
- less churn-prone under rescans,
- more explicit about what is guaranteed versus best-effort.

That is the real goal of the fearless refactor: not more knobs, but fewer ambiguous seams.
