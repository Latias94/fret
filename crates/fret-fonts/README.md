# `fret-fonts`

Bundled font bytes for bootstrapping `fret` apps, primarily targeting Web/WASM where system fonts
are not available.

## Contents

- Inter (roman + italic) — OFL 1.1 (`assets/Inter-OFL.txt`)
- JetBrains Mono (roman + italic) — OFL 1.1 (`assets/JetBrainsMono-OFL.txt`)
- Fira Mono (subset) — OFL 1.1 (`assets/FiraMono-LICENSE`)

The canonical API is `fret_fonts::default_fonts()`, intended to be fed into
`Effect::TextAddFonts`.

