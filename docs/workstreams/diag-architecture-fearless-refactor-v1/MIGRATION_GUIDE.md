# Diagnostics Architecture (Fearless Refactor v1) — Ecosystem Migration Guide

Last updated: 2026-03-02

This guide is for ecosystem crate authors who want to:

- make their components easier to debug with Fret’s diagnostics (inspect/pick/scripts/bundles),
- add extension diagnostics payloads without bloating the core schema,
- and add deterministic regression gates (scripts + assertions) for behavior/layout.

This is a living guide; keep it short and actionable.

---

## 1) Make your UI debuggable (baseline)

### 1.1 Prefer stable semantics selectors

Rules of thumb:

- stamp `test_id` on:
  - triggers,
  - primary actions,
  - important list items,
  - container roots that need stable scope selection.

Avoid:

- selectors based on localized labels for automation,
- pixel-coordinate-based scripts.

### 1.2 Provide debuggable container handles

If a component has an important “scope root” (dialog content, menu content, dock panel root), attach a stable semantics
node so scripts can scope queries (e.g. `exists_under(scope, target)`).

---

## 2) Add a diagnostics extension payload (proposed v1 contract)

### 2.1 Pick an extension key

Use a namespaced, versioned key:

- `fret.<crate_or_domain>.<feature>.v1`

Examples:

- `fret.docking.interaction.v1`
- `fret.ui_kit.virtual_list.v1`

### 2.2 Keep payloads bounded

Requirements:

- include `schema_version` inside the payload,
- cap size (bytes and count),
- include a clip report when dropping data.

### 2.3 Capability-gate optional dependencies

If a script depends on your extension:

- declare a required capability in script meta,
- fail fast with a stable reason code when missing.

---

## 3) Add regression gates (scripts)

### 3.1 Add a script

Put new scripts under `tools/diag-scripts/` with stable selectors.

Prefer asserting:

- focus/overlay state via semantics and traces,
- bounds predicates for layout correctness,
- deterministic “wait until” predicates (avoid raw delays).

### 3.2 Add (or extend) a suite

If the scripts form a group:

- add a suite entry (preferred) rather than many ad-hoc commands.

### 3.3 Gate in CI/dev loops

At minimum:

- `cargo run -p fretboard -- diag run <script> --launch -- <demo cmd>`

For layout-heavy changes:

- add a `diag perf` gate suite and set thresholds appropriate for your scenario.

---

## 4) Layout debugging guidance (correctness-first)

Preferred gate style:

- semantics-driven bounds predicates (`test_id` + stable predicates).

Optional “explainability”:

- capture a layout sidecar (e.g. Taffy dump) only when diagnosing a regression, not as the primary gate.

---

## 5) Common pitfalls

- Shipping huge extension payloads every frame (will hurt perf and artifact size).
- Relying on labels/value strings for selectors (localization breaks scripts).
- Writing scripts that use fixed frame delays instead of `wait_until`.
- Adding new typed fields to the core debug snapshot for ecosystem-only needs (prefer extensions).

