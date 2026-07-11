from __future__ import annotations

from pathlib import Path
from typing import Any, Callable


# High-ceiling examples may own an AppComponentCx helper boundary, but should keep
# typed Ui returns instead of erasing every extracted helper to AnyElement.
ADVANCED_HELPER_CONTEXT_POLICIES = (
    (
        "assets_demo.rs",
        (
            "fn render_view<'a, Cx>(cx: &mut Cx) -> Ui",
            "Cx: fret::app::AppRenderContext<'a>",
            "let theme = cx.theme_snapshot();",
            "let cx = cx.elements();",
            "fn assets_page<C>(cx: &mut AppComponentCx<'_>, theme: &ThemeSnapshot, card: C) -> Ui",
            "C: IntoUiElement<App>",
            "fn render_image_panel(",
            ") -> impl IntoUiElement<App> + use<>",
            "fn render_svg_panel(",
        ),
        (
            "fn render_view(cx: &mut AppComponentCx<'_>) -> Ui",
            "fn render_image_panel(cx: &mut ElementContext<'_, KernelApp>,",
            "fn render_svg_panel(cx: &mut ElementContext<'_, KernelApp>,",
        ),
    ),
    (
        "custom_effect_v1_demo.rs",
        (
            "fn watch_first_f32(cx: &mut AppComponentCx<'_>,",
            "fn stage(",
            "cx: &mut AppComponentCx<'_>,",
            ") -> impl IntoUiElement<App> + use<>",
            "let label_row = |cx: &mut AppComponentCx<'_>, label: &str, value: String|",
        ),
        (
            "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
        ),
    ),
    (
        "custom_effect_v2_demo.rs",
        (
            "fn watch_first_f32(cx: &mut AppComponentCx<'_>,",
            "fn stage(",
            "cx: &mut AppComponentCx<'_>,",
            ") -> impl IntoUiElement<App> + use<>",
            "let label_row = |cx: &mut AppComponentCx<'_>, label: &str, value: String|",
        ),
        (
            "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
        ),
    ),
    (
        "custom_effect_v3_demo.rs",
        (
            "fn animated_backdrop(cx: &mut AppComponentCx<'_>) -> impl IntoUiElement<App> + use<>",
            "fn stage_controls(",
            "cx: &mut AppComponentCx<'_>,",
            "fn lens_shell(",
            ") -> impl IntoUiElement<App> + use<>",
        ),
        (
            "fn animated_backdrop(cx: &mut ElementContext<'_, KernelApp>)",
        ),
    ),
    (
        "postprocess_theme_demo.rs",
        (
            "fn watch_first_f32(cx: &mut AppComponentCx<'_>,",
            "fn stage_cards(cx: &mut AppComponentCx<'_>) -> impl IntoUiElement<App> + use<>",
            "let card = |cx: &mut AppComponentCx<'_>, title: &str, subtitle: &str|",
        ),
        (
            "fn stage_cards(cx: &mut ElementContext<'_, KernelApp>)",
        ),
    ),
    (
        "markdown_demo.rs",
        (
            "let spinner_box = |cx: &mut AppComponentCx<'_>|",
            "fn render_image_placeholder(",
            "cx: &mut AppComponentCx<'_>,",
        ),
        (
            "let spinner_box = |cx: &mut fret_ui::ElementContext<'_, KernelApp>|",
        ),
    ),
    (
        "genui_demo.rs",
        (
            "fn genui_page<L, R>(cx: &mut AppComponentCx<'_>, theme: ThemeSnapshot, left: L, right: R) -> Ui",
            "L: IntoUiElement<KernelApp>",
            "R: IntoUiElement<KernelApp>",
            "genui_page(cx, theme, left, right)",
        ),
        ("let page = ui::container(move |cx| {",),
    ),
    (
        "liquid_glass_demo.rs",
        (
            "fn watch_first_f32(cx: &mut AppComponentCx<'_>,",
            "let mk_card = |cx: &mut AppComponentCx<'_>,",
            "|cx: &mut AppComponentCx<'_>,",
        ),
        (
            "fn watch_first_f32(cx: &mut ElementContext<'_, KernelApp>,",
            "let mk_card = |cx: &mut ElementContext<'_, KernelApp>,",
        ),
    ),
)


CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]


def check_advanced_helper_context_source_policies(
    failures: list[Any],
    *,
    examples_src: Path,
    read_source: ReadSource,
    check_required_forbidden_markers: CheckMarkers,
) -> None:
    for source_name, required, forbidden in ADVANCED_HELPER_CONTEXT_POLICIES:
        path = examples_src / source_name
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=list(required),
            forbidden=list(forbidden),
            failures=failures,
        )
