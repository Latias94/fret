# Font System Audit (2026-02): Parley/fontique vs Zed (cosmic-text) vs Xilem (Parley)

Scope: **font enumeration**, **caching**, **fallback chain semantics**, and **variable font instance identity**.

This audit is intended to inform and de-risk the active font-system workstream:
`docs/workstreams/font-system-v1.md` and ADR 0257:
`docs/adr/0257-font-selection-fallback-and-variable-font-instances-v1.md`.

## Baseline: current Fret direction

Fret’s stable text boundary is locked (ADR 0006). The renderer owns font selection and fallback.
Fret currently shapes via **Parley** and enumerates/selects fonts via **fontique** (Parley’s font system).

Key anchors:

- Renderer text + fallback injection: `crates/fret-render-wgpu/src/text/mod.rs`
- Parley/fontique integration + family metadata: `crates/fret-render-wgpu/src/text/parley_shaper.rs`
- Font bootstrap + `TextFontStackKey` invalidation: `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`

## Upstream patterns (repo-ref)

### Zed / GPUI (Linux): cosmic-text owns fallback, app layers cache and adapt

Zed’s Linux text system (`repo-ref/zed/crates/gpui/src/platform/linux/text_system.rs`) uses:

- `cosmic_text::FontSystem` for shaping and fallback.
- An application-side cache keyed by `(family, features)` to avoid repeated DB scans.
- A pragmatic “fallback font id injection” mechanism: when cosmic-text picks a fallback face, Zed materializes it into
  its own `FontId` list on demand (and explicitly documents that some metadata like features may be arbitrary for those
  fallback-only ids).

Takeaway for Fret:

- **Fallback is real-time and dynamic**: even with a requested family, the shaper may pick multiple fallback faces.
- It’s valuable to have a **debuggable trace** of what was actually selected.
- Caching family resolution results is worth doing at the renderer layer (or in a font-resolver wrapper).

### Parley / fontique: fallback is script+locale aware and backend-driven

Parley’s design doc (`repo-ref/parley/doc/design.md`) positions font selection/fallback as a first-class pipeline stage:

- selection based on script/locale and coverage,
- “no tofu” as a guiding correctness property.

Fontique (inside `repo-ref/parley/fontique/`) provides:

- system font enumeration per platform backend,
- fallback lookup keyed by script and locale (platform-dependent, but structured).

Takeaway for Fret:

- Treat **script+locale fallback** as the baseline on native platforms where system fonts exist.
- Use curated/bundled fallbacks as an **override tier** (especially for wasm/deterministic bundles), rather than the
  only fallback model.

### Xilem / Masonry: Parley-driven font stacks, variable fonts as “just weight”

Xilem exposes Parley-style `FontStack` (generic + named family) and demonstrates variable fonts by animating weight
(`repo-ref/xilem/xilem/examples/variable_clock.rs` + widget plumbing).

Takeaway for Fret:

- A good v1 API can keep variable fonts mostly implicit (weight/slant map onto axes when present).
- “Arbitrary axis UI” is useful, but can stay advanced/debug-only until the pipeline + cache keys are rock-solid.

## Findings: what to optimize in Fret (architecture)

### 1) Make selection auditable (diagnostics-first)

Problem:

- When “tofu” happens, it is hard to answer: “Which family did we request?” and “Which families did shaping actually
  use for runs?”

Recommendation:

- Keep a **renderer-owned per-frame font selection trace** in the diagnostics bundle, scoped and bounded.
- Ensure it can be enabled in a “record all” mode for deep investigation, but defaults to recording only missing-glyph
  cases to keep overhead low.

Status:

- Implemented (2026-02): `RendererTextFontTraceSnapshot` is included in bundles via `fret-bootstrap`.

### 2) Separate “policy composition” from “mechanism” in the renderer

Problem:

- Fallback behavior is currently expressed partially as “stack injection strings” and partially as backend fallback
  behavior inside fontique. Without explicit composition rules, it’s easy for behavior to drift.

Recommendation:

- Introduce a renderer-internal “font fallback policy” object whose inputs are explicit:
  - requested family or generic,
  - locale (bcp47) and derived script,
  - curated/bundled override tiers,
  - system-font availability (wasm vs native).
- Ensure a single “effective policy fingerprint” participates in `TextFontStackKey` so caches cannot alias.

### 3) Cache the expensive pieces at the right boundary

Problem:

- Family enumeration and “resolve family id” calls can be made frequently (settings UIs, debug tooling, catalog refresh).

Recommendation:

- Keep a renderer-side cache for:
  - `family_name(lowercase) -> FamilyId`,
  - `(generic_family, config_revision) -> resolved stack family ids`,
  - (optional) `(script, locale) -> fallback family ids` when system fonts are enabled.

### 4) Variable font instance identity: keep correctness, add debuggability

Problem:

- Normalized coords are correct for cache keys, but not human-friendly for debugging.

Recommendation:

- Keep normalized coords as the authoritative internal identity input,
  and add a **debug-only representation** (e.g. axis tags + values) into diagnostics traces.

## Proposed milestones (workstream alignment)

Use `docs/workstreams/font-system-v1.md` as the living tracker, with these “audit-informed” closures:

- M0: variable font identity correctness (coords + synthesis if needed).
- M1: explicit fallback composition rules (script/locale + curated overrides).
- M2: auditable traces + conformance strings (bundles + diag scripts).
- M3: optional public shaping knobs (features/variations).

