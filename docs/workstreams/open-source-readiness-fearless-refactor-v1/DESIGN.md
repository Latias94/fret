# Open Source Readiness (Fearless Refactor v1)

This workstream focuses on making the repository **friendlier to first-time GitHub users** without
changing hard-to-change runtime contracts.

It is intentionally “fearless refactor”: we are willing to reorganize examples, defaults, and docs
to improve onboarding and reduce perceived chaos, while keeping core architecture boundaries intact.

This is **not** an ADR. If any change affects contracts (input/focus, overlays, docking, rendering
semantics), it must go through the ADR workflow.

## Problem statement

When opened on GitHub, the repo currently feels “large and demo-heavy”:

- Too many runnable-looking targets (demo shells, stress harnesses, galleries, cookbook, web harness).
- UI Gallery is comprehensive, but can be expensive (compile + load time), and links/pages can feel “too much” on day 1.
- The `fret` meta crate aims to be “batteries included”, but default features can still be tuned for:
  - smoother first-time app authoring,
  - fewer surprising side effects (filesystem/config),
  - less dependency weight when users want only a subset.

## Goals

- Make the “what do I run first?” answer obvious in < 60 seconds.
- Prefer **few, boring, repeatable** entry points:
  - templates ladder (`cargo run -p fretboard -- new ...`),
  - cookbook lessons (`cargo run -p fretboard -- dev native --example ...`),
  - gallery (component catalog) as an optional deep dive.
- Reduce cold-start friction:
  - fewer heavy defaults for day-1 examples,
  - optional feature-gating for “higher ceiling” examples/pages.
- Keep layering intact:
  - `crates/` stays mechanism/contract,
  - policy + components stay in `ecosystem/`,
  - runnable harnesses stay in `apps/`.

## Non-goals

- Writing a full tutorial site.
- Removing maintainer harnesses (they remain valuable).
- Re-architecting UI Gallery content (only making it easier to approach).

## Proposed changes

### A) Example suite: “lesson-shaped” demos move into cookbook

We treat `apps/fret-demo/src/bin/*` as a **maintainer/labs bucket** by default.

Policy:

- If a demo is a single concept that fits in one file and is copy/paste-friendly, it belongs in:
  - `apps/fret-cookbook/examples/`.
- If a demo is a stress harness, deep interop boundary, or regression harness, it stays in:
  - `apps/fret-demo` (hidden by default in `cargo run -p fretboard -- list native-demos`).

Expected outcomes:

- The cookbook becomes the primary “learn by running” surface.
- `fret-demo` remains a broad harness, but is no longer the first thing new users see.

### B) UI Gallery: add a “lite” onboarding mode

UI Gallery is valuable as a catalog + conformance surface, but it can be too much on day 1.

We introduce a “lite” mode:

**Implemented (compile-time feature gating + optional deps):**

- Default build shows a curated set of pages (core + shadcn).
- Dev/unfinished pages and their heavier dependencies are opt-in:
  - `--features gallery-dev`
- Material 3 pages are opt-in:
  - `--features gallery-material3`
- Full catalog is opt-in:
  - `--features gallery-full`

Details live in:

- [apps/fret-ui-gallery/README.md](../../../apps/fret-ui-gallery/README.md)
- [apps/fret-ui-gallery/Cargo.toml](../../../apps/fret-ui-gallery/Cargo.toml)

### C) `fret` meta crate: feature profiles for “smooth but not too heavy”

We want `fret` to be:

- pleasant for app authors,
- modular for advanced users,
- predictable for onboarding examples.

We define explicit profiles:

- `default`: `desktop` + `app` (shadcn-first; no filesystem config by default).
- `state`: opt-in for selector/query helpers in `ViewCx`.
- `diagnostics`: opt-in for dev tooling (tracing + panic hook + diag plumbing).
- `batteries`: opt-in bundle for “everything”: `state` + `diagnostics` + config files + ui-assets caches + icon packs + preloading.
- `config-files`: opt-in, because it can create filesystem side effects (`.fret/*`) and can surprise
  first-time users in a template/cookbook context.

In docs, we describe:

- “fastest minimal app”: `default-features = false`, pick a small feature set.
- “recommended app”: depend on `fret` defaults, then opt into batteries as needed.

### D) Cookbook curation (avoid “20 examples is the new chaos”)

Cookbook is valuable only if it has a clear order and avoids becoming a dump.

Policy:

- Keep a **recommended order** (5–8 examples) in `apps/fret-cookbook/README.md`.
- Keep higher-ceiling examples behind Cargo features.
- Tag each example in docs as one of:
  - `Official` (onboarding-friendly),
  - `Lab` (opt-in, higher ceiling),
  - `Maintainer` (regression/stress harness).

## Success metrics

- Time-to-first-run (native) is short and reliable.
- README links a small number of “canonical” things to run.
- Cookbook examples compile without pulling in unrelated subsystems.
- UI Gallery is still comprehensive, but feels optional and approachable.

## Related workstreams

- Examples redesign: `docs/workstreams/example-suite-fearless-refactor-v1/design.md`
- Framework modularity: `docs/workstreams/framework-modularity-fearless-refactor-v1/design.md`
