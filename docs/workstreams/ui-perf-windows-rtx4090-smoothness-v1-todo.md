# UI Perf (Windows RTX4090) — Smoothness v1 (TODO)

## P0 — Gates (must stay green)

- [x] Run `ui-resize-probes` attempts=3 and capture the out-dir summary + worst bundles.
- [x] Run `ui-code-editor-resize-probes` attempts=3 and confirm no regression.
- [x] Run `ui-gallery-steady` repeat=3 against the Windows baseline and record failures (if any).

## P0 — Hitch Classes (make each explainable)

- [ ] Font rescan: confirm worst bundles do not include `TextFontStackKey` bumps inside measured windows.
- [ ] Resize tails: if failures persist, classify top frames by `layout_time_us` vs `paint_time_us`.
- [ ] `ui-gallery-steady`: investigate baseline failures after merging recent `main`:
  - [ ] `ui-gallery-view-cache-toggle-perf-steady` outlier dominated by `layout_time_us`
  - [ ] minor deltas in `material3-tabs-switch-perf-steady` and `virtual-list-torture-steady`

## P1 — Tooling / Protocol

- [x] Add a perf log entry (commit-addressable): command lines + out dirs + worst bundle anchors.
- [x] Lock UI gallery perf env defaults (cache+shell, VirtualList known heights) so `diag perf ui-gallery(-steady)` matches the baseline without manual flags.
- [ ] If needed, add a dedicated `ui-gallery-*` script that isolates a single failing steady workload.

## P1 — Hardening

- [ ] Ensure script changes do not rely on semantics capture (keep `FRET_DIAG_SEMANTICS=0` viable).
- [ ] Keep runner mitigations bounded (avoid deferring font rescan forever; avoid unbounded caches).
