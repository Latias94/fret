# Bottom-Up Fearless Refactor v1 — Crate Audit Template

Status: Template (fill per crate; keep notes concise and evidence-backed)

This template is intentionally lightweight. The goal is to make reviews repeatable and to produce
actionable refactor steps that can land safely (small diffs, clear gates).

## Crate

- Name:
- Path:
- Owners / adjacent crates:
- Current “layer”: `crates/` kernel, backend adapter, renderer, ecosystem policy, app shell

## 1) Purpose (what this crate *is*)

1–5 bullets:

- …

## 2) Public contract surface

- Key exports / stable types:
- “Accidental” exports to consider removing:
- Feature flags and intent:

Evidence anchors:

- `crates/<crate>/src/lib.rs`

## 3) Dependency posture

- Backend coupling risks (`winit`, `wgpu`, `web-sys`, etc.):
- Layering policy compliance:
- Compile-time hotspots / heavy deps:

Evidence anchors:

- `crates/<crate>/Cargo.toml`
- `docs/dependency-policy.md`

## 4) Module ownership map (internal seams)

List the main subsystems and their primary files. Keep this short, but explicit.

- Subsystem A — responsibility
  - Files:
- Subsystem B — responsibility
  - Files:

## 5) Refactor hazards (what can regress easily)

Pick 3–10 hazards. Each hazard should eventually have at least one executable gate.

- Hazard:
  - Failure mode:
  - Existing gates (tests/diag):
  - Missing gate to add:

## 6) Code quality findings (Rust best practices)

Focus on high-impact issues:

- Error handling consistency:
- Ownership / clone discipline:
- Determinism (ordering, hashing, iteration):
- `unsafe` usage:
- Blocking or long work on UI thread:
- Serialization stability (if applicable):

Evidence anchors:

- File paths + key functions

## 7) Recommended refactor steps (small, gated)

Write this as an ordered list of “landable” steps. Each step should have an outcome.

1. Step — outcome — gate
2. Step — outcome — gate

## 8) Open questions / decisions needed

- …

