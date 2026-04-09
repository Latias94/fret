# Notes — GPUI / Flutter-style Example Strategy (Adapted to Fret)

This appendix is a pragmatic “pattern extraction” note:

- what GPUI does well for examples,
- what Flutter does well for examples,
- how to translate those ideas into Fret’s layering and tooling.

## GPUI (Zed) pattern highlights

Pinned reference: `repo-ref/zed/crates/gpui/examples/`.

Observed traits:

1) **Many small examples**: each file demonstrates one concept (layout, popover, input, scrollable, data table).
2) **Runnable by convention**: `cargo run --example <name>` is a low-friction discovery+run loop.
3) **A real app is the final reference**: the Zed editor itself is the “app-scale” integration example.
4) **Examples are not the widget library**: they are usage references; the core library remains mechanism-focused.

## Flutter pattern highlights

Flutter’s public story tends to split into:

- “codelabs / getting started” (boring ladder),
- “cookbook” (how do I do X?),
- “gallery” (component catalog),
- “sample apps” (app-scale reference apps).

Two useful principles:

- Keep the onboarding ladder tiny and stable.
- Treat sample apps as product surfaces with ownership and regression gates.

## Translation for Fret

Fret has additional constraints:

- mechanism vs policy split (`crates/*` vs `ecosystem/*`) must stay clean,
- native + wasm parity is selective (WebGPU path exists, but not all workflows map 1:1),
- diagnostics/scripts are first-class (stable `test_id` is a contract surface).

Therefore, a good Fret example strategy is:

1) **Templates** (boring ladder): `cargo run -p fretboard-dev -- new hello|simple-todo|todo`
2) **Cookbook examples** (GPUI-like): small Cargo `examples/` (App/Interop/Renderer tracks)
3) **Gallery app** (catalog + conformance): `fret-ui-gallery`
4) **Reference apps** (Zed-like): 2–3 app-scale shells (workbench/viz-studio/shader-lab)
5) **Gates** everywhere:
   - diag scripts for behavior outcomes
   - perf baselines for perf-sensitive surfaces

## Suggested “topic set” for the cookbook (GPUI-aligned coverage)

GPUI’s examples cluster naturally into themes. A Fret cookbook can mirror those themes while staying ecosystem-first:

- Layout basics: flex, grid-like patterns, scrollables, responsive sizing.
- Input: text input, focus-visible, shortcuts/commands, hover/press states.
- Overlays: popover, dialog/sheet, menu, combobox/select (focus trap + restore).
- Lists/tables: virtualization, data table interactions, stable identity.
- Rendering semantics: shadows/blur/effects; then bounded custom effects.
- Diagnostics: inspector + scripted repro + bundle export.

## What “app examples” should look like in Fret

Reference apps should:

- have stable IDs and stable workflows,
- be intentionally opinionated (show the recommended seams),
- reuse cookbook/gates rather than duplicating bespoke automation.

They should not:

- become a dumping ground for experiments,
- pull mechanism layers into app code (policy belongs in ecosystem, mechanism stays in `crates/*`).
