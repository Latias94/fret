# Example Suite v1 — Web Support Tiers

This appendix defines what “web support” means for examples, so we can be explicit and avoid drift.

The goal is not to force parity everywhere. The goal is to keep a **curated web subset** high quality.

## Tier definitions

### Tier W0 — Web required (must run on wasm)

Use W0 for examples that:

- are core onboarding surfaces, or
- are portability contracts we want to keep honest early.

Expectations:

- runnable via `fretboard dev web --demo <id>`,
- stable `test_id`s,
- at least one scripted diag scenario.

Recommended W0 set (initial):

- `simple-todo` (onboarding baseline)
- `components_gallery` or a smaller “overlays basics” web-capable cookbook
- `external_texture_imports_web_demo` (interop portability)
- at least one effects/custom-effect demo (capability-gated) to keep WebGPU paths exercised

### Tier W1 — Web optional (nice-to-have)

Use W1 for examples where web support is valuable but not required for v1.

Expectations:

- can be added later without changing the example ID or the teaching intent,
- may have reduced fidelity (document degradations explicitly).

Examples:

- charts/plot cookbook slices
- markdown/code-view cookbook slices
- theme switching + token reads

### Tier W2 — Native-only (explicitly not a web target in v1)

Use W2 when the example depends on:

- multi-window / tear-off docking,
- platform-specific APIs (file dialogs, video import stacks),
- performance assumptions that do not translate to wasm yet,
- native-only diagnostics workflows.

Expectations:

- the example is still teachable and gated on native,
- docs explicitly say “native-only” and why.

Examples:

- multi-window tear-off docking
- some video import demos (platform-specific)
- some stress/perf harnesses

## Rule: web scope is a curated product surface

We do **not** attempt to “make all demos run on web” as a v1 goal.

Instead:

- keep W0 small and well-gated,
- allow W1 to grow organically,
- document W2 reasons and revisit when contracts mature.

## What counts as “supported on web”

For v1, “supported on web” means:

- there is an explicit demo ID in `fretboard` discovery,
- it runs without manual code edits,
- it has at least one gate (diag script or a deterministic smoke).

