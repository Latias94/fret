# Workstream: Font Fallback Conformance v1 (Gates + Diagnostics)

Status: In progress

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
- Font system roadmap: `docs/workstreams/font-system-v1.md`

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

- The script captures two bundles (before/after settings apply).
- The gate asserts `fallback_policy_key` differs between the two labeled captures.

Run (native):

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-text-fallback-policy-key-bumps-on-settings-change.json \
  --check-ui-gallery-text-fallback-policy-key-bumps-on-settings-change \
  --launch -- cargo run -p fret-ui-gallery --release
```

### Gate 1.5: Locale change bumps fallback policy key

- Script: `tools/diag-scripts/ui-gallery-text-fallback-policy-key-bumps-on-locale-change.json`
- Check flag: `--check-ui-gallery-text-fallback-policy-key-bumps-on-locale-change`

Expected:

- The script captures two bundles (before/after locale change via Settings).
- The gate asserts:
  - `locale_bcp47` is `en-US` in the BEFORE capture and `zh-CN` in the AFTER capture.
  - `fallback_policy_key` differs between the two labeled captures.

Run (native):

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-text-fallback-policy-key-bumps-on-locale-change.json \
  --check-ui-gallery-text-fallback-policy-key-bumps-on-locale-change \
  --launch -- cargo run -p fret-ui-gallery --release
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

### Gate 3: Bundled-only mixed-script baseline is tofu-free

- Script: `tools/diag-scripts/ui-gallery-text-mixed-script-bundled-fallback-conformance.json`
- Check flag: `--check-ui-gallery-text-mixed-script-bundled-fallback-conformance`

Expected:

- The script navigates to the text measure overlay page, which includes a mixed-script sample (`m你😀`).
- The gate asserts:
  - `system_fonts_enabled=false`
  - `prefer_common_fallback=true`
  - `frame_missing_glyphs=0`

Run (native, deterministic):

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-text-mixed-script-bundled-fallback-conformance.json \
  --env FRET_TEXT_SYSTEM_FONTS=0 \
  --env FRET_UI_GALLERY_BOOTSTRAP_FONTS=1 \
  --check-ui-gallery-text-mixed-script-bundled-fallback-conformance \
  --launch -- cargo run -p fret-ui-gallery --release
```

## Next (recommended)

1) Add a locale-switch gate that asserts `fallback_policy_key` bumps when the renderer locale changes (bundle-only).
2) Add a bundled-only conformance gate that asserts missing-glyph traces are captured when tofu occurs (debug-only case).
