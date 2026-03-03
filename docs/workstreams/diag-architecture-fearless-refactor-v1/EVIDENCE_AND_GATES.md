# Diagnostics Architecture (Fearless Refactor v1) — Evidence and Gates

Last updated: 2026-03-03

This file defines what “done” means for this workstream beyond subjective UX.

The guiding principle: diagnostics refactors must be protected by **portable artifacts** and **small deterministic gates**.

---

## 1) Required evidence anchors (update as code lands)

Contracts and foundations:

- ADR (bundles + scripts): `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- ADR (semantics): `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- Bundle + script workflow: `docs/ui-diagnostics-and-scripted-tests.md`
- Inspect/pick workflow: `docs/debugging-ui-with-inspector-and-scripts.md`

Workstreams:

- This workstream: `docs/workstreams/diag-architecture-fearless-refactor-v1/DESIGN.md`
- Simplification (artifact parity + transport): `docs/workstreams/diag-simplification-v1.md`
- Extensibility + capabilities: `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`
- DevTools GUI: `docs/workstreams/diag-devtools-gui-v1.md`

---

## 2) Implementation anchors (today)

Protocol (stable serde types):

- `crates/fret-diag-protocol/src/lib.rs`

Tooling engine (CLI/GUI shared):

- `crates/fret-diag/src/lib.rs`
- Artifacts seam: `crates/fret-diag/src/artifact_store.rs`
- Suite resolution seam: `crates/fret-diag/src/registry/suites.rs`
- Checks seam (post-run): `crates/fret-diag/src/registry/checks/mod.rs`
- Builtin suite mapping: `crates/fret-diag/src/diag_suite.rs` (`resolve_builtin_suite_scripts`)
- CLI wrapper: `apps/fretboard/src/cli.rs`

WS transport:

- `crates/fret-diag-ws/src/server.rs`
- `crates/fret-diag-ws/src/client.rs`

Runtime capture/export + script executor + inspector:

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`

Layout deep debug escape hatch (Taffy dump):

- `crates/fret-ui/src/tree/layout/taffy_debug.rs`
- env wiring: `crates/fret-ui/src/runtime_config.rs`

---

## 3) Artifact invariants (non-negotiable)

These are invariants for `fretboard diag run/suite/repro/perf`:

1. Every run writes a **stable script result** (`script.result.json`) even on tooling failures.
2. Every bundle dump yields a **local shareable directory** with a primary artifact:
   - prefer `bundle.schema2.json` when present,
   - keep `bundle.json` optional/compat.
3. Every failure path produces a stable `reason_code` and bounded structured evidence (no “just timeout”).
4. Tooling must be able to resolve “latest bundle for this run/session” deterministically (avoid global `latest.txt` races).
5. Any evidence clipping must be explicit in the artifact (counts, byte caps, and what was dropped).

---

## 4) Regression gates (required)

### 4.1 Format + compile (baseline)

- Prefer scoped formatting on Windows worktrees (avoid `os error 206` path length failures):
  - `cargo fmt --manifest-path crates/fret-diag/Cargo.toml`
- `cargo check -p fret-diag -p fret-diag-protocol -p fret-diag-ws`

Notes:

- `fret-diag-protocol` must remain wasm-friendly (do not pull native-only dependencies into it).

### 4.2 Focused tests (tooling + protocol)

Prefer `cargo nextest run` when available.

- `cargo nextest run -p fret-diag -p fret-diag-protocol -p fret-diag-ws`

### 4.3 Layering checks (when boundaries move)

- `python3 tools/check_layering.py`

### 4.4 Scripted diagnostics gates (interaction-level)

We require at least one “golden” gate suite that:

- runs scripts,
- emits a bundle,
- runs lint/triage checks,
- and exits deterministically when launched.

Recommended starting suites (native):

- `cargo run -p fretboard -- diag suite ui-gallery-layout --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard -- diag suite ui-gallery-shadcn-conformance --launch -- cargo run -p fret-ui-gallery --release`

Recommended perf gates (native):

- `cargo run -p fretboard -- diag perf ui-gallery --repeat 5 --warmup-frames 5 --sort time --launch -- cargo run -p fret-ui-gallery --release`

### 4.5 Layout correctness gates (semantics-first)

Requirements:

- at least one diag script asserts layout geometry using semantics-driven selectors (`test_id`) and bounds predicates,
- the script must not rely on pixel coordinates or screenshot diffs as the primary signal.

Optional “explainability sidecar”:

- if the gate fails, the repro should also produce a layout sidecar (e.g. Taffy subtree dump) to explain why.

---

## 5) Dev loop commands (recommended)

When iterating on tooling/diagnostics, keep artifacts isolated per task:

- Prefer a dedicated out dir:
  - `cargo run -p fretboard -- diag run <script> --dir target/fret-diag-issue-1234 --session-auto --launch -- <cmd...>`

To keep bundles small for AI/CLI loops (see details in `docs/ui-diagnostics-and-scripted-tests.md`):

```powershell
$env:FRET_DIAG=1
$env:FRET_DIAG_BUNDLE_SEMANTICS_MODE="changed"
$env:FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=1
$env:FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS=20
```

---

## 6) “No silent regressions” checklist (review time)

- Protocol changes are additive and versioned (no breaking changes without an ADR).
- New evidence fields are bounded and clipping is reported.
- New runtime diagnostics capture does not introduce per-frame allocation cliffs.
- Any new ecosystem extension path is capability-gated and does not weaken layering.
