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
