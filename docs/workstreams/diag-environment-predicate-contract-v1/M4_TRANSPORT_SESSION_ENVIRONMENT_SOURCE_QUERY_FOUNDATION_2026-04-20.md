# M4 Transport Session Environment Source Query Foundation - 2026-04-20

Status: active implementation note

Related:

- `WORKSTREAM.json`
- `DESIGN.md`
- `M2_ENVIRONMENT_SOURCE_PROVENANCE_AND_AVAILABILITY_CONTRACT_2026-04-20.md`
- `M3_HOST_MONITOR_TOPOLOGY_LAUNCH_TIME_PUBLICATION_AND_CAMPAIGN_PROVENANCE_2026-04-20.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `crates/fret-diag-protocol/src/lib.rs`
- `crates/fret-diag/src/devtools.rs`
- `crates/fret-diag/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `tools/gate_imui_workstream_source.py`

## Purpose

This note records the next additive slice after launch-time filesystem publication landed.

The goal of this slice was still narrow:

1. land an explicit transport/session query surface for admitted environment sources,
2. keep that surface separate from static hello/session capabilities,
3. let existing DevTools sessions publish `host.monitor_topology` truthfully as
   `preflight_transport_session`,
4. and still stop before manifest grammar or environment predicate execution.

## Landed result

### 1) DevTools now has an explicit environment-source query pair

`fret-diag-protocol` now exposes:

- `DevtoolsEnvironmentSourcesGetV1`
- `DevtoolsEnvironmentSourcesGetAckV1`

The transport message names are:

- `environment.sources.get`
- `environment.sources.get_ack`

This keeps environment-source acquisition explicit instead of smuggling it into static
`session.list` descriptors.

### 2) Runtime now publishes `host.monitor_topology` through transport/session

`ecosystem/fret-bootstrap` now advertises `devtools.environment_sources` for DevTools WS sessions.

When tooling sends `environment.sources.get`, the runtime answers with:

- a source list,
- runner identity hints,
- and an inline `host_monitor_topology` payload when the runner currently exposes that inventory.

For this transport/session publication path, `host.monitor_topology` is truthfully classified as
`preflight_transport_session`.

### 3) `fret-diag` now has a transport-session acquisition seam

`crates/fret-diag` now has a tooling helper for:

- sending the explicit request,
- waiting for the matching ack,
- normalizing the returned source list,
- and keeping inline `host_monitor_topology` payloads separate from filesystem payload paths.

This acquisition is additive today: tooling only queries when the selected session advertises
`devtools.environment_sources`.

### 4) The repo now has both truthful acquisition lanes needed before grammar work

After this slice, the first admitted source has two explicit acquisition lanes:

- filesystem publication at `launch_time`,
- transport-session query at `preflight_transport_session`.

That means the next manifest decision no longer has to guess how the source will actually be
acquired.

## Important non-results

This slice intentionally did not:

- add `requires_environment`,
- execute environment predicates during campaign preflight,
- widen `session.list` into a mutable environment-source registry,
- overload `capabilities.json` or hello capabilities with environment facts,
- or reinterpret tool-launched filesystem runs as preflight-ready.

Tool-launched filesystem runs still remain truthful `launch_time` acquisitions.

## Why this is the correct shape

### Static session descriptors are the wrong owner for dynamic environment sources

`DevtoolsSessionDescriptorV1` is a static session list surface.

That is the correct place for stable capabilities such as `devtools.scripts` or
`devtools.environment_sources`, but it is the wrong place for dynamic environment facts like the
current host monitor inventory.

An explicit request/ack keeps the timing honest.

### Support capability and source data stay separate

`devtools.environment_sources` means only:

- this session knows how to answer the explicit source query.

It does not mean:

- which sources are currently available,
- whether `host.monitor_topology` is present right now,
- or that environment facts have become capabilities.

That boundary is the whole point of this slice.

### This slice unlocks syntax work without forcing it

Before this change, the lane still lacked a truthful transport/session publication path.

Now both acquisition lanes exist, so manifest grammar can be chosen from a real source/timing
matrix instead of speculation.

## Evidence

- Protocol:
  - `crates/fret-diag-protocol/src/lib.rs`
- Tooling request/ack helper:
  - `crates/fret-diag/src/devtools.rs`
  - `crates/fret-diag/src/lib.rs`
- Runtime WS publication:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`
- Living docs + source-policy gate:
  - `docs/ui-diagnostics-and-scripted-tests.md`
  - `tools/gate_imui_workstream_source.py`

## Verification

- `cargo nextest run -p fret-diag-protocol --lib environment_sources_get --no-fail-fast`
- `cargo nextest run -p fret-diag --lib environment_source --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics-ws" --lib environment_sources_get --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `git diff --check`
