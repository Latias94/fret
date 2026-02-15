use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::ShadcnResolver;

impl ShadcnResolver {
    pub(super) fn render_card<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let wrap_content = resolved_props
            .get("wrapContent")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if wrap_content {
            // Convenience mode: when the spec doesn't provide CardHeader/CardContent/CardFooter
            // explicitly, provide a reasonable padded body container.
            //
            // Note: `CardContent` is modeled after shadcn/ui's `p-6 pt-0`, which is intended to
            // follow a header. For "single-body" cards, we want top padding too.
            let body = fret_ui_kit::ui::v_flex(cx, move |_cx| children)
                .gap(fret_ui_kit::Space::N0)
                .items_start()
                .w_full()
                .p(fret_ui_kit::Space::N6)
                .into_element(cx);
            fret_ui_shadcn::Card::new([body]).into_element(cx)
        } else {
            fret_ui_shadcn::Card::new(children).into_element(cx)
        }
    }

    pub(super) fn render_card_header<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        fret_ui_shadcn::CardHeader::new(children).into_element(cx)
    }

    pub(super) fn render_card_content<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        fret_ui_shadcn::CardContent::new(children).into_element(cx)
    }

    pub(super) fn render_card_footer<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        fret_ui_shadcn::CardFooter::new(children).into_element(cx)
    }

    pub(super) fn render_card_title<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
    ) -> AnyElement {
        let text = Self::json_to_label(
            resolved_props
                .get("text")
                .or_else(|| resolved_props.get("title")),
        );
        fret_ui_shadcn::CardTitle::new(text).into_element(cx)
    }

    pub(super) fn render_card_description<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
    ) -> AnyElement {
        let text = Self::json_to_label(
            resolved_props
                .get("text")
                .or_else(|| resolved_props.get("description")),
        );
        fret_ui_shadcn::CardDescription::new(text).into_element(cx)
    }

    pub(super) fn render_text<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
    ) -> AnyElement {
        let text = Self::json_to_label(resolved_props.get("text"));
        match resolved_props
            .get("variant")
            .and_then(|v| v.as_str())
            .unwrap_or("body")
        {
            "body" => fret_ui_shadcn::typography::p(cx, text),
            "muted" => fret_ui_shadcn::typography::muted(cx, text),
            "small" => fret_ui_shadcn::typography::small(cx, text),
            "lead" => fret_ui_shadcn::typography::lead(cx, text),
            "large" => fret_ui_shadcn::typography::large(cx, text),
            "h1" => fret_ui_shadcn::typography::h1(cx, text),
            "h2" => fret_ui_shadcn::typography::h2(cx, text),
            "h3" => fret_ui_shadcn::typography::h3(cx, text),
            "h4" => fret_ui_shadcn::typography::h4(cx, text),
            "inlineCode" => fret_ui_shadcn::typography::inline_code(cx, text),
            _ => fret_ui_kit::ui::text(cx, text).into_element(cx),
        }
    }

    pub(super) fn render_vstack<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let gap = Self::parse_space(resolved_props.get("gap")).unwrap_or(fret_ui_kit::Space::N2);
        let p = Self::parse_space(resolved_props.get("p"));
        let px = Self::parse_space(resolved_props.get("px"));
        let py = Self::parse_space(resolved_props.get("py"));
        let items = resolved_props.get("items").and_then(|v| v.as_str());
        let justify = resolved_props.get("justify").and_then(|v| v.as_str());
        let wrap = resolved_props
            .get("wrap")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let w_full = resolved_props
            .get("wFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let h_full = resolved_props
            .get("hFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_w_0 = resolved_props
            .get("minW0")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_h_0 = resolved_props
            .get("minH0")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut v = fret_ui_kit::ui::v_flex(cx, move |_cx| children).gap(gap);
        v = match items {
            Some("center") => v.items_center(),
            Some("end") => v.items_end(),
            Some("stretch") => v.items_stretch(),
            _ => v.items_start(),
        };
        v = match justify {
            Some("center") => v.justify_center(),
            Some("end") => v.justify_end(),
            Some("between") => v.justify_between(),
            _ => v.justify_start(),
        };
        if wrap {
            v = v.wrap();
        }
        if w_full {
            v = v.w_full();
        }
        if h_full {
            v = v.h_full();
        }
        if min_w_0 {
            v = v.min_w_0();
        }
        if min_h_0 {
            v = v.min_h_0();
        }
        if let Some(p) = p {
            v = v.p(p);
        }
        if let Some(px) = px {
            v = v.px(px);
        }
        if let Some(py) = py {
            v = v.py(py);
        }
        v.into_element(cx)
    }

    pub(super) fn render_hstack<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let gap = Self::parse_space(resolved_props.get("gap")).unwrap_or(fret_ui_kit::Space::N2);
        let p = Self::parse_space(resolved_props.get("p"));
        let px = Self::parse_space(resolved_props.get("px"));
        let py = Self::parse_space(resolved_props.get("py"));
        let items = resolved_props.get("items").and_then(|v| v.as_str());
        let justify = resolved_props.get("justify").and_then(|v| v.as_str());
        let wrap = resolved_props
            .get("wrap")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let w_full = resolved_props
            .get("wFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let h_full = resolved_props
            .get("hFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_w_0 = resolved_props
            .get("minW0")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_h_0 = resolved_props
            .get("minH0")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut h = fret_ui_kit::ui::h_flex(cx, move |_cx| children).gap(gap);
        h = match items {
            Some("start") => h.items_start(),
            Some("end") => h.items_end(),
            Some("stretch") => h.items_stretch(),
            _ => h.items_center(),
        };
        h = match justify {
            Some("center") => h.justify_center(),
            Some("end") => h.justify_end(),
            Some("between") => h.justify_between(),
            _ => h.justify_start(),
        };
        if wrap {
            h = h.wrap();
        }
        if w_full {
            h = h.w_full();
        }
        if h_full {
            h = h.h_full();
        }
        if min_w_0 {
            h = h.min_w_0();
        }
        if min_h_0 {
            h = h.min_h_0();
        }
        if let Some(p) = p {
            h = h.p(p);
        }
        if let Some(px) = px {
            h = h.px(px);
        }
        if let Some(py) = py {
            h = h.py(py);
        }
        h.into_element(cx)
    }

    pub(super) fn render_box<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let p = Self::parse_space(resolved_props.get("p"));
        let px = Self::parse_space(resolved_props.get("px"));
        let py = Self::parse_space(resolved_props.get("py"));
        let w_full = resolved_props
            .get("wFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let h_full = resolved_props
            .get("hFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_w_0 = resolved_props
            .get("minW0")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_h_0 = resolved_props
            .get("minH0")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut b = fret_ui_kit::ui::container(cx, move |_cx| children);
        if w_full {
            b = b.w_full();
        }
        if h_full {
            b = b.h_full();
        }
        if min_w_0 {
            b = b.min_w_0();
        }
        if min_h_0 {
            b = b.min_h_0();
        }
        if let Some(p) = p {
            b = b.p(p);
        }
        if let Some(px) = px {
            b = b.px(px);
        }
        if let Some(py) = py {
            b = b.py(py);
        }
        b.into_element(cx)
    }

    pub(super) fn render_separator<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
    ) -> AnyElement {
        let orientation = resolved_props
            .get("orientation")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "horizontal" => Some(fret_ui_shadcn::SeparatorOrientation::Horizontal),
                "vertical" => Some(fret_ui_shadcn::SeparatorOrientation::Vertical),
                _ => None,
            })
            .unwrap_or(fret_ui_shadcn::SeparatorOrientation::Horizontal);
        let flex_stretch_cross_axis = resolved_props
            .get("flexStretchCrossAxis")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        fret_ui_shadcn::Separator::new()
            .orientation(orientation)
            .flex_stretch_cross_axis(flex_stretch_cross_axis)
            .into_element(cx)
    }

    pub(super) fn render_scroll_area<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let axis = resolved_props
            .get("axis")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "x" => Some(fret_ui::element::ScrollAxis::X),
                "y" => Some(fret_ui::element::ScrollAxis::Y),
                "both" => Some(fret_ui::element::ScrollAxis::Both),
                _ => None,
            })
            .unwrap_or(fret_ui::element::ScrollAxis::Y);
        let show_scrollbar = resolved_props
            .get("showScrollbar")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        fret_ui_shadcn::ScrollArea::new(children)
            .axis(axis)
            .show_scrollbar(show_scrollbar)
            .into_element(cx)
    }

    pub(super) fn render_button<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
        on_event: &dyn Fn(&str) -> Option<OnActivate>,
    ) -> AnyElement {
        let label = Self::json_to_label(resolved_props.get("label"));
        let mut button = fret_ui_shadcn::Button::new(label).children(children);
        if let Some(on_activate) = on_event("press") {
            button = button.on_activate(on_activate);
        }
        button.into_element(cx)
    }

    pub(super) fn render_badge<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let label = Self::json_to_label(resolved_props.get("label"));
        let variant = Self::parse_badge_variant(resolved_props.get("variant")).unwrap_or_default();
        fret_ui_shadcn::Badge::new(label)
            .variant(variant)
            .children(children)
            .into_element(cx)
    }
}
