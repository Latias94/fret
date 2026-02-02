---
title: UI Performance: Zed-level Smoothness v1 (Log)
status: draft
date: 2026-02-02
scope: performance, profiling, regression-gates
---

# UI Performance: Zed-level Smoothness v1 (Log)

This document is a **chronological, commit-addressable performance log** for the workstream:

- `docs/workstreams/ui-perf-zed-smoothness-v1.md`

The goal is traceability:

- which commit changed what,
- what was measured,
- what improved/regressed (with numbers),
- where the evidence bundle lives.

Notes:

- These numbers are *machine-dependent*. Always record the machine profile and the exact command.
- Prefer suite-level summaries (p50/p95/max) and keep raw bundle paths for the worst runs.

---

## Test Environment

Fill in / update when the machine profile changes.

- OS: macOS 26.2 (25C56)
- CPU: Apple M4 (arm64)
- GPU: Apple M4 (10 cores, Metal 4)
- Display (refresh rate, scaling): 1920×1080 @ 120Hz (UI looks like 1920×1080 @ 120Hz)
- Rust toolchain: see `rust-toolchain.toml`
- Command runner:
  - `cargo --version`: cargo 1.92.0 (344c4567c 2025-10-21)
  - `rustc --version`: rustc 1.92.0 (ded5c06cf 2025-12-08)
  - `cargo nextest --version`: cargo-nextest 0.9.115 (b8e0d5dcd 2025-12-15)
- Runtime flags (defaults for this log):
  - `FRET_UI_GALLERY_VIEW_CACHE=1`
  - `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`

---

## How We Record Results

We record suite runs via `fretboard diag perf` and store:

- the exact command line,
- the resulting perf JSON summary (p50/p95/max),
- worst bundles for root cause digging.

Recommended workflow:

1) Run the suite and capture output to a file:

```powershell
cargo run -p fretboard -- diag perf ui-gallery ^
  --env FRET_UI_GALLERY_VIEW_CACHE=1 ^
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 ^
  --warmup-frames 5 --repeat 7 --sort time --json ^
  --launch -- cargo run -p fret-ui-gallery --release ^
  > target/fret-diag/perf.ui-gallery.stdout.txt
```

2) Append a new entry to this log (tooling helper optional):

```powershell
python3 tools/perf/perf_log.py append ^
  --stdout target/fret-diag/perf.ui-gallery.stdout.txt ^
  --log docs/workstreams/ui-perf-zed-smoothness-v1-log.md ^
  --suite ui-gallery
```

---

## Entries

<!--
Template:

## YYYY-MM-DD HH:MM (commit <hash>)

Change:
- <short description>

Command:
```powershell
...
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| ... | ... | ... | ... | ... | ... | ... |

Worst overall:
- script:
- top_total_time_us:
- bundle:

Notes:
- <anything relevant>
-->

## 2026-02-02 18:30:01 (commit `eb960a0570b361dd58f14f92683c4b345b2abbc3`)

Change:
- docs(workstreams): add zed smoothness perf workstream plan

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --warmup-frames 5 --repeat 7 --sort time --json --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click.json | 3620 | 3669 | 3669 | 3058 | 47 | 16 | 596 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json | 27579 | 27789 | 27789 | 10398 | 3936 | 24 | 17384 |
| tools/diag-scripts/ui-gallery-dropdown-open-select.json | 27252 | 27450 | 27450 | 10176 | 3923 | 24 | 17307 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf.json | 6774 | 6886 | 6886 | 5776 | 1442 | 21 | 1089 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json | 3022 | 3057 | 3057 | 2585 | 52 | 13 | 472 |
| tools/diag-scripts/ui-gallery-overlay-torture.json | 6932 | 7090 | 7090 | 4350 | 464 | 21 | 2727 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf.json | 11621 | 22584 | 22584 | 18098 | 3646 | 56 | 4430 |
| tools/diag-scripts/ui-gallery-virtual-list-torture.json | 9105 | 9238 | 9238 | 8223 | 776 | 29 | 988 |
| tools/diag-scripts/ui-gallery-window-resize-stress.json | 30504 | 30770 | 30770 | 27569 | 17610 | 47 | 3156 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress.json`
- top_total_time_us: `30770`
- bundle: `target/fret-diag/1770027974556-ui-gallery-window-resize-stress/bundle.json`

## 2026-02-02 19:49:26 (commit `b5542636`)

Change:
- Normalize TextWrap::None measure cache key (ignore max_width); keep ellipsis width override

Suite:
- `ui-gallery-window-resize-stress`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-window-resize-stress.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --warmup-frames 5 --repeat 7 --sort time --json --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-window-resize-stress.json | 30384 | 30916 | 30916 | 27696 | 17630 | 50 | 3187 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress.json`
- top_total_time_us: `30916`
- bundle: `target/fret-diag/1770032393251-ui-gallery-window-resize-stress/bundle.json`

## 2026-02-02 20:57:10 (commit `9440648ae76f5fdc31dc17e930de90e9bb569029`)

Change:
- Fast-path wrapped text measure via shaping cache

Suite:
- `ui-gallery-window-resize-stress`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-window-resize-stress.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --warmup-frames 5 --repeat 7 --sort time --json --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-window-resize-stress.json | 15006 | 15511 | 15511 | 11580 | 1724 | 57 | 4708 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress.json`
- top_total_time_us: `15511`
- bundle: `target/fret-diag/1770036974294-ui-gallery-window-resize-stress/bundle.json`

## 2026-02-02 21:45:22 (commit `9440648ae76f5fdc31dc17e930de90e9bb569029`)

Change:
- Suite after wrapped text measure fast-path

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --warmup-frames 5 --repeat 7 --sort time --json --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click.json | 3392 | 3443 | 3443 | 2853 | 45 | 17 | 588 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json | 25204 | 25396 | 25396 | 8052 | 2251 | 26 | 17342 |
| tools/diag-scripts/ui-gallery-dropdown-open-select.json | 25121 | 25507 | 25507 | 8127 | 2312 | 25 | 17404 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf.json | 5572 | 5628 | 5628 | 4546 | 391 | 22 | 1072 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json | 2091 | 2156 | 2156 | 1673 | 52 | 13 | 470 |
| tools/diag-scripts/ui-gallery-overlay-torture.json | 6726 | 6872 | 6872 | 4070 | 311 | 20 | 2783 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf.json | 11238 | 11495 | 11495 | 10228 | 361 | 46 | 1231 |
| tools/diag-scripts/ui-gallery-virtual-list-torture.json | 7453 | 7574 | 7574 | 6573 | 777 | 30 | 973 |
| tools/diag-scripts/ui-gallery-window-resize-stress.json | 15300 | 15742 | 15742 | 12053 | 1752 | 57 | 4670 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-dropdown-open-select.json`
- top_total_time_us: `25507`
- bundle: `target/fret-diag/1770038785462-script-step-0002-click/bundle.json`
