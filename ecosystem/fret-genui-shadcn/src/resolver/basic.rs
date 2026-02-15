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
        fret_ui_kit::ui::text(cx, text).into_element(cx)
    }

    pub(super) fn render_vstack<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let gap = Self::parse_space(resolved_props.get("gap")).unwrap_or(fret_ui_kit::Space::N2);
        let p = Self::parse_space(resolved_props.get("p"));
        let items = resolved_props.get("items").and_then(|v| v.as_str());
        let justify = resolved_props.get("justify").and_then(|v| v.as_str());
        let wrap = resolved_props
            .get("wrap")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut v = fret_ui_kit::ui::v_flex(cx, move |_cx| children)
            .gap(gap)
            .w_full();
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
        if let Some(p) = p {
            v = v.p(p);
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
        let items = resolved_props.get("items").and_then(|v| v.as_str());
        let justify = resolved_props.get("justify").and_then(|v| v.as_str());
        let wrap = resolved_props
            .get("wrap")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut h = fret_ui_kit::ui::h_flex(cx, move |_cx| children)
            .gap(gap)
            .w_full();
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
        if let Some(p) = p {
            h = h.p(p);
        }
        h.into_element(cx)
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
