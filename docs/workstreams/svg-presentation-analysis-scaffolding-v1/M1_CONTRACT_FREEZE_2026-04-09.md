# M1 Contract Freeze — 2026-04-09

Status: accepted freeze

Related:

- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/DESIGN.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-report-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `crates/fret-icons-generator/src/naming.rs`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/suggest_svg.rs`

## Frozen decisions

### 1. The new public surface is one explicit helper

The public CLI surface is:

- `fretboard icons suggest svg-dir-presentation-overrides --source <dir> --out <file> [--report-out <file>]`

No import flags, runtime APIs, or generator config files are broadened here.

### 2. The helper remains `fretboard`-owned

The analysis/report logic stays in `crates/fretboard`. The generator may expose only the minimal
shared helper needed to keep `icon_name` normalization aligned with `icons import svg-dir ...`.

### 3. Output stays on the shipped config contract

The helper writes the existing versioned `presentation-defaults.json` shape and may optionally
write a separate versioned report. It does not invent another config dialect.

### 4. Strong evidence is required for per-icon overrides

The helper may suggest `original-colors` overrides only when SVG analysis finds strong evidence,
such as:

- multiple distinct solid colors,
- gradients,
- patterns,
- embedded raster images,
- or embedded SVG images.

Single-color assets remain unclassified.

### 5. Pack-level defaults stay out of scope

The helper does not infer `default_render_mode`, does not guess `mask`, and does not alter import
defaults silently.

## Rejected alternatives

- inferring pack-level `default_render_mode` from SVG analysis,
- treating any non-black single-color asset as authored-color by default,
- moving SVG analysis/report logic into `fret-icons-generator`,
- or making parse failures abort the full suggestion run.
