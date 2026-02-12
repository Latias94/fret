# Creative Recipes (v1)

This workstream turns the “creative visuals” ADR set into an implementable, ecosystem-friendly
surface without turning the kernel into a shader zoo.

Primary goals:

- Provide a stable, ergonomic authoring surface for component ecosystems (shadcn, magicui, app UIs)
  to compose creative looks.
- Keep kernel contracts portable and budgetable (capability gating + deterministic degradation).
- Ensure diagnostics/perf baselines remain meaningful during rapid ecosystem growth.

Key ADRs:

- Recipes/catalog authoring surface: `docs/adr/0245-ecosystem-visual-recipes-and-creative-authoring-surface-v1.md`
- Paint primitives (gradients): `docs/adr/0233-paint-primitives-brushes-and-gradients-v1.md`
- Controlled materials registry (Tier B): `docs/adr/0235-controlled-materials-registry-and-procedural-paints-v1.md`
- Masks (alpha masks): `docs/adr/0239-mask-layers-and-alpha-masks-v1.md`
- Compositing groups / blend modes: `docs/adr/0247-compositing-groups-and-blend-modes-v1.md`
- Frame clock + reduced motion: `docs/adr/0240-frame-clock-and-reduced-motion-gates-v1.md`
- Pointer motion snapshots: `docs/adr/0243-pointer-motion-snapshots-and-move-coalescing-v1.md`
- Procedural determinism: `docs/adr/0244-procedural-material-determinism-seeds-and-time-inputs-v1.md`
- Effect steps (threshold/matrix): `docs/adr/0236-effect-steps-color-matrix-and-alpha-threshold-v1.md`

## Target parity (initial)

Use the in-repo MagicUI reference (`repo-ref/magicui`) to define “creative baseline” targets.

Initial targets (v1):

- `magic-card` (pointer-follow radial gradients + border highlight)
- `lens` (radial mask + scaled content)
- `border-beam` (animated beam along border; compositing + mask semantics)
- `dot-pattern` / `grid-pattern` / `striped-pattern` (procedural patterns)
- `animated-grid-pattern` (procedural pattern + deterministic animation)
- `sparkles-text` (procedural sparkle field; determinism + reduced motion gating)

Non-goal (v1): pixel-perfect 1:1 CSS parity. The goal is a portable, budgeted look with stable
fallbacks and diagnostics.

## Layering plan (what goes where)

- `crates/fret-core`: portable value types and scene ops (`Paint`, `MaterialId`, mask/composite ops).
- `crates/fret-render*`: renderer-owned implementations, budgets, conformance tests, telemetry.
- `crates/fret-ui`: mechanism wrappers and derived inputs (pointer snapshots, frame clock reads).
- `ecosystem/fret-ui-kit`: recipes/catalogs and stable developer-facing helpers.
- `ecosystem/fret-ui-magic`: MagicUI-named wrappers that depend on `fret-ui-kit` recipes.

## Worktree strategy (recommended)

Avoid a single mega-branch. Split by “high-conflict kernel churn” vs “ecosystem surface”.

Suggested worktrees under `F:\SourceCodes\Rust\fret-worktrees\`:

- `ws/creative-recipes-foundation`:
  - only `ecosystem/fret-ui-kit` recipe trait + reporting + catalog scaffolding
  - migrate existing `glass`/`pixelate` to the shared shape
- `ws/paint-v1`:
  - `Paint` in `fret-core` + quad/path integration + renderer encode + conformance tests
- `ws/materials-v1`:
  - `MaterialId` + registry + baseline material kinds + diagnostics/telemetry
- `ws/masks-v1`:
  - `PushMask/PopMask` semantics + renderer support + demos/diag scripts
- `ws/composite-groups-v1`:
  - `PushCompositeGroup/PopCompositeGroup` + blend modes + budgeted intermediates
- `ws/pointer-motion-v1`:
  - pointer motion snapshots and local mapping helpers in `fret-ui`
- `ws/effect-steps-v1`:
  - `ColorMatrix` + `AlphaThreshold` effect steps + conformance tests

Rule of thumb:

- If a change touches `SceneOp` or renderer encoding, give it its own worktree.
- Keep recipe evolution in a separate worktree to reduce merge conflicts and maximize iteration speed.

## Acceptance criteria (v1)

- A small set of recipe wrappers exist in `fret-ui-kit` (or `fret-ui-magic`) that can reproduce the
  target parity list with stable fallbacks.
- `fretboard diag` scripts exist for each target parity demo and record:
  - screenshots (native + web when possible),
  - perf baselines for steady-state pointer-move and idle.
- Renderer conformance tests exist for:
  - gradient mapping correctness,
  - mask coverage behavior at edges,
  - blend group compositing order,
  - deterministic degradation behavior under reduced budgets.

## Current status (as of `main`)

Landed kernel primitives:

- Paint v1: gradients via `Paint` (`crates/fret-render-wgpu/tests/paint_gradient_conformance.rs`)
- Materials v1: controlled `MaterialId` registry + baseline procedural kinds
  (`crates/fret-render-wgpu/tests/materials_conformance.rs`)
- Masks v1: gradient alpha masks (`crates/fret-render-wgpu/tests/mask_gradient_conformance.rs`)
- Compositing groups v1: isolated groups + blend modes
  (`crates/fret-render-wgpu/tests/composite_group_conformance.rs`)
- Motion/pointer seams: pointer coordinate helpers + pointer motion snapshots + frame clock + diag
  fixed-delta support
- Effect steps extension: `ColorMatrix` + `AlphaThreshold`
  (`crates/fret-render-wgpu/tests/{effect_color_matrix_conformance.rs,effect_alpha_threshold_conformance.rs}`)
- Sampled materials v2a: renderer-owned catalog textures (ADR 0242)
  (`crates/fret-render-wgpu/tests/materials_sampled_conformance.rs`)

Not yet landed (tracked in this workstream):

1. `ecosystem/fret-ui-magic` Phase 0: seed components + diag scripts (in progress)
2. External texture imports v1: contract-path demo + a first real backend path (ADR 0234)
