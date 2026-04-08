# Baseline Decision 2026-04-08

Status: Accepted for this lane
Date: 2026-04-08

## Decision

For the `font-bundle-release-boundary-v1` lane, the framework-owned bundled bootstrap baseline is:

- `bootstrap-subset` only

The following are explicitly **not** part of the framework bootstrap baseline:

- `cjk-lite`
- `emoji`
- `bootstrap-full`

They remain valid bundled-font surfaces, but they are extension bundles or demo/debug convenience
assets, not framework baseline assets.

## Why this is the correct boundary

### 1. The baseline must guarantee text-system correctness, not broad script coverage

The framework baseline must be strong enough to guarantee:

- portable startup text rendering,
- stable control metrics,
- deterministic diagnostics screenshots,
- and reproducible fallback/cache behavior.

That requires a minimal UI sans/serif/monospace bootstrap set. It does not require broad language
coverage by default.

### 2. Script coverage is product policy, not framework mechanism

`cjk-lite` improves first-party Web/WASM usability, but it is still script coverage policy. The
repo already models it that way:

- ADR 0147 describes CJK fallback as optional.
- `fret-launch` already exposes explicit opt-in features:
  - `wasm-cjk-lite-fonts`
  - `wasm-emoji-fonts`
- first-party web harnesses explicitly choose whether to enable CJK/emoji features.

That means the conceptual model is already "extension bundle"; the main gap is that the published
crate boundary still does not reflect that model.

### 3. Cargo package boundaries, not feature flags, define publication cost

Even if downstream code treats `cjk-lite` as optional, keeping the asset inside the main published
crate means the publication boundary still pays for it. A correct publication design must separate
baseline and extension assets at the package boundary.

### 4. First-party defaults may still include CJK coverage

This decision does **not** say that first-party web demos or gallery surfaces should stop enabling
`cjk-lite` by default. It says that such a choice belongs to the app/product layer, not to the
framework bootstrap baseline.

## Evidence

- `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`
- `crates/fret-launch/Cargo.toml`
- `apps/fret-demo-web/Cargo.toml`
- `apps/fret-ui-gallery-web/Cargo.toml`
- `apps/fret-demo-web/README.md`
- `apps/fret-ui-gallery-web/README.md`
- `docs/workstreams/code-editor-ecosystem-v1/code-editor-ecosystem-v1.md`

## Consequences for the next slice

1. `fret-fonts` must stop making `cjk-lite` part of its published core package boundary.
2. `fret-launch` must stop inheriting `cjk-lite` as an accidental default baseline through
   `fret-fonts` default features.
3. First-party web/demo surfaces should opt into `cjk-lite` explicitly, just as they already do in
   concept for `emoji`.
4. Release preflight must prove that the main published `fret-fonts` package only contains baseline
   assets.
