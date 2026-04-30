from __future__ import annotations

from pathlib import Path
from typing import Any, Callable


MANUAL_UI_TREE_ROOT_WRAPPER_SOURCES = [
    (
        "cjk_conformance_demo.rs",
        [
            "fn cjk_conformance_page<'a, Cx, C>(",
            "Cx: fret_ui::ElementContextAccess<'a, App>,",
            "theme: fret_ui::ThemeSnapshot,",
            "card: C,",
            ") -> impl fret_ui_kit::IntoUiElement<App> + use<Cx, C>",
            "C: fret_ui_kit::IntoUiElement<App>,",
            ".into_element_in(cx)",
            "ui::children![cx; cjk_conformance_page(cx, theme, card)]",
            "ui::v_flex(move |cx| ui::single(cx, card))",
        ],
        [
            "cx: &mut fret_ui::ElementContext<'_, App>,",
            "let page = ui::container(|cx| {",
            "ui::v_flex(move |_cx| [card])",
        ],
    ),
    (
        "emoji_conformance_demo.rs",
        [
            "fn emoji_conformance_page<'a, Cx, C>(",
            "Cx: fret_ui::ElementContextAccess<'a, App>,",
            "theme: fret_ui::ThemeSnapshot,",
            "card: C,",
            ") -> impl fret_ui_kit::IntoUiElement<App> + use<Cx, C>",
            "C: fret_ui_kit::IntoUiElement<App>,",
            ".into_element_in(cx)",
            "ui::children![cx; emoji_conformance_page(cx, theme, card)]",
            "ui::v_flex(move |cx| ui::single(cx, card))",
        ],
        [
            "cx: &mut fret_ui::ElementContext<'_, App>,",
            "let page = ui::container(|cx| {",
            "ui::v_flex(move |_cx| [card])",
        ],
    ),
]

CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]


def check_manual_ui_tree_source_policies(
    failures: list[Any],
    *,
    examples_src: Path,
    read_source: ReadSource,
    check_required_forbidden_markers: CheckMarkers,
) -> None:
    for name, required, forbidden in MANUAL_UI_TREE_ROOT_WRAPPER_SOURCES:
        path = examples_src / name
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["UiTree<App>", *required],
            forbidden=["KernelApp", *forbidden],
            failures=failures,
        )
