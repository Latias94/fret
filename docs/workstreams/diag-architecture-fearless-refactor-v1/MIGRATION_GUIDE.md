# Diagnostics Architecture (Fearless Refactor v1) — Ecosystem Migration Guide

Last updated: 2026-03-03

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

### 2.4 View extension payloads (tooling)

Once you have a bundle (from a script run or suite run), use the CLI viewer:

- list available keys:
  - `fretboard diag extensions <bundle_or_out_dir>`
- print a specific key:
  - `fretboard diag extensions <bundle_or_out_dir> --key dock.graph.v1 --print`
- emit structured JSON (useful for CI artifacts):
  - `fretboard diag extensions <bundle_or_out_dir> --key dock.graph.v1 --json --out exported.dock.graph.v1.json`

Notes:

- prefer `--warmup-frames <n>` when the first frames are noisy or incomplete.
- values may be clipped (look for `_clipped: true` in the payload).

### 2.5 End-to-end example: `dock.graph.v1`

This is the “golden path” example shipped in-tree:

1) Register the writer (runtime)

- Built-in registration:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/extensions.rs` (`default_debug_extensions_registry_v1`)
- Registering from an ecosystem crate:
  - call `fret_bootstrap::ui_diagnostics::register_debug_extension_best_effort(app, "my.feature.v1", writer)`
  - helper lives at: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

2) Capture a bundle

- Run a script/suite that exercises docking and produces a bundle.
- If you need a controlled reproduction, prefer a script that drives the minimal interaction.

3) Inspect the extension

- `fretboard diag extensions <bundle_or_out_dir> --key dock.graph.v1 --print`

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
