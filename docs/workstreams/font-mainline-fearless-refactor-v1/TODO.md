# Font Mainline Fearless Refactor v1 — TODO

Status: Active

## Baseline already in place

- [x] Mixed-script bundled fallback conformance gate exists.
- [x] Locale-change fallback policy key gate exists.
- [x] Settings-change fallback policy key gate exists.
- [x] ADR 0257 implementation alignment records the current diagnostics evidence.
- [x] L0/L1 audits exist for the three crates on the mainline path.

## `fret-fonts`

- [x] FR-FONTS-001 Split `crates/fret-fonts/src/lib.rs` into explicit ownership modules
      (`assets`, `profiles`, `tests`).
- [x] FR-FONTS-002 Add a representative feature-matrix gate:
      `python tools/check_fret_fonts_feature_matrix.py`
      (`default`, `--no-default-features`, `--features bootstrap-full,emoji,cjk-lite`).
- [x] FR-FONTS-003 Decide whether `emoji_fonts()` and `cjk_lite_fonts()` remain public or become
      implementation helpers behind profile-first APIs.

## `fret-render-text`

- [x] FR-RENDER-TEXT-010 Extract font DB/catalog/rescan logic from `parley_shaper.rs` into a
      dedicated ownership seam.
- [x] FR-RENDER-TEXT-011 Reduce `src/lib.rs` to an explicit facade and shrink accidental `pub mod`
      surface.
- [x] FR-RENDER-TEXT-012 Split `wrapper.rs` by responsibility (`metrics`, `hit_test`, `wrapping`,
      selection/layout helpers).
- [x] FR-RENDER-TEXT-013 Add crate-local tests for fallback-policy key transitions on locale,
      injection mode, and system-font availability changes.
- [x] FR-RENDER-TEXT-014 Add a bounded catalog-enumeration regression harness or perf check.

## `fret-launch`

- [x] FR-LAUNCH-020 Keep `runner/font_catalog.rs` as the only font-environment publication facade
      and remove duplicated seeding/policy helpers from runner-specific modules where possible.
- [x] FR-LAUNCH-021 Make desktop and web startup paths go through the same narrow publication story
      (`publish_renderer_font_environment` / startup initializer), differing only in runner
      orchestration.
- [x] FR-LAUNCH-022 Re-audit whether bundled-profile seeding belongs in `fret-launch` or should move
      to `fret-runtime`.

## Cross-crate closure

- [ ] FR-CROSS-030 Keep the diagnostics 3-pack green after every ownership move:
      settings-change, locale-change, mixed-script bundled fallback.
- [ ] FR-CROSS-031 Update crate audits and ADR alignment after each landed slice so the new owner map
      stays explicit.
