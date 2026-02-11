# `fret-docking`

Docking UI and interaction policy for editor-grade apps built on top of the `fret-ui` substrate.

This crate follows ADR 0075 (“Docking Layering, B route”):

- Dock graph, ops, and persistence are stable contracts in `crates/fret-core`.
- `crates/fret-ui` stays mechanism-only (routing, hit-test, overlays, performance substrate).
- Docking UI/policy lives in this crate (`ecosystem/fret-docking`).

Key docs:

- Dock ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Docking layering: `docs/adr/0075-docking-layering-b-route-and-retained-bridge.md`
- Arbitration checklist: `docs/docking-arbitration-checklist.md`
- ImGui parity matrix: `docs/docking-imgui-parity-matrix.md`
- N-ary split graph plan: `docs/workstreams/docking-nary-split-graph-v1.md`

## Reference repos (non-normative)

These are useful for aligning design intent and vocabulary:

- `egui_tiles` (N-ary linear containers + shares + simplification rules)
  - Upstream: https://github.com/rerun-io/egui_tiles
  - Local checkout: `F:\SourceCodes\Rust\fret\repo-ref\egui_tiles\`
- `dockview` (layout tree + panel state map separation; floating/popout state organization)
  - Upstream: https://github.com/mathuo/dockview
  - Local checkout: `F:\SourceCodes\Rust\fret\repo-ref\dockview\`
