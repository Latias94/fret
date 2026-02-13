# `fret-fonts`

Bundled font bytes for bootstrapping `fret` apps, primarily targeting Web/WASM where system fonts
are not available.

## Contents

- Inter (roman + italic) — OFL 1.1 (`assets/Inter-OFL.txt`)
- JetBrains Mono (roman + italic) — OFL 1.1 (`assets/JetBrainsMono-OFL.txt`)
- Fira Mono (subset) — OFL 1.1 (`assets/FiraMono-LICENSE`)
- Noto Color Emoji — Apache 2.0 (`assets/NotoEmoji-LICENSE.txt`) (optional; `emoji` feature)

The canonical API is:

- `fret_fonts::default_fonts()` — bootstrap + CJK-lite (by default) + optional emoji (if enabled).
- `fret_fonts::bootstrap_fonts()` — bootstrap fonts only (never includes emoji).
- `fret_fonts::emoji_fonts()` — emoji fonts only (requires `emoji` feature).

All are intended to be fed into `Effect::TextAddFonts`.

## Size strategy (WASM)

The default feature set uses **subset fonts** to reduce WASM payload size:

- `bootstrap-subset` (default): uses `*-subset.ttf` for Inter/JetBrains Mono.
- `cjk-lite` (default): adds a small subset of `Noto Sans CJK SC` for basic CJK coverage.
- `bootstrap-full`: uses the full font files (much larger).

Emoji:

- `emoji`: includes `assets/NotoColorEmoji.ttf` (large; intended as an explicit opt-in).

## CJK lite (WASM bootstrap)

- `cjk-lite`: includes a subset of `Noto Sans CJK SC` as
  `assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf` (generated via `fonttools`/`pyftsubset`).
  Intended to cover a practical baseline of CJK glyphs for bootstrap UI on Web/WASM without pulling
  in the full font payload.

## Recommended bundles

For a general-purpose app shell:

- Web/WASM: keep `bootstrap-subset` + `cjk-lite` on by default, and gate `emoji` behind an explicit
  feature or user setting (WASM size impact is significant).
- Native: either rely on system UI fonts (plus user-loaded fonts) or use `bootstrap-full` for a
  deterministic demo experience.

For future expansion, prefer feature-gated bundles (e.g. CJK subsets) rather than growing
`default_fonts()` unconditionally.
