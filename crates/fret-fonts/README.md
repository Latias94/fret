# `fret-fonts`

Bundled font bytes for bootstrapping `fret` apps, primarily targeting Web/WASM where system fonts
are not available.

## Contents

- Inter (roman + italic) — OFL 1.1 (`assets/Inter-OFL.txt`)
- JetBrains Mono (roman + italic) — OFL 1.1 (`assets/JetBrainsMono-OFL.txt`)
- Fira Mono (subset) — OFL 1.1 (`assets/FiraMono-LICENSE`)
- Noto Color Emoji — Apache 2.0 (`assets/NotoEmoji-LICENSE.txt`) (optional; `emoji` feature)

The canonical API is `fret_fonts::default_fonts()`, intended to be fed into
`Effect::TextAddFonts`.

## Size strategy (WASM)

The default feature set uses **subset fonts** to reduce WASM payload size:

- `bootstrap-subset` (default): uses `*-subset.ttf` for Inter/JetBrains Mono.
- `bootstrap-full`: uses the full font files (much larger).

Emoji:

- `emoji`: includes `assets/NotoColorEmoji.ttf` (large; intended as an explicit opt-in).
