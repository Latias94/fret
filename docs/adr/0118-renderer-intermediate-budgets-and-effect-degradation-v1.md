# ADR 0118: Renderer Intermediate Budgets and Effect Degradation (v1)

Status: Accepted (initial implementation landed)

## Context

Fret’s renderer is evolving to support optional multi-pass UI composition via an internal `RenderPlan` and
intermediate texture pool (ADR 0116). Public effect semantics (effect layers + backdrop filters) are expressed
in the display list as ordered `SceneOp`s (ADR 0117), and must preserve strict `Scene.ops` ordering (ADR 0002 / ADR 0009).

Multi-pass UI effects have two predictable failure modes if budgets are not explicitly designed:

1) **Unbounded GPU memory growth** (intermediate textures allocate per effect, per frame, per window).
2) **Unbounded GPU work growth** (blur kernels / sampling loops scale with radius and resolution).

In an editor-grade UI framework, both are unacceptable:

- performance must degrade gracefully and deterministically,
- disabling an effect must not break layout, hit-testing, or interaction policy,
- multi-window workloads must remain stable,
- wasm/WebGPU targets must have a clear fallback story.

This ADR locks:

- how the renderer accounts for intermediate resource usage,
- the scope of budgets (per-window vs global),
- deterministic degradation behavior when budgets are exceeded,
- observability requirements.

This ADR does **not** lock a specific effect catalog or exact visual outputs; it defines the framework-level
guardrails that make effects safe to adopt.

Related ADRs:

- Display list ordering: `docs/adr/0002-display-list.md`, `docs/adr/0009-renderer-ordering-and-batching.md`
- Renderer v3 substrate: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Effect semantics: `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Color/compositing: `docs/adr/0040-color-management-and-compositing-contracts.md`
- Scheduling (idle must be idle): `docs/adr/0034-timers-animation-and-redraw-scheduling.md`

## Decision

### 1) Budget scope: per-window budgets with an optional global cap

Budgets are enforced at the **window level** (per rendered surface), because:

- Fret is multi-window by design,
- windows may have different sizes and active effects,
- per-window budgets make degradation more predictable.

Additionally, the renderer may enforce an optional **global cap** as a last-resort safety valve.

### 2) Accounting: intermediate GPU memory is tracked in bytes (approximate but deterministic)

The renderer tracks intermediate texture allocations using an approximate byte accounting model:

- `bytes = width * height * bytes_per_pixel(format) * sample_count` (plus a small fixed overhead per allocation).

Notes:

- This is an accounting mechanism, not a promise about physical residency.
- The accounting must be deterministic for a given window configuration and viewport size.

The budget applies only to **renderer-owned intermediates** used for effects and internal multi-pass steps
(e.g. post-process ping-pong, downsample chains). It does not attempt to include:

- engine-owned viewport targets (registered via `RenderTargetId`),
- atlas textures (glyph/SVG/image atlases), which have their own budgets/policies.

This includes any renderer-owned **clip mask resources** required to implement soft clipping for effect passes
(ADR 0138). Clip masks are treated as intermediates for accounting purposes (format-dependent, often `R8Unorm`).

### 3) Quality tiers: effects must declare a bounded-cost "quality ladder"

Every effect step that can be degraded must expose a bounded-cost ladder. The renderer must be able to move
an effect from higher to lower tiers without introducing undefined behavior.

Minimum v1 ladder requirements:

- **Blur**: clamp radius; increase downsample factor (e.g. 1x → 2x → 4x); reduce passes/samples.
- **Pixelate**: increase pixel size (lower resolution sampling).
- **Dither**: switch to cheaper mode or disable.
- **ColorAdjust/ColorMatrix**: keep (cheap) unless the effect is entirely disabled.
- **Clip masks** (renderer-internal): lower mask resolution / disable mask generation (fall back to scissor-only)
  before disabling the effect group (ADR 0138).

### 4) Deterministic degradation: define an ordered set of degradations

When compiling a `RenderPlan`, if the required intermediates would exceed the current budget, the renderer
must apply degradations in a deterministic order until the plan fits.

Normative degradation order (v1):
 
0) Prefer reducing clip mask cost first (ADR 0138):
   - generate the mask at a lower resolution (aligned to the effect step’s downsample tier),
   - and if still over budget, disable mask generation (fall back to scissor-only behavior).
1) Increase downsample factors for the most expensive steps (typically blur).
2) Clamp blur radii to a budget-derived maximum.
3) Disable non-essential cosmetic steps (e.g. dithering) before disabling core steps.
4) If still over budget, disable the entire effect group (ADR 0117 rules):
   - `FilterContent`: render children directly into the parent target (no effect).
   - `Backdrop`: treat backdrop contribution as transparent; render children normally.

This order is chosen to preserve interaction clarity (glass remains a surface; pixelate remains stylized)
while avoiding pathological resource growth.

### 5) Layout invariants: degradation must not change geometry or hit-testing

Degradation is renderer-only and must not affect:

- layout geometry,
- clip geometry,
- hit-testing results,
- focus/navigation outcomes.

If an effect is disabled, it becomes a pure visual no-op; the subtree still exists and behaves identically.

### 6) Safety limits: hard caps independent of budgets

Independently of budget accounting, the renderer must enforce hard caps to prevent worst-case GPU work:

- Maximum blur radius (per tier).
- Maximum number of fullscreen passes per effect group.
- Maximum number of intermediate textures used by a single effect group.

These caps may be conservative in v1 and relaxed later with telemetry.

### 7) Observability: budgets and degradations must be measurable

The renderer must expose (at least in debug/perf snapshot mode):

- configured budgets (per window, optional global),
- per-frame intermediate peak bytes,
- pool reuse vs allocation counts,
- degradations applied (which step, which tier change, whether effect disabled).

This is required to make performance regressions diagnosable and to guide future contract evolution.

## Consequences

- Effects become safe to roll out incrementally without risking runaway allocations.
- Multi-window behavior becomes predictable: each window degrades independently.
- Future wasm/WebGPU targets have a clear fallback story (lower tiers or disable effects).

## Alternatives Considered

### A) No explicit budgets; rely on the OS/GPU driver

Rejected:

- leads to non-deterministic performance cliffs, OOMs, and hard-to-debug regressions.

### B) Global-only budgets

Rejected for v1:

- makes multi-window behavior surprising (a large window can degrade small windows).

## Open Questions

- Should budgets be user-configurable via theme/settings files, and if so, at which layer?
- How do we best represent budgets for HDR targets or non-8-bit formats when they are introduced?
- Do we need separate budgets per effect class (blur vs pixelate) vs a unified intermediate pool budget?

## Non-goals (v1)

- This ADR does not attempt to budget engine-owned targets (`RenderTargetId`) or long-lived atlases.
- This ADR does not promise a particular “quality” at a given budget value; it only constrains boundedness and
  determinism of degradation.

## Validation / Acceptance Criteria

Implementation is considered conformant when:

- For a fixed window size and config, the set/order of degradations applied is deterministic across runs.
- Peak intermediate byte accounting remains bounded by the configured per-window budget (within the chosen
  accounting model).
- When degradation disables an effect, layout and hit-testing remain unchanged (visual-only behavior).
- Debug/perf snapshots expose budgets, peak intermediate bytes, allocation/reuse counts, and applied degradations.

## References

- Bevy view target ping-pong patterns (conceptual reference): `repo-ref/bevy`
- ADR 0116 and ADR 0117: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`,
  `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
