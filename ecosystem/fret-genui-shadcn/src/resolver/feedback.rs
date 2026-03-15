use fret_genui_core::props::ResolvedProps;
use fret_genui_core::spec::ElementKey;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_shadcn::facade as shadcn;

use super::ShadcnResolver;

fn parse_f32(v: Option<&serde_json::Value>) -> Option<f32> {
    let v = v?;
    if let Some(f) = v.as_f64() {
        return Some(f as f32);
    }
    if let Some(i) = v.as_i64() {
        return Some(i as f32);
    }
    if let Some(u) = v.as_u64() {
        return Some(u as f32);
    }
    if let Some(s) = v.as_str() {
        return s.parse::<f32>().ok();
    }
    None
}

impl ShadcnResolver {
    pub(super) fn render_alert<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        _key: &ElementKey,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let resolved_props = &props.props;
        let variant = resolved_props
            .get("variant")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "default" => Some(shadcn::AlertVariant::Default),
                "destructive" => Some(shadcn::AlertVariant::Destructive),
                _ => None,
            })
            .unwrap_or(shadcn::AlertVariant::Default);

        let title = resolved_props
            .get("title")
            .and_then(|v| (!v.is_null()).then(|| Self::json_to_label(Some(v))));
        let description = resolved_props
            .get("description")
            .and_then(|v| (!v.is_null()).then(|| Self::json_to_label(Some(v))));

        let mut out: Vec<AnyElement> = Vec::new();
        if let Some(title) = title {
            out.push(shadcn::AlertTitle::new(title).into_element(cx));
        }
        if let Some(description) = description {
            out.push(shadcn::AlertDescription::new(description).into_element(cx));
        }
        out.extend(children);

        shadcn::Alert::new(out).variant(variant).into_element(cx)
    }

    pub(super) fn render_spinner<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let resolved_props = &props.props;
        let size_px = resolved_props
            .get("sizePx")
            .and_then(|v| v.as_u64())
            .unwrap_or(16) as f32;
        let speed = parse_f32(resolved_props.get("speed")).unwrap_or(0.12);

        let spinner = shadcn::Spinner::new()
            .speed(speed)
            .refine_layout(
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(fret_core::Px(size_px))
                    .h_px(fret_core::Px(size_px)),
            )
            .into_element(cx);

        if children.is_empty() {
            spinner
        } else {
            fret_ui_kit::ui::v_flex(move |_cx| {
                let mut out = Vec::with_capacity(children.len().saturating_add(1));
                out.push(spinner);
                out.extend(children);
                out
            })
            .gap(fret_ui_kit::Space::N2)
            .items_start()
            .into_element(cx)
        }
    }

    pub(super) fn render_skeleton<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let resolved_props = &props.props;
        let secondary = resolved_props
            .get("secondary")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let animate_pulse = resolved_props
            .get("animatePulse")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let h_px = resolved_props
            .get("hPx")
            .and_then(|v| v.as_u64())
            .unwrap_or(16) as f32;
        let w_full = resolved_props
            .get("wFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut layout = fret_ui_kit::LayoutRefinement::default().h_px(fret_core::Px(h_px));
        if w_full {
            layout = layout.w_full();
        }

        let mut sk = shadcn::Skeleton::new()
            .animate_pulse(animate_pulse)
            .refine_layout(layout);
        if secondary {
            sk = sk.secondary();
        }
        let sk = sk.into_element(cx);

        if children.is_empty() {
            sk
        } else {
            fret_ui_kit::ui::v_flex(move |_cx| {
                let mut out = Vec::with_capacity(children.len().saturating_add(1));
                out.push(sk);
                out.extend(children);
                out
            })
            .gap(fret_ui_kit::Space::N2)
            .items_start()
            .into_element(cx)
        }
    }

    pub(super) fn render_progress<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        props: &ResolvedProps,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let resolved_props = &props.props;
        let min = parse_f32(resolved_props.get("min")).unwrap_or(0.0);
        let max = parse_f32(resolved_props.get("max")).unwrap_or(100.0);
        let v = parse_f32(resolved_props.get("value"))
            .unwrap_or(min)
            .clamp(min, max);

        let model = Self::ensure_f32_model(cx, key, v);
        let cur = cx.app.models().get_copied(&model).unwrap_or(v);
        if (cur - v).abs() > f32::EPSILON {
            let _ = cx.app.models_mut().update(&model, |m| *m = v);
        }

        let w_full = resolved_props
            .get("wFull")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let mirror_in_rtl = resolved_props
            .get("mirrorInRtl")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut layout = fret_ui_kit::LayoutRefinement::default();
        if w_full {
            layout = layout.w_full();
        }

        let progress = shadcn::Progress::new(model)
            .range(min, max)
            .mirror_in_rtl(mirror_in_rtl)
            .refine_layout(layout)
            .into_element(cx);

        if children.is_empty() {
            progress
        } else {
            fret_ui_kit::ui::v_flex(move |_cx| {
                let mut out = Vec::with_capacity(children.len().saturating_add(1));
                out.push(progress);
                out.extend(children);
                out
            })
            .gap(fret_ui_kit::Space::N2)
            .items_start()
            .into_element(cx)
        }
    }
}
