---
name: fret-repo-orientation
description: "Get oriented in the Fret codebase (or when using Fret from an external app repo): identify the right layer/crate, find entry points fast, and choose the smallest runnable target for a change."
---

# Fret repo orientation (find the right place fast)

Fret is intentionally layered: **mechanism** lives in `crates/`, while **policy + recipes** live in
`ecosystem/`. If you start in the wrong layer, you will fight the architecture.

## When to use

- You are new to the Fret mono-repo and don’t know where a change should land.
- You are building an app *outside* the Fret repo and need to locate sources/contracts quickly.
- You need the smallest runnable repro target (demo/gallery) before touching code.

## Inputs to collect (ask the user)

Ask these before you start searching (saves hours of wrong-layer edits):

- What change are we trying to make (bug fix vs new feature vs refactor)?
- What user-facing invariant should change (behavior/UX/perf/contract)?
- What environment: native vs web; which runner; any platform constraints?
- Do we need a runnable repro target (which demo/gallery page) or is this purely contract/doc work?
- What regression artifact is expected (test, diag script, perf gate, ADR alignment)?

Defaults if unclear:

- Pick the smallest runnable demo target and start from architecture/ADR contracts first.

## Quick start

1. Read the “contracts first” docs:
   - `README.md`
   - `docs/README.md`
   - `docs/architecture.md`
   - `docs/runtime-contract-matrix.md`
2. Decide the layer:
   - mechanisms/contracts ⇒ `crates/`
   - interaction policy primitives (roving/typeahead/overlays) ⇒ `ecosystem/fret-ui-kit/`
   - shadcn-aligned composition + styling recipes ⇒ `ecosystem/fret-ui-shadcn/`
3. Pick the smallest runnable target:
   - `cargo run -p fretboard -- dev native --bin todo_demo`

## Workflow

### 1) Map the change to the correct layer (non-negotiable)

Use this mental model:

- `crates/fret-ui`: **mechanism/contract surface**, not a component library.
- `ecosystem/fret-ui-kit`: **headless policy + reusable infra** (roving, typeahead, overlay policy).
- `ecosystem/fret-ui-shadcn`: **shadcn v4 taxonomy + recipes** (composition + tokens + test_id conventions).

If the change is about:

- dismiss rules / focus restore / hover intent / keyboard navigation ⇒ almost always `ecosystem/`
- layout engine / hit testing / semantics contracts ⇒ likely `crates/`

### 2) Find entry points (fast paths)

In the mono-repo:

- UI authoring surface: `crates/fret-ui/src/elements/cx.rs` (`ElementContext`)
- shadcn recipes: `ecosystem/fret-ui-shadcn/src/`
- kit primitives: `ecosystem/fret-ui-kit/src/primitives/`
- diagnostics driver: `apps/fretboard/src/diag.rs`
- diag protocol types: `crates/fret-diag-protocol`

Quick search patterns:

```bash
rg -n "ElementContext" crates ecosystem
rg -n "OverlayController|OverlayRequest" crates ecosystem
rg -n "test_id\\(" ecosystem/fret-ui-shadcn
```

### 3) If you are in an external app repo (no mono-repo checkout)

Preferred: keep a lightweight Fret source checkout for browsing (submodule or sibling clone).

Fallback: browse Cargo registry sources for published crates:

- Registry source root is typically under `~/.cargo/registry/src/`
- Search for a crate folder like `fret-ui-*` then `rg` within it.

Notes:

- You won’t have `apps/fretboard` or `tools/` scripts in the registry sources.
- For “how to use the API”, prefer the published docs + crate `lib.rs` as the index.

### 4) Always leave a regression artifact

If you are changing interaction/state machines:

- Add a `tools/diag-scripts/*.json` scripted repro and gate it (`fret-diag-workflow`).

If you are changing layout/style parity:

- Add a small invariant test and/or parity harness case (`fret-shadcn-source-alignment`).

## Definition of done (what to leave behind)

- The change is mapped to the correct layer/crate (mechanism vs policy vs recipe) with a short rationale.
- A smallest runnable target is chosen (demo/gallery) when behavior is involved.
- The key evidence anchors are identified (docs/ADRs + entry points) so reviewers can verify the rationale quickly.
- A regression artifact exists for any behavior change (test and/or diag script and/or perf gate).

## Evidence anchors

- Repo positioning: `README.md`
- Docs index: `docs/README.md`
- Architecture layering: `docs/architecture.md`
- Runtime contract surface: `docs/runtime-contract-matrix.md`
- Repo structure: `docs/repo-structure.md`

## Common pitfalls

- Fixing a policy mismatch by adding runtime knobs in `crates/fret-ui` (wrong layer).
- Starting from a huge app target instead of a minimal demo/gallery page (slow iteration).
- Changing behavior without a gate (regressions return as “human timing” bugs).

## Related skills

- `fret-component-authoring`
- `fret-shadcn-app-recipes`
- `fret-shadcn-source-alignment`
- `fret-diag-workflow`
