from __future__ import annotations

from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "default teaching snippets use UiCx"


def curated_default_snippet_paths() -> list[Path]:
    root = WORKSPACE_ROOT / "apps/fret-ui-gallery/src/ui/snippets"
    paths = [
        root / "command/action_first_view.rs",
        root / "navigation_menu/demo.rs",
        root / "navigation_menu/docs_demo.rs",
        root / "navigation_menu/rtl.rs",
        root / "breadcrumb/responsive.rs",
        root / "date_picker/dropdowns.rs",
        root / "form/notes.rs",
        root / "sidebar/rtl.rs",
        root / "slider/usage.rs",
        root / "toast/deprecated.rs",
        root / "data_table/basic_demo.rs",
        root / "data_table/default_demo.rs",
        root / "data_table/guide_demo.rs",
        root / "data_table/rtl_demo.rs",
        root / "table/actions.rs",
        root / "table/demo.rs",
        root / "table/footer.rs",
        root / "table/rtl.rs",
        root / "tabs/demo.rs",
        root / "tabs/disabled.rs",
        root / "tabs/extras.rs",
        root / "tabs/icons.rs",
        root / "tabs/line.rs",
        root / "tabs/list.rs",
        root / "tabs/rtl.rs",
        root / "tabs/vertical.rs",
        root / "tabs/vertical_line.rs",
    ]

    for subdir in [
        "progress",
        "combobox",
        "chart",
        "carousel",
        "item",
        "motion_presets",
        "card",
    ]:
        subdir_root = root / subdir
        for path in sorted(subdir_root.glob("*.rs")):
            if path.name == "mod.rs":
                continue
            paths.append(path)

    return paths


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.relative_to(WORKSPACE_ROOT)}: {exc}")


def main() -> None:
    problems: list[str] = []
    for path in curated_default_snippet_paths():
        if not path.is_file():
            problems.append(f"missing curated snippet: {path.relative_to(WORKSPACE_ROOT)}")
            continue

        text = read_text(path)
        rel = path.relative_to(WORKSPACE_ROOT)

        if "UiCx<'_>" not in text:
            problems.append(
                f"{rel} must keep the default app-facing helper signature on UiCx<'_>"
            )

        for forbidden in [
            "ElementContext<'_, App>",
            "ElementContext<'_, KernelApp>",
            "use fret_app::App;",
            "use fret::advanced::KernelApp;",
        ]:
            if forbidden in text:
                problems.append(f"{rel} reintroduced forbidden default-surface snippet: {forbidden}")

    if problems:
        fail(GATE_NAME, "\n".join(problems))

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
