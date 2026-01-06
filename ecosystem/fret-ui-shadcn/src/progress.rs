use fret_core::{Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::progress as radix_progress;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius};

#[derive(Clone)]
enum ProgressModel {
    Determinate(Model<f32>),
    Optional(Model<Option<f32>>),
}

#[derive(Clone)]
pub struct Progress {
    model: ProgressModel,
    min: f32,
    max: f32,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Progress {
    pub fn new(model: Model<f32>) -> Self {
        Self {
            model: ProgressModel::Determinate(model),
            min: 0.0,
            max: 100.0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Creates a progress indicator with an optional value.
    ///
    /// When the value is `None`, the indicator renders as 0% (matching shadcn/ui's
    /// `value || 0` behavior).
    pub fn new_opt(model: Model<Option<f32>>) -> Self {
        Self {
            model: ProgressModel::Optional(model),
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

    fn value<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Option<f32> {
        match &self.model {
            ProgressModel::Determinate(model) => Some(cx.watch_model(model).copied().unwrap_or(self.min)),
            ProgressModel::Optional(model) => cx.watch_model(model).copied().flatten(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let height = theme
                .metric_by_key("component.progress.height")
                .unwrap_or(Px(16.0));
            let radius = theme
                .metric_by_key("component.progress.radius")
                .unwrap_or_else(|| MetricRef::radius(Radius::Full).resolve(&theme));

            let track_bg = theme.color_required("secondary");
            let fill = theme.color_required("primary");

            let track_border = theme
                .color_by_key("border")
                .or_else(|| theme.color_by_key("input"))
                .expect("missing theme token: border/input");

            let v = self.value(cx);
            let t = v
                .map(|v| radix_progress::normalize_progress(v, self.min, self.max))
                .unwrap_or(0.0);

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
                let track_w = cx
                    .last_bounds_for_element(track_id)
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

pub fn progress<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<f32>) -> AnyElement {
    Progress::new(model).into_element(cx)
}
