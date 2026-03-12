use std::sync::Arc;

use fret_genui_core::props::ResolvedProps;
use fret_genui_core::spec::ElementKey;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::IntoUiElement;

use super::ShadcnResolver;

impl ShadcnResolver {
    pub(super) fn render_heading<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
    ) -> AnyElement {
        let text = Self::json_to_label(resolved_props.get("text"));
        let level = resolved_props
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("h2");
        match level {
            "h1" => fret_ui_shadcn::typography::h1(text).into_element(cx),
            "h2" => fret_ui_shadcn::typography::h2(text).into_element(cx),
            "h3" => fret_ui_shadcn::typography::h3(text).into_element(cx),
            "h4" => fret_ui_shadcn::typography::h4(text).into_element(cx),
            _ => fret_ui_shadcn::typography::h2(text).into_element(cx),
        }
    }

    pub(super) fn render_stack<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let direction = resolved_props
            .get("direction")
            .and_then(|v| v.as_str())
            .unwrap_or("vertical");
        let gap = resolved_props
            .get("gap")
            .and_then(|v| v.as_str())
            .unwrap_or("md");
        let gap = match gap {
            "sm" => fret_ui_kit::Space::N2,
            "lg" => fret_ui_kit::Space::N6,
            _ => fret_ui_kit::Space::N4,
        };
        match direction {
            "horizontal" => fret_ui_kit::ui::h_flex(move |_cx| children)
                .gap(gap)
                .items_center()
                .into_element(cx),
            _ => fret_ui_kit::ui::v_flex(move |_cx| children)
                .gap(gap)
                .items_start()
                .into_element(cx),
        }
    }

    pub(super) fn render_avatar<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        resolved_props: &serde_json::Map<String, serde_json::Value>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let fallback = resolved_props
            .get("fallback")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Note: `src` is a URL in json-render; Fret's AvatarImage expects an `ImageId`.
        // Until we have a URL → ImageId pipeline in GenUI, always render the fallback.
        let fallback = fret_ui_shadcn::AvatarFallback::new(Arc::<str>::from(fallback));

        let mut out: Vec<AnyElement> = Vec::new();
        out.extend(children);
        out.push(fallback.into_element(cx));

        fret_ui_shadcn::Avatar::new(out).into_element(cx)
    }

    pub(super) fn render_bar_chart<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
    ) -> AnyElement {
        self.render_chart_placeholder(cx, key, props, "BarChart")
    }

    pub(super) fn render_line_chart<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
    ) -> AnyElement {
        self.render_chart_placeholder(cx, key, props, "LineChart")
    }

    fn render_chart_placeholder<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
        ty: &str,
    ) -> AnyElement {
        let resolved = &props.props;
        let title = resolved
            .get("title")
            .and_then(|v| v.as_str())
            .map(Arc::<str>::from)
            .unwrap_or_else(|| Arc::<str>::from(ty));

        let msg = Arc::<str>::from(format!("{ty} placeholder (not implemented yet): {:?}", key));

        let body = fret_ui_kit::ui::v_flex(move |_cx| {
            vec![
                fret_ui_shadcn::CardTitle::new(title).into_element(_cx),
                fret_ui_shadcn::CardDescription::new(msg).into_element(_cx),
            ]
        })
        .gap(fret_ui_kit::Space::N2)
        .items_start()
        .w_full()
        .into_element(cx);

        fret_ui_shadcn::Card::new([fret_ui_shadcn::CardContent::new([body]).into_element(cx)])
            .into_element(cx)
    }
}
