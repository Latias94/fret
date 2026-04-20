# Resizable Adaptive Panel Proof v1 — Evidence and Gates

Status: Closed

## Smallest current repro

Use this sequence to reopen the shipped proof promotion:

```bash
rg -n "Adaptive Panel Proof|ui-gallery-resizable-adaptive-panel" apps/fret-ui-gallery/src/ui/pages/resizable.rs apps/fret-ui-gallery/src/ui/snippets/resizable/adaptive_panel.rs
cargo nextest run -p fret-ui-gallery --test resizable_docs_surface --no-fail-fast
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-adaptive-panel-proof.json --dir target/fret-diag/ui-gallery-resizable-adaptive-panel-proof --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery
```

What this proves:

- the `Resizable` docs path now teaches a fixed-window container-query proof directly,
- the promoted proof flips from `Wide panel` to `Compact panel` by moving only the splitter,
- and the run leaves bundle, screenshot, and layout-sidecar artifacts for review.

## Gate set

### Source-policy + behavior gates

```bash
cargo nextest run -p fret-ui-gallery --test resizable_docs_surface --no-fail-fast
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-adaptive-panel-proof.json --dir target/fret-diag/ui-gallery-resizable-adaptive-panel-proof --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery
```

### Lane hygiene

```bash
python3 tools/check_workstream_catalog.py
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/resizable-adaptive-panel-proof-v1/WORKSTREAM.json > /dev/null
git diff --check
```

## Current evidence after landing

- `apps/fret-ui-gallery/src/ui/pages/resizable.rs` now places `Adaptive Panel Proof` between
  `API Reference` and `Notes`.
- `apps/fret-ui-gallery/src/ui/snippets/resizable/adaptive_panel.rs` keeps a fixed shell, a real
  request-pane composition, and explicit
  `ui-gallery-resizable-adaptive-panel-state-{wide,compact}` selectors.
- `apps/fret-ui-gallery/tests/resizable_docs_surface.rs` locks the new section, ordering, and
  snippet proof vocabulary.
- `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-adaptive-panel-proof.json`
  captures before/after layout-sidecar, screenshot, and bundle evidence without relying on the
  docking demo.
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md` now cites this gallery proof as the promoted first-party
  fixed-window teaching surface for ADR 0325.

## Evidence anchors

- `docs/workstreams/resizable-adaptive-panel-proof-v1/DESIGN.md`
- `docs/workstreams/resizable-adaptive-panel-proof-v1/TODO.md`
- `docs/workstreams/resizable-adaptive-panel-proof-v1/MILESTONES.md`
- `docs/workstreams/resizable-adaptive-panel-proof-v1/CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/workstreams/resizable-adaptive-panel-proof-v1/WORKSTREAM.json`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/roadmap.md`
- `apps/fret-ui-gallery/src/ui/pages/resizable.rs`
- `apps/fret-ui-gallery/src/ui/snippets/resizable/adaptive_panel.rs`
- `apps/fret-ui-gallery/src/ui/snippets/resizable/notes.rs`
- `apps/fret-ui-gallery/tests/resizable_docs_surface.rs`
- `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-adaptive-panel-proof.json`
