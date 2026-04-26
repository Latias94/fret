# Evidence And Gates

Status: closed

## Commands

```bash
cargo fmt --package fret-diag-protocol --package fret-bootstrap --package fret-diag --package fret-examples
cargo check -p fret-diag-protocol -p fret-bootstrap -p fret-diag -p fret-examples --jobs 2
cargo nextest run -p fret-diag-protocol --lib environment_sources_get_ack_round_trips_and_omits_missing_payloads --no-fail-fast
cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics diagnostics-ws" refresh_environment_source_files_publishes_launch_time_platform_capabilities_sidecar environment_sources_get_ack_publishes_transport_session_monitor_topology --no-fail-fast
cargo nextest run -p fret-diag --lib manifest_campaign_parses_platform_capabilities_environment_requirement environment_admission_skips_when_platform_capabilities_requirement_is_unsatisfied environment_admission_allows_satisfied_platform_capabilities_requirement --no-fail-fast
cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-wayland-real-host.json --json
python3 tools/check_workstream_catalog.py
git diff --check
```

## Evidence Anchors

- Protocol payload: `crates/fret-diag-protocol/src/lib.rs`
- Filesystem publication: `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
- Transport publication: `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs`
- Manifest grammar: `crates/fret-diag/src/registry/campaigns.rs`
- Admission evaluation: `crates/fret-diag/src/diag_campaign.rs`
- First consumer: `tools/diag-campaigns/imui-p3-wayland-real-host.json`
