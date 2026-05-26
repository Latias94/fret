# IMUI Editor Workbench Golden Path v1

Status: Closed
Last updated: 2026-05-25

## Purpose

Fret has enough IMUI/editor/docking pieces to build real editor panels, but the current first-party
proofs are still distributed across focused demos:

- `imui_editor_proof_demo` proves rich IMUI/editor controls and app-owned collection behavior.
- `workspace_shell_demo` proves shell-mounted panes, tab strips, command scope, and editor rails.
- `editor_notes_demo` proves a smaller shell-mounted editor surface.
- `docking_arbitration_demo` proves docking and multi-window stress behavior.
- DevTools and diagnostics lanes prove script/bundle/compare/perf evidence.

This lane creates one canonical editor workbench golden path that can absorb those proofs over time
without blurring ownership. It is the product-facing route for "Fret can build a Dear ImGui-class
editor workbench" while keeping the implementation layered.

## Target State

- `cargo run -p fret-demo --bin imui_editor_workbench_demo` opens the canonical editor workbench.
- The route starts from the current shell/workspace proof and gradually converges the scattered
  editor proof surfaces into one coherent workflow.
- Focused demos remain useful as smaller proof surfaces; they are no longer the only way to discover
  the editor-grade story.
- Every meaningful expansion has a repro, a gate, and evidence.

## Owner Split

- `apps/fret-examples` owns the first-party demo composition and app-owned collection behavior.
- `ecosystem/fret-workspace` owns shell slots, tab strips, command scope, and workbench frame policy.
- `ecosystem/fret-docking` plus runner/backend owners own multi-window hand-feel.
- `ecosystem/fret-ui-editor` owns editor controls/composites and exposes thin IMUI adapters.
- `ecosystem/fret-ui-kit::imui` owns generic immediate widget policy and response vocabulary.
- `ecosystem/fret-imui` stays thin and policy-light.
- `apps/fret-devtools`, `apps/fret-devtools-mcp`, `crates/fret-diag`, and `ecosystem/fret-bootstrap`
  own Demo/Metrics/Debug discoverability and diagnostic productization.

## Non-Goals

- Do not introduce a second immediate runtime.
- Do not widen `crates/fret-ui` for Dear ImGui parity without ADR-backed mechanism evidence.
- Do not copy Dear ImGui's mutable stack grammar or string-label parser by default.
- Do not move app-owned collection semantics into `fret-ui-kit::imui` until the two-proof helper
  budget is satisfied.
- Do not close docking/Wayland/perf/DevTools gaps from this lane unless their owner-lane evidence
  actually proves them.

## Assumptions

- Confident: `imui-imgui-gap-closure-v1` is the current source-backed audit lane for IMUI gaps.
- Confident: `imui-editor-grade-product-closure-v1` is a maintenance umbrella and should not carry
  implementation-heavy work.
- Confident: `workspace_shell_demo` is the best current base for a canonical workbench route because
  it already owns workspace shell, tab strip, command scope, and editor rail proof.
- Likely: the first slice should create a named canonical entrypoint before moving large content.
- Likely: later content convergence should be vertical slices, not a single demo rewrite.

## First Slice

Add `imui_editor_workbench_demo` as the canonical route. Initially it delegates to the existing
`workspace_shell_demo` driver so the product entrypoint is runnable immediately. A source-policy test
locks the route as the canonical path and prevents it from forking the workspace shell proof before a
bounded migration task exists.
