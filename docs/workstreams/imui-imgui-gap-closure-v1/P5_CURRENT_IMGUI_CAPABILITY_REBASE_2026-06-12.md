# ImUi Dear ImGui Capability Rebase - 2026-06-12

Status: Active source audit
Last updated: 2026-06-12

## Summary

This note refreshes the current IMUI capability read against `repo-ref/imgui` and the current Fret
sources. The main conclusion is unchanged, but sharper:

- Fret IMUI is no longer missing basic widget surface breadth.
- The remaining gap is product coherence, diagnostics reachability, multi-window feel, and public
  surface discipline.
- The next useful work is to deepen the workbench/product chain and keep new public surface behind
  evidence, not to mirror more Dear ImGui helper names.

## Current Read

The current Fret stack already has:

- a thin `fret-imui` facade,
- a policy-heavy `fret-ui-kit::imui` layer,
- editor-facing adapters in `fret-ui-editor::imui`,
- collection and workbench proof surfaces,
- diagnostics gates and recent-evidence tooling,
- docking and multi-window lanes that own the shell/backend feel.

Compared with `repo-ref/imgui`, the obvious missing class is not another basic widget:

- `ShowDemoWindow` / `ShowMetricsWindow` style always-available product entrypoints still want a
  cleaner first-open chain.
- The editor-grade proof route still needs to feel like one integrated workbench rather than a set
  of adjacent demos.
- Multi-window / tear-off hand-feel remains owned mostly by the docking lane, not by generic IMUI
  surface growth.
- `fret-ui-kit::imui` should keep acting as a contract/mechanism layer, not as a clone of Dear
  ImGui's helper namespace.

## Fearless Refactor Candidates

The next refactors that look justified by current evidence are:

1. Keep the public IMUI surface frozen unless a second first-party proof surface exists.
2. Promote the editor/workbench product route as the canonical first-open story.
3. Keep diagnostics/demo/metrics discovery in app-facing layers and out of `fret-imui`.
4. Continue splitting large IMUI owner hubs only when a hub is still doing too much real work.
5. Keep performance review work in perf-oriented lanes, not in widget/API backlog lanes.

## Not Yet Justified

Do not start a broad Dear ImGui helper cloning effort from this lane.
Do not reopen closed narrow follow-ons unless fresh source evidence says the closeout is wrong.
Do not move diagnostics UI into `fret-imui`.

## Next Slice

The next narrow slice should be one of:

- product-chain evidence refresh,
- editor workbench proof deepening,
- or a new narrow follow-on only if a specific missing capability is proven with a repro.
