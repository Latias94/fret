# Text Shaping Surface v1 — Milestones

This is a milestone checklist for:

- `docs/workstreams/text-shaping-surface-v1.md`

The milestones are intentionally shippable in small steps and aligned with the existing text v2
pipeline work.

## M0 — Contracts + Parley plumbing

Exit criteria:

- `fret-core` exposes:
  - `TextFontFeatureSetting { tag, value }`
  - `TextShapingStyle.features: Vec<TextFontFeatureSetting>`
- Parley mapping exists:
  - `StyleProperty::FontFeatures` is emitted when `features` is non-empty.
- Cache correctness:
  - a feature toggle changes shaping keys and invalidates prepared outputs deterministically.
- Tests:
  - canonicalization (invalid tags, duplicates, stable ordering),
  - keying correctness (feature toggle changes shaping key),
  - one behavior smoke test (feature affects shaping) OR a documented fallback plan if no stable
    fixture exists yet.

Evidence checklist:

- `cargo nextest run -p fret-render-wgpu` (or the crate that owns the text tests)
- `cargo nextest run -p fret-render`
- `cargo nextest run -p fret-ui` (sanity; should be unaffected)

## M1 — Editor-grade policy adoption (ecosystem)

Exit criteria:

- `ecosystem/fret-code-view` or `ecosystem/fret-code-editor` can define a “code font policy”:
  - disable `liga`/`calt` by default (or provide a toggle),
  - keep UI text defaults unchanged.
- Attributed spans produced for syntax highlighting do not accidentally pull paint-only changes into
  shaping keys (regression test).
- Optional: a demo page exists to visualize feature toggles on a known string (and documents which
  fonts exhibit visible differences).

Evidence checklist:

- `cargo nextest run -p fret-code-view` (if tests exist)
- `cargo nextest run -p fret-code-editor` (if tests exist)

## M2 — Settings surface (optional, not required for correctness)

Exit criteria:

- A stable configuration surface exists at the ecosystem layer (not `fret-ui` mechanism):
  - UI font features
  - code/editor font features
- Defaults are documented, and changing them bumps the correct invalidation keys (no stale caches).

Notes:

- This milestone should not block the core refactor; it is a “productization” pass.

