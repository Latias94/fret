# M2 Proof Surface — 2026-04-09

Lane: `bootstrap-known-startup-failure-taxonomy-v1`
Status: Closed proof record

## Landed implementation

### 1) Bootstrap taxonomy types and mappings

Implemented in `ecosystem/fret-bootstrap/src/lib.rs`:

- `BootstrapKnownFailureStage`
- `BootstrapKnownFailureKind`
- `BootstrapKnownFailureReport`
- `BootstrapError::known_failure_report()`
- conversions from returned startup errors and icon panic-time reports

The shipped taxonomy covers:

- settings read/parse
- keymap read/parse
- menu bar read/parse
- asset manifest read/parse/serialize/write/bundle-root/invalid/duplicate-key
- asset startup missing development/packaged lane
- icon install registry-freeze and metadata-conflict failures

### 2) Unified diagnostics panic fields

Implemented in `ecosystem/fret-bootstrap/src/lib.rs`:

- `init_panic_hook()` now maps the existing icon panic-time report into the bootstrap taxonomy and
  logs:
  - `known_panic_kind = "bootstrap_known_failure"`
  - `bootstrap_failure_stage`
  - `bootstrap_failure_kind`
  - `bootstrap_failure_surface`
  - `bootstrap_failure_pack_id`
  - `bootstrap_failure_summary`
  - `bootstrap_failure_details`

Ordinary panic text remains human-readable because the lane kept string panic payloads.

### 3) `fret` facade bridge

Implemented in `ecosystem/fret/src/lib.rs`:

- `Error::known_bootstrap_failure_report()`

This method maps:

- `Error::Bootstrap(...)`
- `Error::AssetManifest(...)`
- `Error::AssetStartup(...)`

into one shared taxonomy and intentionally returns `None` for unrelated runner failures.

### 4) Root-surface budget stayed closed

Proof discovered during `cargo nextest run -p fret --lib`:

- a first attempt to add root-level `pub use` re-exports for the taxonomy violated the existing
  `root_surface_direct_pub_use_budget_is_curated_and_closed` guard,
- the landed fix kept the bridge method but removed the new root re-exports.

That keeps the public surface aligned with the existing authoring-surface policy.

## Verification

The following commands passed on 2026-04-09:

```bash
cargo nextest run -p fret-bootstrap
cargo nextest run -p fret --lib
cargo check -p fret-bootstrap --features diagnostics
python3 tools/check_layering.py
git diff --check
```

## Proof anchors

- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret/src/lib.rs`
- `crates/fret-app/src/settings.rs`
- `crates/fret-app/src/keymap.rs`
- `crates/fret-app/src/menu_bar.rs`
- `crates/fret-assets/src/file_manifest.rs`
- `crates/fret-launch/src/assets.rs`
- `ecosystem/fret-icons/src/lib.rs`
