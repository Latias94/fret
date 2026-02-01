# UI Gallery Performance (Interaction Latency) — TODO Tracker

Status: Draft

Tracking format:

- ID: `UIP-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

---

## Repro & Instrumentation

- [x] UIP-010 Identify the specific “card click latency” surface (page + element) and document the exact steps.
  - Target: UI Gallery sidebar nav item `Card` (`ui-gallery-nav-card`).
  - Start: `FRET_UI_GALLERY_START_PAGE=button`.
- [x] UIP-020 Add stable `test_id` anchors for the clickable card (or trigger) and the delayed region (e.g. detail panel).
  - Prefer: `apps/fret-ui-gallery/src/ui.rs` (demo surface), then shadcn component layer if needed.
  - Existing anchors used: `ui-gallery-nav-card`, `ui-gallery-nav-search`, `ui-gallery-page-card`.
- [x] UIP-030 Add a deterministic script that reproduces the latency and captures a bundle.
  - Repro: `tools/diag-scripts/ui-gallery-nav-card-click-latency.json`
- [ ] UIP-040 Add a perf harness invocation to the workstream doc with a captured “worst frame” bundle path and summary.

## Hypotheses to Validate

- [ ] UIP-100 Layout solve dominated by a large subtree re-measure on click (invalidations too wide).
- [ ] UIP-110 Text shaping/cache churn dominates the first post-click frame.
- [ ] UIP-120 GPU resource uploads or pipeline compilation stalling the first post-click paint.
