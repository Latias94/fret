use fret_core::{Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, FractionalRenderTransformProps, LayoutStyle, Length};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::progress as radix_progress;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius};

#[derive(Clone)]
enum ProgressModel {
    Determinate(Model<f32>),
    Optional(Model<Option<f32>>),
    ValuesFirst(Model<Vec<f32>>),
}

#[derive(Clone)]
pub struct Progress {
    model: ProgressModel,
    min: f32,
    max: f32,
    mirror_in_rtl: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Progress {
    pub fn new(model: Model<f32>) -> Self {
        Self {
            model: ProgressModel::Determinate(model),
            min: 0.0,
            max: 100.0,
            mirror_in_rtl: false,
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
            mirror_in_rtl: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Creates a progress indicator driven by the first entry in a values model.
    ///
    /// This is primarily useful for parity with slider-style value models (`Vec<f32>`).
    pub fn new_values_first(model: Model<Vec<f32>>) -> Self {
        Self {
            model: ProgressModel::ValuesFirst(model),
            min: 0.0,
            max: 100.0,
            mirror_in_rtl: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Mirror the fill direction when rendered under an RTL direction provider.
    ///
    /// This models upstream Tailwind recipes like `rtl:rotate-180` without requiring a
    /// layout-time rotation transform.
    pub fn mirror_in_rtl(mut self, mirror_in_rtl: bool) -> Self {
        self.mirror_in_rtl = mirror_in_rtl;
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
            ProgressModel::Determinate(model) => {
                Some(cx.watch_model(model).copied().unwrap_or(self.min))
            }
            ProgressModel::Optional(model) => cx.watch_model(model).copied().flatten(),
            ProgressModel::ValuesFirst(model) => cx
                .watch_model(model)
                .layout()
                .read_ref(|values| values.first().copied())
                .ok()
                .flatten(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let mirror_in_rtl = self.mirror_in_rtl;
            let theme = Theme::global(&*cx.app).clone();
            let height = theme
                .metric_by_key("component.progress.height")
                .unwrap_or(Px(8.0));
            let radius = theme
                .metric_by_key("component.progress.radius")
                .unwrap_or_else(|| MetricRef::radius(Radius::Full).resolve(&theme));
            let radius = Px(radius.0.min(height.0 * 0.5));

            // shadcn v4 Progress uses `bg-primary/20` for the track.
            let mut track_bg = theme.color_required("primary");
            track_bg.a *= 0.2;
            let fill = theme.color_required("primary");

            let v = self.value(cx);
            let t = v
                .map(|v| radix_progress::normalize_progress(v, self.min, self.max))
                .unwrap_or(0.0);

            let base_layout = LayoutRefinement::default()
                .w_full()
                .h_px(height)
                .overflow_hidden()
                .merge(self.layout);

            let mut track_props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(track_bg))
                    .rounded(Radius::Full)
                    .merge(self.chrome),
                base_layout,
            );

            // `container_props` uses a resolved radius; override with `component.progress.radius` if present.
            track_props.corner_radii = fret_core::Corners::all(radius);

            cx.container(track_props, move |cx| {
                // Match the upstream DOM structure:
                // - the indicator is full-width (`w-full`)
                // - it is shifted with a translate so the left edge is clipped by the track's
                //   `overflow-hidden`, keeping the right edge rounded.
                let dir = direction_prim::use_direction_in_scope(cx, None);
                let translate_x_fraction =
                    if mirror_in_rtl && dir == direction_prim::LayoutDirection::Rtl {
                        1.0 - t
                    } else {
                        t - 1.0
                    };

                let mut transform_layout = LayoutStyle::default();
                transform_layout.size.width = Length::Fill;
                transform_layout.size.height = Length::Fill;

                vec![cx.fractional_render_transform_props(
                    FractionalRenderTransformProps {
                        layout: transform_layout,
                        translate_x_fraction,
                        translate_y_fraction: 0.0,
                    },
                    move |cx| {
                        let mut fill_layout = LayoutStyle::default();
                        fill_layout.size.width = Length::Fill;
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
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        )]
                    },
                )]
            })
        })
    }
}

pub fn progress<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<f32>) -> AnyElement {
    Progress::new(model).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::ElementKind;
    use fret_ui_kit::primitives::direction as direction_prim;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        )
    }

    fn find_translate_x_fraction(el: &AnyElement) -> Option<f32> {
        match &el.kind {
            ElementKind::FractionalRenderTransform(props) => Some(props.translate_x_fraction),
            _ => el.children.iter().find_map(find_translate_x_fraction),
        }
    }

    #[test]
    fn progress_values_first_reads_first_entry() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let values = cx.app.models_mut().insert(vec![42.0]);
            let empty = cx.app.models_mut().insert(Vec::<f32>::new());

            let p = Progress::new_values_first(values);
            assert_eq!(p.value(cx), Some(42.0));

            let p = Progress::new_values_first(empty);
            assert_eq!(p.value(cx), None);
        });
    }

    #[test]
    fn progress_mirror_in_rtl_flips_translate_fraction() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let model = cx.app.models_mut().insert(25.0f32);
            let el = Progress::new(model.clone())
                .mirror_in_rtl(true)
                .into_element(cx);
            let tx = find_translate_x_fraction(&el).expect("translate_x_fraction");
            assert!((tx + 0.75).abs() <= 1e-6);

            let el = direction_prim::with_direction_provider(
                cx,
                direction_prim::LayoutDirection::Rtl,
                |cx| {
                    Progress::new(model.clone())
                        .mirror_in_rtl(true)
                        .into_element(cx)
                },
            );
            let tx = find_translate_x_fraction(&el).expect("translate_x_fraction (rtl)");
            assert!((tx - 0.75).abs() <= 1e-6);
        });
    }
}
