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

## 2026-02-02 22:46:39 (commit `686bebe182fc2ca94c1ee1b072680549d3426f21`)

Change:
- feat(fretboard): add ui-gallery-steady perf suite

Suite:
- `ui-gallery-steady`

Command:
```powershell
# Preferred (single command; reuses a single launched process):
cargo run -p fretboard -- diag perf ui-gallery-steady ^
  --reuse-launch --repeat 7 --sort time --top 15 --json ^
  --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 ^
  --launch -- cargo run -p fret-ui-gallery --release

# Fallback (when you already have a running demo or cannot use `--launch`):
# 1) Terminal A:
set FRET_DIAG=1
set FRET_DIAG_DIR=target/fret-diag-steady
set FRET_UI_GALLERY_VIEW_CACHE=1
set FRET_UI_GALLERY_VIEW_CACHE_SHELL=1
cargo run -p fret-ui-gallery --release

# 2) Terminal B:
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-steady ^
  --repeat 7 --sort time --top 15 --json
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3006 | 3095 | 3095 | 2769 | 65 | 15 | 330 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3619 | 3740 | 3740 | 3063 | 176 | 19 | 682 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3373 | 3935 | 3935 | 3217 | 156 | 15 | 703 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2870 | 3033 | 3033 | 2450 | 41 | 18 | 599 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1692 | 2028 | 2028 | 1554 | 42 | 12 | 462 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3714 | 6342 | 6342 | 3801 | 293 | 21 | 2523 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10737 | 11162 | 11162 | 9901 | 346 | 47 | 1221 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6315 | 7325 | 7325 | 6041 | 753 | 28 | 1260 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 15165 | 15613 | 15613 | 11736 | 1824 | 54 | 4235 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `15613`
- bundle: `target/fret-diag-steady/1770043506957-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-02 23:24:09 (commit `b6f1b5803a89ecbdad47fbccd85fef4208e3e515`)

Change:
- perf(fret-ui): stabilize view-cache key

Suite:
- `ui-gallery-steady`

Command:
```powershell
# Preferred:
cargo run -p fretboard -- diag perf ui-gallery-steady ^
  --reuse-launch --repeat 7 --sort time --top 15 --json ^
  --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 ^
  --launch -- cargo run -p fret-ui-gallery --release

# Fallback:
set FRET_DIAG=1
set FRET_DIAG_DIR=target/fret-diag-steady2
set FRET_UI_GALLERY_VIEW_CACHE=1
set FRET_UI_GALLERY_VIEW_CACHE_SHELL=1
cargo run -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-steady2 ^
  --repeat 7 --sort time --top 15 --json
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3136 | 3367 | 3367 | 3019 | 62 | 17 | 331 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3731 | 3863 | 3863 | 3138 | 185 | 19 | 706 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3533 | 4075 | 4075 | 3320 | 161 | 16 | 739 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2970 | 3106 | 3106 | 2503 | 42 | 16 | 629 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1752 | 2018 | 2018 | 1537 | 42 | 12 | 469 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3903 | 6641 | 6641 | 3937 | 291 | 20 | 2684 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 11368 | 11592 | 11592 | 10287 | 334 | 48 | 1302 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6571 | 7478 | 7478 | 6215 | 760 | 28 | 1277 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13576 | 14894 | 14894 | 12389 | 1876 | 59 | 2446 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `14894`
- bundle: `target/fret-diag-steady2/1770045822918-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- View-cache keys no longer include the parent context bounds. Responsive branching that depends on
  window size should incorporate that into `ViewCacheProps.cache_key`.

## 2026-02-03 00:22:17 (commit `05d2d56c`)

Change:
- Defer scroll unbounded probe while viewport resizes (debounced); keep view-cache reuse stable

Suite:
- `ui-gallery-window-resize-stress-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-window-resize-stress-steady.json --reuse-launch --repeat 7 --sort time --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 10370 | 10527 | 10527 | 8168 | 2109 | 50 | 2310 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `10527`
- bundle: `target/fret-diag/1770049134799-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-03 00:46:46 (commit `448c34ad`)

Change:
- Replace WindowFrame HashMaps with slotmap::SecondaryMap (reduce per-frame hashing)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --reuse-launch --repeat 7 --sort time --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 2872 | 2984 | 2984 | 2656 | 61 | 17 | 317 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3434 | 3500 | 3500 | 2814 | 181 | 19 | 683 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3195 | 3745 | 3745 | 3002 | 166 | 15 | 728 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2712 | 2799 | 2799 | 2200 | 41 | 15 | 587 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1576 | 1879 | 1879 | 1401 | 41 | 12 | 469 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3650 | 6460 | 6460 | 3724 | 316 | 20 | 2716 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10100 | 10443 | 10443 | 9197 | 346 | 47 | 1210 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6061 | 6974 | 6974 | 5717 | 761 | 27 | 1264 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 12436 | 12587 | 12587 | 10261 | 1701 | 52 | 2357 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `12587`
- bundle: `target/fret-diag/1770050763291-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-03 01:11:08 (commit `a540829e`)

Change:
- Generation-stamp invalidation visited tables (propagate_observation_masks) to reduce per-frame hashing

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-inv-stamp --reuse-launch --repeat 7 --sort time --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3152 | 3249 | 3249 | 2891 | 77 | 18 | 341 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3787 | 3822 | 3822 | 3059 | 198 | 22 | 750 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3587 | 4053 | 4053 | 3279 | 179 | 17 | 757 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2918 | 8293 | 8293 | 8058 | 43 | 17 | 642 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1801 | 2101 | 2101 | 1571 | 50 | 14 | 518 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3889 | 6708 | 6708 | 3889 | 316 | 21 | 2800 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10792 | 11261 | 11261 | 9845 | 388 | 51 | 1365 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6445 | 7406 | 7406 | 6086 | 826 | 31 | 1380 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13559 | 15094 | 15094 | 12174 | 2118 | 59 | 2861 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `15094`
- bundle: `target/fret-diag-inv-stamp/1770052220451-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-03 01:13:26 (commit `a540829e`)

Change:
- Re-run ui-gallery-steady after generation-stamped invalidation tables (noise check)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-inv-stamp.v2 --reuse-launch --repeat 7 --sort time --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3183 | 3276 | 3276 | 2884 | 76 | 17 | 378 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3819 | 3871 | 3871 | 3083 | 203 | 21 | 783 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3560 | 4042 | 4042 | 3256 | 179 | 17 | 769 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2900 | 3089 | 3089 | 2462 | 43 | 17 | 661 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1775 | 2089 | 2089 | 1566 | 48 | 13 | 511 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3889 | 6817 | 6817 | 3927 | 328 | 21 | 2870 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10797 | 10942 | 10942 | 9638 | 375 | 50 | 1322 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6484 | 8164 | 8164 | 6708 | 871 | 32 | 1484 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13554 | 13575 | 13575 | 11006 | 1885 | 58 | 2644 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `13575`
- bundle: `target/fret-diag-inv-stamp.v2/1770052373457-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- The first run at `01:11:08` shows a large outlier on `ui-gallery-material3-tabs-switch-perf-steady` (p95=8293us).
  The rerun at `01:13:26` drops to p95=3089us, which suggests that spike is noise (e.g. one-off warmup / background work).
- Compared to the most recent recorded `ui-gallery-steady` baseline (commit `448c34ad`), some heavy scripts remain higher:
  `ui-gallery-window-resize-stress-steady` p95 total `12587 -> 13575` and `ui-gallery-virtual-list-torture-steady`
  p95 total `6974 -> 8164` (see the two entries above).
- Bundle stats snapshots used for local comparison (not versioned): `target/fret-diag/stats.ui-gallery-window-resize-stress-steady.448c34ad.txt`,
  `target/fret-diag/stats.ui-gallery-window-resize-stress-steady.a540829e.txt`.

## 2026-02-03 06:24:54 (commit `50bfcc54ff7d62d7b726dcce2ce126fad770b6d0`)

Change:
- Record macOS (Apple M4) ui-gallery-steady baseline (perf-baseline-out v1)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.macos-m4.v1 --reuse-launch --repeat 7 --sort time --top 15 --json --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v1.json --perf-baseline-headroom-pct 20 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3162 | 3248 | 3248 | 2898 | 76 | 17 | 349 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3820 | 3889 | 3889 | 3123 | 210 | 20 | 789 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3568 | 4066 | 4066 | 3270 | 185 | 19 | 777 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2850 | 3228 | 3228 | 2559 | 43 | 18 | 686 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1792 | 2187 | 2187 | 1649 | 53 | 13 | 525 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3882 | 6897 | 6897 | 3988 | 333 | 21 | 2888 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10757 | 10992 | 10992 | 9684 | 386 | 50 | 1331 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6569 | 7623 | 7623 | 6245 | 846 | 30 | 1605 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13811 | 13988 | 13988 | 11135 | 1977 | 58 | 2936 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `13988`
- bundle: `target/fret-diag-perf/ui-gallery-steady.macos-m4.v1/1770071057385-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- Baseline file written via `--perf-baseline-out`:
  - `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v1.json`
- A `--perf-baseline` check with repeat=3 can be slightly flaky on `ui-gallery-window-resize-stress-steady`
  `max_top_solve_us` (evidence: `target/fret-diag-perf/ui-gallery-steady.macos-m4.v1.check/check.perf_thresholds.json`).
  Prefer the v2 baseline (headroom 30%) for gating.
- Quick triage comparison against the previously logged `ui-gallery-steady` run at commit `448c34ad`:
  - `ui-gallery-window-resize-stress-steady` bundle stats show higher totals (sum `338183us -> 371826us`)
    and higher invalidation counts (sum calls/nodes `321/2784 -> 357/3096`). Treat as “needs confirmation”
    until we pin baselines and rerun under tighter noise control.
  - `ui-gallery-virtual-list-bottom-steady` invalidation counts are identical (sum calls/nodes `760/2521`),
    but layout/paint totals are higher (sum `24414us -> 26642us`).

## 2026-02-03 06:33:07 (commit `fd7ed84b`)

Notes:
- v2 baseline threshold check passed with repeat=3:
  - evidence: `target/fret-diag-perf/ui-gallery-steady.macos-m4.v2.check/check.perf_thresholds.json`

## 2026-02-03 06:41:07 (commit `fd7ed84b`)

Change:
- Record macOS (Apple M4) ui-gallery-steady baseline v2 (headroom 30%)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.macos-m4.v2 --reuse-launch --repeat 7 --sort time --top 15 --json --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v2.json --perf-baseline-headroom-pct 30 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3189 | 3435 | 3435 | 3000 | 90 | 17 | 418 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3814 | 3907 | 3907 | 3134 | 206 | 21 | 800 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3585 | 4092 | 4092 | 3301 | 185 | 17 | 774 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2840 | 3089 | 3089 | 2472 | 42 | 17 | 637 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1787 | 2137 | 2137 | 1598 | 51 | 13 | 543 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3870 | 6903 | 6903 | 3991 | 329 | 21 | 2891 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10898 | 11271 | 11271 | 9916 | 393 | 50 | 1335 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6542 | 7476 | 7476 | 6120 | 831 | 29 | 1360 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13769 | 14022 | 14022 | 11308 | 1930 | 58 | 2684 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `14022`
- bundle: `target/fret-diag-perf/ui-gallery-steady.macos-m4.v2/1770071470742-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-03 06:45:59 (commit `448c34ad`)

Change:
- Re-run ui-gallery-steady at 448c34ad (A/B vs a540+ baselines; same machine)

Suite:
- `ui-gallery-steady`

Command:
```powershell
(in detached worktree @448c34ad) cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.448c34ad.rerun --reuse-launch --repeat 7 --sort time --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| /Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3193 | 3321 | 3321 | 2964 | 81 | 17 | 340 |
| /Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3847 | 3888 | 3888 | 3139 | 202 | 20 | 769 |
| /Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3596 | 4166 | 4166 | 3378 | 184 | 17 | 771 |
| /Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2939 | 3181 | 3181 | 2557 | 46 | 20 | 654 |
| /Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1811 | 2150 | 2150 | 1623 | 51 | 13 | 515 |
| /Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3935 | 6928 | 6928 | 4041 | 332 | 20 | 2867 |
| /Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10923 | 11260 | 11260 | 9935 | 393 | 51 | 1284 |
| /Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6608 | 7515 | 7515 | 6201 | 807 | 31 | 1408 |
| /Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13707 | 13762 | 13762 | 11160 | 1888 | 55 | 2597 |

Worst overall:
- script: `/Users/frankorz/codes/rust/fret-perf-448c34ad/tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `13762`
- bundle: `/Users/frankorz/codes/rust/fret-perf-448c34ad/target/fret-diag-perf/ui-gallery-steady.448c34ad.rerun/1770072315614-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- This rerun suggests the earlier “`a540829e` regressed vs `448c34ad`” signal was mostly noise. On the same machine:
  - `ui-gallery-window-resize-stress-steady` p95 total is within ~2% (`13762us @448c34ad` vs `14022us @fd7ed84b baseline v2`).
  - `ui-gallery-virtual-list-torture-steady` is essentially flat (`7515us @448c34ad` vs `7476us @fd7ed84b baseline v2`).
- Script paths are absolute here because the run was performed from a detached worktree (`fret-perf-448c34ad`).

## 2026-02-03 07:05:31 (commit `cce827ad`)

Change:
- Skip rewriting WindowFrame.children when unchanged (reduce per-frame Arc allocations)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.after-windowframe-children-skip.r7 --reuse-launch --repeat 7 --sort time --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3157 | 3320 | 3320 | 2969 | 78 | 18 | 342 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3809 | 3878 | 3878 | 3126 | 214 | 20 | 757 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3589 | 4129 | 4129 | 3323 | 194 | 17 | 789 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2914 | 3082 | 3082 | 2442 | 42 | 19 | 641 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1786 | 2155 | 2155 | 1597 | 54 | 13 | 545 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3948 | 6943 | 6943 | 3970 | 349 | 29 | 2950 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10789 | 11237 | 11237 | 9904 | 418 | 52 | 1345 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6604 | 7504 | 7504 | 6157 | 876 | 30 | 1441 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13763 | 13825 | 13825 | 11165 | 2051 | 65 | 2783 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `13825`
- bundle: `target/fret-diag-perf/ui-gallery-steady.after-windowframe-children-skip.r7/1770073483221-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- `--perf-baseline` gating is currently sensitive to rare outliers on small scripts (e.g. menubar nav).
  During one baseline-gated run for this change, a single run hit `~8ms` on the menubar script and failed the gate:
  `target/fret-diag-perf/ui-gallery-steady.after-windowframe-children-skip/check.perf_thresholds.json`.
  A standalone baseline-gated rerun for just the menubar script passed:
  `target/fret-diag-perf/menubar-nav.after-windowframe-children-skip/check.perf_thresholds.json`.

## 2026-02-03 07:16:05 (commit `089bac9b`)

Change:
- Avoid cloning child lists for UiTree set_children during declarative mount (1x copy instead of 2x)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.after-mount-avoid-children-clone.r7 --reuse-launch --repeat 7 --sort time --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3175 | 3310 | 3310 | 2950 | 80 | 19 | 346 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3810 | 3862 | 3862 | 3096 | 204 | 24 | 779 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3645 | 4050 | 4050 | 3248 | 178 | 17 | 785 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2939 | 3091 | 3091 | 2452 | 50 | 17 | 652 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1755 | 2132 | 2132 | 1592 | 52 | 14 | 527 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3924 | 6905 | 6905 | 3911 | 335 | 21 | 2973 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10773 | 11247 | 11247 | 9903 | 441 | 52 | 1333 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6430 | 7565 | 7565 | 6150 | 826 | 30 | 1387 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13611 | 13643 | 13643 | 10969 | 1924 | 60 | 2636 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `13643`
- bundle: `target/fret-diag-perf/ui-gallery-steady.after-mount-avoid-children-clone.r7/1770074129791-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- Baseline gate check passed (repeat=3):
  - evidence: `target/fret-diag-perf/ui-gallery-steady.after-mount-avoid-children-clone.check/check.perf_thresholds.json`

## 2026-02-03 07:45:06 (commit `ac04f3dd`)

Change:
- Record macOS (Apple M4) ui-gallery-steady baseline v3 (adds hover layout steady script)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.macos-m4.v3 --reuse-launch --repeat 7 --sort time --top 15 --json --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v3.json --perf-baseline-headroom-pct 30 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3198 | 3344 | 3344 | 2989 | 77 | 17 | 348 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3814 | 3884 | 3884 | 3116 | 205 | 20 | 767 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3595 | 4157 | 4157 | 3367 | 177 | 16 | 774 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 1778 | 1808 | 1808 | 1257 | 16 | 12 | 544 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2921 | 3120 | 3120 | 2481 | 44 | 17 | 629 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1792 | 2127 | 2127 | 1593 | 53 | 13 | 525 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3925 | 6953 | 6953 | 4026 | 344 | 21 | 2906 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 11093 | 11440 | 11440 | 10384 | 393 | 55 | 1347 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6533 | 7575 | 7575 | 6189 | 833 | 29 | 1362 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13748 | 16940 | 16940 | 14381 | 2859 | 61 | 2768 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `16940`
- bundle: `target/fret-diag-perf/ui-gallery-steady.macos-m4.v3/1770075716969-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-03 07:50:39 (commit `d7e2c1db`)

Change:
- Record macOS (Apple M4) ui-gallery-steady baseline v4 (hover script cleanup)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.macos-m4.v4 --reuse-launch --repeat 7 --sort time --top 15 --json --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v4.json --perf-baseline-headroom-pct 30 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 3205 | 3297 | 3297 | 2936 | 83 | 18 | 348 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3825 | 3893 | 3893 | 3125 | 208 | 35 | 781 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3629 | 4067 | 4067 | 3255 | 178 | 17 | 795 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 1788 | 1807 | 1807 | 1286 | 17 | 12 | 526 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2899 | 3115 | 3115 | 2467 | 47 | 18 | 645 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1787 | 2140 | 2140 | 1603 | 53 | 13 | 525 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3904 | 6858 | 6858 | 3970 | 374 | 23 | 2865 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10835 | 10930 | 10930 | 9588 | 381 | 54 | 1343 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6511 | 7503 | 7503 | 6140 | 845 | 30 | 1403 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13699 | 16051 | 16051 | 13410 | 2177 | 59 | 2711 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `16051`
- bundle: `target/fret-diag-perf/ui-gallery-steady.macos-m4.v4/1770076076714-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-03 08:31:07 (commit `05cd5691`)

Change:
- perf(fret-ui): stamp layout engine solve state (SecondaryMap + frame-stamped solved tracking)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.after-layout-engine-solved-stamp.autodump-off --reuse-launch --repeat 7 --timeout-ms 120000 --sort time --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 2957 | 3032 | 3032 | 2702 | 65 | 19 | 324 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3574 | 3637 | 3637 | 2897 | 186 | 19 | 721 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3397 | 3937 | 3937 | 3153 | 171 | 16 | 768 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 1594 | 1623 | 1623 | 1111 | 9 | 11 | 501 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2630 | 2836 | 2836 | 2226 | 30 | 15 | 615 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1644 | 1976 | 1976 | 1463 | 48 | 12 | 501 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3665 | 6576 | 6576 | 3715 | 305 | 25 | 2841 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10352 | 10712 | 10712 | 9406 | 338 | 52 | 1277 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6267 | 7334 | 7334 | 5994 | 810 | 32 | 1335 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13092 | 13211 | 13211 | 10643 | 1768 | 56 | 2526 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `13211`
- bundle: `target/fret-diag-perf/ui-gallery-steady.after-layout-engine-solved-stamp.autodump-off/1770078589779-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- Delta vs `ui-gallery-steady.macos-m4.v4` (commit `d7e2c1db`, repeat=7):
  - `ui-gallery-window-resize-stress-steady`: p95 total `16051us -> 13211us` (-2840us, ~-17.7%)
  - `ui-gallery-hover-layout-torture-steady`: p95 total `1807us -> 1623us` (-184us, ~-10.2%)
  - `ui-gallery-overlay-torture-steady`: p95 total `6858us -> 6576us` (-282us, ~-4.1%)
  - Most other scripts improved by ~2–9% on p95 total (see table above).

## 2026-02-03 08:33:43 (commit `05cd5691`)

Change:
- Record baseline gate check (macos m4 v5; FRET_DIAG_SCRIPT_AUTO_DUMP=0)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.macos-m4.v5.check --reuse-launch --repeat 3 --timeout-ms 120000 --sort time --top 15 --json --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v5.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 2957 | 3055 | 3055 | 2719 | 64 | 16 | 328 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3570 | 3633 | 3633 | 2874 | 190 | 22 | 740 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3441 | 3862 | 3862 | 3079 | 164 | 16 | 767 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 1589 | 1617 | 1617 | 1107 | 9 | 13 | 497 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2689 | 2867 | 2867 | 2241 | 30 | 16 | 610 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1602 | 1965 | 1965 | 1440 | 46 | 12 | 513 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3625 | 6594 | 6594 | 3735 | 299 | 20 | 2839 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10249 | 10424 | 10424 | 9150 | 339 | 48 | 1275 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6220 | 7261 | 7261 | 5937 | 793 | 27 | 1338 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13039 | 13043 | 13043 | 10519 | 1777 | 59 | 2487 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `13043`
- bundle: `target/fret-diag-perf/ui-gallery-steady.macos-m4.v5.check/1770078789978-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-03 08:49:15 (commit `b038cbf7`)

Change:
- perf(fret-ui): reuse layout measure cache scratch (avoid per-solve HashMap alloc)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.after-layout-measure-scratch --reuse-launch --repeat 7 --timeout-ms 120000 --sort time --top 15 --json --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v5.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 2932 | 3050 | 3050 | 2718 | 63 | 16 | 323 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3554 | 3629 | 3629 | 2895 | 187 | 20 | 728 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3371 | 3849 | 3849 | 3078 | 163 | 16 | 755 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 1568 | 1602 | 1602 | 1088 | 8 | 11 | 503 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2643 | 2830 | 2830 | 2231 | 34 | 16 | 620 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1609 | 1914 | 1914 | 1410 | 43 | 12 | 492 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3628 | 6659 | 6659 | 3766 | 290 | 24 | 2873 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10200 | 10736 | 10736 | 9383 | 338 | 51 | 1302 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6172 | 7261 | 7261 | 5938 | 791 | 28 | 1334 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13018 | 16312 | 16312 | 13769 | 2241 | 60 | 2530 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `16312`
- bundle: `target/fret-diag-perf/ui-gallery-steady.after-layout-measure-scratch/1770079724231-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-03 08:50:52 (commit `b038cbf7`)

Change:
- Validate resize steady outlier: script-only run (repeat=11)

Suite:
- `ui-gallery-window-resize-stress-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-window-resize-stress-steady.json --dir target/fret-diag-perf/resize-steady.after-layout-measure-scratch --reuse-launch --repeat 11 --timeout-ms 120000 --sort time --top 15 --json --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v5.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 12557 | 12942 | 12942 | 10441 | 1725 | 59 | 2442 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `12942`
- bundle: `target/fret-diag-perf/resize-steady.after-layout-measure-scratch/1770079809090-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-03 01:44:57 (commit `75a9fde3`)

Change:
- perf(fret-ui): add bounds tree hit-test index (prepaint-built per layer; axis-aligned transforms only)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.after-bounds-tree.r7 --reuse-launch --repeat 7 --timeout-ms 120000 --sort time --top 15 --json --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v5.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3666 | 6777 | 6777 | 3882 | 300 | 19 | 2876 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3368 | 3834 | 3834 | 3060 | 157 | 16 | 758 |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 2945 | 3060 | 3060 | 2719 | 64 | 16 | 329 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3577 | 3635 | 3635 | 2888 | 184 | 21 | 739 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 1576 | 1599 | 1599 | 1089 | 8 | 11 | 500 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1608 | 1933 | 1933 | 1419 | 42 | 12 | 502 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6149 | 7105 | 7105 | 5803 | 787 | 28 | 1336 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2639 | 2834 | 2834 | 2223 | 33 | 16 | 619 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10337 | 10686 | 10686 | 9380 | 359 | 49 | 1283 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 12982 | 13033 | 13033 | 10494 | 1734 | 61 | 2548 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `13033`
- bundle: `target/fret-diag-perf/ui-gallery-steady.after-bounds-tree.r7/1770083128949-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- Gate check passed (no failures): `target/fret-diag-perf/ui-gallery-steady.after-bounds-tree.r7/check.perf_thresholds.json`.
- Compared to the last logged suite run at commit `b038cbf7`, `ui-gallery-hover-layout-torture-steady` is slightly lower
  (`p95 total 1602us -> 1599us`), while `ui-gallery-overlay-torture-steady` shows a higher outlier in this run.

## 2026-02-03 02:29:18 (commit `4b0be50e`)

Change:
- perf(diag): expose dispatch and hit-test timing (adds `--sort dispatch|hit_test` and exports `top_dispatch_time_us` / `top_hit_test_time_us`)

Suite:
- `tools/diag-scripts/ui-gallery-hit-test-drag-sweep-steady.json` (added by commit `8a08ff1d`)

Commands (A/B):
```powershell
# Bounds tree ON:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-drag-sweep-steady.json --dir target/fret-diag-perf/drag-hit-test.metrics.bounds-tree-on.r7 --reuse-launch --repeat 7 --timeout-ms 120000 --sort hit_test --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release

# Bounds tree OFF:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-drag-sweep-steady.json --dir target/fret-diag-perf/drag-hit-test.metrics.bounds-tree-off.r7 --reuse-launch --repeat 7 --timeout-ms 120000 --sort hit_test --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| variant | p95 dispatch_time_us | p95 hit_test_time_us | dispatch_events | hit_test_queries |
| --- | ---: | ---: | ---: | ---: |
| bounds tree ON | 47474 | 392 | 604 | 303 |
| bounds tree OFF | 47274 | 385 | 604 | 303 |

Notes:
- This script intentionally emits a high density of pointer events in a single frame (by design of `drag_pointer`), so
  `dispatch_time_us` is a “per-frame sum” of many event dispatches. A quick sanity check at p50 indicates ~74us/event.
- In this workload, the bounds tree does not materially reduce `hit_test_time_us` (delta is within noise); keep it as an
  optional path and revisit once we have a more realistic “pointer moves spread across frames” driver.

## 2026-02-03 11:03:38 (commit `4941baa1`)

Change:
- Add `move_pointer_sweep` (multi-frame pointer move) to diagnostics scripts so we can measure hover/hit-test cost per
  frame (instead of batching many events into one frame via `drag_pointer`).

Scripts:
- `tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json`
- `tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json`

Commands (A/B):
```powershell
# Bounds tree ON:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json --dir target/fret-diag-perf/move-hit-test.metrics.bounds-tree-on.r7 --reuse-launch --repeat 7 --timeout-ms 180000 --sort hit_test --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json --dir target/fret-diag-perf/data-table-move-hit-test.metrics.bounds-tree-on.r7d --reuse-launch --repeat 7 --timeout-ms 180000 --sort hit_test --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release

# Bounds tree OFF:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json --dir target/fret-diag-perf/move-hit-test.metrics.bounds-tree-off.r7 --reuse-launch --repeat 7 --timeout-ms 180000 --sort hit_test --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1 --launch -- cargo run -p fret-ui-gallery --release
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json --dir target/fret-diag-perf/data-table-move-hit-test.metrics.bounds-tree-off.r7d --reuse-launch --repeat 7 --timeout-ms 180000 --sort hit_test --top 15 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | variant | p50 total | p95 total | max total | p95 dispatch_time_us | p95 hit_test_time_us |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| ui-gallery-hit-test-move-sweep-steady | bounds tree ON | 1025 | 1176 | 1176 | 108 | 5 |
| ui-gallery-hit-test-move-sweep-steady | bounds tree OFF | 1015 | 1050 | 1050 | 98 | 6 |
| ui-gallery-hit-test-data-table-move-sweep-steady | bounds tree ON | 1386 | 1414 | 1414 | 137 | 8 |
| ui-gallery-hit-test-data-table-move-sweep-steady | bounds tree OFF | 1377 | 1720 | 1720 | 248 | 8 |

Worst bundles:
- `ui-gallery-hit-test-move-sweep-steady` (ON): `target/fret-diag-perf/move-hit-test.metrics.bounds-tree-on.r7/1770086918445-ui-gallery-hit-test-move-sweep-steady/bundle.json`
- `ui-gallery-hit-test-move-sweep-steady` (OFF): `target/fret-diag-perf/move-hit-test.metrics.bounds-tree-off.r7/1770086988815-ui-gallery-hit-test-move-sweep-steady/bundle.json`
- `ui-gallery-hit-test-data-table-move-sweep-steady` (ON): `target/fret-diag-perf/data-table-move-hit-test.metrics.bounds-tree-on.r7d/1770087539969-ui-gallery-hit-test-data-table-move-sweep-steady/bundle.json`
- `ui-gallery-hit-test-data-table-move-sweep-steady` (OFF): `target/fret-diag-perf/data-table-move-hit-test.metrics.bounds-tree-off.r7d/1770087596313-ui-gallery-hit-test-data-table-move-sweep-steady/bundle.json`

Notes:
- In these “one pointer move per frame” workloads, `hit_test_time_us` is still in single-digit microseconds, which
  suggests hit testing is not currently a dominant cost (or the scripts are not yet stressing the right shape).
- Next: find or synthesize a workload where hit testing is a meaningful slice of the frame budget, then re-run the
  bounds tree A/B in that context.

## 2026-02-03 06:17:40 (commit `26de29bd`)

Change:
- feat(ui-gallery): add hit-test torture harness

Adds:
- New gallery page: `hit_test_torture`
- Harness-only mode (skips gallery chrome): `FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture`
- Script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`

Goal:
- Provide a deterministic workload where hit-test CPU time is intentionally measurable (tens/hundreds of microseconds),
  so bounds-tree vs fallback traversal A/B is meaningful.

## 2026-02-03 06:19:06 (commit `ad9d5091`)

Change:
- perf(diag): expose bounds-tree query stats

Adds:
- `UiDebugFrameStats` counters for bounds-tree query outcomes (queries / disabled / miss / hit / candidate_rejected).
- `fretboard diag perf` JSON fields for the top frame:
  - `top_hit_test_bounds_tree_queries`
  - `top_hit_test_bounds_tree_disabled`
  - `top_hit_test_bounds_tree_misses`
  - `top_hit_test_bounds_tree_hits`
  - `top_hit_test_bounds_tree_candidate_rejected`

## 2026-02-03 06:24:19 (commit `811101c3`)

Change:
- perf(fret-ui): support overflow-visible in bounds tree

Context:
- Previously the bounds tree was disabled for an entire layer if any node had `clips_hit_test=false` (overflow visible),
  which is common in mechanism-heavy UI trees (semantics wrappers, pointer regions, etc.). This made the index hard to
  activate in practice, and the A/B runs stayed within noise.
- After this change, the bounds tree keeps building even when some ancestors do not clip hit-testing, by propagating
  the ancestor clip (or "no clip") down the stack. This makes the index usable on more real trees.

Commands (A/B; noise=20k; harness-only):
```powershell
# Bounds tree ON:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json --dir target/fret-diag-perf/hit-test-torture.harness-only.surface.bounds-tree-on.noise20k.after-overflow-visible-support.r7 --repeat 7 --timeout-ms 600000 --sort hit_test --top 5 --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=20000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- target/release/fret-ui-gallery

# Bounds tree OFF:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json --dir target/fret-diag-perf/hit-test-torture.harness-only.surface.bounds-tree-off.noise20k.after-overflow-visible-support.r7 --repeat 7 --timeout-ms 600000 --sort hit_test --top 5 --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=20000 --env FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- target/release/fret-ui-gallery
```

Results (us):
| variant | p50 total | p95 total | max total | p95 dispatch_time_us | p95 hit_test_time_us |
| --- | ---: | ---: | ---: | ---: | ---: |
| bounds tree ON | 29729 | 31348 | 31348 | 967 | 240 |
| bounds tree OFF | 28695 | 29408 | 29408 | 1600 | 797 |

Worst bundles:
- bounds tree ON: `target/fret-diag-perf/hit-test-torture.harness-only.surface.bounds-tree-on.noise20k.after-overflow-visible-support.r7/1770098586674-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- bounds tree OFF: `target/fret-diag-perf/hit-test-torture.harness-only.surface.bounds-tree-off.noise20k.after-overflow-visible-support.r7/1770099309508-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

Notes:
- Under this workload, bounds tree materially reduces `hit_test_time_us` (~3.3x at p95).

## 2026-02-03 16:09:00 (commit `1b3d2db3`)

Change:
- Add a smaller "mini" variant of the hit-test torture sweep script to make higher-noise scaling runs more practical.

Script:
- `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-mini.json`

Run shape:
- `FRET_DIAG_SCRIPT_AUTO_DUMP=0` so the app only writes the explicitly requested `capture_bundle` (avoids per-step bundles).
- `FRET_DIAG_SEMANTICS=0` and `FRET_DIAG_MAX_SNAPSHOTS=120` to keep bundle sizes stable.
- `--sort hit_test` to ensure we are sampling frames where hit testing is actually present.

Commands (A/B; harness-only; mini script; bounds tree forced on by `FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS=0`):
```powershell
# noise=50k, bounds tree ON:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-mini.json --dir target/fret-diag-perf-mini/hit-test-torture.mini.harness-only.bounds-tree-on.noise50k.r5 --repeat 5 --timeout-ms 600000 --sort hit_test --top 5 --json --reuse-launch --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=50000 --env FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS=0 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=120 --launch -- target/release/fret-ui-gallery

# noise=50k, bounds tree OFF:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-mini.json --dir target/fret-diag-perf-mini/hit-test-torture.mini.harness-only.bounds-tree-off.noise50k.r5 --repeat 5 --timeout-ms 600000 --sort hit_test --top 5 --json --reuse-launch --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=50000 --env FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS=0 --env FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=120 --launch -- target/release/fret-ui-gallery

# noise=100k, bounds tree ON:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-mini.json --dir target/fret-diag-perf-mini/hit-test-torture.mini.harness-only.bounds-tree-on.noise100k.r3 --repeat 3 --timeout-ms 600000 --sort hit_test --top 5 --json --reuse-launch --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=100000 --env FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS=0 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=120 --launch -- target/release/fret-ui-gallery

# noise=100k, bounds tree OFF:
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-mini.json --dir target/fret-diag-perf-mini/hit-test-torture.mini.harness-only.bounds-tree-off.noise100k.r3 --repeat 3 --timeout-ms 600000 --sort hit_test --top 5 --json --reuse-launch --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=100000 --env FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS=0 --env FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=120 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort hit_test`):
| noise | variant | p95 total | p95 dispatch_time_us | p95 hit_test_time_us | hit-test A/B (p95) |
| ---: | --- | ---: | ---: | ---: | ---: |
| 50k | bounds tree ON | 81983 | 2606 | 551 | - |
| 50k | bounds tree OFF | 77695 | 5332 | 2778 | ~5.0x slower |
| 100k | bounds tree ON | 160612 | 7399 | 1425 | - |
| 100k | bounds tree OFF | 148981 | 12360 | 5866 | ~4.1x slower |

Worst bundles:
- 50k ON: `target/fret-diag-perf-mini/hit-test-torture.mini.harness-only.bounds-tree-on.noise50k.r5/1770104974868-ui-gallery-hit-test-torture-stripes-move-sweep-mini/bundle.json`
- 50k OFF: `target/fret-diag-perf-mini/hit-test-torture.mini.harness-only.bounds-tree-off.noise50k.r5/1770105356574-ui-gallery-hit-test-torture-stripes-move-sweep-mini/bundle.json`
- 100k ON: `target/fret-diag-perf-mini/hit-test-torture.mini.harness-only.bounds-tree-on.noise100k.r3/1770105986938-ui-gallery-hit-test-torture-stripes-move-sweep-mini/bundle.json`
- 100k OFF: `target/fret-diag-perf-mini/hit-test-torture.mini.harness-only.bounds-tree-off.noise100k.r3/1770106187140-ui-gallery-hit-test-torture-stripes-move-sweep-mini/bundle.json`

Notes:
- The top frames in this torture workload are still layout-dominant (tens to hundreds of milliseconds) even when sorting
  by `hit_test`. The bounds tree improvement is real for hit test, but overall "Zed smoothness" will depend on reducing
  layout/prepaint cost under pointer moves as well.

## 2026-02-03 16:12:00 (commit `0003d978`)

Change:
- Clean up extremely large local diagnostics artifacts under `target/fret-diag-perf/` after scaling experiments.

Rationale:
- Some earlier torture runs produced multi-GB `bundle.json` files per repeat (e.g. ~4.7GB each at noise=20k), and
  accumulated to hundreds of GB. These are not intended to be kept long-term in-repo.
- The key A/B evidence is already captured as metrics + commands in this log. When needed, bundles can be regenerated
  by re-running the logged commands.

What was preserved:
- The two bundles explicitly referenced in this log (noise=20k A/B worst bundles):
  - `target/fret-diag-perf/hit-test-torture.harness-only.surface.bounds-tree-on.noise20k.after-overflow-visible-support.r7/1770098586674-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
  - `target/fret-diag-perf/hit-test-torture.harness-only.surface.bounds-tree-off.noise20k.after-overflow-visible-support.r7/1770099309508-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

Outcome:
- `target/fret-diag-perf/` size: ~292GB → ~29GB (local machine; macOS).

## 2026-02-03 16:20:00 (commit `21ceabc3`)

Change:
- `fretboard diag stats --json` now includes bounds-tree hit-test counters in `top[]` rows:
  - `hit_test_bounds_tree_queries`
  - `hit_test_bounds_tree_disabled`
  - `hit_test_bounds_tree_misses`
  - `hit_test_bounds_tree_hits`
  - `hit_test_bounds_tree_candidate_rejected`

Why:
- `diag perf` already exported these for top frames, but `diag stats` JSON did not, which made ad-hoc inspection
  confusing when validating whether the bounds tree path was actually exercised.

## 2026-02-03 16:34:00 (commit `8788389d`)

Change:
- Run a steady hover torture baseline and enforce the “no hover layout invalidations” gate.

Script:
- `tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json --dir target/fret-diag-perf-hover/hover-layout-torture.steady.baseline.r7 --repeat 7 --timeout-ms 240000 --sort dispatch --top 10 --json --reuse-launch --check-hover-layout --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort dispatch`):
| p95 total | p95 dispatch_time_us | p95 hit_test_time_us | p95 layout_time_us | p95 prepaint_time_us | p95 paint_time_us |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1196 | 348 | 2 | 874 | 40 | 293 |

Hover gates:
- `snapshots_with_hover_layout_invalidations`: 0 (PASS)
- `sum.hover_layout_invalidations`: 0 (PASS)

Worst bundle:
- `target/fret-diag-perf-hover/hover-layout-torture.steady.baseline.r7/1770107613569-ui-gallery-hover-layout-torture-steady/bundle.json`

Notes:
- In this scenario, hover edges do not trigger declarative layout invalidations; pointer-move cost is dominated by
  dispatch + the usual per-frame work (sub-2ms top frames).

## 2026-02-03 16:44:00 (commit `c579fce4`)

Change:
- `fretboard diag perf` now falls back to `latest.txt` (or scanning export dirs) when a script run completes without
  a `last_bundle_dir` in `script.result.json`.

Why:
- Some older scripts end immediately after `capture_bundle`, which requests a dump and may finish before the dump
  completes. In those cases, `last_bundle_dir` is missing even though a bundle is eventually written to disk.
- This fallback makes perf tooling more resilient while scripts are migrated to the steadier “reset + wait” protocol.

## 2026-02-03 16:48:00 (commit `2549d976`)

Change:
- Make the code-view scroll baseline script “steady” by resetting diagnostics after warmup, and giving the bundle dump
  enough frames to complete before the script exits.

Script:
- `tools/diag-scripts/ui-gallery-code-view-scroll-refresh-baseline.json`

Command (cached; steady; repeat=7):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-code-view-scroll-refresh-baseline.json --dir target/fret-diag-perf-editor/code-view-scroll-refresh.baseline.cached.steady.r7 --repeat 7 --timeout-ms 240000 --sort time --top 10 --json --reuse-launch --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| p95 total | p95 dispatch_time_us | p95 layout_time_us | p95 prepaint_time_us | p95 paint_time_us |
| ---: | ---: | ---: | ---: | ---: |
| 1289 | 129 | 764 | 25 | 510 |

Worst bundle:
- `target/fret-diag-perf-editor/code-view-scroll-refresh.baseline.cached.steady.r7/1770108556310-ui-gallery-code-view-scroll-refresh-baseline/bundle.json`

## 2026-02-03 17:55:00 (commit `bd709f88`)

Change:
- Establish a baseline for the code editor “autoscroll torture” scenario (syntax highlighting on).

Script:
- `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`

Command (release; steady; repeat=5):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json --dir target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.pre-81159325.bd709f88.r5 --repeat 5 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 23541 | 23856 | 23856 | 885 | 26 | 22947 |

Worst bundle:
- `target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.pre-81159325.bd709f88.r5/1770112756836-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Notes:
- The hot cost is overwhelmingly in `paint_time_us` for editor text rendering.

## 2026-02-03 18:05:00 (commit `81159325`)

Change:
- Speed up syntax-rich line rendering in the code editor by:
  - avoiding per-row `Theme` cloning when materializing `AttributedText`, and
  - adding an optional per-row `AttributedText` cache (LRU-like, keyed by buffer/theme revision + language + row).

Script:
- `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`

Command (release; steady; repeat=5):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json --dir target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.rich-row-cache.on.r5 --repeat 5 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5734 | 5881 | 5881 | 856 | 24 | 5001 |

Worst bundle:
- `target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.rich-row-cache.on.r5/1770111718534-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

A/B (same commit; cache disabled):
- Disable rich-row cache: add `--env FRET_CODE_EDITOR_RICH_ROW_CACHE_DISABLE=1`
- Results (us): p95 total `6009`, p95 paint `5128`
- Delta vs cache enabled: total `-2.1%`, paint `-2.5%`

Notes:
- The majority of the win comes from removing the `Theme` clone from the per-row rich-text path; the row cache is a
  smaller steady-state improvement in this specific probe.

## 2026-02-03 20:40:00 (commit `43f9c73e`)

Change:
- Export view-cache reuse “miss reasons” as first-class per-frame counters and include them in `fretboard diag perf`
  JSON output.

Why:
- We want perf regressions to be explainable: when view-cache reuse drops, we need to know whether it’s due to
  layout invalidations, deferred rerender flags, or cache key mismatches.

New `diag perf` JSON fields (for the top frame in each run):
- `top_view_cache_roots_total`
- `top_view_cache_roots_reused`
- `top_view_cache_roots_cache_key_mismatch`
- `top_view_cache_roots_needs_rerender`
- `top_view_cache_roots_layout_invalidated`

Notes:
- The per-root `reuse_reason` string in bundle snapshots now includes `needs_rerender` and `layout_invalidated`
  (in addition to existing reasons like `cache_key_mismatch`).

## 2026-02-03 19:40:00 (commit `a39e79c4`)

Change:
- Reuse a small set of per-frame scratch buffers to reduce allocator churn:
  - mount pending invalidations (`HashMap<NodeId, u8>`) is now reused across frames,
  - paint-cache replay translation uses a reusable `Vec<NodeId>` stack,
  - interaction-cache replay uses a reusable `Vec<InteractionRecord>` scratch.

Script:
- `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`

Command (release; steady; repeat=5):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json --dir target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.framescratch.r5 --repeat 5 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 5845 | 5949 | 5949 | 871 | 25 | 5053 |

Top view-cache counters (top frame):
- `top_view_cache_roots_total`: 2
- `top_view_cache_roots_reused`: 1
- `top_view_cache_roots_cache_key_mismatch`: 0
- `top_view_cache_roots_needs_rerender`: 0
- `top_view_cache_roots_layout_invalidated`: 0

Worst bundle:
- `target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.framescratch.r5/1770118714777-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Notes:
- Compared to the previous code-editor autoscroll entry (commit `81159325`), this is within expected noise.

## 2026-02-03 20:25:00 (commit `cb3ff2d9`)

Change:
- Reuse view-cache “keep-alive” scratch collections (HashSet/Vec) during reachability/GC to reduce per-frame
  allocator churn when cache roots are reused.

Script:
- `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`

Command (release; steady; repeat=5):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json --dir target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.keepalive-scratch.r7 --repeat 5 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 6274 | 6379 | 6379 | 933 | 29 | 5437 |

Top view-cache counters (top frame):
- `top_view_cache_roots_total`: 2
- `top_view_cache_roots_reused`: 1
- `top_view_cache_roots_cache_key_mismatch`: 0
- `top_view_cache_roots_needs_rerender`: 0
- `top_view_cache_roots_layout_invalidated`: 0

Worst bundle:
- `target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.keepalive-scratch.r7/1770121359579-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Notes:
- Compared to the previous code-editor autoscroll entry (commit `a39e79c4`), this run regressed:
  - p95 total: `5949` -> `6379` (+`430us`, +`7.2%`)
  - p95 paint: `5053` -> `5437` (+`384us`, +`7.6%`)
- This scenario has only 2 cache roots and is paint-dominated; the keep-alive scratch reuse is expected to matter
  mostly for cases with many reused roots/elements. Re-run with more repeats and additional probes before deciding
  whether this change should be kept or reverted.

## 2026-02-03 20:45:00 (commit `968305b9`)

Change:
- Add an A/B gate for the view-cache GC keep-alive scratch reuse:
  - `FRET_UI_VIEW_CACHE_KEEPALIVE_SCRATCH_DISABLE=1` forces the pre-`cb3ff2d9` allocation behavior.

Script:
- `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`

Command (release; steady; repeat=9; scratch enabled):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json --dir target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.keepalive-scratch.ab-default.r8 --repeat 9 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| scratch enabled (default) | 6282 | 6336 | 6336 | 925 | 26 | 5385 |

Worst bundle:
- `target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.keepalive-scratch.ab-default.r8/1770122017768-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Command (release; steady; repeat=9; scratch disabled):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json --dir target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.keepalive-scratch.ab-disabled.r8 --repeat 9 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_VIEW_CACHE_KEEPALIVE_SCRATCH_DISABLE=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| scratch disabled | 6294 | 6322 | 6322 | 921 | 29 | 5398 |

Worst bundle:
- `target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.keepalive-scratch.ab-disabled.r8/1770122258799-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Notes:
- In this paint-dominated probe (only 2 cache roots), the scratch reuse has no meaningful impact (A/B deltas are
  within noise). The earlier perceived regression in the `cb3ff2d9` entry should be treated as noise until
  confirmed by a broader suite or a cache-root-heavy script.

## 2026-02-03 21:05:00 (commit `968305b9`)

Change:
- A/B validation: verify the keep-alive scratch gate across cache-root-heavy scripts.

### Script: `tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json`

Command (release; steady; repeat=7; scratch enabled):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json --dir target/fret-diag-perf-view-cache/view-cache-toggle-perf.steady.keepalive-scratch.ab-default.r8 --repeat 7 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| scratch enabled (default) | 10539 | 10654 | 10654 | 9327 | 79 | 1259 |

Top view-cache counters (top frame):
- `top_view_cache_roots_total`: 3
- `top_view_cache_roots_reused`: 1
- `top_view_cache_roots_cache_key_mismatch`: 0
- `top_view_cache_roots_needs_rerender`: 0
- `top_view_cache_roots_layout_invalidated`: 2

Worst bundle:
- `target/fret-diag-perf-view-cache/view-cache-toggle-perf.steady.keepalive-scratch.ab-default.r8/1770122617532-ui-gallery-view-cache-toggle-perf-steady/bundle.json`

Command (release; steady; repeat=7; scratch disabled):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json --dir target/fret-diag-perf-view-cache/view-cache-toggle-perf.steady.keepalive-scratch.ab-disabled.r8 --repeat 7 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_VIEW_CACHE_KEEPALIVE_SCRATCH_DISABLE=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| scratch disabled | 10533 | 10674 | 10674 | 9333 | 80 | 1271 |

Worst bundle:
- `target/fret-diag-perf-view-cache/view-cache-toggle-perf.steady.keepalive-scratch.ab-disabled.r8/1770122688732-ui-gallery-view-cache-toggle-perf-steady/bundle.json`

Notes:
- A/B deltas are within expected noise for this script.

### Renderer churn signals: export text atlas + intermediate pool counters

Commits:
- `feat(render): add text atlas + intermediate churn perf stats` (`d10cac5a`)
- `feat(fretboard): add renderer churn sort modes` (`c9a8b168`)

Goal:
- Make tail hitches explainable by correlating “slow frames” with renderer churn:
  - text atlas uploads / evictions / resets
  - intermediate pool pressure / evictions (blur/effects)

#### Quick validation: text atlas uploads appear in bundles

Command (dev; steady script; renderer perf enabled):
```powershell
FRET_DIAG_RENDERER_PERF=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json --dir target/fret-diag-churn-verify2 --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery
```

Evidence bundle:
- `target/fret-diag-churn-verify2/1770175418448-ui-gallery-context-action-steady/bundle.json`

Observed churn (sum/max over snapshots in that bundle):
- `renderer_text_atlas_upload_bytes`: sum `2560`, max `2560`
- `renderer_text_atlas_evicted_pages`: sum `0`, max `0`

#### Churn signature example: “cold-ish” UI step triggers a large atlas upload

Command (dev; screenshots enabled because the script requests them):
```powershell
FRET_DIAG_RENDERER_PERF=1 FRET_DIAG_SCREENSHOTS=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-overlay-modals-visible.json --dir target/fret-diag-churn-verify5b --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery
```

Evidence bundle:
- `target/fret-diag-churn-verify5b/1770175626589-script-step-0078-click/bundle.json`

Top atlas upload frame (computed from `layout+prepaint+paint+dispatch+hit_test`):
- `renderer_text_atlas_upload_bytes`: `835328` bytes
- `renderer_prepare_text_us`: `2067`
- `total_us`: `5546` (`layout/prepaint/paint = 5072/71/403`)

Note:
- This is the intended shape of the new metrics: large atlas uploads show up alongside elevated `prepare_text_us`.

#### Suite check: `ui-gallery-steady` stays “churn-free” after warmup

Command (release; steady; `--reuse-launch`; repeat=3; warmup=5):
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf-churn2 --reuse-launch --repeat 3 --warmup-frames 5 --sort time --json --env FRET_DIAG_RENDERER_PERF=1 --launch -- cargo run -p fret-ui-gallery --release
```

Summary (repeat=3; `--sort time`; p95 total):
- Worst script: `ui-gallery-window-resize-stress-steady.json` p95 total `19713us`
- In this steady-state suite run, `top_renderer_text_atlas_upload_bytes` stays `0` on the sampled top frames
  (i.e. no per-frame glyph churn after warmup).

Worst bundle (from `worst_overall`):
- `target/fret-diag-perf-churn2/1770175928782-ui-gallery-window-resize-stress-steady/bundle.json`

### Renderer churn: deterministic effects workload to exercise intermediate pool

Goal:
- Ensure the diagnostics/perf pipeline can capture effect intermediate pressure (blur/effects), so we can correlate
  tail hitches with intermediate pool churn and then close it.

Commits:
- `feat(ui-gallery): add effects_blur_torture harness + script` (`7519d318`)

Command (dev; harness-only; renderer perf enabled):
```powershell
FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture FRET_DIAG_RENDERER_PERF=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-effects-blur-torture-steady.json --dir target/fret-diag-effects-blur --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery
```

Evidence bundle:
- `target/fret-diag-effects-blur/1770177186090-ui-gallery-effects-blur-torture-steady/bundle.json`

Observed intermediate pool signals (sum/max across snapshots in this bundle):
- `renderer_intermediate_peak_in_use_bytes`: sum `2042074800`, max `8403600`
- `renderer_intermediate_release_targets`: sum `972`, max `4`
- `renderer_intermediate_pool_reuses`: sum `4860`, max `20`
- `renderer_intermediate_pool_releases`: sum `4860`, max `20`
- `renderer_intermediate_pool_evictions`: sum `0`, max `0`

#### Eviction stress: force pool evictions with a reduced intermediate budget (1080p)

Purpose:
- Generate a “high churn” signature (`renderer_intermediate_pool_evictions > 0`) for tail-hitch correlation work.

Command (dev; harness-only; 1080p; reduced pool budget; renderer perf enabled):
```powershell
FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture FRET_UI_GALLERY_RENDERER_INTERMEDIATE_BUDGET_BYTES=20971520 FRET_DIAG_RENDERER_PERF=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json --dir target/fret-diag-effects-blur-thrash-b20 --timeout-ms 240000 --launch -- target/debug/fret-ui-gallery
```

Evidence bundle:
- `target/fret-diag-effects-blur-thrash-b20/1770177939950-ui-gallery-effects-blur-thrash-steady/bundle.json`

Observed intermediate pool churn (sum/max across snapshots in this bundle):
- `renderer_intermediate_budget_bytes`: max `20971520`
- `renderer_intermediate_peak_in_use_bytes`: sum `3944706480`, max `16233360`
- `renderer_intermediate_pool_allocations`: sum `243`, max `1`
- `renderer_intermediate_pool_evictions`: sum `243`, max `1`

---

### Renderer perf exported into diagnostics bundles (primitive-level correlation)

Commits:

- `feat(diag): export renderer perf into bundles` (`0e4928fe`)
- `feat(fretboard): add renderer perf sort modes` (`cf8975ca`)

Verification (macOS; wgpu Metal; short script):

```bash
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-context-menu-right-click.json \
  --dir target/fret-diag-verify-renderer-perf.v2 \
  --timeout-ms 240000 \
  --launch -- cargo run -p fret-ui-gallery --release
```

Evidence bundle:

- `target/fret-diag-verify-renderer-perf.v2/1770168912611-ui-gallery-context-action/bundle.json`

Sanity check (sort by renderer text prep time):

```bash
cargo run -p fretboard -- diag stats \
  target/fret-diag-verify-renderer-perf.v2/1770168912611-ui-gallery-context-action/bundle.json \
  --sort prepare_text \
  --top 5
```

`diag perf --json` output now includes `top_renderer_*` fields:

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-context-menu-right-click.json \
  --dir target/fret-diag-verify-renderer-perf-perf.v4 \
  --repeat 1 \
  --timeout-ms 240000 \
  --sort encode_scene \
  --json \
  --launch -- target/release/fret-ui-gallery
```

Evidence bundle:

- `target/fret-diag-verify-renderer-perf-perf.v4/1770169414415-script-step-0007-click/bundle.json`

---

### Renderer metrics baseline: editor autoscroll + chrome torture (bundle-embedded)

Commit: `54e4c587` (includes `0e4928fe` + `cf8975ca`).

#### Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`

Command (release; relaunch-per-repeat; repeat=7):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json \
  --dir target/fret-diag-perf-editor/renderer-metrics.r1 \
  --repeat 7 \
  --timeout-ms 240000 \
  --sort prepare_text \
  --top 10 \
  --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_DIAG_MAX_SNAPSHOTS=180 \
  --launch -- target/release/fret-ui-gallery
```

Results (us; per-run “top frame” selected by `--sort prepare_text`):

| metric | p50 | p95 |
| --- | ---: | ---: |
| total | 1288 | 1442 |
| layout | 906 | 961 |
| prepaint | 27 | 30 |
| paint | 359 | 454 |
| renderer.encode_scene | 625 | 645 |
| renderer.prepare_text | 548 | 585 |
| renderer.draw_calls | 59 | 59 |
| renderer.pipeline_switches | 41 | 41 |
| renderer.bind_group_switches | 56 | 56 |
| renderer.scissor_sets | 39 | 39 |
| renderer.scene_encoding_cache_misses | 1 | 1 |

Worst bundle:

- `target/fret-diag-perf-editor/renderer-metrics.r1/1770170286951-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

#### Script: `tools/diag-scripts/ui-gallery-chrome-torture-steady.json`

Command (release; relaunch-per-repeat; repeat=7):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-chrome-torture-steady.json \
  --dir target/fret-diag-perf-chrome/renderer-metrics.r1 \
  --repeat 7 \
  --timeout-ms 240000 \
  --sort pipeline_switches \
  --top 10 \
  --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_DIAG_MAX_SNAPSHOTS=180 \
  --launch -- target/release/fret-ui-gallery
```

Results (us; per-run “top frame” selected by `--sort pipeline_switches`):

| metric | p50 | p95 |
| --- | ---: | ---: |
| total | 901 | 910 |
| layout | 745 | 758 |
| prepaint | 21 | 26 |
| paint | 131 | 143 |
| renderer.encode_scene | 0 | 0 |
| renderer.prepare_text | 108 | 110 |
| renderer.draw_calls | 74 | 74 |
| renderer.pipeline_switches | 65 | 65 |
| renderer.bind_group_switches | 79 | 79 |
| renderer.scissor_sets | 46 | 46 |
| renderer.scene_encoding_cache_hits | 1 | 1 |
| renderer.scene_encoding_cache_misses | 0 | 0 |

Worst bundle:

- `target/fret-diag-perf-chrome/renderer-metrics.r1/1770170482121-ui-gallery-chrome-torture-steady/bundle.json`

### Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json` (validation)

Command (release; steady; repeat=9; relaunch-per-repeat):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json --dir target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.element-vec-pool.r9 --repeat 9 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| children vec pool (v0) | 6330 | 6525 | 6525 | 936 | 32 | 5558 |

Element build pool counters (top frame):
- `top_element_children_vec_pool_reuses`: p50 `197`, p95 `197`
- `top_element_children_vec_pool_misses`: p50 `0`, p95 `0`

Frame arena counters (top frame; proxy signals):
- `top_frame_arena_capacity_estimate_bytes`: p50 `24016`, p95 `24064`
- `top_frame_arena_grow_events`: p50 `0`, p95 `0`

Worst bundle:
- `target/fret-diag-perf-editor/code-editor-torture.autoscroll.steady.element-vec-pool.r9/1770134649492-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Notes:
- The element children vec pool stays in a stable “0 misses” steady state on this workload.
- This page is paint-dominant (`p95 paint 5558us / p95 total 6525us`), so allocation-churn wins in element build are not expected to move `p95 total` much here.

### Script: `tools/diag-scripts/ui-gallery-chrome-torture-steady.json` (new steady perf script; validation)

Command (release; steady; repeat=7; relaunch-per-repeat):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-chrome-torture-steady.json --dir target/fret-diag-perf-chrome/chrome-torture.steady.element-vec-pool.r7 --repeat 7 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=180 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| chrome torture (steady) | 968 | 988 | 988 | 655 | 23 | 334 |

Element build pool counters (top frame):
- `top_element_children_vec_pool_reuses`: p50 `132`, p95 `132`
- `top_element_children_vec_pool_misses`: p50 `0`, p95 `0`

Frame arena counters (top frame; proxy signals):
- `top_frame_arena_capacity_estimate_bytes`: p50 `20896`, p95 `20896`
- `top_frame_arena_grow_events`: p50 `0`, p95 `0`

Worst bundle:
- `target/fret-diag-perf-chrome/chrome-torture.steady.element-vec-pool.r7/1770135044798-ui-gallery-chrome-torture-steady/bundle.json`

Notes:
- This script is intentionally “perf-safe”: no screenshots and includes a `reset_diagnostics` after warmup.
- The element children vec pool stays in a stable “0 misses” steady state on this page as well.

### Renderer primitive profiling (UI gallery): periodic `RenderPerfSnapshot` logging

Commit:
- `feat(ui-gallery): log renderer perf snapshots` (`68e31129`)

Enable:
- `FRET_UI_GALLERY_RENDERER_PERF=1` enables renderer perf accumulation + periodic snapshot logging.
- `FRET_RENDERER_PERF_PIPELINES=1` prints pipeline-switch breakdown (optional).

Usage (scripted steady workload; can be paired with `diag repro --with tracy` or `--with renderdoc`):
```bash
cargo run -p fretboard -- diag repro tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json \
  --env FRET_UI_GALLERY_RENDERER_PERF=1 \
  --env FRET_RENDERER_PERF_PIPELINES=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --launch -- target/release/fret-ui-gallery
```

What it reports (stdout; once per ~1s while enabled):
- CPU slices: `encode_scene_us`, `prepare_text_us`, `prepare_svg_us`
- Complexity proxies: `draw_calls`, `pipeline_switches`, bind/scissor counts, upload bytes
- Cache stability: `scene_encoding_cache_hits` / `scene_encoding_cache_misses`

Notes:
- This is a profiling aid (not a speedup). Keep it disabled for normal perf baselines.

Run (code editor autoscroll steady; renderer perf enabled):
- `feat(ui-gallery): log renderer perf snapshots` (`68e31129`)
- Date: 2026-02-03

Command:
```bash
cargo run -p fretboard -- diag repro tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json \
  --dir target/fret-diag-repro-renderer-perf/editor-autoscroll.r2 \
  --timeout-ms 240000 --poll-ms 50 \
  --env FRET_UI_GALLERY_RENDERER_PERF=1 \
  --env FRET_RENDERER_PERF_PIPELINES=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_DIAG_MAX_SNAPSHOTS=180 \
  --launch -- target/release/fret-ui-gallery
```

Artifacts:
- stdout log: `target/fret-diag-repro-renderer-perf/editor-autoscroll.r2.stdout.log`
- bundle: `target/fret-diag-repro-renderer-perf/editor-autoscroll.r2/1770138298097-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Renderer perf (aggregated per ~1s window; per-frame values derived by dividing by `frames`):
- Sample windows: `n=22`, frames/window p50 `124` (min `115`, max `129`).
- Encode (CPU) per-frame: p50 `0.606ms`, mean `0.598ms` (min `0.387ms`, max `0.645ms`).
- Text prepare (CPU) per-frame: p50 `0.457ms`, mean `0.454ms` (min `0.352ms`, max `0.484ms`).
- SVG prepare (CPU) per-frame: p50 `0.00094ms` (~0.94µs; negligible).
- Draw-call complexity per-frame (proxies):
  - `draws`: p50 `59`, p95 `61`
  - `pipeline_switches`: p50 `41`, p95 `43`
  - `bind_group_switches`: p50 `56`, p95 `57`
  - `scissor_sets`: p50 `39`, p95 `39`

UI diagnostics (same bundle; 180 frames extracted from snapshots):
- `layout_time_us`: p50 `910`, p95 `943`, max `969`
- `prepaint_time_us`: p50 `26`, p95 `31`, max `34`
- `paint_time_us`: p50 `401`, p95 `476`, max `5475` (spike at tick_id=339/frame_id=341)
- `paint_cache_misses`: always `0`; `paint_cache_replayed_ops`: always `270` (paint cache replay stable)

Notes:
- This workload looks “CPU-cheap per frame” for scene building + encoding, but the **state-change density** is high (pipeline/bind/scissor counts).
  If we want Zed-like smoothness under heavier scenes, reducing pipeline/bind churn and making cache keys more stable should be high leverage.

### FrameArenaScratch v0: GC + semantics scratch reuse (exports `top_frame_arena_*`)

Commits:
- `perf(fret-ui): reuse GC/semantics scratch via frame arena` (`3d6e2431`)
- `feat(diag): export frame arena scratch stats` (`fe0ad7c3`)
- `fix(fret-ui): restore keepalive scratch after diagnostics` (`1b0364e9`)

Command (release; steady; repeat=7; relaunch-per-repeat):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture-steady.json --dir target/fret-diag-perf-overlay/overlay-torture.steady.frame-arena.r5.match-log.no-reuse-launch --repeat 7 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame arena scratch (v0) | 6624 | 6737 | 6737 | 3806 | 39 | 2904 |

Frame arena counters (top frame; proxy signals):
- `top_frame_arena_capacity_estimate_bytes`: p50 `24064`, p95 `24480`
- `top_frame_arena_grow_events`: p50 `1`, p95 `1` (expected with relaunch-per-repeat)

Worst bundle:
- `target/fret-diag-perf-overlay/overlay-torture.steady.frame-arena.r5.match-log.no-reuse-launch/1770128903097-ui-gallery-overlay-torture-steady/bundle.json`

Delta note (vs the earlier “keepalive scratch enabled (default)” entry above):
- `p95 total 6828us -> 6737us` (-91us, ~-1.3%); likely within noise. Primary benefit is allocator churn reduction + observability.

Command (release; steady; repeat=7; `--reuse-launch` warm process):
```powershell
cargo run -q -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture-steady.json --dir target/fret-diag-perf-overlay/overlay-torture.steady.frame-arena.r4-reuse-launch.match-log --repeat 7 --reuse-launch --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Warm-process highlights:
- `top_frame_arena_grow_events`: p50 `0`, p95 `1` (growth only shows up in the first run; subsequent repeats stay stable)
- `p95 total`: `6487us` (this is not directly comparable to relaunch-per-repeat baselines)

### Element build: remove per-scope `HashMap` churn in callsite counters

Commit:
- `perf(fret-ui): remove callsite counter HashMap churn` (`2dd36fde`)

Command (release; steady; repeat=7; relaunch-per-repeat):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture-steady.json --dir target/fret-diag-perf-overlay/overlay-torture.steady.callsite-smallvec.r6 --repeat 7 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| callsite counters: `HashMap -> SmallVec` | 6312 | 6373 | 6373 | 3608 | 37 | 2784 |

Worst bundle:
- `target/fret-diag-perf-overlay/overlay-torture.steady.callsite-smallvec.r6/1770130218798-ui-gallery-overlay-torture-steady/bundle.json`

Delta note (vs `1b0364e9` relaunch-per-repeat run above):
- `p95 total 6737us -> 6373us` (-364us, ~-5.4%)

### Element build: pool `Vec<AnyElement>` children buffers (arena-adjacent, v0)

Commits:
- `perf(fret-ui): pool element children vectors` (`07a4c252`)
- `feat(diag): export element build pool counters` (`cbcd81ed`)
- `perf(fret-ui): make element children vec pool LIFO` (`693a55b0`)

Command (release; steady; repeat=7; relaunch-per-repeat):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture-steady.json --dir target/fret-diag-perf-overlay/overlay-torture.steady.children-vec-pool.pop.r8 --repeat 7 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| children vec pool (v0) | 6663 | 6803 | 6803 | 3817 | 41 | 2957 |

Element build pool counters (top frame):
- `top_element_children_vec_pool_reuses`: p50 `293`, p95 `293`
- `top_element_children_vec_pool_misses`: p50 `0`, p95 `0`

Worst bundle:
- `target/fret-diag-perf-overlay/overlay-torture.steady.children-vec-pool.pop.r8/1770132990787-ui-gallery-overlay-torture-steady/bundle.json`

Notes:
- The pool reaches a stable “0 misses” steady state for the sampled top frame.
- This script's `p95 total` did not improve in this run; the primary win is allocator-churn reduction + a measurable signal we can correlate on heavier pages.

### Script: `tools/diag-scripts/ui-gallery-overlay-torture-steady.json`

Command (release; steady; repeat=7; scratch enabled):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture-steady.json --dir target/fret-diag-perf-overlay/overlay-torture.steady.keepalive-scratch.ab-default.r8 --repeat 7 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| scratch enabled (default) | 6613 | 6828 | 6828 | 3880 | 42 | 2906 |

Top view-cache counters (top frame):
- `top_view_cache_roots_total`: 3
- `top_view_cache_roots_reused`: 1
- `top_view_cache_roots_cache_key_mismatch`: 0
- `top_view_cache_roots_needs_rerender`: 0
- `top_view_cache_roots_layout_invalidated`: 2

Worst bundle:
- `target/fret-diag-perf-overlay/overlay-torture.steady.keepalive-scratch.ab-default.r8/1770122908340-ui-gallery-overlay-torture-steady/bundle.json`

Command (release; steady; repeat=7; scratch disabled):
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture-steady.json --dir target/fret-diag-perf-overlay/overlay-torture.steady.keepalive-scratch.ab-disabled.r8 --repeat 7 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_VIEW_CACHE_KEEPALIVE_SCRATCH_DISABLE=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
| mode | p50 total | p95 total | max total | p95 layout | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| scratch disabled | 6657 | 6759 | 6759 | 3788 | 40 | 2947 |

Worst bundle:
- `target/fret-diag-perf-overlay/overlay-torture.steady.keepalive-scratch.ab-disabled.r8/1770122979000-ui-gallery-overlay-torture-steady/bundle.json`

Notes:
- A/B deltas are within expected noise for this script.

## 2026-02-04 12:16:14 (commit `f4ac7a12ef9e94d686df39c6367c8ae7955893c1`)

Change:
- measure churn: effects blur thrash (budget=20MB)

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json --repeat 3 --warmup-frames 5 --sort time --json --env FRET_DIAG_RENDERER_PERF=1 --env FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture --env FRET_UI_GALLERY_RENDERER_INTERMEDIATE_BUDGET_BYTES=20971520 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json | 440 | 443 | 443 | 168 | 24 | 5 | 289 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json | 0 | 0 | 0 | 0 | 16233360 | 16233360 | 1 | 1 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`
- top_total_time_us: `443`
- bundle: `target/fret-diag/1770178521003-script-step-0008-press_key/bundle.json`

## 2026-02-04 13:54:55 (commit `dfbc02d3`)

Change:
- Add svg/image upload churn metrics + svg upload torture harness

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json --repeat 3 --warmup-frames 5 --sort svg_upload_bytes --json --env FRET_DIAG_RENDERER_PERF=1 --env FRET_UI_GALLERY_HARNESS_ONLY=svg_upload_torture --env FRET_UI_GALLERY_SVG_RASTER_BUDGET_BYTES=262144 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json | 18 | 19 | 19 | 15 | 4 | 0 | 4 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json | 0 | 0 | 0 | 0 | 2359296 | 2359296 | 0 | 0 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json`
- top_total_time_us: `19`
- bundle: `target/fret-diag/1770184393082-script-step-0008-press_key/bundle.json`

## 2026-02-04 14:36:03 (commit `3d1510a7`)

Change:
- rerun: svg_upload_thrash_steady (repeat=5) incl svg cache churn fields

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json --repeat 5 --warmup-frames 5 --sort svg_upload_bytes --json --env FRET_DIAG_RENDERER_PERF=1 --env FRET_UI_GALLERY_HARNESS_ONLY=svg_upload_torture --env FRET_UI_GALLERY_SVG_RASTER_BUDGET_BYTES=262144 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json | 15 | 28 | 28 | 23 | 8 | 0 | 5 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json | 0 | 0 | 0 | 0 | 2506752 | 2506752 | 0 | 0 | 17 | 17 | 16 | 16 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json`
- top_total_time_us: `28`
- bundle: `target/fret-diag/1770186886544-script-step-0008-press_key/bundle.json`

## 2026-02-04 15:38:07 (commit `dd8bc0f8`)

Change:
- Add invalidation-driven svg scroll churn harness + scripted wheel workload

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-svg-scroll-thrash-steady.json --repeat 5 --warmup-frames 5 --sort svg_upload_bytes --json --env FRET_DIAG_RENDERER_PERF=1 --env FRET_UI_GALLERY_HARNESS_ONLY=svg_scroll_torture --env FRET_UI_GALLERY_SVG_RASTER_BUDGET_BYTES=262144 --launch -- cargo run -p fret-ui-gallery --release
```

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-svg-scroll-thrash-steady.json | 17 | 17 | 17 | 14 | 0 | 1 | 2 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-svg-scroll-thrash-steady.json | 0 | 0 | 0 | 0 | 1179648 | 1179648 | 0 | 0 | 8 | 8 | 7 | 7 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-svg-scroll-thrash-steady.json`
- top_total_time_us: `17`
- bundle: `target/fret-diag/1770190559929-script-step-0216-press_key/bundle.json`

## 2026-02-04 16:02:02 (commit `52f555d5`)

Change:
- rerun: effects blur thrash with intermediate pool lifecycle stats (budget=20MB, repeat=5)

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json --repeat 5 --warmup-frames 5 --sort time --json --env FRET_DIAG_RENDERER_PERF=1 --env FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture --env FRET_UI_GALLERY_RENDERER_INTERMEDIATE_BUDGET_BYTES=20971520 --launch -- cargo run -p fret-ui-gallery --release
```

Stdout:
- `target/fret-perf-stdout-effects-blur-thrash-steady-1770191925.txt`

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json | 428 | 446 | 446 | 152 | 36 | 2 | 294 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 16233360 | 16233360 | 1 | 1 |

Intermediate pool signals (top frame; p95/max):
| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json | 20971520 | 20971520 | 0 | 0 | 16233360 | 16233360 | 4 | 4 | 1 | 1 | 19 | 19 | 20 | 20 | 1 | 1 | 18763600 | 18763600 | 10 | 10 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`
- top_total_time_us: `446`
- bundle: `target/fret-diag/1770191928695-script-step-0008-press_key/bundle.json`

## 2026-02-04 16:19:21 (commit `3b792646`)

Change:
- perf(fret-render): defer intermediate pool budget enforcement; rerun effects blur thrash (budget=20MB, repeat=5)

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json --repeat 5 --warmup-frames 5 --sort time --json --env FRET_DIAG_RENDERER_PERF=1 --env FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture --env FRET_UI_GALLERY_RENDERER_INTERMEDIATE_BUDGET_BYTES=20971520 --launch -- cargo run -p fret-ui-gallery --release
```

Stdout:
- `target/fret-perf-stdout-effects-blur-thrash-steady-1770192979.txt`

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json | 387 | 434 | 434 | 196 | 26 | 2 | 267 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 16233360 | 16233360 | 1 | 1 |

Intermediate pool signals (top frame; p95/max):
| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json | 20971520 | 20971520 | 0 | 0 | 16233360 | 16233360 | 4 | 4 | 1 | 1 | 19 | 19 | 20 | 20 | 1 | 1 | 18763600 | 18763600 | 10 | 10 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`
- top_total_time_us: `434`
- bundle: `target/fret-diag/1770193126521-script-step-0008-press_key/bundle.json`

## 2026-02-04 16:34:03 (commit `0b8d3bb208f304ea9d4ef4eea7c2938091fe2081`)

Change:
- baseline: hit-test data table move sweep (repeat=5, reuse-launch, sort=hit_test)

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json --repeat 5 --warmup-frames 5 --sort hit_test --timeout-ms 180000 --reuse-launch --json --env FRET_DIAG_RENDERER_PERF=1 --launch -- cargo run -p fret-ui-gallery --release
```

Stdout:
- `target/fret-perf-stdout-hit-test-data-table-move-sweep-steady-1770193939.txt`

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint | p95 dispatch | p95 hit_test |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json | 1635 | 1700 | 1700 | 1208 | 0 | 53 | 449 | 260 | 4 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Intermediate pool signals (top frame; p95/max):
| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json`
- top_total_time_us: `1700`
- bundle: `target/fret-diag/1770193962388-script-step-0017-press_key/bundle.json`

## 2026-02-04 16:50:44 (commit `9b2f9fc9`)

Change:
- baseline: hit-test torture stripes sweep via nav (noise=5000, stripes=256, repeat=5, reuse-launch, sort=hit_test)

Suite:
- `ui-gallery`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-via-nav-steady.json --repeat 5 --warmup-frames 5 --sort hit_test --timeout-ms 180000 --reuse-launch --json --env FRET_DIAG_RENDERER_PERF=1 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=5000 --env FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES=256 --launch -- cargo run -p fret-ui-gallery --release
```

Stdout:
- `target/fret-perf-stdout-hit-test-torture-stripes-via-nav-steady-1770194549.txt`

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint | p95 dispatch | p95 hit_test |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-via-nav-steady.json | 6564 | 7142 | 7142 | 6547 | 0 | 518 | 77 | 1136 | 5 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-via-nav-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Intermediate pool signals (top frame; p95/max):
| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-via-nav-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-via-nav-steady.json`
- top_total_time_us: `7142`
- bundle: `target/fret-diag/1770194564827-script-step-0027-press_key/bundle.json`

## 2026-02-04 16:59:06 (commit `9b2f9fc9de58f2e99178f3c6bc8af1adf813a294`)

Change:
- baseline: ui-gallery-steady (repeat=7, reuse-launch, sort=time)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.1770195466 --repeat 7 --sort time --top 15 --timeout-ms 180000 --reuse-launch --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_RENDERER_PERF=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release
```

Stdout:
- `target/fret-perf-stdout-ui-gallery-steady-1770195466.txt`

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint | p95 dispatch | p95 hit_test |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 2956 | 2983 | 2983 | 2630 | 67 | 33 | 341 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3583 | 3641 | 3641 | 2897 | 185 | 38 | 722 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3330 | 3681 | 3681 | 2935 | 156 | 31 | 716 | 0 | 0 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 1595 | 3134 | 3134 | 2468 | 14 | 131 | 535 | 0 | 0 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2626 | 2890 | 2890 | 2254 | 33 | 38 | 635 | 0 | 0 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1642 | 2165 | 2165 | 1579 | 56 | 33 | 553 | 0 | 0 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3565 | 6407 | 6407 | 3611 | 277 | 37 | 2759 | 168 | 0 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10268 | 10393 | 10393 | 9064 | 335 | 76 | 1255 | 0 | 0 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6280 | 7212 | 7212 | 5852 | 789 | 57 | 1376 | 0 | 0 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 12934 | 15552 | 15552 | 13020 | 1883 | 89 | 2492 | 2160 | 0 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Intermediate pool signals (top frame; p95/max):
| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `15552`
- bundle: `target/fret-diag-perf/ui-gallery-steady.1770195466/1770195504962-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-04 17:01:05 (commit `9b2f9fc9de58f2e99178f3c6bc8af1adf813a294`)

Change:
- gate check: ui-gallery-steady vs macos-m4.v5 baseline (repeat=7, reuse-launch)

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --dir target/fret-diag-perf/ui-gallery-steady.norenderperf.1770195597 --repeat 7 --sort time --top 15 --timeout-ms 180000 --reuse-launch --json --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v5.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --launch -- cargo run -p fret-ui-gallery --release
```

Stdout:
- `target/fret-perf-stdout-ui-gallery-steady-norenderperf-1770195597.txt`

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint | p95 dispatch | p95 hit_test |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 2974 | 3088 | 3088 | 2737 | 63 | 33 | 323 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 3608 | 3673 | 3673 | 2915 | 188 | 37 | 723 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 3376 | 3875 | 3875 | 3086 | 159 | 34 | 755 | 0 | 0 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 1584 | 1603 | 1603 | 1092 | 9 | 27 | 486 | 0 | 0 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 2660 | 2857 | 2857 | 2243 | 34 | 33 | 614 | 0 | 0 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 1643 | 1856 | 1856 | 1357 | 40 | 28 | 491 | 0 | 0 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 3628 | 6483 | 6483 | 3648 | 278 | 36 | 2799 | 0 | 0 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 10391 | 10753 | 10753 | 9450 | 338 | 79 | 1255 | 611 | 0 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 6166 | 7077 | 7077 | 5735 | 779 | 55 | 1319 | 269 | 0 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 13042 | 13844 | 13844 | 10897 | 1753 | 196 | 2751 | 2222 | 0 |

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Intermediate pool signals (top frame; p95/max):
| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-context-menu-right-click-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-material3-tabs-switch-perf-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-overlay-torture-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| tools/diag-scripts/ui-gallery-window-resize-stress-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `13844`
- bundle: `target/fret-diag-perf/ui-gallery-steady.norenderperf.1770195597/1770195633326-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-04 19:06:00 (perf commit `1905de1e4e5bbda5ccab9e2f6d9c2dbd9f968ff0`)

Change:
- Skip layout-engine rebuild (`request_build_window_roots_if_final`) on frames where no visible roots need layout/bounds updates.
- Still runs prepaint/focus repair/cleanup so hit-testing and interaction caches stay correct.

Probe:
- Script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
- Harness-only: `FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture`
- Env: `FRET_DIAG_SCRIPT_AUTO_DUMP=0`, `FRET_DIAG_SEMANTICS=0`, `FRET_DIAG_MAX_SNAPSHOTS=240`

Baseline (commit `f90bbe181d8a4d821b64d0a17e4a4d2cd011a74e`):
- bundle: `target/fret-diag-perf/stripes-sweep-perf-baseline.head/1770200313185-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- `diag perf` worst top_total_time_us: `83237`
- max stats (us): layout=`74017`, prepaint=`9647`, dispatch=`3734`, hit_test=`909`, paint=`417`

After (commit `1905de1e4e5bbda5ccab9e2f6d9c2dbd9f968ff0`):
- bundle: `target/fret-diag-perf/stripes-sweep-perf-fastpath.v6/1770203253914-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- `diag perf` worst top_total_time_us: `40406`
- max stats (us): layout=`30671`, prepaint=`9585`, dispatch=`3594`, hit_test=`664`, paint=`575`

## 2026-02-04 21:12:15 (perf commit `470708b2`)

Change:
- Gate `UiTree::request_semantics_snapshot()` per-frame requests based on current diagnostics/script needs.
- During long-running scripted sweeps, avoid refreshing semantics every frame once target geometry is cached.

Probe:
- Script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
- Harness-only: `FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture`
- Env: `FRET_DIAG_SCRIPT_AUTO_DUMP=0`, `FRET_DIAG_SEMANTICS=0`, `FRET_DIAG_MAX_SNAPSHOTS=240`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json `
  --dir target/fret-diag-perf/stripes-sweep-semanticgate.470708b2 `
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json `
  --timeout-ms 300000 --poll-ms 200 `
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture `
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 `
  --launch -- cargo run -p fret-ui-gallery --release
```

Baseline (commit `b02744a8`, before gating semantics requests):
- dir: `target/fret-diag-perf/stripes-sweep-layoutbreakdown.b02744a8`
- top frame p50/p95/max total (us): `42225 / 56190 / 56190`
- top frame p50/p95/max layout (us): `32660 / 39619 / 39619`
- top frame p50/p95/max prepaint (us): `9761 / 15433 / 15433`
- semantics refresh was observed on **201/201** sampled frames (bundle inspection).

After (commit `470708b2`):
- dir: `target/fret-diag-perf/stripes-sweep-semanticgate.470708b2`
- top frame p50/p95/max total (us): `37866 / 38637 / 38637`
- top frame p50/p95/max layout (us): `28387 / 29251 / 29251`
- top frame p50/p95/max prepaint (us): `8984 / 9074 / 9074`
- semantics refresh was observed on **3/201** sampled frames (bundle inspection).

Notes:
- This makes the “hit-test torture” probe far more representative: long multi-frame pointer sweeps are no longer
  dominated by per-frame semantics refresh.

## 2026-02-04 22:15:07 (perf commit `ba3fd15d`)

Change:
- Fix a diagnostics accounting bug: `layout_time_us` no longer includes (and thus double-counts) the time spent in
  `prepaint_after_layout`.

Notes:
- From this commit onward, `top_total_time_us = layout_time_us + prepaint_time_us + paint_time_us` is no longer
  inflated by `prepaint` being counted twice.
- Perf numbers recorded **before** `ba3fd15d` are not directly comparable to later runs without adjusting for this.

## 2026-02-04 22:15:07 (perf commit `6cca2cf1`)

Change:
- On layout stable frames (where `layout_all_with_pass_kind` takes the “skip layout-engine rebuild” fast path),
  avoid rebuilding interaction/prepaint state and instead reuse the existing hit-test bounds trees.

Probe:
- Script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
- Harness-only: `FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture`
- Env: `FRET_DIAG_SCRIPT_AUTO_DUMP=0`, `FRET_DIAG_SEMANTICS=0`, `FRET_DIAG_MAX_SNAPSHOTS=240`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json `
  --dir target/fret-diag-perf/stripes-sweep-prepaintreuse.6cca2cf1 `
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json `
  --timeout-ms 300000 --poll-ms 200 `
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture `
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 `
  --launch -- cargo run -p fret-ui-gallery --release
```

Results (top frame; p50/p95/max across 7 runs; us):
- `top_total_time_us`: `19917 / 20086 / 20086`
- `top_layout_time_us`: `19500 / 19674 / 19674` (dominated by one-time semantics refresh frames)
- `top_prepaint_time_us`: `0 / 0 / 0`
- `top_paint_time_us`: `405 / 417 / 417`

Pointer-move frames (within the captured bundle; filtered to frames where `dispatch_events > 0`):
- Worst-per-run total (layout+prepaint+paint) p50/p95/max (us): `464 / 693 / 693`
- Worst-per-run hit-test (subset of dispatch) in the worst pointer frame (us): `669`
- Worst-per-run dispatch in the worst pointer frame (us): `3912`

Notes:
- The “worst overall” frame in this probe is now typically a **selector resolution** frame (no dispatched events),
  which is expected for scripted tooling. The pointer-move steady-state frames are now effectively **paint-only**
  with `layout_time_us ~ 0` and `prepaint_time_us ~ 0`.

## 2026-02-04 23:01:54 (commit `1a9c1238`)

Change:
- perf(fret-ui): avoid redundant hit-test in dispatch (validate)

Suite:
- `ui-gallery-hit-test-torture-stripes-move-sweep-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json --dir target/fret-diag-perf/2026-02-04-hit-test-stripes-move-sweep-1a9c1238-r1 --warmup-frames 5 --repeat 7 --sort time --top 15 --json --timeout-ms 300000 --poll-ms 200 --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- cargo run -p fret-ui-gallery --release
```

Stdout:
- `target/fret-diag-perf/2026-02-04-hit-test-stripes-move-sweep-1a9c1238-r1/stdout.txt`

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint | p95 dispatch | p95 hit_test |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json | 20318 | 20954 | 20954 | 20547 | 0 | 0 | 409 | 0 | 0 |

Notes:
- In this probe, the worst “top frame” by total time is typically an initial mount/settle frame with no dispatched
  pointer events, so `p95 dispatch` / `p95 hit_test` show up as `0` in the table above (because `perf_log.py`
  reports top-frame metrics).

Pointer-move frames (dispatch-focused; per-run **max** across 7 bundles; us):
- `dispatch_time_us`: `2845 / 4145 / 4145` (p50 / p95 / max)
- `hit_test_time_us`: `893 / 922 / 922` (p50 / p95 / max)
- Worst dispatch bundle: `target/fret-diag-perf/2026-02-04-hit-test-stripes-move-sweep-1a9c1238-r1/1770216342891-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- Worst hit-test bundle: `target/fret-diag-perf/2026-02-04-hit-test-stripes-move-sweep-1a9c1238-r1/1770216466940-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- Common churn signal in these bundles: `WindowInputContextService` and `WindowCommandActionAvailabilityService`
  are reported as changed on most snapshots but are frequently unobserved (`unobs.globals`), suggesting a
  “changed-but-unobserved global churn” dispatch tail candidate (tracked in the TODO).

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Intermediate pool signals (top frame; p95/max):
| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
- top_total_time_us: `20954`
- bundle: `target/fret-diag-perf/2026-02-04-hit-test-stripes-move-sweep-1a9c1238-r1/1770217083405-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-04 23:31:02 (commit `d4adf37f`)

Change:
- perf(fret-ui): avoid global churn on hover moves

Suite:
- `ui-gallery-hit-test-torture-stripes-move-sweep-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json --dir target/fret-diag-perf/2026-02-05-hit-test-stripes-move-sweep-global-churn-gate --warmup-frames 5 --repeat 7 --sort time --top 15 --json --timeout-ms 300000 --poll-ms 200 --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- cargo run -p fret-ui-gallery --release
```

Stdout:
- `target/fret-diag-perf/2026-02-05-hit-test-stripes-move-sweep-global-churn-gate/stdout.txt`

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint | p95 dispatch | p95 hit_test |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json | 19727 | 20720 | 20720 | 20363 | 0 | 0 | 417 | 0 | 0 |

Notes:
- This change targets “changed-but-unobserved global churn” on hover-only pointer moves:
  - avoid publishing `WindowInputContextService` snapshots when unchanged,
  - avoid publishing `WindowCommandActionAvailabilityService` snapshots on hover-only moves.
- As with prior entries, the “top frame” totals are dominated by a non-dispatch settle/mount frame, so `p95 dispatch`
  / `p95 hit_test` can appear as `0` in the table above.

Pointer-move frames (dispatch-focused; per-run **max** across 7 bundles; us):
- `dispatch_time_us`: `1090 / 1176 / 1176` (p50 / p95 / max)
- `hit_test_time_us`: `851 / 905 / 905` (p50 / p95 / max)
- `snapshots_with_global_changes`: `0` (for all 7 bundles)
- Worst dispatch/hit-test bundle: `target/fret-diag-perf/2026-02-05-hit-test-stripes-move-sweep-global-churn-gate/1770218744032-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Intermediate pool signals (top frame; p95/max):
| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
- top_total_time_us: `20720`
- bundle: `target/fret-diag-perf/2026-02-05-hit-test-stripes-move-sweep-global-churn-gate/1770218867587-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 00:42:09 (commit `6da92d3d`)

Change:
- feat(diag): add pointer-move perf thresholds (validate)

Suite:
- `ui-gallery-hit-test-torture-stripes-move-sweep-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json --dir target/fret-diag-perf/2026-02-05-hit-test-stripes-move-sweep-pointer-move-gate-6da92d3d-r1 --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json --timeout-ms 300000 --poll-ms 200 --max-pointer-move-dispatch-us 2000 --max-pointer-move-hit-test-us 1500 --max-pointer-move-global-changes 0 --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- cargo run -p fret-ui-gallery --release
```

Stdout:
- `target/fret-diag-perf/2026-02-05-hit-test-stripes-move-sweep-pointer-move-gate-6da92d3d-r1/stdout.txt`

Results (us):
| script | p50 total | p95 total | max total | p95 layout | p95 solve | p95 prepaint | p95 paint | p95 dispatch | p95 hit_test |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json | 19647 | 19954 | 19954 | 19554 | 0 | 0 | 417 | 0 | 0 |

Notes:
- Pointer-move frames (derived from bundle snapshots; per-run **max** over frames where `dispatch_events > 0`; us):
  - `dispatch_time_us`: `1105 / 1551 / 1551` (p50 / p95 / max)
  - `hit_test_time_us`: `886 / 967 / 967` (p50 / p95 / max)
  - `snapshots_with_global_changes` (within that frame set): `0 / 0 / 0` (p50 / p95 / max)
  - Worst dispatch bundle: `target/fret-diag-perf/2026-02-05-hit-test-stripes-move-sweep-pointer-move-gate-6da92d3d-r1/1770223086674-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
  - Worst hit-test bundle: `target/fret-diag-perf/2026-02-05-hit-test-stripes-move-sweep-pointer-move-gate-6da92d3d-r1/1770223086674-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

Churn signals (top frame; p95/max):
| script | p95 atlas_upload_bytes | max atlas_upload_bytes | p95 atlas_evicted_pages | max atlas_evicted_pages | p95 svg_upload_bytes | max svg_upload_bytes | p95 image_upload_bytes | max image_upload_bytes | p95 svg_cache_misses | max svg_cache_misses | p95 svg_evictions | max svg_evictions | p95 intermediate_peak_bytes | max intermediate_peak_bytes | p95 pool_evictions | max pool_evictions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Intermediate pool signals (top frame; p95/max):
| script | p95 budget_bytes | max budget_bytes | p95 in_use_bytes | max in_use_bytes | p95 peak_in_use_bytes | max peak_in_use_bytes | p95 release_targets | max release_targets | p95 allocations | max allocations | p95 reuses | max reuses | p95 releases | max releases | p95 evictions | max evictions | p95 free_bytes | max free_bytes | p95 free_textures | max free_textures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json | 268435456 | 268435456 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Worst overall:
- script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
- top_total_time_us: `19954`
- bundle: `target/fret-diag-perf/2026-02-05-hit-test-stripes-move-sweep-pointer-move-gate-6da92d3d-r1/1770222686711-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-04 16:50:52 (commit `dd1a22e8`)

Change:
- docs-only: validate pointer-move gate still passes on current HEAD

Suite:
- `ui-gallery-hit-test-torture-stripes-move-sweep-steady` (sorted by `dispatch`)

Command:
```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json --dir target/fret-diag-perf/2026-02-04-pointer-move-dispatch-top --reuse-launch --warmup-frames 5 --repeat 3 --sort dispatch --top 15 --json --timeout-ms 300000 --poll-ms 200 --max-pointer-move-dispatch-us 2000 --max-pointer-move-hit-test-us 1500 --max-pointer-move-global-changes 0 --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- cargo run -p fret-ui-gallery --release
```

Results (pointer-move frames; derived; per-run **max** over frames where `dispatch_events > 0`; us):
- `dispatch_time_us`: `1094 / 1751 / 1751` (p50 / p95 / max; repeat=3)
- `hit_test_time_us`: `883 / 1465 / 1465` (p50 / p95 / max; repeat=3)
- `snapshots_with_global_changes` (within that frame set): `0 / 0 / 0` (p50 / p95 / max)

Bundles:
- run 0: `target/fret-diag-perf/2026-02-04-pointer-move-dispatch-top/1770223952625-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 1: `target/fret-diag-perf/2026-02-04-pointer-move-dispatch-top/1770224052396-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 2: `target/fret-diag-perf/2026-02-04-pointer-move-dispatch-top/1770224151980-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-04 17:18:40 (commit `eb6c6b2e`)

Change:
- perf(ui-gallery): avoid per-frame undo/redo model churn

Why:
- The gallery driver updated `settings_edit_can_undo/settings_edit_can_redo` via `ModelStore::update` every frame.
  `update` marks the model dirty unconditionally, so this created `changed_models=2` even when values were unchanged,
  showing up as changed-but-unobserved model churn in pointer-move probes.

Suite:
- `ui-gallery-hit-test-torture-stripes-move-sweep-steady` (sorted by `dispatch`)

Command:
```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json --dir target/fret-diag-perf/2026-02-04-pointer-move-model-churn-release-after --reuse-launch --warmup-frames 5 --repeat 3 --sort dispatch --top 15 --json --timeout-ms 300000 --poll-ms 200 --max-pointer-move-dispatch-us 2000 --max-pointer-move-hit-test-us 1500 --max-pointer-move-global-changes 0 --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- cargo run -p fret-ui-gallery --release
```

Results (pointer-move frames; derived; per-run **max** over frames where `dispatch_events > 0`; us):
- `dispatch_time_us`: `1042 / 1189 / 1189` (p50 / p95 / max; repeat=3)
- `hit_test_time_us`: `860 / 884 / 884` (p50 / p95 / max; repeat=3)
- `snapshots_with_global_changes` (within that frame set): `0 / 0 / 0` (p50 / p95 / max)
- `changed_models` (top frame on the worst-dispatch bundle): `0`

Bundles:
- run 0: `target/fret-diag-perf/2026-02-04-pointer-move-model-churn-release-after/1770225617609-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 1: `target/fret-diag-perf/2026-02-04-pointer-move-model-churn-release-after/1770225715527-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 2: `target/fret-diag-perf/2026-02-04-pointer-move-model-churn-release-after/1770225814534-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-04 18:09:12 (commit `b3d13e51`)

Change:
- perf(fret-ui): reuse invalidation dedup in dispatch (commit `bcb329e6`)

Suite:
- `ui-gallery-hit-test-torture-stripes-move-sweep-steady` (sorted by `dispatch`)

Command:
```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json --dir target/fret-diag-perf/2026-02-04-pointer-move-dispatch-invalidation-dedup-bcb329e6 --reuse-launch --warmup-frames 5 --repeat 3 --sort dispatch --top 15 --json --timeout-ms 300000 --poll-ms 200 --max-pointer-move-dispatch-us 2000 --max-pointer-move-hit-test-us 1500 --max-pointer-move-global-changes 0 --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- cargo run -p fret-ui-gallery --release
```

Results (pointer-move frames; derived; per-run **max** over frames where `dispatch_events > 0`; us):
- `dispatch_time_us`: `1114 / 1136 / 1136` (p50 / p95 / max; repeat=3)
- `hit_test_time_us`: `877 / 891 / 891` (p50 / p95 / max; repeat=3)
- `snapshots_with_global_changes` (within that frame set): `0 / 0 / 0` (p50 / p95 / max)
- `changed_models` (top frame on the worst-dispatch bundle): `0`

Bundles:
- run 0: `target/fret-diag-perf/2026-02-04-pointer-move-dispatch-invalidation-dedup-bcb329e6/1770228652839-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 1: `target/fret-diag-perf/2026-02-04-pointer-move-dispatch-invalidation-dedup-bcb329e6/1770228751450-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 2: `target/fret-diag-perf/2026-02-04-pointer-move-dispatch-invalidation-dedup-bcb329e6/1770228848106-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 02:49:41 (commit `f1ce6599`)

Change:
- perf(fret-ui): reduce dispatch allocations on pointer-move

Why:
- Pointer-move is the “Zed feel” hot path. This change removes two small but steady allocation sources in dispatch:
  - reuse a scratch `Vec<UiLayerId>` instead of collecting `visible_layers_in_paint_order()` per dispatch path
  - use `HashMap::retain` to drop stale pointer captures without allocating a temporary `Vec`

Suite:
- `ui-gallery-hit-test-torture-stripes-move-sweep-steady` (sorted by `time`)

Command:
```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/hit-test-stripes-move-sweep-pointer-move-gate-scratch-r3 \
  --timeout-ms 300000 --poll-ms 100 \
  --reuse-launch --warmup-frames 5 --repeat 3 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 2000 \
  --max-pointer-move-hit-test-us 1500 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- cargo run -p fret-ui-gallery --release
```

Results (pointer-move frames; derived; per-run **max** over frames; us):
- `dispatch_time_us`: `1089 / 1104 / 1104` (p50 / p95 / max; repeat=3)
- `hit_test_time_us`: `859 / 911 / 911` (p50 / p95 / max; repeat=3)
- `snapshots_with_global_changes` (within that frame set): `0 / 0 / 0` (p50 / p95 / max)

Bundles:
- run 0: `target/fret-diag-perf/hit-test-stripes-move-sweep-pointer-move-gate-scratch-r3/1770230769311-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 1: `target/fret-diag-perf/hit-test-stripes-move-sweep-pointer-move-gate-scratch-r3/1770230866422-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 2: `target/fret-diag-perf/hit-test-stripes-move-sweep-pointer-move-gate-scratch-r3/1770230960458-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 03:08:26 (commit `b83ae7a5`)

Change:
- perf(fret-ui): avoid visible-layer Vec allocs in routing

Why:
- Pointer move/wheel routing calls `active_input_layers` / `active_focus_layers` / `topmost_pointer_occlusion_layer`
  frequently. These helpers previously collected `visible_layers_in_paint_order()` into a temporary `Vec` to support
  reverse traversal and barrier discovery. This commit replaces those allocations with direct scans of `layer_order`.

Suite:
- `ui-gallery-hit-test-torture-stripes-move-sweep-steady` (sorted by `time`)

Command:
```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc \
  --timeout-ms 300000 --poll-ms 100 \
  --reuse-launch --warmup-frames 5 --repeat 3 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 2000 \
  --max-pointer-move-hit-test-us 1500 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (pointer-move frames; derived; per-run **max** over frames; us):
- `dispatch_time_us`: `1075 / 1082 / 1082` (p50 / p95 / max; repeat=3)
- `hit_test_time_us`: `839 / 886 / 886` (p50 / p95 / max; repeat=3)
- `snapshots_with_global_changes` (within that frame set): `0 / 0 / 0` (p50 / p95 / max)

Bundles:
- run 0: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc/1770231841210-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 1: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc/1770231941595-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 2: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc/1770232040946-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 06:57:50 (commit `b83ae7a5`)

Change:
- perf(fret-ui): avoid visible-layer Vec allocs in routing (commit `b83ae7a5`)

Suite:
- `ui-gallery-hit-test-torture-stripes-move-sweep-steady` (sorted by `time`)

Command:
```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc-r7 \
  --timeout-ms 300000 --poll-ms 100 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 2000 \
  --max-pointer-move-hit-test-us 1500 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (pointer-move frames; derived; per-run **max** over frames; us):
- `dispatch_time_us`: `1085 / 1481 / 1639` (p50 / p95 / max; repeat=7)
- `hit_test_time_us`: `887 / 1252 / 1391` (p50 / p95 / max; repeat=7)
- `snapshots_with_global_changes` (within that frame set): `0 / 0 / 0` (p50 / p95 / max)

Notes:

- Run 0 had a noticeably higher pointer-move max than the other repeats (still within the gate thresholds). At the
  moment we do not export the worst pointer-move frame id in bundles, so tying this outlier to a specific frame
  requires additional instrumentation.

Bundles:
- run 0: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc-r7/1770245252655-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 1: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc-r7/1770245352324-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 2: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc-r7/1770245451304-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 3: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc-r7/1770245551128-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 4: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc-r7/1770245650104-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 5: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc-r7/1770245750183-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- run 6: `target/fret-diag-perf/2026-02-05-pointer-move-layer-scan-no-alloc-r7/1770245849788-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 07:05:30 (commit `c2ea017b`)

Change:
- feat(diag): include pointer-move max frame ids in triage

Why:
- The repeat=7 pointer-move gate had a visible “single-run outlier” (run 0 max much higher than others). Without the
  ability to locate the exact snapshot id, explaining and fixing dispatch/hit-test tails is unnecessarily slow.

Notes:

- `fretboard diag triage --json` now includes:
  - `stats.pointer_move.max_dispatch_at.{window,tick_id,frame_id}`
  - `stats.pointer_move.max_hit_test_at.{window,tick_id,frame_id}`
- On the run 0 bundle above, the outlier snapshot was:
  - `max_dispatch_at`: `window=4294967297 tick=128 frame=130`
  - `max_hit_test_at`: `window=4294967297 tick=128 frame=130`

Next:

- Use this snapshot identity to add a more detailed breakdown for the dispatch/hit-test time (so the outlier can be
  explained in terms of concrete work, not just wall time).

## 2026-02-05 07:26:44 (commit `913ee260`)

Change:
- feat(fret-ui): track bounds-tree query work in debug stats

Why:
- Pointer-move hit testing is currently gated by `hit_test_time_us`, but without a “work” proxy it is hard to
  distinguish:
  - algorithmic cost (too many nodes visited / too much overlap), vs
  - wall-time noise (preemption, scheduling jitter).

Notes:

- Diagnostics snapshots now include two new per-frame counters (accumulated across queries in a frame):
  - `debug.stats.hit_test_bounds_tree_nodes_visited`
  - `debug.stats.hit_test_bounds_tree_nodes_pushed`
- Example (single run; max-hit-test pointer-move frame from the bundle below):
  - `hit_test_time_us=896` with `hit_test_bounds_tree_nodes_visited=17` and `hit_test_bounds_tree_nodes_pushed=17`

Command:
```sh
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-run/2026-02-05-pointer-move-bounds-tree-query-stats \
  --timeout-ms 300000 --poll-ms 100 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Bundle:
- `target/fret-diag-run/2026-02-05-pointer-move-bounds-tree-query-stats/1770247519772-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 07:38:02 (commit `913ee260`)

Change:
- (no code change) Re-run the pointer-move gate at repeat=7 to validate that the new bounds-tree “work” counters
  (visited/pushed) can explain the tail.

Why:
- The pointer-move gate previously showed a few ~0.9ms `hit_test_time_us` outliers. Without a work proxy it was not
  clear whether this was algorithmic cost (too many nodes visited) or wall-time jitter.

Command:
```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-bounds-tree-work \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --timeout-ms 300000 --poll-ms 200 \
  --max-pointer-move-dispatch-us 2000 \
  --max-pointer-move-hit-test-us 1500 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- cargo run -p fret-ui-gallery --release
```

Results (median across 7 runs; 192 pointer-move frames per run):
- `dispatch_time_us`: p50 ~800, p95 ~936.6, max (across runs) 1106
- `hit_test_time_us`: p50 ~581.5, p95 ~785.9, max (across runs) 925

Worst pointer-move hit-test frame (from the worst-by-hit bundle below):
- `tick_id=893 frame_id=895`
- `hit_test_time_us=925`, `dispatch_time_us=946`
- `hit_test_bounds_tree_queries=1`, `nodes_visited=12`, `nodes_pushed=12`
- `bounds_tree_hit=1`, `candidate_rejected=0`

Takeaway:
- The tail is **not** explained by a bounds-tree explosion (visited/pushed stays small even at the max frame). The
  remaining ~0.9ms is likely fixed per-query overhead (clip/corner-radii checks, transform work, widget hit-test),
  or wall-time jitter. Next step is to add sub-step timing inside hit testing.

Bundles:
- Worst-by-hit: `target/fret-diag-perf/2026-02-05-pointer-move-r7-bounds-tree-work/1770248282947-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`
- Worst-by-dispatch: `target/fret-diag-perf/2026-02-05-pointer-move-r7-bounds-tree-work/1770248580579-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 08:21:34 (commit `55dd923d`)

Change:
- feat(diag): track hit-test path-cache reuse

Why:
- We need a concrete signal for “did the cached-path fast path actually help?” on pointer-move workloads.
- This enables A/B experiments (bounds-tree on/off, different hover policies) without guesswork.

Notes:
- New per-frame counters exported in diagnostics bundles:
  - `debug.stats.hit_test_path_cache_hits`
  - `debug.stats.hit_test_path_cache_misses`
- Semantics:
  - `hits`: a hit-test query was satisfied via `try_hit_test_along_cached_path` (no bounds-tree query needed).
  - `misses`: a cached-path hit-test was attempted for the cached layer root but did not hit, so we fell back.

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-path-cache-stats-55dd923d \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --timeout-ms 300000 --poll-ms 200 \
  --max-pointer-move-dispatch-us 2000 \
  --max-pointer-move-hit-test-us 1500 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; 192 pointer-move frames per run):
- `dispatch_time_us`: p50 ~797, p95 ~946.0, max (across runs) 1180
- `hit_test_time_us`: p50 ~586.0, p95 ~783.5, max (across runs) 943

Path-cache reuse (worst-by-hit bundle below; 192 pointer-move frames):
- `hit_test_path_cache_hits_total=4`
- `hit_test_path_cache_misses_total=188`
- Hit rate: ~2.1% (4 / 192)

Interpretation:
- On this stripes sweep workload, the pointer crosses many regions per frame, so cached-path reuse is expected to be
  low. The counter is still useful to confirm whether a change improves locality on more realistic pages.

Bundles:
- `target/fret-diag-perf/2026-02-05-pointer-move-r7-path-cache-stats-55dd923d/1770250128271-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 08:40:01 (commit `763bf8e7`)

Change:
- feat(diag): break down hit-test timing

Why:
- The pointer-move gate (stripes sweep) showed ~0.6–0.9ms `hit_test_time_us` even when the bounds-tree index was
  enabled. This entry explains *where the time actually went*.

Notes:

- New hit-test micro timers were added (commit `763bf8e7`), and the repeat=7 pointer-move gate run below shows that:
  - almost all hit-test time was spent inside `try_hit_test_along_cached_path`, and
  - bounds-tree query + candidate validation were ~single-digit microseconds.
- This indicates the cached-path fast path can be actively harmful on workloads with many siblings (it performs a
  conservative sibling scan to ensure correctness).

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-hit-test-breakdown-763bf8e7 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --timeout-ms 300000 --poll-ms 200 \
  --max-pointer-move-dispatch-us 2000 \
  --max-pointer-move-hit-test-us 1500 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; 192 pointer-move frames per run):
- `hit_test_time_us`: p50 ~575.0, p95 ~792.3, max (across runs) 907
- Sub-step median breakdown (per pointer-move frame; derived from bundle stats):
  - `hit_test_cached_path_time_us`: p50 ~572.0, p95 ~788.3, max 903
  - `hit_test_bounds_tree_query_time_us`: p50 ~2.0, p95 ~2.0, max 5
  - `hit_test_candidate_self_only_time_us`: p50 ~0.0, p95 ~0.0, max 2
  - `hit_test_fallback_traversal_time_us`: p50 ~0.0, p95 ~0.0, max 0

Takeaway:
- The bounds-tree index was *already* doing the right thing; the remaining ~0.6–0.9ms tail was the cached-path
  attempt itself. Next step: avoid attempting cached-path hit testing when the bounds-tree is enabled.

Bundles:
- Worst-by-hit: `target/fret-diag-perf/2026-02-05-pointer-move-r7-hit-test-breakdown-763bf8e7/1770252192036-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 08:57:12 (commit `8bc15eda`)

Change:
- perf(fret-ui): skip cached-path hit-test under bounds-tree

Why:
- Cached-path hit testing was dominating `hit_test_time_us` even when bounds-tree was enabled, due to conservative
  sibling scanning on miss. When bounds-tree is enabled for a layer, cached-path becomes redundant and costly.

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-skip-cached-path-8bc15eda \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --timeout-ms 300000 --poll-ms 200 \
  --max-pointer-move-dispatch-us 2000 \
  --max-pointer-move-hit-test-us 1500 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; 192 pointer-move frames per run):
- `dispatch_time_us`: p50 ~129.0, p95 ~250.0, max (across runs) 357
- `hit_test_time_us`: p50 ~3.0, p95 ~5.0, max (across runs) 10
- Sub-step median breakdown:
  - `hit_test_cached_path_time_us`: p50 ~0.0 (skipped under bounds-tree)
  - `hit_test_bounds_tree_query_time_us`: p50 ~2.0, p95 ~3.0, max 9
  - `hit_test_candidate_self_only_time_us`: p50 ~0.0, p95 ~0.0, max 3

Takeaway:
- This closes the pointer-move hit-test hot path for the stripes torture probe: `hit_test_time_us` drops from
  ~0.58ms → ~0.003ms (≈ 190× reduction at p50).
- The remaining dispatch time is now dominated by non-hit-test routing + bookkeeping.

Bundles:
- Worst overall: `target/fret-diag-perf/2026-02-05-pointer-move-r7-skip-cached-path-8bc15eda/1770253131674-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 09:09:47 (commit `8bc15eda`)

Change:
- (experiment) Disable bounds-tree hit-test index to measure the fallback cost.

Why:
- This validates that the bounds-tree index is load-bearing for “Zed feel” pointer-move workloads, and it provides
  an upper bound for how costly the full traversal path is under the same script.

Command:
```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r3-bounds-tree-disabled-8bc15eda \
  --reuse-launch --warmup-frames 5 --repeat 3 --sort time --top 15 --json \
  --timeout-ms 300000 --poll-ms 200 \
  --max-pointer-move-dispatch-us 2000 \
  --max-pointer-move-hit-test-us 1500 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Result:
- The perf gate fails (expected) because `hit_test_time_us` rises above the 1500us threshold:
  - evidence: `target/fret-diag-perf/2026-02-05-pointer-move-r3-bounds-tree-disabled-8bc15eda/check.perf_thresholds.json`
- Metrics (median across 3 runs; 192 pointer-move frames per run):
  - `dispatch_time_us`: p50 ~2140.0, p95 ~2444.9, max 4362
  - `hit_test_time_us`: p50 ~1998.0, p95 ~2256.0, max 4311
  - `hit_test_fallback_traversal_time_us`: p50 ~1422.0, p95 ~1591.8, max 3226
  - `hit_test_cached_path_time_us`: p50 ~570.0, p95 ~774.9, max 1082

Takeaway:
- Without bounds-tree, this workload is ~2ms per pointer-move frame (and can spike to ~4ms). For Tier B “Zed feel”,
  bounds-tree (or an equivalent spatial index) is mandatory.

## 2026-02-05 10:08:53 (commit `7fa76fd5`)

Change:
- feat(diag): break down dispatch timing

Why:
- After `8bc15eda`, pointer-move hit testing is in the single-digit microseconds for the stripes torture probe, but the
  remaining dispatch time still matters for Tier B “Zed feel”.
- We need concrete, per-frame signals for **where dispatch time goes** (input bookkeeping vs routing vs widget hooks)
  so future refactors have a measurable target.

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-dispatch-breakdown-7fa76fd5 \
  --timeout-ms 300000 --poll-ms 200 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 800 \
  --max-pointer-move-hit-test-us 100 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; 192 pointer-move frames per run):
- Pointer-move frame costs:
  - `dispatch_time_us`: p50 ~221, p95 ~242, max (across runs) 289
  - `hit_test_time_us`: p50 ~3, p95 ~3, max (across runs) 10
- Hit-test sub-steps (per frame, accumulated across hit-test queries):
  - `hit_test_bounds_tree_query_time_us`: p50 ~2, p95 ~2, max 9
  - `hit_test_cached_path_time_us`: p50 ~0 (skipped under bounds-tree)
- Dispatch sub-steps (per frame):
  - `dispatch_widget_bubble_time_us`: p50 ~3, p95 ~5, max 13
  - `dispatch_input_context_time_us`: p50 ~1, p95 ~2, max 12
  - `dispatch_hover_update_time_us`: p50 ~1, p95 ~2, max 11
  - `dispatch_cursor_query_time_us`: p50 ~1, p95 ~1, max 3
  - `dispatch_active_layers_time_us`: p50 ~0, p95 ~0, max 3
  - `dispatch_event_chain_build_time_us`: p50 ~0 (sub-micro in this probe; rounds down)

Takeaway:
- The newly exported micro timers explain only a small fraction of the observed `dispatch_time_us`. This likely means
  a significant part of pointer-move dispatch cost is currently in **pointer routing / bookkeeping** not covered by the
  initial instrumentation points (or in code paths that round down to 0us at microsecond granularity).
- Next step: add a coarse “dispatch pointer routing” timer around the pointer-specific dispatch block to close the
  accounting gap before attempting deeper algorithmic refactors.

Bundles:
- Run dir: `target/fret-diag-perf/2026-02-05-pointer-move-r7-dispatch-breakdown-7fa76fd5/`
- Worst-by-dispatch (also worst-by-hit): `target/fret-diag-perf/2026-02-05-pointer-move-r7-dispatch-breakdown-7fa76fd5/1770256617791-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

Errata (2026-02-05):
- The pointer-move frame distribution for this probe is **bimodal**: half the frames are “no timer dispatch” and
  half are “timer dispatch” frames. With nearest-rank percentiles, this means `dispatch_time_us` p50 is closer to
  the no-timer baseline (≈ 20–40us), while p95 reflects the timer frames (≈ 240–260us).
- The original p50 number above (~221us) was computed from a timer-heavy subset and is not the nearest-rank p50 over
  *all* pointer-move frames. A follow-up attribution in commit `5ab4ba71` confirms the timer/other split explicitly.

## 2026-02-05 12:21:00 (commit `95806541`)

Change:
- feat(diag): time synthetic hover observer dispatch

Why:
- Verify whether synthetic hover observers account for the remaining pointer-move dispatch tail after `8bc15eda`.

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-synth-observer-timer-95806541 \
  --timeout-ms 300000 --poll-ms 200 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 800 \
  --max-pointer-move-hit-test-us 100 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; pointer-move frames; nearest-rank percentiles):
- `dispatch_synth_hover_observer_time_us`: p50 ~1, p95 ~1, max (across runs) 11

Takeaway:
- Synthetic hover observer dispatch is not a meaningful contributor to pointer-move dispatch time for this probe.

Bundles:
- Run dir: `target/fret-diag-perf/2026-02-05-pointer-move-r7-synth-observer-timer-95806541/`

## 2026-02-05 12:21:10 (commit `72e24f51`)

Change:
- feat(diag): time pointer-move layer observers

Why:
- Verify whether post-dispatch pointer-move observers (layer observers) are responsible for the remaining dispatch cost.

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-pointer-move-observers-timer-72e24f51-v2 \
  --timeout-ms 300000 --poll-ms 200 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 800 \
  --max-pointer-move-hit-test-us 100 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; pointer-move frames; nearest-rank percentiles):
- `dispatch_pointer_move_layer_observers_time_us`: p50 ~0, p95 ~0, max (across runs) 4

Takeaway:
- Pointer-move layer observers are not a meaningful contributor to pointer-move dispatch time for this probe.

Bundles:
- Run dir: `target/fret-diag-perf/2026-02-05-pointer-move-r7-pointer-move-observers-timer-72e24f51-v2/`

## 2026-02-05 12:21:20 (commit `51ad7cc9`)

Change:
- feat(diag): time post-dispatch snapshot and cursor effects

Why:
- Verify whether post-dispatch snapshots and cursor effects account for the remaining pointer-move dispatch tail.

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-post-dispatch-snapshot-timers-51ad7cc9 \
  --timeout-ms 300000 --poll-ms 200 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 800 \
  --max-pointer-move-hit-test-us 100 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; pointer-move frames; nearest-rank percentiles):
- `dispatch_cursor_effect_time_us`: p50 ~0, p95 ~0, max (across runs) 0
- `dispatch_post_dispatch_snapshot_time_us`: p50 ~0, p95 ~1, max (across runs) 2

Takeaway:
- Cursor effects and post-dispatch snapshots are not meaningful contributors to pointer-move dispatch time for this probe.

Bundles:
- Run dir: `target/fret-diag-perf/2026-02-05-pointer-move-r7-post-dispatch-snapshot-timers-51ad7cc9/`

## 2026-02-05 12:21:30 (commit `5ab4ba71`)

Change:
- feat(diag): attribute dispatch time by event class

Why:
- `dispatch_events` can be > 1 on pointer-move frames, but the bundle event log only captures injected events
  (e.g. `pointer.move`). We need to attribute dispatch time by **what kinds of events** were actually dispatched
  during the frame to explain the remaining dispatch tail.

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-event-class-breakdown-5ab4ba71 \
  --timeout-ms 300000 --poll-ms 200 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 800 \
  --max-pointer-move-hit-test-us 100 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; pointer-move frames; nearest-rank percentiles):
- Overall pointer-move distribution (bimodal due to timer dispatch):
  - `dispatch_time_us`: p50 ~30, p95 ~250, max (across runs) 303
  - `hit_test_time_us`: p50 ~3, p95 ~5, max (across runs) 12
- Pointer-move frames *without* timer dispatch (96/192 frames per run):
  - `dispatch_time_us`: p50 ~16, p95 ~25, max 38
  - `dispatch_pointer_event_time_us`: p50 ~16, p95 ~25, max 38
- Pointer-move frames *with* timer dispatch (96/192 frames per run):
  - `dispatch_time_us`: p50 ~241, p95 ~254, max 303
  - `dispatch_timer_event_time_us`: p50 ~223, p95 ~232, max 288
  - `dispatch_pointer_event_time_us`: p50 ~17, p95 ~25, max 36

Key attribution example (worst pointer-move dispatch frame in the worst run):
```sh
cargo run -p fretboard -- diag stats \
  target/fret-diag-perf/2026-02-05-pointer-move-r7-event-class-breakdown-5ab4ba71/1770264315951-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json \
  --sort dispatch --top 50 --json \
  | jq '. as $r | ($r.pointer_move.max_dispatch_at + {max_dispatch_time_us: $r.pointer_move.max_dispatch_time_us}) as $m | {pointer_move_max: $m, row: ($r.top[] | select(.frame_id==$m.frame_id and .tick_id==$m.tick_id and .window==$m.window) | {dispatch_time_us, dispatch_events, dispatch_pointer_events, dispatch_timer_events, dispatch_pointer_event_time_us, dispatch_timer_event_time_us})}'
```

Takeaway:
- The pointer-move “dispatch tail” for this probe is dominated by **timer event dispatch**.
- Pointer routing itself is already cheap in the no-timer baseline (~10–40us).
- Next: identify and eliminate/defang the timers that fire on alternating pointer-move frames.

Bundles:
- Run dir: `target/fret-diag-perf/2026-02-05-pointer-move-r7-event-class-breakdown-5ab4ba71/`
- Worst-by-dispatch: `target/fret-diag-perf/2026-02-05-pointer-move-r7-event-class-breakdown-5ab4ba71/1770264315951-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 15:10:00 (commit `5690e068`)

Change:
- perf(fret-ui): skip timer broadcast for targeted timers

Why:
- If the timer token has a recorded element target, broadcasting the same timer event across all timer-enabled layers
  should be unnecessary. This change makes the targeted routing path return early (no fallback broadcast).

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-skip-timer-broadcast-5690e068 \
  --timeout-ms 300000 --poll-ms 200 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 800 \
  --max-pointer-move-hit-test-us 100 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; pointer-move frames; nearest-rank percentiles):
- `dispatch_time_us`: p50 ~31, p95 ~250, max (across runs) 277
- `dispatch_timer_event_time_us`: p50 ~0, p95 ~229, max (across runs) 253

Takeaway:
- This does not materially change p95 for the probe (timer frames remain expensive), but it reduces the run-level max.
- Next: attribute whether the expensive timer frames are targeted or fallback broadcasts (and measure broadcast work).

Bundles:
- Run dir: `target/fret-diag-perf/2026-02-05-pointer-move-r7-skip-timer-broadcast-5690e068/`
- Worst-by-dispatch: `target/fret-diag-perf/2026-02-05-pointer-move-r7-skip-timer-broadcast-5690e068/1770266641499-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 16:40:00 (commit `7c40fcd3`)

Change:
- perf(fret-ui): avoid bubbling targeted timer events

Why:
- Hypothesis: the timer dispatch tail might come from bubbling a `Event::Timer` through a deep ancestor chain even when
  only the target element cares about the token.
- This change dispatches targeted timer events to the target element only (no bubbling).

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-timer-target-only-7c40fcd3 \
  --timeout-ms 300000 --poll-ms 200 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 800 \
  --max-pointer-move-hit-test-us 100 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; pointer-move frames; nearest-rank percentiles):
- `dispatch_time_us`: p50 ~31, p95 ~252, max (across runs) 503
- `dispatch_timer_event_time_us`: p50 ~0, p95 ~231, max (across runs) 479

Takeaway:
- This does not improve the probe’s p95 and introduces a large run-level max outlier (likely timer-related).
- This suggests the dominant timer cost is not simply “ancestor bubbling”, or that the probe is still hitting the
  fallback broadcast path for a timer token that has no element target.
- Next: add explicit counters for targeted-vs-broadcast timer routing and measure the broadcast loop (layers visited).

Bundles:
- Run dir: `target/fret-diag-perf/2026-02-05-pointer-move-r7-timer-target-only-7c40fcd3/`
- Worst-by-dispatch: `target/fret-diag-perf/2026-02-05-pointer-move-r7-timer-target-only-7c40fcd3/1770267697192-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 19:10:00 (commit `98ca4fe3`)

Change:
- feat(diag): break down timer dispatch

Why:
- The stripes pointer-move probe showed a large dispatch tail that attribution (commit `5ab4ba71`) already narrowed to
  timer event dispatch. We still needed to answer:
  - Is this timer work coming from targeted timer routing, or fallback broadcast routing?
  - Is the broadcast loop (layers visited) itself expensive, or is the cost elsewhere?
  - Which timer token is responsible for the slow frames?

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-timer-route-breakdown-dirty-21c14e33 \
  --timeout-ms 300000 --poll-ms 200 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 800 \
  --max-pointer-move-hit-test-us 100 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; pointer-move frames; nearest-rank percentiles):
- `dispatch_time_us`: p50 ~29, p95 ~247, max (across runs) 736
- `dispatch_pointer_event_time_us`: p50 ~16, p95 ~23, max (across runs) 32
- `dispatch_timer_event_time_us`: p50 ~0, p95 ~229, max (across runs) 714
- Timer routing detail:
  - `dispatch_timer_targeted_events`: p95 ~0 (no targeted timer delivery observed)
  - `dispatch_timer_broadcast_time_us`: p50 ~0, p95 ~223, max (across runs) 703
  - `dispatch_timer_broadcast_loop_time_us`: p50 ~0, p95 ~4, max (across runs) 22
  - Slowest token observed: `dispatch_timer_slowest_token` = 1 (broadcast)

Takeaway:
- The tail is a **single broadcast timer token** (`TimerToken(1)`).
- The broadcast **layer loop is not the cost** (loop time stays tiny); most of the time is “outside the loop”, i.e. due
  to other work performed during timer event handling.
- Next: verify whether the timer tail is avoidable background work (and if so, remove it from the probe), or else make
  it cheap enough to coexist with pointer-move events.

Bundles:
- Run dir: `target/fret-diag-perf/2026-02-05-pointer-move-r7-timer-route-breakdown-dirty-21c14e33/`
- Worst-by-dispatch: `target/fret-diag-perf/2026-02-05-pointer-move-r7-timer-route-breakdown-dirty-21c14e33/1770270312252-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 20:10:00 (commit `06feeb41`)

Change:
- perf(ui-gallery): skip config watcher in harness

Why:
- The timer token dominating the pointer-move tail (`TimerToken(1)`) was consistent with ui-gallery’s dev-only
  config-file poller (`with_config_files_watcher(...)`), which installs a repeating global timer.
- Scripted harness runs (especially perf probes) should isolate UI dispatch costs. Periodic background polling adds
  unrelated timer traffic that can co-occur with pointer-move frames and dominate p95/maximum dispatch time.

Command:
```sh
cargo build -p fret-ui-gallery --release

cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-harness-skip-config-watcher-dirty-21c14e33 \
  --timeout-ms 300000 --poll-ms 200 \
  --reuse-launch --warmup-frames 5 --repeat 7 --sort time --top 15 --json \
  --max-pointer-move-dispatch-us 800 \
  --max-pointer-move-hit-test-us 100 \
  --max-pointer-move-global-changes 0 \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; pointer-move frames; nearest-rank percentiles):
- `dispatch_time_us`: p50 ~16, p95 ~26, max (across runs) 37
- `dispatch_timer_event_time_us`: p95 ~0 (no timer dispatch observed during pointer-move frames)
- `hit_test_time_us`: p50 ~2, p95 ~4, max (across runs) 13

Takeaway:
- The pointer-move dispatch tail was dominated by **dev-only config polling timer traffic** in ui-gallery.
- With config watcher suppressed during scripted harness runs, the probe reflects the intended UI mechanisms:
  pointer routing + hit-test remain in the ~tens-of-microseconds range on this machine.

Bundles:
- Run dir: `target/fret-diag-perf/2026-02-05-pointer-move-r7-harness-skip-config-watcher-dirty-21c14e33/`
- Worst-by-dispatch: `target/fret-diag-perf/2026-02-05-pointer-move-r7-harness-skip-config-watcher-dirty-21c14e33/1770272814649-ui-gallery-hit-test-torture-stripes-move-sweep-steady/bundle.json`

## 2026-02-05 15:59:00 (commit `1293364f`, built on `e978fe85`)

Change:
- `perf(ui-gallery): add hit-test torture redraw knob`
  - New env: `FRET_UI_GALLERY_HIT_TEST_TORTURE_REDRAW_ON_MOVE=1`
  - Goal: keep pointer-move probes deterministic when the torture surface itself is paint-stable.

Why:
- `e978fe85` reintroduced a way to *force-enable* the ui-gallery config watcher in harness runs
  (`FRET_UI_GALLERY_ENABLE_CONFIG_WATCHER=1`) so we can reproduce and measure timer-driven behavior on demand.
- The earlier log entries showed that config watcher polling could dominate pointer-move dispatch tail latency.
  This entry re-checks whether that tail still exists on current `main`.

Commands (macOS Apple M4, repeat=7, `sort=dispatch`):
```sh
cargo build -p fret-ui-gallery --release

# Baseline: harness-only hit-test torture, config watcher suppressed by default.
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-config-watcher-off \
  --timeout-ms 180000 --repeat 7 --sort dispatch --top 15 --json \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_UI_GALLERY_HIT_TEST_TORTURE_REDRAW_ON_MOVE=1 \
  --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=2000 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery

# Forced: enable the config watcher poller even in harness-only mode.
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json \
  --dir target/fret-diag-perf/2026-02-05-pointer-move-r7-config-watcher-on \
  --timeout-ms 180000 --repeat 7 --sort dispatch --top 15 --json \
  --env FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture \
  --env FRET_UI_GALLERY_HIT_TEST_TORTURE_REDRAW_ON_MOVE=1 \
  --env FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE=2000 \
  --env FRET_UI_GALLERY_ENABLE_CONFIG_WATCHER=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Results (median across 7 runs; pointer-move frames; nearest-rank percentiles):

- Config watcher **off**:
  - `pointer_move_max_dispatch_time_us`: p50 ~14us, p95 ~16us, max 16us
  - `pointer_move_max_hit_test_time_us`: p50 ~2us, p95 ~2us, max 2us
  - `pointer_move_snapshots_with_global_changes`: p95 ~0
- Config watcher **forced on**:
  - `pointer_move_max_dispatch_time_us`: p50 ~14us, p95 ~16us, max 16us
  - `pointer_move_max_hit_test_time_us`: p50 ~2us, p95 ~2us, max 2us
  - `pointer_move_snapshots_with_global_changes`: p95 ~0

Takeaway:
- On current `main`, forcing the ui-gallery config watcher back on does **not** reintroduce a measurable pointer-move
  dispatch tail for this probe. This suggests the earlier timer-driven hitch mechanism has been eliminated or reduced
  to “noise floor” for this workload.

## 2026-02-05 16:12:00 (commit `b87bf64d` → `5b5d3fe3`)

Change:
- Run the steady-state gate on current `main` against the older macOS M4 baseline (v5), then reduce timer noise:
  - `perf(ui-gallery): suppress config watcher during diag perf` (commit `5b5d3fe3`)

Why:
- The v5 baseline (`05cd5691`) predates several diagnostics/runtime changes; it is still useful as a regression signal,
  but we must keep timer-driven background work out of gate runs (the earlier pointer-move probe already showed how
  a dev-only polling timer can dominate tails when it lines up with an interaction).

Gate run (v5 baseline; repeat=7; `ui-gallery-steady`; `sort=time`):
- Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v5.json`
- Run dir (before watcher suppression): `target/fret-diag-perf/ui-gallery-steady.gap-check.1770279063/`
- Result: gate failed (failures=4)
  - `ui-gallery-window-resize-stress-steady`: one run hit `top_total_time_us=19447` (thr `17201`)
    - Worst bundle: `target/fret-diag-perf/ui-gallery-steady.gap-check.1770279063/1770279097844-ui-gallery-window-resize-stress-steady/bundle.json`
    - Attribution: dispatch contained `dispatch_post_dispatch_snapshot_time_us~2810us` (timer-aligned noise).
  - `ui-gallery-menubar-keyboard-nav-steady`: consistent `top_total_time_us~3.0ms` across runs (thr `2642us`)
    - Worst bundle: `target/fret-diag-perf/ui-gallery-steady.gap-check.1770279063/1770279078981-ui-gallery-menubar-file-escape-steady/bundle.json`

Fix:
- Suppress the ui-gallery config watcher when running under diagnostics (detect `FRET_DIAG_DIR`), unless explicitly
  forced via `FRET_UI_GALLERY_ENABLE_CONFIG_WATCHER=1`:
  - Commit: `5b5d3fe3`

Re-run (v5 baseline; repeat=7):
- Run dir: `target/fret-diag-perf/ui-gallery-steady.gap-check.after-suppress-watcher.1770279883/`
- Result: gate failed (failures=1)
  - Only remaining failure: `ui-gallery-menubar-keyboard-nav-steady` max `2941us` (thr `2642us`).

Baseline update (macOS M4 v6; repeat=7):
- New baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v6.json` (generated at commit `5b5d3fe3`)
- Run dir: `target/fret-diag-perf/ui-gallery-steady.macos-m4.v6.1770280087/`
- Note: the v6 baseline includes pointer-move maxima per script in addition to `top_total/layout/solve` thresholds.

Gate check (v6 baseline; repeat=3):
- `target/fret-diag-perf/ui-gallery-steady.macos-m4.v6.check.1770280162/` showed a resize outlier (`top_total_time_us=21780`)
  and failed; immediate re-run passed:
  - `target/fret-diag-perf/ui-gallery-steady.macos-m4.v6.check2.1770280248/` (passed; worst `top_total_time_us=13293`)
  - Takeaway: `ui-gallery-window-resize-stress-steady` can still be flaky at low repeat counts; prefer repeat=7 for
    contract checks, and keep investigating rare solve/layout outliers (text measure cache / intrinsic probes).

## 2026-02-05 18:00:00 (commit `f2bee87a`)

Change:
- Export paint-pass breakdown metrics into diagnostics bundles and `fretboard diag stats`:
  - `paint_cache_replay_time_us`
  - `paint_cache_bounds_translate_time_us` / `paint_cache_bounds_translated_nodes`
  - `paint_record_visual_bounds_time_us` / `paint_record_visual_bounds_calls`

Why:
- Several “steady-state” probes (notably the menubar script) show non-trivial `paint_time_us` even with view-cache reuse.
  We needed to confirm whether paint-cache replay (or subtree bounds translation) was responsible.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-f2bee87a.1770300800/`
- Command (repeat=7; `sort=time`):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --dir target/fret-diag-perf/menubar-kbd-nav.after-f2bee87a.1770300800 \
  --reuse-launch --repeat 7 --timeout-ms 180000 --sort time --top 15 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
- `top_total_time_us`: p50 ~3504, p95 ~3740, max 3740
- Worst bundle:
  - `target/fret-diag-perf/menubar-kbd-nav.after-f2bee87a.1770300800/1770285619385-ui-gallery-menubar-file-escape-steady/bundle.json`
- Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
  - `paint_time_us=2669`
  - `paint_cache_replayed_ops=453`
  - `paint_cache_replay_time_us=6`
  - `paint_cache_bounds_translate_time_us=0` (`paint_cache_bounds_translated_nodes=0`)
  - `paint_record_visual_bounds_time_us=15` (`paint_record_visual_bounds_calls=155`)

Takeaway:
- For this workload, paint-cache replay and paint-cache bounds translation are **not** the hotspot.
- The remaining paint cost likely comes from other paint-phase work (per-node traversal overhead, widget paint costs,
  observation bookkeeping, or window snapshot plumbing). Next step: add paint micro timers to explain this slice
  (tracked in `docs/workstreams/ui-perf-paint-pass-breakdown-v1.md`).

## 2026-02-05 18:28:00 (commit `b20a1280`)

Change:
- Add initial paint micro-breakdown timers (paint-all plumbing) and export them into bundles + `fretboard diag stats`:
  - `paint_input_context_time_us`
  - `paint_scroll_handle_invalidation_time_us`
  - `paint_collect_roots_time_us`
  - `paint_publish_text_input_snapshot_time_us`
  - `paint_collapse_observations_time_us`

Why:
- The menubar steady probe still shows ~2.6ms `paint_time_us` with view-cache reuse and near-free paint-cache replay.
  We needed to prove/disprove that “paint-all plumbing” was the culprit before instrumenting per-node traversal.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-b20a1280.micro.1770287305/`
- Command (repeat=7; `sort=time`):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --dir target/fret-diag-perf/menubar-kbd-nav.after-b20a1280.micro.1770287305 \
  --reuse-launch --repeat 7 --timeout-ms 180000 --sort time --top 15 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
- `top_total_time_us`: p50 ~3386, p95 ~3776, max 3776
- Worst bundle:
  - `target/fret-diag-perf/menubar-kbd-nav.after-b20a1280.micro.1770287305/1770287306932-ui-gallery-menubar-file-escape-steady/bundle.json`
- Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
  - `paint_time_us=2693`
  - `paint_cache_replayed_ops=453`
  - `paint_cache_replay_time_us=6`
  - `paint_cache_bounds_translate_time_us=0` (`paint_cache_bounds_translated_nodes=0`)
  - `paint_record_visual_bounds_time_us=15` (`paint_record_visual_bounds_calls=155`)
  - `paint_breakdown.us(input_ctx/scroll_inv/collect_roots/text_snapshot/collapse)=0/0/0/0/46`

Takeaway:
- The paint-all “plumbing” micro timers are not where the ~2.6ms paint slice goes for this probe.
- Next: instrument per-node paint traversal and widget paint (cache hit vs miss) to explain the remaining slice
  (tracked in `docs/workstreams/ui-perf-paint-pass-breakdown-v1.md`).

## 2026-02-05 19:11:00 (commit `c512be81`)

Change:
- Add paint node breakdown timers and export them into bundles + `fretboard diag stats`:
  - `paint_cache_key_time_us`
  - `paint_cache_hit_check_time_us`
  - `paint_widget_time_us` (exclusive; pauses while painting children)
  - `paint_observation_record_time_us`

Why:
- The menubar steady probe still shows ~2.6ms `paint_time_us` with view-cache reuse. We needed to confirm whether the
  remaining slice is “widget paint” vs paint-cache bookkeeping vs observation recording.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-c512be81.1770289882/`
- Command (repeat=7; `sort=time`):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --dir target/fret-diag-perf/menubar-kbd-nav.after-c512be81.1770289882 \
  --reuse-launch --repeat 7 --timeout-ms 180000 --sort time --top 15 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results (us; `--sort time`):
- `top_total_time_us`: p50 ~3568, p95 ~3734, max 3734
- Worst bundle:
  - `target/fret-diag-perf/menubar-kbd-nav.after-c512be81.1770289882/1770289882739-ui-gallery-menubar-file-escape-steady/bundle.json`
- Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
  - `paint_time_us=2655`
  - `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2555/11`
  - `paint_breakdown.us(input_ctx/scroll_inv/collect_roots/text_snapshot/collapse)=0/0/0/0/43`

Takeaway:
- For this stable workload, paint is dominated by exclusive widget paint code (`paint_widget_time_us`), not paint-cache
  replay/key checks, and not paint-all plumbing.

## 2026-02-05 19:25:00 (commit `f3078d25`)

Change:
- Add an experimental knob to relax the paint-cache view-cache gating:
  - Env: `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING=1`
  - Effect: when view-cache is active, allow paint-cache candidates beyond view-cache roots.

Why:
- `paint_widget_time_us` dominates the menubar steady paint slice. We wanted a quick A/B to see whether broadening
  the paint-cache eligibility surface reduces widget paint overhead on stable frames.

Probe (A/B):
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Baseline (no relax knob): see 2026-02-05 19:11 (commit `c512be81`).
- Relaxed run:
  - Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-relax-paint-cache.1770290717/`
  - Command (repeat=7; `sort=time`):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --dir target/fret-diag-perf/menubar-kbd-nav.after-relax-paint-cache.1770290717 \
  --reuse-launch --repeat 7 --timeout-ms 180000 --sort time --top 15 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING=1 \
  --launch -- target/release/fret-ui-gallery
```

Results (us; relaxed run; `--sort time`):
- `top_total_time_us`: p50 ~3438, p95 ~3718, max 3718
- Worst bundle:
  - `target/fret-diag-perf/menubar-kbd-nav.after-relax-paint-cache.1770290717/1770290719459-ui-gallery-menubar-file-escape-steady/bundle.json`
- Worst-frame paint breakdown:
  - `paint_time_us=2610`
  - `paint_nodes_performed=30` (baseline was 153)
  - `paint_cache_hits=12` (`paint_cache_replayed_ops=500`)
  - `paint_widget_time_us=2540`

Takeaway:
- Relaxing the view-cache gating increased paint-cache hits and reduced the number of widgets that run `paint()`,
  but did **not** materially reduce `paint_widget_time_us` or `paint_time_us` on this probe.
- Next: identify which nodes still dominate `paint_widget_time_us` (need per-node paint hotspots or cache-disabled
  reason counters) and evaluate higher-level caching boundaries.

## 2026-02-05 20:03:00 (commit `e1132c95`)

Change:
- Export paint widget hotspots into diag bundles and surface them in `fretboard diag stats`:
  - `debug.paint_widget_hotspots[]` (top-N by exclusive widget paint time)
  - Includes `widget_type`, `exclusive_time_us`, `inclusive_time_us`, and `scene_ops_delta` (exclusive + inclusive)

Why:
- `paint_widget_time_us` dominates the menubar steady paint slice, but we needed to know which widgets are actually
  responsible before attempting more aggressive caching/refactors.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-paint-widget-hotspots.1770292980/`
- Command (repeat=7; `sort=time`):

```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --dir target/fret-diag-perf/menubar-kbd-nav.after-paint-widget-hotspots.1770292980 \
  --reuse-launch --repeat 7 --timeout-ms 180000 --sort time --top 15 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results (worst frame; `fretboard diag stats --sort time --top 1`):
- Worst bundle:
  - `target/fret-diag-perf/menubar-kbd-nav.after-paint-widget-hotspots.1770292980/1770292982106-ui-gallery-menubar-file-escape-steady/bundle.json`
- Worst-frame paint breakdown:
  - `paint_time_us=2592`
  - `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2487/12`
  - `paint_widget_hotspots` (top 3):
    - `us=1117 type=fret_ui::declarative::host_widget::ElementHostWidget ops(excl/incl)=1/1`
    - `us=942  type=fret_ui::declarative::host_widget::ElementHostWidget ops(excl/incl)=1/1`
    - `us=373  type=fret_ui::declarative::host_widget::ElementHostWidget ops(excl/incl)=1/1`
  - Top-3 sum: ~2432us (~98% of `paint_widget_time_us=2487`).

Takeaway:
- Stable-frame widget paint time is extremely concentrated in a few `ElementHostWidget` nodes.
- The ops deltas (`1/1`) suggest the cost is not scene encoding, but CPU bookkeeping inside the host-widget paint path
  (likely element-runtime observation access and/or instance lookup).
- Next: remove per-frame allocation/clone in element-runtime observation accessors
  (`elements::{observed_models_for_element, observed_globals_for_element}` or equivalent) and re-run this probe.

## 2026-02-05 20:28:06 (commit `424ca9fc`)

Change:
- Replace per-call cloning of element-runtime observation vectors with a zero-allocation iterator/closure API:
  - `observed_models_for_element(...) -> Vec<_>` becomes `with_observed_models_for_element(..., |items| ...)`
  - Same for globals.

Why:
- Hypothesis: the stable-frame `ElementHostWidget` paint hotspots were dominated by per-frame `Vec` clones of observed
  model/global dependencies.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-observed-models-no-clone.424ca9fc.1770294486/`
- Command (repeat=7; `sort=time`): same as 20:03 entry, with the new `--dir`.

Results (us; `--sort time`):
- `top_total_time_us`: p50 ~3510, p95 ~3724, max 3724 (note: slightly worse than the 20:03 run; could be noise)
- Worst bundle:
  - `target/fret-diag-perf/menubar-kbd-nav.after-observed-models-no-clone.424ca9fc.1770294486/1770294488214-ui-gallery-menubar-file-escape-steady/bundle.json`
- Worst-frame paint breakdown:
  - `paint_time_us=2654`
  - `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2545/12`
  - `paint_widget_hotspots` (top 3):
    - `us=1140 type=ElementHostWidget ops(excl/incl)=1/1`
    - `us=965  type=ElementHostWidget ops(excl/incl)=1/1`
    - `us=383  type=ElementHostWidget ops(excl/incl)=1/1`

Takeaway:
- This change did **not** reduce the `ElementHostWidget` paint hotspots for this probe.
- Likely the dominant cost is elsewhere in the host-widget paint path (instance lookup, view-cache bookkeeping, or
  first-call per-frame preparation in `ElementRuntime`), not the `Vec` clone itself.

## 2026-02-05 20:37:01 (commit `df5df0b7`)

Change:
- When `observed_*_next` is missing for an element, fall back to `observed_*_rendered` without cloning into `*_next`.

Why:
- Hypothesis: stable cached frames were paying hidden clone cost via `touch_observed_*_for_element_if_recorded(...)`.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Run dir: `target/fret-diag-perf/menubar-kbd-nav.after-observed-models-merge-rendered.df5df0b7.1770295021/`

Results (us; `--sort time`):
- `top_total_time_us`: p50 ~3523, p95 ~3857, max 3857 (worse; likely extra lookup overhead + noise)
- Worst bundle:
  - `target/fret-diag-perf/menubar-kbd-nav.after-observed-models-merge-rendered.df5df0b7.1770295021/1770295023042-ui-gallery-menubar-file-escape-steady/bundle.json`
- Worst-frame paint breakdown:
  - `paint_time_us=2761`
  - `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2649/13`
  - `paint_widget_hotspots` remains dominated by `ElementHostWidget` (top-3 sum ~2.59ms).

Takeaway:
- The “missing observed_*_next” fallback did not improve stable-frame paint for this probe.
- Next: instrument `ElementHostWidget::paint_impl` with sub-timers (obs-models, obs-globals, instance lookup) to locate
  the remaining ~1ms+ slices, and only then attempt a targeted refactor.

## 2026-02-05 13:20:04 (commit `188d7da1`)

Change:
- Export `ElementHostWidget::paint_impl` sub-timers:
  - observed models iteration time + item count
  - observed globals iteration time + item count
  - element instance lookup time + call count

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Worst bundle:
  - `target/fret-diag/1770297604582-ui-gallery-menubar-file-escape-steady/bundle.json`

Command:
```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --reuse-launch --repeat 7 --timeout-ms 180000 --sort time --top 15 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results (us; repeat=7):
- `total_time_us`: p50=3303 p95=3552 max=3552

Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
- `paint_time_us=2551`
- `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2452/12`
- `paint_host_widget.us(models/globals/instance)=16/10/16 items=14/1 calls=153`
- `paint_widget_hotspots` (top 3):
  - `us=1101 type=ElementHostWidget ops(excl/incl)=1/1`
  - `us=933  type=ElementHostWidget ops(excl/incl)=1/1`
  - `us=352  type=ElementHostWidget ops(excl/incl)=1/1`

Takeaway:
- Observed deps access + instance lookup are **not** the cause of the ~1ms+ host-widget paint hotspots (they are
  O(10us) each on this probe).
- Next: time the remaining host-widget paint overhead candidates (child traversal / bounds queries / clip setup), then
  only attempt an aggressive refactor once the sub-slice is confirmed.

## 2026-02-05 13:31:54 (commit `c80525b9`)

Change:
- Add `ElementInstance` kind strings to exported widget paint hotspots (so `ElementHostWidget` hotspots can be
  attributed to `Text` vs `Container` vs `ViewCache`, etc).

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Worst bundle:
  - `target/fret-diag/1770298314770-ui-gallery-menubar-file-escape-steady/bundle.json`

Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
- `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2727/13`
- `paint_widget_hotspots` (top 3):
  - `us=1205 kind=Text type=ElementHostWidget ops(excl/incl)=1/1`
  - `us=1033 kind=Text type=ElementHostWidget ops(excl/incl)=1/1`
  - `us=421  kind=Text type=ElementHostWidget ops(excl/incl)=1/1`

Takeaway:
- The stable-frame `ElementHostWidget` hotspots are specifically `ElementInstance::Text` paint paths (not generic
  container/bookkeeping).

## 2026-02-05 13:42:10 (commit `07d2ccf2`)

Change:
- Export paint-phase counters for text blob preparation:
  - `paint_text_prepare_time_us`
  - `paint_text_prepare_calls`

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Worst bundle:
  - `target/fret-diag/1770298930506-ui-gallery-menubar-file-escape-steady/bundle.json`

Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
- `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2617/13`
- `paint_text_prepare.us(time/calls)=2543/3`
- `paint_widget_hotspots` (top 3) remain `kind=Text` and sum to ~2.44ms.

Takeaway:
- Worst frames on this probe are spending ~2.5ms in `TextService::prepare` (3 calls), which largely explains the
  paint hotspots.
- Follow-up evidence suggests `paint_text_prepare_calls` is often `0` on many frames, with spikes concentrated in a
  smaller subset of frames (e.g. first appearance / cache miss frames). Treat this as a **tail-latency** issue until
  per-element attribution confirms true per-frame churn.

## 2026-02-05 14:13:54 (commit `80a46d49`)

Change:
- Export per-reason counters for text prepares (why `needs_prepare` fired).
- Also quantize paint-time text `max_width` to device pixel boundaries when building `TextConstraints` (to reduce
  cache churn from subpixel widths; expected to help some cases).

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Worst bundle:
  - `target/fret-diag/1770300835921-ui-gallery-menubar-file-escape-steady/bundle.json`

Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
- `paint_node.us(cache_key/hit_check/widget/obs_record)=3/0/2517/14`
- `paint_text_prepare.us(time/calls)=2447/3`
- `paint_text_prepare.reasons(blob/scale/text/rich/style/wrap/overflow/width/font)=3/3/0/0/0/0/0/3/0`

Takeaway:
- Worst-frame text prepares are dominated by `blob_missing` (and derived "key changed" fields), i.e. the
  `ElementHostWidget` text blob cache is missing when the hitch occurs.
- `blob_missing` can mean either “first prepare for this widget” **or** “cache was cleared between frames”, so this is
  not yet proof of per-frame churn.
- Next: attribute prepares to **stable element ids** across frames (top-N prepare hotspots), then explain whether misses
  are due to subtree churn / cleanup paths or simply first-appearance spikes; aim for warm stable frames where
  `paint_text_prepare_calls==0` and no >1ms prepare spikes.

## 2026-02-05 14:56:31 (commit `22e1b538`)

Change:
- Re-run the menubar steady probe with consistent env + warmup/repeat (no code change; baseline evidence refresh).

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Worst bundle:
  - `target/fret-diag-codex-vcache/1770303391967-ui-gallery-menubar-file-escape-steady/bundle.json`

Command:
```bash
target/codex-perf/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --warmup-frames 10 --repeat 5 --reuse-launch --sort time --json \
  --dir target/fret-diag-codex-vcache \
  --launch -- target/codex-perf/release/fret-ui-gallery
```

Results (us; repeat=5):
- `total_time_us`: min=3500 p50=3711 p95=3886 max=3886

Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
- `time.us(total/layout/prepaint/paint)=3886/1220/29/2637`
- `paint_text_prepare.us(time/calls)=2439/3`
- `paint_text_prepare.reasons(blob/scale/text/rich/style/wrap/overflow/width/font)=3/3/0/0/0/0/0/3/0`

Takeaway:
- This probe still hits multi-millisecond text prepare spikes even with warmup + view cache enabled; next step remains
  per-element attribution to distinguish “first appearance” from “cache cleared/recreated” spikes.

## 2026-02-05 15:15:57 (commit `77979100`)

Change:
- Export `paint_text_prepare_hotspots` (top-N per frame) into diagnostics bundles and surface it in `fretboard diag stats`.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json`
- Worst bundle:
  - `target/fret-diag-codex-preparehot/1770304558320-ui-gallery-menubar-file-escape-steady/bundle.json`

Command:
```bash
target/codex-perf/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --warmup-frames 10 --repeat 1 --reuse-launch --sort time --json \
  --dir target/fret-diag-codex-preparehot \
  --launch -- target/codex-perf/release/fret-ui-gallery
```

Worst-frame paint breakdown (from `fretboard diag stats --sort time --top 1`):
- `paint_text_prepare.us(time/calls)=2365/3`
- `paint_text_prepare_hotspots` (top 3):
  - `us=1085 node=12884902507 kind=Text len=652 max_width=494.0 wrap=word overflow=clip reasons=blob|scale|width element=3279273990770790565`
  - `us=917  node=4294967930 kind=Text len=586 max_width=494.0 wrap=word overflow=clip reasons=blob|scale|width element=1046958583803201156`
  - `us=361  node=4294967931 kind=Text len=258 max_width=494.0 wrap=word overflow=clip reasons=blob|scale|width element=15496724796638654331`

Takeaway:
- We can now track whether the *same element ids* are repeatedly missing their blobs across frames, or whether these
  are first-appearance spikes. Next: correlate these element ids with cleanup/remove-subtree records and cache-root
  reuse reasons.
- In the captured bundle above, each `paint_text_prepare_hotspots` element id only appears in a single snapshot,
  consistent with “first appearance” prepares (not per-frame churn).

## 2026-02-05 15:25:21 (commit `21198872`)

Change:
- Refresh steady-state suite evidence (no runtime changes expected; captures current tail metrics + bundles).

Suite:
- `ui-gallery-steady`

Command:
```bash
target/codex-perf/debug/fretboard diag perf ui-gallery-steady \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --warmup-frames 10 --repeat 3 --reuse-launch --sort time --json \
  --dir target/fret-diag-codex-suite \
  --launch -- target/codex-perf/release/fret-ui-gallery
```

Worst overall:
- script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- top_total_time_us: `14447`
- bundle: `target/fret-diag-codex-suite/1770305149472-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- The worst frame is layout-dominant (`layout_time_us=10591`) and includes resize-driven text re-prepare
  (`paint_text_prepare.us(time/calls)=2008/20`, `reasons=width_changed=20`), which is expected for a resize stress probe.

## 2026-02-05 15:36:09 (commit `0a8191eb`)

Change:
- Add a steady-state menubar probe that opens the File menu, resets diagnostics after mount, then runs a pointer-move
  sweep to validate “hover frames do not re-prepare text”.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-open-hover-sweep-steady.json`
- Bundle:
  - `target/fret-diag-codex-menubar-sweep/1770305770074-script-step-0013-press_key/bundle.json`

Command:
```bash
target/codex-perf/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-open-hover-sweep-steady.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --warmup-frames 5 --repeat 1 --reuse-launch --sort time --json \
  --dir target/fret-diag-codex-menubar-sweep \
  --launch -- target/codex-perf/release/fret-ui-gallery
```

Results:
- `paint_text_prepare_calls==0` across the measured sweep frames (no `paint_text_prepare_hotspots` recorded).
- Derived pointer-move maxima: `dispatch<=20us`, `hit_test<=1us` across 25 pointer-move frames.

## 2026-02-05 15:41:52 (commit `e6b1e228`)

Change:
- Add a “reopen after close” probe for the File menubar menu to validate that close/open does not drop text caches
  inside the same session.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-reopen-after-close.json`
- Bundle:
  - `target/fret-diag-codex-menubar-reopen/1770306112488-script-step-0016-press_key/bundle.json`

Command:
```bash
target/codex-perf/debug/fretboard diag perf tools/diag-scripts/ui-gallery-menubar-reopen-after-close.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --warmup-frames 5 --repeat 1 --reuse-launch --sort time --json \
  --dir target/fret-diag-codex-menubar-reopen \
  --launch -- target/codex-perf/release/fret-ui-gallery
```

Results:
- After the post-close `reset_diagnostics`, the second open stays at `paint_text_prepare_calls==0` (no prepare hotspots),
  indicating the menu subtree stays live / cached across close/open.

## 2026-02-05 15:43:13 (commit `5eaf5884`)

Change:
- Refresh baseline evidence for a code-view scroll probe with the new text-prepare hotspot export enabled.

Probe:
- Script: `tools/diag-scripts/ui-gallery-code-view-scroll-refresh-baseline.json`
- Bundle:
  - `target/fret-diag-codex-codeview/1770306194398-script-step-0019-press_key/bundle.json`

Results:
- Worst frame: `time.us(total/layout/prepaint/paint)=1288/1050/29/209`
- `paint_text_prepare_calls==0` (no prepare hotspots recorded).

## 2026-02-05 15:43:55 (commit `5eaf5884`)

Change:
- Refresh baseline evidence for the editor-class autoscroll torture page to find the current top CPU paint hotspot.

Probe:
- Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
- Bundle:
  - `target/fret-diag-codex-codeeditor/1770306238481-script-step-0011-press_key/bundle.json`

Worst frame (from `fretboard diag stats --sort time --top 1`):
- `time.us(total/layout/prepaint/paint)=6340/902/26/5412`
- `paint_widget_hotspots` dominated by `kind=Canvas`:
  - `us=5126 ops=581/581 node=4294968005 test_id=ui-gallery-code-editor-torture-root`
- Renderer signals on the same worst run:
  - `top_renderer_encode_scene_us=641`
  - `top_renderer_prepare_text_us=523`

Takeaway:
- This workload is currently bounded by CPU-side scene construction inside a `Canvas` element (not text prepares).
  Closing the gap to GPUI/Zed here likely requires more aggressive retained/replay strategies for editor-class surfaces
  (e.g. windowed line reuse + cheaper per-frame scene rebuild).

## 2026-02-05 17:48:00 (commit `78a7cd87`)

Change:
- Rerun a small “sanity baseline” set to verify whether earlier numbers drift (they can, due to timing and warmup).
- Generate a fresh `ui-gallery-steady` perf baseline snapshot (`macos-m4.v7`).
- Stabilize the menubar hover-sweep “steady” script by adding an extra post-reset warmup + reset.

Rerun probes:
- Script: `tools/diag-scripts/ui-gallery-menubar-open-hover-sweep-steady.json`
- Bundle:
  - `target/fret-diag-codex-rerun-menubar-sweep/1770313101809-script-step-0013-press_key/bundle.json`

Results:
- Observed `paint_text_prepare_calls=sum=1 (max=1)` in the captured bundle.
  - Single hotspot: `kind=Text`, `text_len=167`, `prepare_time_us=325`, `reasons_mask=blob_missing|scale_changed|width_changed`.
- Interpretation: still not a per-frame churn pattern (a single late “first visible paint” can slip past the script reset).
  The script now includes an extra warmup + reset to reduce this flakiness for future runs.

Follow-up (same commit, updated script shape):
- Bundle (with an additional warmup sweep before the measured sweep):
  - `target/fret-diag-codex-rerun-menubar-sweep-v3/1770313661905-script-step-0016-press_key/bundle.json`
- Still observed `paint_text_prepare_calls=sum=1`, suggesting the remaining prepare may be gated by a delayed hover policy
  (e.g. tooltip/intent) rather than purely “first paint after open”.

Rerun probes:
- Script: `tools/diag-scripts/ui-gallery-menubar-reopen-after-close.json`
- Bundle:
  - `target/fret-diag-codex-rerun-menubar-reopen/1770313229786-script-step-0016-press_key/bundle.json`

Results:
- After the post-close `reset_diagnostics`, the second open stays at `paint_text_prepare_calls==0` (no prepare hotspots).

Rerun probes:
- Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
- Bundle:
  - `target/fret-diag-codex-rerun-codeeditor-autoscroll/1770313271320-script-step-0011-press_key/bundle.json`

Worst frame (by `paint_time_us`):
- `paint_time_us=5149` (`paint_widget_time_us=5113`)
- `paint_widget_hotspots` dominated by `kind=Canvas`: `us=5096 ops=581/581`
- Renderer signals on the same worst run: `encode_scene_us=633`, `prepare_text_us=495`

Perf baseline snapshot:
- Baseline file: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v7.json`
- Worst overall script in the run: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
  - Evidence bundle: `target/fret-diag/1770313439094-ui-gallery-window-resize-stress-steady/bundle.json`

## 2026-02-06 01:50:00 (commit `72e6c32df`)

Change:
- Merge the latest `origin/main` into the local perf work branch (large upstream delta).
- Fix post-merge compilation issues caused by `slotmap` API expectations (`SecondaryMap::get` takes keys by value).
- Update the view-cache toggle perf scripts to avoid waiting for a now-missing popover close `test_id`
  (`ui-gallery-view-cache-popover-close`) and close via `Escape` instead.

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-open-hover-sweep-steady.json`
- Bundle:
  - `target/fret-diag-codex-postmerge-menubar-sweep/1770341327163-script-step-0016-press_key/bundle.json`

Results:
- `paint_text_prepare_calls==0` across the measured sweep frames (no prepare hotspots recorded).

Probe:
- Script: `tools/diag-scripts/ui-gallery-menubar-reopen-after-close.json`
- Bundle:
  - `target/fret-diag-codex-postmerge-menubar-reopen/1770341382081-script-step-0016-press_key/bundle.json`

Results:
- Observed `paint_text_prepare_calls=sum=1 (max=1)`, `paint_text_prepare_time_us=306`.
  - Single hotspot: `kind=Text`, `text_len=164`, `prepare_time_us=306`, `reasons_mask=blob_missing|scale_changed|width_changed`.

Probe:
- Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
- Bundle:
  - `target/fret-diag-codex-postmerge-codeeditor-autoscroll/1770341454895-script-step-0011-press_key/bundle.json`

Results:
- This workload regressed dramatically vs the earlier baseline: `paint_time_us` p50/p95/max = `27085/30223/33968`.
- `paint_widget_hotspots` remains dominated by `kind=Canvas`:
  - worst `Canvas us=33907 ops=581/581`, `scene_ops=1104`
  - same-frame renderer: `encode_scene_us=655`, `prepare_text_us=552`

Perf baseline snapshot:
- Baseline file: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v8.json`
- Worst overall script in the run: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
  - Evidence bundle: `target/fret-diag-codex-postmerge-perf/1770342116675-ui-gallery-window-resize-stress-steady/bundle.json`
- Notable drift vs v7 (max `top_total_time_us`):
  - `ui-gallery-view-cache-toggle-perf-steady`: `4757 → 13046` (script updated to close popover via `Escape`)
  - `ui-gallery-window-resize-stress-steady`: `22721 → 25156`

## 2026-02-06 10:05:00 (commit `b9ba410f6`)

Change:
- `CanvasPainter::{text,text_with_blob}` no longer bypass stable keys by using the “shared text cache” implicitly.
  - Shared text caching is now **explicit** (`CanvasPainter::shared_text*`), so high-entropy call sites can’t
    accidentally pollute a global/shared cache map.

Rationale:
- The post-merge `code-editor autoscroll` regression still showed `paint_widget_hotspots kind=Canvas`, and renderer
  self-time was not the dominant slice (`encode_scene_us` / `prepare_text_us` both sub-millisecond).
- Before this change, `text_with_blob(..)` could still land in the shared cache due to internal plumbing. That made it
  too easy for a tight loop (e.g. per-row paint) to do high-entropy “cache by content” and effectively turn the cache
  into a hashmap-backed allocation sink.
- This commit makes the intended contract match the workstream goal: caching is deterministic + keyed unless the
  call site explicitly opts into shared caching.

Evidence:
- See the post-merge regression bundle (commit `72e6c32df`) for the “Canvas dominates paint” symptom:
  - `target/fret-diag-codex-postmerge-codeeditor-autoscroll/1770341454895-script-step-0011-press_key/bundle.json`

## 2026-02-06 10:45:00 (commit `0d8ad27ac`)

Change:
- Fix code-editor syntax paint hot path: avoid cloning the full `Theme` per painted row.

Root cause (post-merge regression):
- The `code-editor autoscroll` probe became “allocation dominated” due to an accidental per-row `Theme` clone during
  syntax span → rich text construction. This caused extreme allocator churn (malloc/free + `drop_in_place<Theme>`)
  and made `Canvas` paint time explode to ~30ms per frame.

Probe:
- Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
- Worst frame bundle (pre-fix, from commit `72e6c32df`):
  - `target/fret-diag-codex-postmerge-codeeditor-autoscroll/1770341454895-script-step-0011-press_key/bundle.json`
- Worst frame bundle (after fix, commit `0d8ad27ac`):
  - `target/fret-diag-codex-codeeditor-autoscroll-after-0d8ad27ac/1770345867196-script-step-0011-press_key/bundle.json`

Results (from the 247 snapshots captured in the `script-step-0011-press_key` bundle; `paint_time_us` p50/p95/max):
- Pre-fix (`72e6c32df`): `27085 / 30215 / 33968`
- After fix (`0d8ad27ac`): `594 / 690 / 5699`

Interpretation:
- This was not a renderer encode or text-prepare bottleneck; it was CPU-side allocation churn in the editor paint path.
- The “Zed feel” gap is often dominated by allocation discipline, not just caching algorithms.

## 2026-02-06 11:14:00 (commit `0d8ad27ac`)

Change:
- Refresh the `ui-gallery-steady` baseline after the post-merge editor regression fix.

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --repeat 7 --warmup-frames 5 --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v9.json --perf-baseline-headroom-pct 30 --dir target/fret-diag-codex-perf-v9 --launch -- cargo run -p fret-ui-gallery --release
```

Perf baseline snapshot:
- Baseline file: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v9.json`
- Worst overall script in the run: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
  - `top_total_time_us=24017`
  - Evidence bundle: `target/fret-diag-codex-perf-v9/1770347631408-ui-gallery-window-resize-stress-steady/bundle.json`

Notable drift vs v8 (max `top_total_time_us`):
- `ui-gallery-dialog-escape-focus-restore-steady`: `3392 → 6947` (no obvious related code change; likely noise due to
  per-run process launches + warmup settings; consider re-running with `--reuse-launch` for a steadier baseline).
- `ui-gallery-window-resize-stress-steady`: `25156 → 24017`

## 2026-02-06 11:20:00 (commit `87de73754`)

Change:
- Merge the latest upstream `origin/main` on top of the editor regression fix work (refresh local main).
- Re-validate the editor-class autoscroll torture probe after the merge.

Probe:
- Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`

Command:
```powershell
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json --dir target/fret-diag-codex-after-origin-main-87de73754/editor-autoscroll.perf.r1 --repeat 1 --warmup-frames 5 --timeout-ms 240000 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Artifacts:
- Bundle: `target/fret-diag-codex-after-origin-main-87de73754/editor-autoscroll.perf.r1/1770347988112-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Results (from 240 captured snapshots; `paint_time_us` p50/p95/max):
- `802 / 889 / 5798`

Notes:
- The probe remains in the “sub-millisecond paint” regime after pulling upstream. Any further “Zed feel” work should
  focus on reducing tail outliers and on end-to-end GPU/present timing, not on baseline CPU paint throughput.

## 2026-02-06 11:47:00 (commit `09ecac494`)

Change:
- Refresh the `ui-gallery-steady` baseline using the **steady-state protocol** (`--reuse-launch`).

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --reuse-launch --repeat 7 --warmup-frames 5 --timeout-ms 300000 --sort time --top 15 --json --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v10.json --perf-baseline-headroom-pct 30 --dir target/fret-diag-codex-perf-v10 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Perf baseline snapshot:
- Baseline file: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v10.json`
- Worst overall script in the run: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
  - `top_total_time_us=16307` (baseline is max-based; see the suite JSON output for p95/max)
  - Evidence bundle: `target/fret-diag-codex-perf-v10/1770349612209-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- This baseline is **not directly comparable** to v9 because the protocol changed:
  - v9: per-script launches (more cold-start noise).
  - v10: `--reuse-launch` (intended steady-state).
- The purpose of v10 is to reduce noise so future regressions are explainable and stable.

## 2026-02-06 11:50:00 (commit `09ecac494`)

Probe:
- Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`

Command (repro; renderer perf snapshots recorded by the runner):
```bash
cargo run -p fretboard -- diag repro tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json \
  --dir target/fret-diag-codex-renderer-perf-09ecac494/editor-autoscroll.r2 \
  --timeout-ms 240000 --poll-ms 50 \
  --env FRET_DIAG_RENDERER_PERF=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_DIAG_MAX_SNAPSHOTS=240 \
  --launch -- target/release/fret-ui-gallery
```

Artifacts:
- stdout log: `target/fret-diag-codex-renderer-perf-09ecac494/editor-autoscroll.r2.stdout.log`
- bundle: `target/fret-diag-codex-renderer-perf-09ecac494/editor-autoscroll.r2/1770349792705-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Results (from 240 captured snapshots; per-frame values from `debug.stats.*`):
- `paint_time_us` p50/p95/max: `826 / 916 / 5967`
- `renderer_encode_scene_us` p50/p95/max: `~600 / 655 / 935`
- `renderer_prepare_text_us` p50/p95/max: `472 / 568 / 593`
- `renderer_draw_calls`: `69` (stable)
- `renderer_pipeline_switches`: `47` (stable)
- `renderer_text_atlas_upload_bytes`: `0` (no churn in this run)
- `renderer_text_atlas_evicted_pages`: `0`

Interpretation:
- On this workload, renderer CPU time is ~1.1–1.2ms/frame in the steady regime (encode + text prepare), while UI paint
  stays sub-millisecond p95. End-to-end 120Hz feel will likely require keeping this renderer slice stable (avoid upload
  churn) and making present timing observable (GPU/present hitches can dominate even when CPU is stable).

## 2026-02-06 12:04:00 (commit `f21a0aa82`)

Change:
- Add `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json` to the `ui-gallery-steady` suite.
- Refresh the suite baseline to include the new editor-grade row.

Suite:
- `ui-gallery-steady`

Command:
```powershell
cargo run -p fretboard -- diag perf ui-gallery-steady --reuse-launch --repeat 7 --warmup-frames 5 --timeout-ms 300000 --sort time --top 15 --json --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v11.json --perf-baseline-headroom-pct 30 --dir target/fret-diag-codex-perf-v11 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery
```

Perf baseline snapshot:
- Baseline file: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v11.json`
- Added row:
  - Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
  - `measured_max.top_total_time_us=7772`
  - Evidence bundle: `target/fret-diag-codex-perf-v11/1770350649172-ui-gallery-code-editor-torture-autoscroll-steady/bundle.json`

Drift vs v10:
- Existing rows are broadly stable (max `top_total_time_us` drift is small; see `v11 - v10` diff summary in local notes).
- Worst overall script remains `ui-gallery-window-resize-stress-steady` with `top_total_time_us=16136`
  (bundle: `target/fret-diag-codex-perf-v11/1770350673752-ui-gallery-window-resize-stress-steady/bundle.json`).

## 2026-02-06 12:36:00 (commit `65f8af318`)

Change:
- Make perf-baseline pointer-move thresholds less flaky by adding slack + quantum rounding (commit `43a9eb124`).
- Refresh `ui-gallery-steady` perf baseline (v12).

Context:
- Baseline v11 validation run was flaky by 1us:
  - Script: `tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json`
  - `pointer_move_max_dispatch_time_us=33` exceeded `threshold_us=32`
  - Evidence: `target/fret-diag-codex-perf-v11-validate/check.perf_thresholds.json`

Baseline command:
```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v12b \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --sort time --top 5 \
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v12.json \
  --perf-baseline-headroom-pct 20 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --launch -- target/release/fret-ui-gallery
```

Perf baseline snapshot:
- Baseline file: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v12.json`
- Worst overall script in the run: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
  - `top_total_time_us=16935`
  - Evidence bundle: `target/fret-diag-codex-perf-v12b/1770352388770-ui-gallery-window-resize-stress-steady/bundle.json`

Validation command:
```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v12-validate \
  --timeout-ms 300000 \
  --reuse-launch --repeat 3 --sort time --top 3 \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v12.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --launch -- target/release/fret-ui-gallery
```

Validation notes:
- Gate passes on repeat=3 (no threshold failures).
- Worst overall in the validation run was still the resize stress script:
  - `top_total_time_us=15954`
  - Bundle: `target/fret-diag-codex-perf-v12-validate/1770352514340-ui-gallery-window-resize-stress-steady/bundle.json`

Notes:
- This change is harness-level only (no runtime perf improvement expected).
- The next real smoothness win still needs to come from the resize path:
  - reduce `layout_roots_time_us` and `paint_text_prepare_time_us (width_changed)` tail outliers.

## 2026-02-06 13:20:00 (commit `beb2fa315`)

Change:
- Coalesce `WindowEvent::SurfaceResized` handling to once per frame (apply pending resize on `RedrawRequested`).

Why:
- GPUI/Zed effectively applies resize at the frame boundary (resize marks dirty; draw happens via request-frame).
  Several platforms can emit multiple resize notifications per vblank during interactive drags. Applying each one
  immediately can waste time reconfiguring the surface and relayouting more often than we can present.

Probe (single script):
- Script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`

Command:
```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-window-resize-stress-steady.json \
  --dir target/fret-diag-codex-perf-resize-coalesce-v2 \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --sort time --top 5 --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results (us):
- Worst overall `top_total_time_us=14219`
- Evidence bundle: `target/fret-diag-codex-perf-resize-coalesce-v2/1770355071995-ui-gallery-window-resize-stress-steady/bundle.json`

Suite baseline refresh:
- Baseline file: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v13.json`
- Run dir: `target/fret-diag-codex-perf-v13`
- Worst overall script in the run remains `ui-gallery-window-resize-stress-steady`:
  - `top_total_time_us=15532`
  - Evidence bundle: `target/fret-diag-codex-perf-v13/1770355191996-ui-gallery-window-resize-stress-steady/bundle.json`

Delta vs v12 baseline:
- `ui-gallery-window-resize-stress-steady` max `top_total_time_us` improves from `16935` (v12) → `15532` (v13).

Notes:
- This does not “avoid relayout during resize”. It reduces *redundant* work when multiple size updates arrive before a frame is drawn.
- The remaining gap for resize smoothness is still dominated by:
  - layout traversal/root build costs, and
  - text prepare on `width_changed` (wrap reflow) for chrome-heavy pages.

## 2026-02-06 13:45:00 (experiment; no code change)

Change:
- Enable deferred unbounded scroll probes during interactive resize:
  - `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1`
  - `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES=2`

Why:
- In `Scroll` layout, the default “unbounded probe” behavior measures scroll content with
  `AvailableSpace::MaxContent` on the scroll axis to compute extents.
- During window resize stress, this can become a large repeated cost when content reflows (wrap)
  on every width change.
- The scroll implementation already supports deferring the deep measure walk and reusing the last
  measured size for a small number of frames while the viewport is changing.

Probe (single script):
- Script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`

Command:
```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-window-resize-stress-steady.json \
  --dir target/fret-diag-codex-perf-resize-scroll-defer-v3 \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES=2 \
  --launch -- target/release/fret-ui-gallery
```

Results (us):
- Worst overall `top_total_time_us=11810`
- Evidence bundle: `target/fret-diag-codex-perf-resize-scroll-defer-v3/1770356485833-ui-gallery-window-resize-stress-steady/bundle.json`

Delta vs the coalesced resize run (same script; commit `beb2fa315` entry above):
- `top_total_time_us` improves from `14219` → `11810` (~-17%).

Notes:
- This is an env-gated experiment only; it does not ship as default behavior yet.
- The effect size and behavioral risk depend on scroll offset clamping semantics:
  if content extents lag behind viewport changes, offsets can clamp earlier/later than “perfect”
  wrap-aware extents. Before making this default, we should add a correctness probe:
  - assert scroll offset remains stable across a resize stress sequence, and
  - validate scrollbar thumb sizing does not glitch (or at least stays within an acceptable tolerance).

## 2026-02-06 14:26:00 (correctness gate; commit `6c248d9e1`)

Change:
- Add per-frame scroll telemetry in UI diagnostics bundles (`debug.scroll_nodes[]`):
  - `node`, `element`, `axis`, `offset_{x,y}`, `viewport_{w,h}`, `content_{w,h}`.
- Add a post-run diagnostics gate to ensure scroll offsets remain stable across a script run:
  - `fretboard diag run ... --check-scroll-offset-stable <test_id>`
- Add a dedicated correctness repro script that scrolls the view-cache page, then performs the
  resize stress sequence:
  - `tools/diag-scripts/ui-gallery-window-resize-scroll-offset-stable.json`

Why:
- The “deferred unbounded scroll probe” resize optimization is intentionally allowed to lag
  content extents while the viewport is changing.
- We need a scripted gate that catches catastrophic offset clamping/jumps while we iterate on the
  policy (and before considering a default-on switch).

Probe (single script; gate pass):
- Script: `tools/diag-scripts/ui-gallery-window-resize-scroll-offset-stable.json`
- Gate: `--check-scroll-offset-stable ui-gallery-content-viewport`

Command:
```bash
target/debug/fretboard diag run tools/diag-scripts/ui-gallery-window-resize-scroll-offset-stable.json \
  --dir target/fret-diag-codex-scroll-offset-stable-v1b \
  --timeout-ms 300000 --poll-ms 50 \
  --check-scroll-offset-stable ui-gallery-content-viewport \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES=2 \
  --launch -- cargo run -p fret-ui-gallery --release
```

Result:
- PASS
- Evidence bundle: `target/fret-diag-codex-scroll-offset-stable-v1b/1770359181990-ui-gallery-window-resize-scroll-offset-stable/bundle.json`

## 2026-02-06 15:01:00 (correctness gate; commits `8375df091`, `e20637f92`)

Change:
- Export per-frame scrollbar telemetry in UI diagnostics bundles (`debug.scrollbars[]`):
  - `node`, `element`, `axis`, `scroll_target`, `offset_{x,y}`, `viewport_{w,h}`, `content_{w,h}`,
    `track`, `thumb`, `hovered`, `dragging`.
- Add a post-run diagnostics gate to ensure scrollbar thumb geometry remains valid:
  - `fretboard diag run ... --check-scrollbar-thumb-valid all`
- Add a dedicated correctness repro script covering the resize stress sequence:
  - `tools/diag-scripts/ui-gallery-window-resize-scrollbar-thumb-valid.json`

Why:
- The “deferred unbounded scroll probe” resize optimization is intentionally allowed to lag
  content extents while the viewport is changing.
- We need a scripted gate that catches catastrophic scrollbar thumb glitches (negative sizes,
  thumb escaping the track) while we iterate on resize policy.

Probe (single script; gate pass):
- Script: `tools/diag-scripts/ui-gallery-window-resize-scrollbar-thumb-valid.json`
- Gate: `--check-scrollbar-thumb-valid all`

Command:
```bash
target/debug/fretboard diag run tools/diag-scripts/ui-gallery-window-resize-scrollbar-thumb-valid.json \
  --dir target/fret-diag-codex-scrollbar-thumb-valid-v1b \
  --timeout-ms 300000 --poll-ms 50 \
  --check-scrollbar-thumb-valid all \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES=2 \
  --launch -- cargo run -p fret-ui-gallery --release
```

Result:
- PASS
- Evidence bundle: `target/fret-diag-codex-scrollbar-thumb-valid-v1b/1770361216367-ui-gallery-window-resize-scrollbar-thumb-valid/bundle.json`

## 2026-02-06 15:28:00 (recheck; no code change)

Change:
- Re-run `ui-gallery-window-resize-stress-steady` after recent mainline changes to verify whether
  the earlier resize conclusions still hold.
- Compare default behavior vs deferred unbounded scroll probe behavior under the same protocol.

Probe (single script):
- Script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`

Command (default):
```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-window-resize-stress-steady.json \
  --dir target/fret-diag-codex-perf-resize-recheck-default-v1 \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Command (defer probe):
```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-window-resize-stress-steady.json \
  --dir target/fret-diag-codex-perf-resize-recheck-defer-v1 \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES=2 \
  --launch -- target/release/fret-ui-gallery
```

Results (us):
- Default (`target/fret-diag-codex-perf-resize-recheck-default-v1`):
  - `total_time_us`: min/p50/p95/max = `14862/15164/15323/15323`
  - `layout_time_us`: min/p50/p95/max = `11366/11671/11830/11830`
  - `paint_time_us`: min/p50/p95/max = `3346/3399/3417/3417`
- Defer probe (`target/fret-diag-codex-perf-resize-recheck-defer-v1`):
  - `total_time_us`: min/p50/p95/max = `11640/11672/11889/11889`
  - `layout_time_us`: min/p50/p95/max = `8171/8220/8393/8393`
  - `paint_time_us`: min/p50/p95/max = `3319/3347/3407/3407`

Delta (defer vs default):
- Worst `total_time_us`: `15323 -> 11889` (`-3434us`, about `-22%`).
- Worst `layout_time_us`: `11830 -> 8393` (`-3437us`, about `-29%`).
- Worst `paint_time_us`: `3417 -> 3407` (nearly unchanged).

Worst-frame attribution (recheck):
- Default worst bundle:
  - `target/fret-diag-codex-perf-resize-recheck-default-v1/1770362421483-ui-gallery-window-resize-stress-steady/bundle.json`
  - Top frame (`tick=256/frame=332`):
    - `layout_time_us=11830`, `paint_time_us=3395`, `paint_text_prepare_time_us=1378`
    - `paint_text_prepare_reason_width_changed=17`
- Defer worst bundle:
  - `target/fret-diag-codex-perf-resize-recheck-defer-v1/1770362463869-ui-gallery-window-resize-stress-steady/bundle.json`
  - Top frame (`tick=305/frame=386`):
    - `layout_time_us=8393`, `paint_time_us=3390`, `paint_text_prepare_time_us=1409`
    - `paint_text_prepare_reason_width_changed=18`

Node-level mapping (semantics-enabled one-shot):
- Bundle:
  - `target/fret-diag-codex-perf-resize-map-v1/1770362652598-ui-gallery-window-resize-stress-steady/bundle.json`
- Hottest layout nodes map to:
  - `node=4294968132` -> `test_id=ui-gallery-content-viewport`
  - `node=4294968244` -> descendant under `test_id=ui-gallery-view-cache-root`
- Interpretation:
  - the current dominant resize cost is still inside the content viewport subtree,
  - not paint-cache churn,
  - and not a broad cache-root miss (the sampled worst frames still show `cache_roots_reused=2/2`).

Notes:
- This recheck confirms the prior finding: deferred unbounded probe is primarily a layout-tail optimization.
- It does not reduce `paint_text_prepare` width-change work; text reflow remains a separate hotspot.

## 2026-02-06 16:12:00 (commit `e50173f13`)

Change:
- Add an experiment gate to decouple paint-cache replay from `HitTestOnly` invalidation:
  - `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1`
- Keep default behavior unchanged (gate-off by default).
- Add targeted unit coverage for gate off/on behavior and non-`HitTestOnly` regressions.

Why:
- `HitTestOnly` currently marks both `hit_test` and `paint` dirty, which can block paint-cache replay
  even when only interaction geometry changes.
- This experiment checks whether allowing replay in that narrow case improves resize smoothness.

Command (A/B template):
```bash
target/debug/fretboard diag perf <script.json> \
  --dir <out-dir> \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1 \
  --env FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES=2 \
  [--env FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1] \
  --launch -- target/release/fret-ui-gallery
```

Probe A: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- Gate off (`target/fret-diag-codex-paint-hit-test-off-v1`):
  - `total_time_us`: `11358/11483/11621/11621` (min/p50/p95/max)
  - `layout_time_us`: `8059/8146/8224/8224`
  - `paint_time_us`: `3198/3219/3305/3305`
- Gate on (`target/fret-diag-codex-paint-hit-test-on-v1`):
  - `total_time_us`: `11347/11417/11513/11513`
  - `layout_time_us`: `8046/8088/8231/8231`
  - `paint_time_us`: `3191/3232/3282/3282`
- Delta (on vs off):
  - worst `total_time_us`: `11621 -> 11513` (`-108us`, about `-0.93%`)
  - worst `paint_time_us`: `3305 -> 3282` (`-23us`)
  - worst `layout_time_us`: `8224 -> 8231` (`+7us`, noise-level)

Probe B: `tools/diag-scripts/ui-gallery-window-resize-scroll-offset-stable.json`
- Round 1:
  - off (`target/fret-diag-codex-paint-hit-test-off-v1b`): `total max=12006`
  - on (`target/fret-diag-codex-paint-hit-test-on-v1b`): `total max=14591` (single heavy outlier)
- Round 2 (recheck):
  - off (`target/fret-diag-codex-paint-hit-test-off-v2b`): `total max=12005`
  - on (`target/fret-diag-codex-paint-hit-test-on-v2b`): `total max=11603`

Outlier attribution note (Probe B round 1):
- Worst ON bundle:
  - `target/fret-diag-codex-paint-hit-test-on-v1b/1770365327865-ui-gallery-window-resize-scroll-offset-stable/bundle.json`
- Top frame (`tick=132/frame=179`) is dominated by broader frame work:
  - `layout_time_us=10311`, `paint_time_us=4179`, `dispatch_time_us=2947`
  - `paint_cache_hits=0`, `paint_cache_misses=3` (new gate path not clearly exercised in that frame)

Notes:
- Current evidence is mixed and noisy across resize probes; no robust, repeatable win yet.
- Keep `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` as an experiment-only gate.
- Next step: add diagnostics counters for “replay permitted by hit-test-only gate” and build a
  focused script where `HitTestOnly` dominates but layout is stable.

## 2026-02-06 17:32:00 (commit `f38f8c1d5`)

Change:
- Export two hit-test-only paint-cache gate counters end-to-end:
  - `paint_cache_hit_test_only_replay_allowed`
  - `paint_cache_hit_test_only_replay_rejected_key_mismatch`
- Wire counters through diagnostics and perf summaries:
  - `fret-ui` frame stats
  - `fret-bootstrap` bundle export
  - `fretboard diag` bundle parser + `--json` top metrics
- Add targeted unit assertions for both counter paths:
  - replay-allowed case
  - key-mismatch rejection case

Validation:
- `cargo nextest run -p fret-ui paint_cache_hit_test_only_invalidation_replays_when_toggle_on paint_cache_hit_test_only_replay_reject_counter_tracks_key_mismatch`
- `cargo check -q -p fret-ui -p fret-bootstrap -p fretboard`

Probe A: hit-test move sweep (counter visibility check)
- Script: `tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json`

Command (gate off):
```bash
target/release/fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json \
  --dir target/fret-diag-codex-paint-hit-test-counter-off-v3 \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Command (gate on):
```bash
target/release/fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json \
  --dir target/fret-diag-codex-paint-hit-test-counter-on-v3 \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1 \
  --launch -- target/release/fret-ui-gallery
```

Results (us):
- Gate off (`target/fret-diag-codex-paint-hit-test-counter-off-v3`):
  - `total_time_us`: `1647/1688/2104/2104` (min/p50/p95/max)
  - `layout_time_us`: `1140/1442/1504/1504`
  - `paint_time_us`: `188/197/964/964`
- Gate on (`target/fret-diag-codex-paint-hit-test-counter-on-v3`):
  - `total_time_us`: `1597/1681/1749/1749`
  - `layout_time_us`: `1376/1459/1525/1525`
  - `paint_time_us`: `187/192/194/194`

Counter evidence:
- For all 14 runs (off + on):
  - `top_paint_cache_hit_test_only_replay_allowed = 0`
  - `top_paint_cache_hit_test_only_replay_rejected_key_mismatch = 0`

Probe B: resize stress recheck with counters
- Script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
- Env includes resize-defer probe:
  - `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1`
  - `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES=2`

Results (us):
- Gate off (`target/fret-diag-codex-paint-hit-test-counter-resize-off-v2`):
  - `total_time_us`: `11319/11413/11499/11499`
  - `layout_time_us`: `8036/8112/8190/8190`
  - `paint_time_us`: `3172/3195/3222/3222`
- Gate on (`target/fret-diag-codex-paint-hit-test-counter-resize-on-v2`):
  - `total_time_us`: `11649/11722/12257/12257`
  - `layout_time_us`: `8281/8372/8696/8696`
  - `paint_time_us`: `3214/3315/3513/3513`

Counter evidence:
- For all 14 runs (off + on):
  - `top_paint_cache_hit_test_only_replay_allowed = 0`
  - `top_paint_cache_hit_test_only_replay_rejected_key_mismatch = 0`

Worst bundles:
- Hit-test off worst:
  - `target/fret-diag-codex-paint-hit-test-counter-off-v3/1770367752601-ui-gallery-hit-test-move-sweep-steady/bundle.json`
- Hit-test on worst:
  - `target/fret-diag-codex-paint-hit-test-counter-on-v3/1770367829971-ui-gallery-hit-test-move-sweep-steady/bundle.json`
- Resize off worst:
  - `target/fret-diag-codex-paint-hit-test-counter-resize-off-v2/1770367861503-ui-gallery-window-resize-stress-steady/bundle.json`
- Resize on worst:
  - `target/fret-diag-codex-paint-hit-test-counter-resize-on-v2/1770367893335-ui-gallery-window-resize-stress-steady/bundle.json`

Interpretation:
- The new counters prove these two current gallery probes do **not** exercise the hit-test-only replay gate path.
- Therefore, observed on/off timing deltas here are not causal evidence for the gate itself.
- Keep `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` experiment-only until we add a dedicated script that
  deterministically drives `HitTestOnly` invalidation on cache-eligible nodes.

## 2026-02-06 18:09:00 (commit `3cd778cce`)

Change:
- Ensure the new hit-test-only paint-cache counters are present in all `diag perf --json` shapes:
  - single-run row output (`--repeat 1`)
  - multi-run summary stats (`--repeat > 1`)
- Rationale: previous wiring covered the per-run list path but missed some JSON surfaces used by quick triage scripts.

Validation:
- `cargo fmt`
- `cargo check -q -p fretboard`

Probe A (single-run JSON shape):
- Script: `tools/diag-scripts/ui-gallery-hit-test-drag-sweep-steady.json`
- Command:
```bash
target/release/fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-drag-sweep-steady.json \
  --dir target/fret-diag-codex-hit-test-counter-scan/ui-gallery-hit-test-drag-sweep-steady-v3 \
  --timeout-ms 180000 \
  --repeat 1 --warmup-frames 1 --sort time --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1 \
  --launch -- target/release/fret-ui-gallery
```
- Result: output row now includes
  - `top_paint_cache_hit_test_only_replay_allowed`
  - `top_paint_cache_hit_test_only_replay_rejected_key_mismatch`
  (both `0` in this probe)

Probe B (multi-run summary JSON shape):
- Script: `tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json`
- Command:
```bash
target/release/fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json \
  --dir target/fret-diag-codex-hit-test-counter-scan/ui-gallery-hit-test-move-sweep-v4 \
  --timeout-ms 240000 \
  --repeat 3 --warmup-frames 3 --sort time --json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1 \
  --launch -- target/release/fret-ui-gallery
```
- Result: output `stats` now includes
  - `top_paint_cache_hit_test_only_replay_allowed` summary (`min/p50/p95/max`)
  - `top_paint_cache_hit_test_only_replay_rejected_key_mismatch` summary (`min/p50/p95/max`)
  (all `0` in this probe)

Notes:
- These probes still do not exercise the gate path itself (counters remain zero),
  but JSON surface completeness is now fixed for downstream tooling.

## 2026-02-06 18:30:00 (working tree)

Change:
- Added a dedicated probe page in UI Gallery:
  - `hit_test_only_paint_cache_probe`
  - pointer-move hook now calls `host.invalidate(Invalidation::HitTestOnly)` on the probe region.
- Added focused script:
  - `tools/diag-scripts/ui-gallery-hit-test-only-paint-cache-probe-sweep.json`
- Goal: produce deterministic `HitTestOnly` invalidation while keeping layout stable, then verify whether the
  `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` gate is actually exercised.

Validation:
- `cargo fmt`
- `cargo check -q -p fret-ui-gallery`

A/B probe command (repeat 5):
```bash
target/release/fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-only-paint-cache-probe-sweep.json \
  --dir target/fret-diag-codex-hit-test-only-probe-off-v4 \
  --timeout-ms 240000 --repeat 5 --warmup-frames 5 --sort time --json \
  --env FRET_UI_GALLERY_START_PAGE=hit_test_only_paint_cache_probe \
  --env FRET_UI_GALLERY_VIEW_CACHE=0 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=0 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery

target/release/fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-only-paint-cache-probe-sweep.json \
  --dir target/fret-diag-codex-hit-test-only-probe-on-v4 \
  --timeout-ms 240000 --repeat 5 --warmup-frames 5 --sort time --json \
  --env FRET_UI_GALLERY_START_PAGE=hit_test_only_paint_cache_probe \
  --env FRET_UI_GALLERY_VIEW_CACHE=0 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=0 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1 \
  --launch -- target/release/fret-ui-gallery
```

Perf summary (from `diag perf` JSON output):
- Gate off (`target/fret-diag-codex-hit-test-only-probe-off-v4`):
  - `total_time_us`: `1332 / 1386 / 1400 / 1400` (min / p50 / p95 / max)
  - `top_layout_time_us`: `1262 / 1313 / 1325 / 1325`
- Gate on (`target/fret-diag-codex-hit-test-only-probe-on-v4`):
  - `total_time_us`: `1348 / 1384 / 1419 / 1419`
  - `top_layout_time_us`: `1271 / 1310 / 1339 / 1339`

Counter evidence:
- `diag perf` top-row fields still report
  - `top_paint_cache_hit_test_only_replay_allowed = 0`
  - `top_paint_cache_hit_test_only_replay_rejected_key_mismatch = 0`
- Bundle-level max check (per run) shows the gate is actually hit when enabled:
```bash
for dir in \
  target/fret-diag-codex-hit-test-only-probe-off-v4 \
  target/fret-diag-codex-hit-test-only-probe-on-v4; do
  for b in $(find "$dir" -name bundle.json | sort); do
    jq '[.windows[0].snapshots[].debug.stats.paint_cache_hit_test_only_replay_allowed] | max' "$b"
  done
done
```
- Result:
  - gate off: `[0, 0, 0, 0, 0]`
  - gate on: `[12, 17, 17, 17, 17]`
- Also observed in every run:
  - `hit_test_only` invalidation walks: `191`
  - `paint_cache_hits` max: `50`
  - `paint_cache_hit_test_only_replay_rejected_key_mismatch` max: `0`

Interpretation:
- The new probe now provides direct evidence that `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1` opens replay attempts
  on real runs.
- Current latency impact in this micro-probe is neutral/mixed (p50 nearly unchanged; p95 slightly worse), so this
  is correctness/path-validation evidence, not a speedup claim.
- Follow-up: improve `diag perf --json` to expose per-run counter maxima directly (not only the selected `top_*` row)
  to avoid false negatives when validating gate-path counters.

## 2026-02-06 19:28:00 (commit `4c88f6696`)

Change:
- Extend `diag perf --json` to export per-run maxima for hit-test-only replay gate counters:
  - `run_paint_cache_hit_test_only_replay_allowed_max`
  - `run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max`
- Keep existing `top_*` fields unchanged for compatibility with existing triage tooling.

Validation:
- `cargo fmt`
- `cargo check -q -p fretboard`
- `target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-only-paint-cache-probe-sweep.json --dir target/fret-diag-codex-hit-test-only-probe-json-surface-v6c-r2-debug --repeat 2 --warmup-frames 1 --sort time --json --env FRET_UI_GALLERY_START_PAGE=hit_test_only_paint_cache_probe --env FRET_UI_GALLERY_VIEW_CACHE=0 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=0 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1 --launch -- target/release/fret-ui-gallery`

Results:
- Run-level evidence (`rows[0].runs`):
  - run 0: `top_paint_cache_hit_test_only_replay_allowed=0`, `run_paint_cache_hit_test_only_replay_allowed_max=17`, `run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max=0`
  - run 1: `top_paint_cache_hit_test_only_replay_allowed=0`, `run_paint_cache_hit_test_only_replay_allowed_max=17`, `run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max=0`
- Summary evidence (`rows[0].stats`):
  - `run_paint_cache_hit_test_only_replay_allowed_max`: `min/p50/p95/max = 17/17/17/17`
  - `run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max`: `0/0/0/0`
  - `top_paint_cache_hit_test_only_replay_allowed`: `0/0/0/0`

Evidence files:
- Perf run bundles: `target/fret-diag-codex-hit-test-only-probe-json-surface-v6c-r2-debug/*/bundle.json`
- Captured perf output (clean JSON): `target/fret-diag-codex-summaries/hit-test-only-probe-v6c-r2-debug-perf.clean.json`

Interpretation:
- `top_*` remains tied to the selected top snapshot (time-sorted), so it can legitimately stay `0`.
- New `run_*_max` fields provide the missing counter surface and prevent false negatives in gate-path validation.

## 2026-02-06 19:56:00 (commit `f4a6f422b`)

Change:
- Wire hit-test-only replay run-max counters into perf gating + baseline flow:
  - New perf CLI thresholds:
    - `--min-run-paint-cache-hit-test-only-replay-allowed-max`
    - `--max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max`
  - `scan_perf_threshold_failures` now evaluates:
    - lower-bound gate for `run_paint_cache_hit_test_only_replay_allowed_max`
    - upper-bound gate for `run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max`
  - `--perf-baseline-out` now emits thresholds + measured max for the two run-max counters.

Validation:
- `cargo fmt`
- `cargo check -q -p fretboard`
- `cargo nextest run -p fretboard perf_threshold_scan`
- `cargo nextest run -p fretboard perf_baseline_parse_reads_script_thresholds`

Probe A (threshold gate wired):
```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-only-paint-cache-probe-sweep.json \
  --dir target/fret-diag-codex-hit-test-only-probe-threshold-v1-r1-debug \
  --repeat 1 --warmup-frames 1 --sort time --json \
  --min-run-paint-cache-hit-test-only-replay-allowed-max 10 \
  --max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max 0 \
  --env FRET_UI_GALLERY_START_PAGE=hit_test_only_paint_cache_probe \
  --env FRET_UI_GALLERY_VIEW_CACHE=0 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=0 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1 \
  --launch -- target/release/fret-ui-gallery
```

Result highlights:
- JSON row fields:
  - `run_paint_cache_hit_test_only_replay_allowed_max = 17`
  - `run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max = 0`
- `check.perf_thresholds.json`:
  - `rows[0].thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max = 10`
  - `rows[0].thresholds.max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max = 0`
  - `failures = 0`

Probe B (baseline export wired):
```bash
target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-only-paint-cache-probe-sweep.json \
  --dir target/fret-diag-codex-hit-test-only-probe-baseline-v1-r1-debug \
  --repeat 1 --warmup-frames 1 --sort time --json \
  --perf-baseline-out target/fret-diag-codex-summaries/hit-test-only-probe-threshold-v1-baseline.json \
  --perf-baseline-headroom-pct 20 \
  --env FRET_UI_GALLERY_START_PAGE=hit_test_only_paint_cache_probe \
  --env FRET_UI_GALLERY_VIEW_CACHE=0 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=0 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1 \
  --launch -- target/release/fret-ui-gallery
```

Baseline output highlights:
- `measured_max.run_paint_cache_hit_test_only_replay_allowed_max = 17`
- `measured_max.run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max = 0`
- `thresholds.min_run_paint_cache_hit_test_only_replay_allowed_max = 13`
  - derived via floor policy at `headroom_pct=20` (17 → 13)
- `thresholds.max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max = 0`

Evidence files:
- Threshold-gate run output: `target/fret-diag-codex-summaries/hit-test-only-probe-threshold-v1-r1-debug-perf.json`
- Threshold gate report: `target/fret-diag-codex-hit-test-only-probe-threshold-v1-r1-debug/check.perf_thresholds.json`
- Baseline output: `target/fret-diag-codex-summaries/hit-test-only-probe-threshold-v1-baseline.json`

Interpretation:
- The run-max counters are now first-class perf-gate signals (baseline + CLI + failure scan).
- This removes the remaining manual `bundle.json` max extraction step for gate-path regressions.

## 2026-02-06 20:12:00 (commit `f4a6f422b`)

Change:
- Refresh `ui-gallery-steady` baseline to include the latest perf-threshold schema with run-max gate fields:
  - baseline file: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v14.json`
  - includes threshold keys:
    - `min_run_paint_cache_hit_test_only_replay_allowed_max`
    - `max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max`

Baseline command (final v14):
```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v14h20c \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --top 5 --json \
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v14.json \
  --perf-baseline-headroom-pct 20 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Validation command:
```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v14-validate2 \
  --timeout-ms 300000 \
  --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 3 --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v14.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results:
- Gate status:
  - `check.perf_thresholds.json` failures: `0` (validation passes).
- Baseline v14 worst overall (generation run):
  - script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
  - `top_total_time_us=22645`
  - bundle: `target/fret-diag-codex-perf-v14h20c/1770379813412-ui-gallery-window-resize-stress-steady/bundle.json`
- Validation worst overall:
  - script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
  - `top_total_time_us=15856`
  - bundle: `target/fret-diag-codex-perf-v14-validate2/1770379937450-ui-gallery-window-resize-stress-steady/bundle.json`
- Drift vs v13 baseline (`docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v13.json`):
  - `window-resize-stress-steady` measured max `top_total_time_us`: `15532 -> 22645`.

Run-max gate fields in v14 baseline:
- Present in `thresholds` and `measured_max` for every row.
- Current `ui-gallery-steady` run keeps both values at `0` (expected because this suite does not enable
  `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` nor target the dedicated probe page).

Evidence files:
- Baseline generation JSON: `target/fret-diag-codex-summaries/ui-gallery-steady.macos-m4.v14.h20c.gen.perf.clean.json`
- Baseline validation JSON: `target/fret-diag-codex-summaries/ui-gallery-steady.macos-m4.v14.validate2.perf.clean.json`
- Threshold report: `target/fret-diag-codex-perf-v14-validate2/check.perf_thresholds.json`

Interpretation:
- Baseline schema migration is complete and validated (new run-max gate fields are now part of the canonical baseline).
- The resize script remains the dominant noise source; one high outlier in the baseline generation run significantly
  raised `max_top_total_us` for that script. Follow-up should consider robust baseline generation
  (e.g., percentile-capped thresholding for known noisy scripts) to avoid over-loose gates.

## 2026-02-06 21:05:00 (commit: feat(diag) anti-noise seeds for steady baseline thresholds)

Change:
- `diag perf --perf-baseline-out` now records anti-noise seed metadata per row:
  - `measured_p95`
  - `threshold_seed`
  - `threshold_seed_source`
- Added script-specific threshold-seed policy:
  - `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
    uses p95 seed for `top_total_time_us`, `top_layout_time_us`, `top_layout_engine_solve_time_us`.
  - other scripts/metrics keep max-seeded thresholds.
- p95 seed computation for baseline generation uses linear interpolation over run samples so repeat=7
  does not collapse to max-only seeding.

Validation:
- `cargo fmt`
- `cargo check -q -p fretboard`
- `cargo nextest run -p fretboard baseline_threshold_seed_policy_for_resize_script perf_percentile_linear_interpolated_reduces_small_sample_tail_noise perf_threshold_scan`

Baseline command (v15):
```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v15h20p95i \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --top 5 --json \
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json \
  --perf-baseline-headroom-pct 20 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Validation command:
```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v15-validate-p95i \
  --timeout-ms 300000 \
  --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 3 --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results:
- Gate status:
  - `target/fret-diag-codex-perf-v15-validate-p95i/check.perf_thresholds.json`: `failures = 0`.
- Baseline v15 resize row (`tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`):
  - `measured_max.top_total_time_us = 16566`
  - `measured_p95.top_total_time_us = 16379`
  - `threshold_seed_source.top_total_time_us = "p95"`
  - `thresholds.max_top_total_us = 19655` (20% headroom over p95 seed)
- Drift vs v14 baseline (`docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v14.json`):
  - resize measured max top-total: `22645 -> 16566` (`-26.84%`)
  - resize threshold max-top-total: `27174 -> 19655` (`-27.67%`)
- Validation run worst overall:
  - script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
  - `top_total_time_us = 15893`
  - bundle: `target/fret-diag-codex-perf-v15-validate-p95i/1770382935955-ui-gallery-window-resize-stress-steady/bundle.json`

Interpretation:
- Baseline rows now expose enough metadata to audit threshold derivation without reverse-engineering scripts.
- Resize steady thresholds are no longer tied to raw max-only seeds; this tightens gates against single-run
  outliers while keeping repeat=3 validation stable on the current machine profile.
- Follow-up: if suite noise rises again, tune seed policy per script (e.g., p90/p95 or higher repeat for
  specific workloads) and record the policy update in this log.

## 2026-02-06 21:35:00 (working tree)

Change:
- Added configurable baseline seed policy for `diag perf --perf-baseline-out`:
  - new CLI flag: `--perf-baseline-seed <script@metric=max|p90|p95>` (repeatable)
  - default policy remains max-seeded globally, with built-in resize override:
    - `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
    - metrics `top_total/layout/solve` default to `p95`
- Baseline payload now records policy header:
  - `threshold_seed_policy.default_seed`
  - `threshold_seed_policy.rules[]`
- Baseline row now records both `measured_p90` and `measured_p95` (for seed provenance and future tuning).

Validation:
- `cargo fmt`
- `cargo check -q -p fretboard`
- `cargo nextest run -p fretboard baseline_threshold_seed_policy_for_resize_script baseline_threshold_seed_policy_can_override_with_p90 baseline_threshold_seed_policy_rejects_bad_spec perf_percentile_linear_interpolated_reduces_small_sample_tail_noise perf_threshold_scan`

Baseline command (v15 refresh with policy header):
```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v15h20seed \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --top 5 --json \
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json \
  --perf-baseline-headroom-pct 20 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Validation command:
```bash
target/debug/fretboard diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v15-validate-seed \
  --timeout-ms 300000 \
  --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 3 --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results:
- Gate status:
  - `target/fret-diag-codex-perf-v15-validate-seed/check.perf_thresholds.json`: `failures = 0`.
- Baseline header includes policy metadata:
  - `threshold_seed_policy.default_seed = "max"`
  - resize steady `top_total/layout/solve` rules seeded by `p95`.
- Baseline v15 resize row (`tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`):
  - `measured_max.top_total_time_us = 15763`
  - `measured_p90.top_total_time_us = 15683`
  - `measured_p95.top_total_time_us = 15723`
  - `threshold_seed_source.top_total_time_us = "p95"`
  - `thresholds.max_top_total_us = 18868`
- Drift vs v14 baseline (`docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v14.json`):
  - resize measured max top-total: `22645 -> 15763` (`-30.39%`)
  - resize threshold max-top-total: `27174 -> 18868` (`-30.56%`)
- Validation run tightest total-time margin:
  - script: `tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json`
  - observed `2170` vs threshold `2552` (margin `382` us)
- CLI override smoke check (`--perf-baseline-seed`):
  - command: `target/debug/fretboard diag perf tools/diag-scripts/ui-gallery-overlay-torture-steady.json --repeat 1 --perf-baseline-out target/fret-diag-codex-summaries/perf-seed-flag-smoke-baseline.json --perf-baseline-seed tools/diag-scripts/ui-gallery-overlay-torture-steady.json@top_total_time_us=p90 ...`
  - baseline header adds a `source="cli"` rule for the override.
  - row seed source reports `threshold_seed_source.top_total_time_us = "p90"`.

Interpretation:
- Seed policy is now explicit and versioned in baseline JSON, so threshold provenance is auditable.
- With `--perf-baseline-seed`, we can tighten or relax noisy scripts without code changes and still keep a
  reproducible evidence trail in the baseline artifact.

## 2026-02-06 22:10:00 (commit: feat(diag) add suite-scoped baseline seed templates)

Change:
- Extended baseline seed scope from per-script to template scopes:
  - `ui-gallery@...`
  - `ui-gallery-steady@...`
  - `this-suite@...`
  - `suite:<name>@...`
  - `*@...`
- Kept rule precedence “last match wins” and preserved default resize `p95` policy.
- Added a policy template document for repeatable usage:
  - `docs/workstreams/perf-baselines/seed-policy-template.md`

Validation:
- `cargo fmt`
- `cargo check -q -p fretboard`
- `cargo nextest run -p fretboard baseline_threshold_seed_policy_for_resize_script baseline_threshold_seed_policy_can_override_with_p90 baseline_threshold_seed_policy_rejects_bad_spec baseline_threshold_seed_policy_supports_suite_scope baseline_threshold_seed_policy_supports_this_suite_scope baseline_threshold_seed_policy_rejects_this_suite_without_named_suite perf_percentile_linear_interpolated_reduces_small_sample_tail_noise perf_threshold_scan`

Result highlights:
- New suite/template scopes are verified by unit tests in `apps/fretboard/src/diag/mod.rs`.
- No baseline numbers were changed in this step; this is a tooling-surface extension.

Interpretation:
- Baseline seed tuning is now script-group aware, so tightening policy can happen by suite-level commands without
  introducing one-off code branches.

## 2026-02-06 22:50:00 (working tree)

Change:
- Added JSON preset support for perf baseline seed policy in `diag perf`:
  - new CLI flag: `--perf-baseline-seed-preset <path>` (repeatable)
  - preset schema validation: `schema_version=1`, `kind=perf_baseline_seed_policy`
  - supported fields: optional `default_seed`, required `rules[]` (`scope`, `metric`, `seed`)
- Policy merge precedence is now explicit:
  - built-in defaults -> preset rules (CLI order) -> explicit `--perf-baseline-seed` rules
- Added versioned preset artifact:
  - `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json`
- Updated docs/help surfaces:
  - `apps/fretboard/src/cli.rs` usage + example
  - `docs/workstreams/perf-baselines/seed-policy-template.md`

Validation:
- `cargo fmt`
- `cargo check -q -p fretboard`
- `cargo nextest run -p fretboard baseline_threshold_seed_policy_for_resize_script baseline_threshold_seed_policy_can_override_with_p90 baseline_threshold_seed_policy_rejects_bad_spec baseline_threshold_seed_policy_supports_suite_scope baseline_threshold_seed_policy_supports_this_suite_scope baseline_threshold_seed_policy_rejects_this_suite_without_named_suite baseline_threshold_seed_policy_supports_preset_file baseline_threshold_seed_policy_rejects_bad_preset_schema baseline_threshold_seed_policy_cli_overrides_preset_rule baseline_threshold_seed_policy_preset_can_override_default_seed perf_percentile_linear_interpolated_reduces_small_sample_tail_noise perf_threshold_scan`

Result highlights:
- Nextest summary: `14 passed, 0 failed` for the targeted policy/perf-threshold test set.
- New tests cover:
  - preset parse success
  - preset schema validation failure
  - CLI rule overriding preset rule
  - preset `default_seed` override while preserving built-in resize `p95` default rule

Interpretation:
- Seed policy is now file-versionable and replayable without code edits.
- Teams can keep policy profiles in-repo, then layer temporary CLI overrides for experiments while preserving reproducibility.

## 2026-02-06 23:20:00 (working tree)

Change:
- Ran a first preset-driven steady baseline trial (`v16`) using:
  - `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json`
- Goal: quantify how much threshold tightening we gain over `v15`, and measure gate stability (`false fail` risk)
  under the same validation profile.

Commands:
```bash
cargo run -q -p fretboard -- diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v16-preset \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --top 5 --json \
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v16.json \
  --perf-baseline-headroom-pct 20 \
  --perf-baseline-seed-preset docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery

cargo run -q -p fretboard -- diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v16-validate \
  --timeout-ms 300000 \
  --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 3 --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v16.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Additional stability sampling:
- Repeated two more validation runs with the same settings:
  - `target/fret-diag-codex-perf-v16-validate2`
  - `target/fret-diag-codex-perf-v16-validate3`
- Rechecked `v15` once for control:
  - `target/fret-diag-codex-perf-v15-validate-recheck`

Results:
- `v16` validation gate status:
  - `target/fret-diag-codex-perf-v16-validate/check.perf_thresholds.json`: `failures = 1`
  - `target/fret-diag-codex-perf-v16-validate2/check.perf_thresholds.json`: `failures = 1`
  - `target/fret-diag-codex-perf-v16-validate3/check.perf_thresholds.json`: `failures = 1`
- Stable failing metric across all 3 validation runs:
  - script: `tools/diag-scripts/ui-gallery-overlay-torture-steady.json`
  - metric: `top_total_time_us`
  - threshold (`v16`): `6664`
  - observed actuals: `7351`, `7403`, `7188`
  - over-threshold margins: `+687`, `+739`, `+524` us
- `v15` control recheck:
  - `target/fret-diag-codex-perf-v15-validate-recheck/check.perf_thresholds.json`: `failures = 0`

v15 -> v16 threshold-delta summary (`ui-gallery-steady`, 11 scripts x 8 gated metrics = 88 checks):
- tightened: `20`
- unchanged: `43`
- loosened: `25`
- aggregate threshold sums:
  - `max_top_total_us`: `85809 -> 82475` (`-3.89%`)
  - `max_top_layout_us`: `59762 -> 58147` (`-2.70%`)
  - `max_top_solve_us`: `4229 -> 4279` (`+1.18%`)

Key root cause candidate:
- Overlay steady `top_total` got over-tightened by p90 seeding:
  - `v15 threshold`: `9066` (max-seeded)
  - `v16 threshold`: `6664` (p90-seeded)
  - delta: `-2402` (`-26.5%`)
- This exceeds observed run-to-run jitter envelope on current machine profile.

Interpretation:
- Preset strategy works technically and provides measurable tightening.
- Current `ui-gallery-steady.v1` policy is too aggressive for overlay `top_total_time_us`; it introduces consistent
  false gate failures under repeat=3 validation.
- Recommended next action: publish `ui-gallery-steady.v2.json` with overlay `top_total_time_us` switched to `p95`
  (or keep overlay on `max`) while retaining p90 for scripts that remain stable.

## 2026-02-06 23:55:00 (working tree)

Change:
- Published preset v2 to address the known overlay false-fail hotspot from v1:
  - `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v2.json`
  - key adjustment: override `tools/diag-scripts/ui-gallery-overlay-torture-steady.json@top_total_time_us` from `p90` to `p95`.
- Generated new baseline with preset v2:
  - `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v17.json`

Baseline command (v17):
```bash
cargo run -q -p fretboard -- diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v17-preset-v2 \
  --timeout-ms 300000 \
  --reuse-launch --repeat 7 --warmup-frames 5 --sort time --top 5 --json \
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v17.json \
  --perf-baseline-headroom-pct 20 \
  --perf-baseline-seed-preset docs/workstreams/perf-baselines/policies/ui-gallery-steady.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Validation sample (3 runs):
```bash
cargo run -q -p fretboard -- diag perf ui-gallery-steady \
  --dir target/fret-diag-codex-perf-v17-validate{1|2|3} \
  --timeout-ms 300000 \
  --reuse-launch --repeat 3 --warmup-frames 5 --sort time --top 3 --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v17.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --launch -- target/release/fret-ui-gallery
```

Results:
- Gate status:
  - `target/fret-diag-codex-perf-v17-validate1/check.perf_thresholds.json`: `failures = 0`
  - `target/fret-diag-codex-perf-v17-validate2/check.perf_thresholds.json`: `failures = 0`
  - `target/fret-diag-codex-perf-v17-validate3/check.perf_thresholds.json`: `failures = 0`
- Overlay false-fail fixed vs v16:
  - `ui-gallery-overlay-torture-steady` `max_top_total_us`: `6664 (v16) -> 7868 (v17)`
  - v16 had repeated failures at this point; v17 passed all sampled validations.
- Threshold delta overview (v15 -> v17, 88 checks):
  - tightened: `22`, unchanged: `45`, loosened: `21`
- Aggregate threshold sums:
  - `max_top_total_us`: `85809 -> 88118` (`+2.69%`)
  - `max_top_layout_us`: `59762 -> 61061` (`+2.17%`)
  - `max_top_solve_us`: `4229 -> 6105` (`+44.36%`)

Interpretation:
- Preset v2 resolves the known overlay false fail and restores validation stability.
- However, this particular v17 generation run carries a resize-heavy outlier (`window-resize-stress-steady`),
  which loosens global guard strength despite stable gate pass.
- Follow-up should add robustness against resize-run outliers (multi-pass baseline selection / outlier rejection)
  before promoting v17 as the long-term canonical baseline.

## 2026-02-07 00:35:00 (working tree)

Change:
- Added baseline candidate-selection automation script:
  - `tools/perf/diag_perf_baseline_select.sh`
- Script behavior:
  - generates multiple baseline candidates (`diag perf --perf-baseline-out`)
  - validates each candidate multiple times (`diag perf --perf-baseline`)
  - selects winner by: `fail_total` -> resize `p90(top_total)` -> `sum(max_top_total_us)`
  - writes machine-readable evidence:
    - candidate list: `<work-dir>/candidate-results.json`
    - final summary: `<work-dir>/selection-summary.json`

Selection run (v18):
```bash
tools/perf/diag_perf_baseline_select.sh \
  --suite ui-gallery-steady \
  --baseline-out docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v18.json \
  --preset docs/workstreams/perf-baselines/policies/ui-gallery-steady.v2.json \
  --candidates 2 \
  --validate-runs 3 \
  --repeat 7 \
  --warmup-frames 5 \
  --headroom-pct 20 \
  --work-dir target/fret-diag-codex-perf-v18-select2 \
  --launch-bin target/release/fret-ui-gallery
```

Selection result:
- Summary: `target/fret-diag-codex-perf-v18-select2/selection-summary.json`
- Candidate-1:
  - `fail_total = 0`
  - `resize_p90_top_total_us = 16110`
  - `threshold_sum_max_top_total_us = 84611`
- Candidate-2:
  - `fail_total = 0`
  - `resize_p90_top_total_us = 16012`
  - `threshold_sum_max_top_total_us = 83564`
- Winner: `candidate-2` copied to
  - `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v18.json`

Validation stability:
- Both candidates passed `3/3` validation runs with `failures=0`.
- This closes the earlier instability issue where single-run baseline promotion could keep a resize-heavy outlier.

Threshold impact:
- Aggregate sums (`ui-gallery-steady`):
  - `max_top_total_us`: `v15=85809`, `v17=88118`, `v18=83564`
  - `max_top_layout_us`: `v15=59762`, `v17=61061`, `v18=57829`
  - `max_top_solve_us`: `v15=4229`, `v17=6105`, `v18=4348`
- Delta structure:
  - `v15 -> v18`: tightened `28`, unchanged `47`, loosened `13` (88 checks)
  - `v17 -> v18`: tightened `28`, unchanged `46`, loosened `14` (88 checks)

Interpretation:
- Candidate selection recovers stability and avoids promoting resize-outlier baselines.
- v18 is both stable (`failures=0` in sampled validations) and tighter than v15/v17 at the suite aggregate level.
- This workflow is a better default for baseline refreshes than single-pass generation.

## 2026-02-07 00:46:00 (working tree)

Change:
- Added a dedicated retained-virtual-list boundary-crossing probe script:
  - `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json`
- Calibrated how this probe should be executed for meaningful window-shift diagnostics.

Initial run (insufficient env; counters stayed zero):
```bash
cargo run -q -p fretboard -- diag run tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json \
  --dir target/fret-diag-codex-window-boundary-crossing-steady-sample-r1 \
  --timeout-ms 300000 \
  --check-vlist-window-shifts-explainable \
  --check-vlist-window-shifts-have-prepaint-actions \
  --check-vlist-window-shifts-non-retained-max 9999 \
  --check-vlist-window-shifts-prefetch-max 9999 \
  --check-vlist-window-shifts-escape-max 9999 \
  --launch -- target/release/fret-ui-gallery
```

Observation from `r1`/`r2`:
- `virtual_list_window_shifts_total = 0`
- `virtual_list_visible_range_refreshes = 0`
- Root cause: view-cache env was not enabled, so this probe did not exercise the intended retained-window path.

Calibrated sampling command (meaningful path):
```bash
cargo run -q -p fretboard -- diag run tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json \
  --dir target/fret-diag-codex-window-boundary-crossing-steady-sample-r3 \
  --timeout-ms 300000 \
  --check-vlist-window-shifts-explainable \
  --check-vlist-window-shifts-have-prepaint-actions \
  --check-vlist-window-shifts-non-retained-max 9999 \
  --check-vlist-window-shifts-prefetch-max 9999 \
  --check-vlist-window-shifts-escape-max 9999 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_UI_GALLERY_VLIST_MINIMAL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --launch -- target/release/fret-ui-gallery
```

Sampled runs:
- `target/fret-diag-codex-window-boundary-crossing-steady-sample-r3`: `total_shifts=1`, `prefetch=1`, `escape=0`, `non_retained=0`
- `target/fret-diag-codex-window-boundary-crossing-steady-sample-r4`: `total_shifts=1`, `prefetch=1`, `escape=0`, `non_retained=0`
- `target/fret-diag-codex-window-boundary-crossing-steady-sample-r5`: `total_shifts=1`, `prefetch=1`, `escape=0`, `non_retained=0`
- `target/fret-diag-codex-window-boundary-crossing-steady-sample-r6`: `total_shifts=1`, `prefetch=1`, `escape=0`, `non_retained=0`

Interpretation:
- The script consistently exercises one retained prefetch window shift when launched with view-cache env enabled.
- A practical first gate target is:
  - `prefetch <= 3`
  - `escape <= 0`
  - `non_retained <= 0`
- Next step: promote this command profile into the M4 acceptance recipe and require repeated `failures=0` validation runs.


## 2026-02-07 00:56:00 (working tree)

Change:
- Promoted the boundary-crossing probe into a reusable gate recipe:
  - `tools/perf/diag_vlist_boundary_gate.sh`
- Gate defaults are now explicit and reproducible:
  - `prefetch_max=3`, `escape_max=0`, `non_retained_max=0`, `runs=3`

Gate command:
```bash
tools/perf/diag_vlist_boundary_gate.sh \
  --runs 3 \
  --out-dir target/fret-diag-codex-vlist-boundary-gate-r1 \
  --launch-bin target/release/fret-ui-gallery
```

Result summary:
- Summary file: `target/fret-diag-codex-vlist-boundary-gate-r1/summary.json`
- Gate status: `pass=true`, `run_failures=0`
- Per-run metrics:
  - run-1: `total_shifts=1`, `prefetch=1`, `escape=0`, `non_retained=0`
  - run-2: `total_shifts=1`, `prefetch=1`, `escape=0`, `non_retained=0`
  - run-3: `total_shifts=1`, `prefetch=1`, `escape=0`, `non_retained=0`

Interpretation:
- M4.2 boundary-crossing gate promotion is complete for the retained VirtualList path.
- Next focus stays on M4.3: reduce rerender-triggering shifts on non-retained fallback and tighten cache-key stability.


## 2026-02-07 01:04:00 (working tree)

Change:
- Tuned VirtualList prepaint window-shift policy for non-retained + view-cache path:
  - file: `crates/fret-ui/src/tree/prepaint.rs`
  - behavior: suppress preemptive/forced prefetch rerender for non-retained lists while
    the current visible range is still covered by the rendered overscan envelope.
- Intent:
  - keep retained-host prefetch behavior unchanged,
  - reduce avoidable cache-root rerender churn on non-retained fallback.

Baseline (before change, non-retained fallback profile):
```bash
cargo run -q -p fretboard -- diag run tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json \
  --dir target/fret-diag-codex-vlist-boundary-nonretained-before-r1 \
  --timeout-ms 300000 \
  --check-vlist-window-shifts-explainable \
  --check-vlist-window-shifts-have-prepaint-actions \
  --check-vlist-window-shifts-non-retained-max 9999 \
  --check-vlist-window-shifts-prefetch-max 9999 \
  --check-vlist-window-shifts-escape-max 9999 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_UI_GALLERY_VLIST_MINIMAL=1 \
  --env FRET_UI_GALLERY_VLIST_RETAINED=0 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --launch -- target/release/fret-ui-gallery
```
(Repeated for `r1..r3`)

Validation after change (non-retained fallback profile, same command shape):
- `target/fret-diag-codex-vlist-boundary-nonretained-after3-r1`: `shifts=0`, `prefetch=0`, `escape=0`, `non_retained=0`
- `target/fret-diag-codex-vlist-boundary-nonretained-after3-r2`: `shifts=0`, `prefetch=0`, `escape=0`, `non_retained=0`
- `target/fret-diag-codex-vlist-boundary-nonretained-after3-r3`: `shifts=0`, `prefetch=0`, `escape=0`, `non_retained=0`

Delta (3-run aggregate):
- `prefetch`: `3 -> 0` (`-100%`)
- `non_retained`: `3 -> 0` (`-100%`)
- `escape`: `0 -> 0` (unchanged)

Retained-path regression check:
```bash
tools/perf/diag_vlist_boundary_gate.sh \
  --runs 3 \
  --out-dir target/fret-diag-codex-vlist-boundary-gate-r2 \
  --launch-bin target/release/fret-ui-gallery
```
- Summary: `target/fret-diag-codex-vlist-boundary-gate-r2/summary.json`
- Result: `pass=true`, with retained profile still at `prefetch=1`, `escape=0`, `non_retained=0` per run.

Interpretation:
- M4.3 first optimization slice lands: non-retained fallback no longer pays avoidable prefetch-triggered rerender churn on this steady crossing probe.
- Next M4.3 slice should audit cache-key instability and add a bounded non-retained escape gate so regressions are caught early.


## 2026-02-07 01:16:00 (working tree)

Change:
- Extended `tools/perf/diag_vlist_boundary_gate.sh` to cover both retained and non-retained profiles.
- Added new gate options:
  - `--retained <0|1>`
  - `--max-cache-key-mismatch <n>`
  - `--max-needs-rerender <n>`
- Gate now records per-run maxima from `bundle.json` snapshots:
  - `view_cache_roots_cache_key_mismatch`
  - `view_cache_roots_needs_rerender`

Retained profile validation:
```bash
tools/perf/diag_vlist_boundary_gate.sh \
  --runs 3 \
  --out-dir target/fret-diag-codex-vlist-boundary-gate-r3 \
  --launch-bin target/release/fret-ui-gallery
```
- Summary: `target/fret-diag-codex-vlist-boundary-gate-r3/summary.json`
- Result: `pass=true` (3/3), sample remains `prefetch=1`, `escape=0`, `non_retained=0`,
  `cache_key_mismatch_max=0`, `needs_rerender_max=0`.

Non-retained strict gate validation:
```bash
tools/perf/diag_vlist_boundary_gate.sh \
  --runs 3 \
  --retained 0 \
  --prefetch-max 0 \
  --escape-max 0 \
  --non-retained-max 0 \
  --max-cache-key-mismatch 0 \
  --max-needs-rerender 0 \
  --out-dir target/fret-diag-codex-vlist-boundary-nonretained-gate-r1 \
  --launch-bin target/release/fret-ui-gallery
```
- Summary: `target/fret-diag-codex-vlist-boundary-nonretained-gate-r1/summary.json`
- Result: `pass=true` (3/3)
- Per-run sample: `prefetch=0`, `escape=0`, `non_retained=0`,
  `cache_key_mismatch_max=0`, `needs_rerender_max=0`.

Interpretation:
- We now have a bounded non-retained fallback gate that tracks both shift behavior and cache-key/rerender hygiene.
- This closes the earlier “non-retained escape budget gate” TODO at tooling level and makes M4.3 regressions easier to catch.


## 2026-02-07 01:34:00 (working tree)

Change:
- Added a stronger non-retained boundary script:
  - `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-nonretained-stress-steady.json`
- Script intent:
  - same target surface as boundary-crossing probe,
  - larger wheel deltas (`±360`) with denser cadence to stress window-boundary behavior,
  - keep diagnostics bounded via explicit `reset_diagnostics` + `capture_bundle`.

Strict gate command (non-retained profile):
```bash
tools/perf/diag_vlist_boundary_gate.sh \
  --runs 3 \
  --script tools/diag-scripts/ui-gallery-virtual-list-window-boundary-nonretained-stress-steady.json \
  --retained 0 \
  --prefetch-max 0 \
  --escape-max 0 \
  --non-retained-max 0 \
  --max-cache-key-mismatch 0 \
  --max-needs-rerender 0 \
  --out-dir target/fret-diag-codex-vlist-boundary-nonretained-stress-gate-r1 \
  --launch-bin target/release/fret-ui-gallery
```

Results:
- Summary: `target/fret-diag-codex-vlist-boundary-nonretained-stress-gate-r1/summary.json`
- Gate status: `pass=true`, `run_failures=0` (3/3)
- Per-run sample:
  - `prefetch=0`, `escape=0`, `non_retained=0`
  - `cache_key_mismatch_max=0`, `needs_rerender_max=0`

Interpretation:
- Even under a stronger wheel stress profile, non-retained fallback keeps zero shift/rerender churn on this probe.
- Escape remained zero in this stress script; next M4.3 work should focus on an explicit out-of-band escape trigger path (or dedicated telemetry) if we want a positive escape expectation gate.
