use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius};
use fret_core::{Edges, Px};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{AnyElement, LayoutStyle, Length};
use fret_ui::{ElementCx, Theme, UiHost, elements};

#[derive(Clone)]
pub struct Progress {
    model: Model<f32>,
    min: f32,
    max: f32,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Progress {
    pub fn new(model: Model<f32>) -> Self {
        Self {
            model,
            min: 0.0,
            max: 100.0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    fn normalized(&self, v: f32) -> f32 {
        let span = self.max - self.min;
        if !span.is_finite() || span.abs() <= f32::EPSILON {
            return 0.0;
        }
        ((v - self.min) / span).clamp(0.0, 1.0)
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            cx.observe_model(self.model, Invalidation::Paint);

            let theme = Theme::global(&*cx.app).clone();
            let height = theme
                .metric_by_key("component.progress.height")
                .unwrap_or(Px(16.0));
            let radius = theme
                .metric_by_key("component.progress.radius")
                .unwrap_or_else(|| MetricRef::radius(Radius::Full).resolve(&theme));

            let track_bg = theme
                .color_by_key("secondary")
                .or_else(|| theme.color_by_key("muted"))
                .unwrap_or(theme.colors.panel_background);
            let fill = theme
                .color_by_key("primary")
                .or_else(|| theme.color_by_key("accent"))
                .unwrap_or(theme.colors.accent);

            let track_border = theme
                .color_by_key("border")
                .or_else(|| theme.color_by_key("input"))
                .unwrap_or(theme.colors.panel_border);

            let v = cx.app.models().get(self.model).copied().unwrap_or(self.min);
            let t = self.normalized(v);

            let base_layout = LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(height))
                .overflow_hidden()
                .merge(self.layout);

            let mut track_props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(track_bg))
                    .border_1()
                    .border_color(ColorRef::Color(track_border))
                    .rounded(Radius::Full)
                    .merge(self.chrome),
                base_layout,
            );

            // `container_props` uses a resolved radius; override with `component.progress.radius` if present.
            track_props.corner_radii = fret_core::Corners::all(radius);

            cx.container(track_props, move |cx| {
                let track_id = cx.root_id();
                let track_w = elements::bounds_for_element(&mut *cx.app, cx.window, track_id)
                    .map(|r| r.size.width)
                    .unwrap_or(Px(0.0));

                let fill_w = Px((track_w.0 * t).max(0.0));

                let mut fill_layout = LayoutStyle::default();
                fill_layout.size.width = Length::Px(fill_w);
                fill_layout.size.height = Length::Fill;

                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: fill_layout,
                        padding: Edges::all(Px(0.0)),
                        background: Some(fill),
                        shadow: None,
                        border: Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: fret_core::Corners::all(radius),
                    },
                    |_cx| Vec::new(),
                )]
            })
        })
    }
}

pub fn progress<H: UiHost>(cx: &mut ElementCx<'_, H>, model: Model<f32>) -> AnyElement {
    Progress::new(model).into_element(cx)
}
