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
