from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "table source policy"


@dataclass(frozen=True)
class SourceCheck:
    path: Path
    required: list[str]
    forbidden: list[str]


def normalize(text: str) -> str:
    return "".join(text.split())


def normalized_source(path: Path) -> str:
    full_path = WORKSPACE_ROOT / path
    try:
        return normalize(full_path.read_text(encoding="utf-8"))
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.as_posix()}: {exc}")


def check_source(check: SourceCheck, failures: list[str]) -> None:
    source = normalized_source(check.path)
    for marker in check.required:
        if normalize(marker) not in source:
            failures.append(f"{check.path.as_posix()}: missing {marker}")
    for marker in check.forbidden:
        if normalize(marker) in source:
            failures.append(f"{check.path.as_posix()}: forbidden {marker}")


def main() -> None:
    checks = [
        SourceCheck(
            Path("apps/fret-examples/src/table_demo.rs"),
            required=[
                "table_state: LocalState<TableState>,",
                "let table_state = LocalState::new_in(app.models_mut(), table_state);",
                "let (selected, sorting) = table_state.layout_read_ref_in(cx, |st| {",
                "fret_ui_kit::declarative::table::table_virtualized(",
                "&table_state,",
                "let view_options_open = view_options_open.clone();",
                "let header_menu_id_open = header_menu_id_open.clone();",
                "let header_menu_name_open = header_menu_name_open.clone();",
                "let header_menu_role_open = header_menu_role_open.clone();",
                "let header_menu_score_open = header_menu_score_open.clone();",
                "shadcn::DropdownMenu::from_open(open).build(",
                "shadcn::DropdownMenuCheckboxItem::new(",
                "&enable_grouping,",
                "shadcn::DropdownMenuRadioGroup::new(",
                "&grouped_column_mode,",
                "shadcn::ContextMenu::from_open(open).into_element(",
                "let table_debug_ids = fret_ui_kit::declarative::table::TableDebugIds {",
                'header_row_test_id: Some(Arc::<str>::from("table-demo-header-row",)),',
                'header_cell_test_id_prefix: Some(Arc::<str>::from("table-demo-header-"),',
                'row_test_id_prefix: Some(Arc::<str>::from("table-demo-row-",)),',
                "Prefer table-owned diagnostics anchors over renderer-local markers.",
            ],
            forbidden=[
                "let view_options_open = view_options_open.clone_model();",
                "let header_menu_id_open = header_menu_id_open.clone_model();",
                "let header_menu_name_open = header_menu_name_open.clone_model();",
                "let header_menu_role_open = header_menu_role_open.clone_model();",
                "let header_menu_score_open = header_menu_score_open.clone_model();",
                "enable_grouping_state.clone_model();",
                "grouped_column_mode_state.clone_model();",
                "table_state: Model<TableState>,",
                "cx.observe_model(&table_state, Invalidation::Layout);",
                ".models().read(&table_state, |st|",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/datatable_demo.rs"),
            required=[
                "use fret::advanced::prelude::LocalState;",
                "table_state: LocalState<TableState>,",
                "let table_state = LocalState::new_in(app.models_mut(), table_state);",
                "let (selected, sorting) = table_state.layout_read_ref_in(cx, |st| {",
                "shadcn::DataTableToolbar::new(",
                "shadcn::DataTablePagination::new(&table_state, table_output.clone())",
                "&table_state,",
                ".debug_ids(fret_ui_kit::declarative::table::TableDebugIds {",
                'header_row_test_id: Some(Arc::<str>::from("datatable-demo-header-row")),',
                'header_cell_test_id_prefix: Some(Arc::<str>::from("datatable-demo-header-")),',
                'row_test_id_prefix: Some(Arc::<str>::from("datatable-demo-row-")),',
            ],
            forbidden=[
                "table_state: Model<TableState>,",
                ".models().read(&table_state, |st|",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/table_stress_demo.rs"),
            required=[
                "let table_debug_ids = fret_ui_kit::declarative::table::TableDebugIds {",
                'header_row_test_id: Some(Arc::<str>::from("table-stress-header-row",)),',
                'header_cell_test_id_prefix: Some(Arc::<str>::from("table-stress-header-"),',
                'row_test_id_prefix: Some(Arc::<str>::from("table-stress-row-",)),',
                "Keep stress/perf diagnostics on table-owned layout wrappers.",
            ],
            forbidden=[
                "fret_ui_kit::declarative::table::TableDebugIds::default()",
            ],
        ),
    ]

    failures: list[str] = []
    for check in checks:
        check_source(check, failures)

    if failures:
        fail(GATE_NAME, f"{len(failures)} source marker problem(s):\n  - " + "\n  - ".join(failures))

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
