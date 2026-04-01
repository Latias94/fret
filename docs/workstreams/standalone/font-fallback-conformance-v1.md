# Workstream: Font Fallback Conformance v1 (Gates + Diagnostics)

Status: background conformance planning note; active execution now lives in
`docs/workstreams/font-system-fearless-refactor-v1/` (especially M4).

Use this file for historical gate rationale. For the current execution lane, see:

- `docs/workstreams/font-system-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/font-system-fearless-refactor-v1/TODO.md`
- `docs/workstreams/font-system-fearless-refactor-v1/MILESTONES.md`

This workstream defines **portable, CI-friendly conformance gates** for Fret’s text fallback behavior.

Goals:

1) Ensure fallback policy changes are **observable** in diagnostics bundles.
2) Ensure cache invalidation boundaries are **gateable** without depending on a machine’s system font set.
3) Keep scripts minimal and explainable (bundle evidence first, pixels second).

Out of scope:

- Per-script fallback customization in settings.
- Mandating specific system font family names.

Related contracts / trackers:

- ADR 0257: `docs/adr/0257-font-selection-fallback-and-variable-font-instances-v1.md`
- ADR 0258: `docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`
- ADR 0259: `docs/adr/0259-system-font-rescan-and-injected-font-retention-v1.md`
- Font system roadmap: `docs/workstreams/standalone/font-system-v1.md`

## Evidence anchor (bundles)

These gates assert against the renderer’s fallback policy snapshot in the bundle:

- `resource_caches.render_text_fallback_policy.fallback_policy_key`
- `resource_caches.render_text_fallback_policy.prefer_common_fallback`
- `resource_caches.render_text_fallback_policy.common_fallback_stack_suffix`

The key invariant is:

- **Policy changes bump `fallback_policy_key`**, and therefore participate in `TextFontStackKey` via the renderer’s
  `font_stack_key`.

## Gates (scripts + checks)

### Gate 1: Settings change bumps fallback policy key

- Script: `tools/diag-scripts/ui-gallery-text-fallback-policy-key-bumps-on-settings-change.json`
- Check flag: `--check-ui-gallery-text-fallback-policy-key-bumps-on-settings-change`

Expected:

- The script opens the UI Gallery settings sheet, switches `fonts.common_fallback_injection`
  from `platform_default` to `common_fallback`, then captures BEFORE/AFTER bundles around the
  apply step. On Windows, the script disables the gallery's curated common-fallback override so
  the BEFORE capture stays on the true `platform_default -> prefer_system_fallback` lane.
- The gate asserts:
  - `system_fonts_enabled` is `true` in both captures.
  - `common_fallback_injection` changes from `platform_default` to `common_fallback`.
  - `prefer_common_fallback` changes from `false` to `true`.
  - `common_fallback_candidates` / `common_fallback_stack_suffix` change from empty to non-empty.
  - `fallback_policy_key` differs between the two labeled captures.

Run (native):

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-text-fallback-policy-key-bumps-on-settings-change.json \
  --check-ui-gallery-text-fallback-policy-key-bumps-on-settings-change \
  --launch -- cargo run -p fret-ui-gallery --release
```

### Gate 1.5: Locale change bumps fallback policy key

- Script: `tools/diag-scripts/ui-gallery/text/ui-gallery-text-fallback-policy-key-bumps-on-locale-change.json`
- Check flag: `--check-ui-gallery-text-fallback-policy-key-bumps-on-locale-change`

Expected:

- The script starts on `text_mixed_script_fallback`, captures a BEFORE bundle, triggers
  `app.locale.switch_next` via the default `primary+alt+l` shortcut, then captures an AFTER
  bundle. This page lives on the lightweight text harness surface, so the launch command must
  include `--features gallery-web-ime-harness`. On Windows, the script opts out of the UI
  Gallery's curated common-fallback override so the run exercises the framework-owned native
  `platform_default` hybrid baseline directly.
- The gate asserts:
  - `system_fonts_enabled` is `true` in both captures.
  - `common_fallback_injection` stays `platform_default` and `prefer_common_fallback` stays
    `false` in both captures, so named-family stacks remain on the system-fallback lane.
  - `prefer_common_fallback_for_generics` stays `true` in both captures, and
    `common_fallback_candidates` stay non-empty, so generic UI text keeps the renderer-owned
    no-tofu baseline.
  - `locale_bcp47` is `en-US` in the BEFORE capture and `zh-CN` in the AFTER capture.
  - The mixed-script sample traces (`m`, `你`, `😀`, `m你😀`) appear in both captures.
  - Those sample trace locales settle to `["en-US"]` in the BEFORE capture and `["zh-CN"]` in
    the AFTER capture.
  - The sample trace classes stay `requested` for latin and `common_fallback` for CJK / emoji.
  - The mixed-script trace preserves `latin -> cjk -> emoji` family order while the CJK / emoji
    sample families resolve inside the renderer-owned `common_fallback_candidates` lane, and the
    mixed trace classes stay `requested -> common_fallback -> common_fallback`.
  - `frame_missing_glyphs` is `0` in both captures.
  - `fallback_policy_key` differs between the two labeled captures.

Run (native):

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/text/ui-gallery-text-fallback-policy-key-bumps-on-locale-change.json \
  --check-ui-gallery-text-fallback-policy-key-bumps-on-locale-change \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-web-ime-harness
```

### Gate 2: System font rescan bumps font stack key (native)

- Script: `tools/diag-scripts/ui-gallery-text-rescan-system-fonts-font-stack-key-bumps.json`
- Check flag: `--check-ui-gallery-text-rescan-system-fonts-font-stack-key-bumps`

Expected:

- The script captures two bundles (before/after rescan).
- The gate asserts `(font_stack_key, font_db_revision)` differs between the two labeled captures.

Run (native):

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-text-rescan-system-fonts-font-stack-key-bumps.json \
  --check-ui-gallery-text-rescan-system-fonts-font-stack-key-bumps \
  --launch -- cargo run -p fret-ui-gallery --release
```

### Gate 3: Bundled-only mixed-script baseline is tofu-free and profile-auditable

- Script: `tools/diag-scripts/ui-gallery/shadcn-conformance/ui-gallery-text-mixed-script-bundled-fallback-conformance.json`
- Check flag: `--check-ui-gallery-text-mixed-script-bundled-fallback-conformance`

Expected:

- The script starts directly on the dedicated mixed-script fallback page
  (`text_mixed_script_fallback`) and captures a bundle after the page settles.
- The gate asserts:
  - `system_fonts_enabled=false`
  - `prefer_common_fallback=true`
  - bundled-profile defaults and common fallback candidates stay aligned with the bundled profile
    contract
  - bundle-scoped font trace evidence remains interpretable under bundled-only mode
  - registered-font-blob counters stay populated
  - `frame_missing_glyphs=0`

Run (native, deterministic):

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/shadcn-conformance/ui-gallery-text-mixed-script-bundled-fallback-conformance.json \
  --env FRET_TEXT_SYSTEM_FONTS=0 \
  --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 \
  --check-ui-gallery-text-mixed-script-bundled-fallback-conformance \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-web-ime-harness
```

`FRET_UI_GALLERY_BOOTSTRAP_FONTS=1` now means "publish the already-installed bundled startup
baseline into the UI gallery font catalog immediately" rather than "inject extra bundled font
bytes through a separate app-local path".

## Next (recommended)

1) Add a bundled-only conformance gate that asserts missing-glyph traces are captured when tofu occurs (debug-only case).
