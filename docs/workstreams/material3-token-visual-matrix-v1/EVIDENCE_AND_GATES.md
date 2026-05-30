# Material3 Token Visual Matrix v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Smallest Current Repro

The initial repro is schema/catalog validation:

```powershell
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null
python -m json.tool ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json | Out-Null
python tools/check_workstream_catalog.py
```

## Gate Set

### Workstream State

```powershell
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null
python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null
python -m json.tool ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json | Out-Null
python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-30 --output docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json
python tools/check_workstream_catalog.py
git diff --check
```

### Material Token Inner Loop

Use narrow gates while the inventory and fixture harness are young:

```powershell
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Evidence Anchors

- `docs/workstreams/material3-token-visual-matrix-v1/DESIGN.md`
- `docs/workstreams/material3-token-visual-matrix-v1/TODO.md`
- `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json`
- `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_schema_v1.md`
- `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json`
- `tools/parity-discovery/material3_token_inventory.py`
- `ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json`
- `ecosystem/fret-ui-material3/src/tokens/visual_fixtures.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_v2_closeout_audit.md`
- `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`
- `ecosystem/fret-ui-material3/tests`
- `goldens/material3-headless/v1`
- `repo-ref/compose-multiplatform-core`
- `repo-ref/base-ui`

## Fresh Evidence Log

- 2026-05-30: M3TVM-010 opened the token visual matrix lane.
  - Sources inspected: M3PV2 closeout audit, `tokens::v30` injection surface, generated
    `material_web_v30`, component token module inventory, and workstream catalog conventions.
  - Result: the initial matrix covers all 39 M3PV2 components and records matrix dimensions,
    source precedence, first family packet ownership, and initial `inventory_seeded` state.
  - Evidence note:
    `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_schema_v1.md`.
- 2026-05-30: M3TVM-020 added the generated token inventory/fallback audit.
  - Report:
    `docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json`.
  - Result: all 38 component token modules are mapped to matrix rows; the report classifies 3,580
    generated Material Web token keys, 107 v30 injection functions, 67 non-generated v30 manual
    writes, 1,136 component token fallback sites, 504 component token visual constants, 45
    foundation/interaction fallback sites, and 50 foundation/interaction visual constants.
  - Main finding: the next fix should build fixture rows before deleting fallback logic, with
    field-family, slider/list/chip/fab/icon-button, and time-picker modules highest in the queue.
  - Fresh verification:
    - `python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-30 --output docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json`: passed.
    - `python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null`; `python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null`; `python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null`: passed.
    - `python tools/check_workstream_catalog.py`: passed with 511 dedicated directories and 47 standalone markdown files.
    - `cargo nextest run -p fret-ui-material3 --lib tokens::v30`: passed, 6 tests run.
    - `git diff --check`: passed.
  - Broader gates not run: `cargo check -p fret-ui-material3 --features diagnostics --tests` and
    diagnostics automation tests were skipped because M3TVM-020 only adds a Python report generator
    plus docs/artifact updates and does not change Rust component behavior.
- 2026-05-30: M3TVM-030 added the fixture-driven token visual harness pilot.
  - Fixture suite: `ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json`.
  - Runner: `ecosystem/fret-ui-material3/src/tokens/visual_fixtures.rs`.
  - Result: Button and TextField matrix rows moved to `covered_fixture`; the suite covers Button
    label/container/icon/state-layer/elevation/shadow/typography outcomes and TextField
    container/background/outline/active-indicator/label/supporting/input/caret/shape/typography
    outcomes.
  - Bug fixed: TextField disabled input text was applying disabled opacity twice in the private
    token helper plus style assembly path; the helper now returns the base token color and the
    style assembly path applies disabled opacity once.
  - Fresh verification:
    - `cargo fmt -p fret-ui-material3 -- --check`: passed.
    - `cargo check -p fret-ui-material3`: passed.
    - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`: passed.
    - `python -m json.tool ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json | Out-Null`: passed.
    - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures`: passed, 1 test run.
    - `cargo nextest run -p fret-ui-material3 --lib tokens::v30`: passed, 6 tests run.
    - `python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-30 --output docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json`: passed.
    - `python tools/check_workstream_catalog.py`: passed with 511 dedicated directories and 47
      standalone markdown files.
    - `git diff --check`: passed.
- 2026-05-30: M3TVM-040A extended the fixture-driven token visual harness to the field-overlay
  subset.
  - Fixture suite:
    `ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json`.
  - Runner:
    `ecosystem/fret-ui-material3/src/tokens/visual_fixtures.rs`.
  - Result: Select, Autocomplete, and ExposedDropdown matrix rows moved to `covered_fixture`.
    The suite now covers Select field chrome, disabled alpha, filled/outlined shape, outline,
    active-indicator, hover state-layer, menu surface/list item/icon outcomes, and typography
    source outcomes. Autocomplete and ExposedDropdown are covered through the Autocomplete token
    namespace and menu surface/list item outcomes.
  - Bugs fixed:
    - Autocomplete label, supporting text, and hover state-layer rendering were still routed through
      ordinary TextField tokens despite using `TextFieldTokenNamespace::Autocomplete`.
    - Autocomplete disabled input text applied disabled opacity in both the private token helper and
      the style assembly path.
    - Select filled disabled container color used a transparent disabled color directly instead of
      blending the disabled overlay token over the filled container token.
    - Select and Autocomplete shape accessors now consume metric-backed Material Web `*.shape`
      tokens where generated tokens are scalar metrics rather than `Corners`.
  - Fresh verification:
    - `python -m json.tool ecosystem/fret-ui-material3/tests/fixtures/material3_token_visual_cases_v1.json | Out-Null`: passed.
    - `cargo fmt -p fret-ui-material3 -- --check`: passed.
    - `cargo check -p fret-ui-material3`: passed.
    - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`: passed.
    - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures`: passed, 1 test run.
    - `cargo nextest run -p fret-ui-material3 --lib tokens::v30`: passed, 6 tests run.
    - `python tools/parity-discovery/material3_token_inventory.py --generated-date 2026-05-30 --output docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json`: passed.
    - `python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json | Out-Null`; `python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_visual_matrix_v1.json | Out-Null`; `python -m json.tool docs/workstreams/material3-token-visual-matrix-v1/artifacts/material3_token_inventory_report_v1.json | Out-Null`: passed.
    - `python tools/check_workstream_catalog.py`: passed with 511 dedicated directories and 47
      standalone markdown files.
    - `git diff --check`: passed.
