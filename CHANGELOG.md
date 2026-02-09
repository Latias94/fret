# Changelog

All notable changes to this repository are documented in this file.

Fret currently uses a **single repository-level changelog** plus GitHub Release notes.
We do not maintain one `CHANGELOG.md` per crate in `v0.1`.

Release automation:

- `release-plz.toml`
- `.github/workflows/release-plz.yml`

Release runbooks:

- `docs/release/v0.1.0-release-checklist.md`
- `docs/release/v0.1.0-publish-order.txt`

## Unreleased

### Added

- Adopt `release-plz` workflow with explicit publish allowlist.
- Add release closure and publish-order checking script (`tools/release_closure_check.py`).
- Add release skill guidance under `.agents/skills/fret-release-check-and-publish`.

### Changed

- Add publish-ready internal dependency version requirements (`path + version`) for v0.1 release scope.
- Add crate descriptions for the v0.1 public publish set.
- Clarify MSRV policy: Rust `1.92` aligned with current `wgpu` minimum requirements.

### Documentation

- Add release adoption analysis and v0.1 release checklist docs.
- Reorganize `README.md` for quick-start + architecture + release discoverability.

