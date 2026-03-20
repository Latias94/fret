use std::sync::Arc;

use crate::direction::LayoutDirection;
use crate::float_value_model::IntoFloatValueModel;
use crate::float_vec_model::IntoFloatVecModel;
use crate::optional_float_value_model::IntoOptionalFloatValueModel;
use fret_core::{Edges, Px, SemanticsOrientation, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, FractionalRenderTransformProps, LayoutStyle, Length, SemanticsDecoration,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion as decl_motion;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::progress as radix_progress;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius};

use crate::overlay_motion;

const SHADCN_PROGRESS_TRANSITION_EASE: fret_ui_kit::headless::easing::CubicBezier =
    fret_ui_kit::headless::easing::CubicBezier::new(0.4, 0.0, 0.2, 1.0);

fn shadcn_progress_transition_ease(t: f32) -> f32 {
    SHADCN_PROGRESS_TRANSITION_EASE.sample(t)
}

#[derive(Clone)]
enum ProgressModel {
    Determinate(Model<f32>),
    Optional(Model<Option<f32>>),
    ValuesFirst(Model<Vec<f32>>),
    Value(Option<f32>),
}

#[derive(Clone)]
pub struct Progress {
    model: ProgressModel,
    min: f32,
    max: f32,
    mirror_in_rtl: bool,
    a11y_label: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Progress {
    pub fn new(model: impl IntoFloatValueModel) -> Self {
        Self {
            model: ProgressModel::Determinate(model.into_float_value_model()),
            min: 0.0,
            max: 100.0,
            mirror_in_rtl: false,
            a11y_label: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Creates a progress indicator from a plain snapshot value, mirroring the upstream `value`
    /// prop without forcing a `Model<f32>` at the call site.
    pub fn from_value(value: f32) -> Self {
        Self::from_optional_value(Some(value))
    }

    /// Creates a progress indicator from an optional snapshot value.
    ///
    /// This mirrors upstream `value?: number | null`, where `None` keeps the bar visually at 0%
    /// while omitting numeric semantics for indeterminate progress.
    pub fn from_optional_value(value: Option<f32>) -> Self {
        Self {
            model: ProgressModel::Value(value),
            min: 0.0,
            max: 100.0,
            mirror_in_rtl: false,
            a11y_label: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Creates a progress indicator with an optional value.
    ///
    /// When the value is `None`, the indicator renders as 0% (matching shadcn/ui's
    /// `value || 0` behavior).
    pub fn new_opt(model: impl IntoOptionalFloatValueModel) -> Self {
        Self {
            model: ProgressModel::Optional(model.into_optional_float_value_model()),
            min: 0.0,
            max: 100.0,
            mirror_in_rtl: false,
            a11y_label: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Creates a progress indicator driven by the first entry in a values model.
    ///
    /// This is primarily useful for parity with slider-style value models (`Vec<f32>`).
    pub fn new_values_first(model: impl IntoFloatVecModel) -> Self {
        Self {
            model: ProgressModel::ValuesFirst(model.into_float_vec_model()),
            min: 0.0,
            max: 100.0,
            mirror_in_rtl: false,
            a11y_label: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
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
            ProgressModel::Value(value) => *value,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let motion_key = match &self.model {
            ProgressModel::Determinate(model) => Some(("determinate", model.id())),
            ProgressModel::Optional(model) => Some(("optional", model.id())),
            ProgressModel::ValuesFirst(model) => Some(("values_first", model.id())),
            ProgressModel::Value(_) => None,
        };

        match motion_key {
            Some(motion_key) => cx.keyed(("shadcn-progress", motion_key), |cx| {
                self.into_element_scoped(cx)
            }),
            None => self.into_element_scoped(cx),
        }
    }

    fn into_element_scoped<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let mirror_in_rtl = self.mirror_in_rtl;
            let a11y_label = self.a11y_label.clone();
            let theme = Theme::global(&*cx.app).snapshot();
            let height = theme
                .metric_by_key("component.progress.height")
                .unwrap_or(Px(8.0));
            let radius = theme
                .metric_by_key("component.progress.radius")
                .unwrap_or_else(|| MetricRef::radius(Radius::Full).resolve(&theme));
            let radius = Px(radius.0.min(height.0 * 0.5));

            // shadcn v4 Progress uses `bg-primary/20` for the track.
            let mut track_bg = theme.color_token("primary");
            track_bg.a *= 0.2;
            let fill = theme.color_token("primary");

            let v = self.value(cx);
            let t = v
                .map(|v| radix_progress::normalize_progress(v, self.min, self.max))
                .unwrap_or(0.0);
            let value_text =
                v.and_then(|value| default_progress_value_label(value, self.min, self.max));

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

            let mut out = cx.container(track_props, move |cx| {
                // Match the upstream DOM structure:
                // - the indicator is full-width (`w-full`)
                // - it is shifted with a translate so the left edge is clipped by the track's
                //   `overflow-hidden`, keeping the right edge rounded.
                let motion_owner = cx.root_id();
                let dir = crate::direction::use_direction(cx, None);
                let translate_x_fraction_target = if mirror_in_rtl && dir == LayoutDirection::Rtl {
                    1.0 - t
                } else {
                    t - 1.0
                };

                // Upstream shadcn/ui uses `transition-all` on the indicator, so value changes
                // animate the translate transform (Tailwind default easing/duration).
                let duration = overlay_motion::shadcn_motion_duration_150(cx);
                let translate_x_fraction = decl_motion::drive_tween_f32_for_element(
                    cx,
                    motion_owner,
                    "indicator-translate-x",
                    translate_x_fraction_target,
                    duration,
                    shadcn_progress_transition_ease,
                )
                .value;

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
                                padding: Edges::all(Px(0.0)).into(),
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
            });

            let mut semantics = SemanticsDecoration::default()
                .role(SemanticsRole::ProgressBar)
                .orientation(SemanticsOrientation::Horizontal);
            if let Some(label) = a11y_label {
                semantics = semantics.label(label);
            }
            if self.min.is_finite() && self.max.is_finite() {
                semantics = semantics.numeric_range(self.min as f64, self.max as f64);
            }
            if let Some(value_text) = value_text {
                semantics = semantics.value(value_text);
            }
            if let Some(value) = v
                && value.is_finite()
            {
                semantics = semantics.numeric_value(value as f64);
            }
            out = out.attach_semantics(semantics);
            out
        })
    }
}

fn default_progress_value_label(value: f32, min: f32, max: f32) -> Option<Arc<str>> {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() {
        return None;
    }
    let span = max - min;
    if !span.is_finite() || span.abs() <= f32::EPSILON {
        return None;
    }
    let percent = (radix_progress::normalize_progress(value, min, max) * 100.0).round() as i32;
    Some(Arc::from(format!("{percent}%")))
}

pub fn progress<H: UiHost, M: IntoFloatValueModel>(model: M) -> impl IntoUiElement<H> + use<H, M> {
    Progress::new(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::direction::LayoutDirection;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size, WindowFrameClockService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::element::ElementKind;

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
    fn progress_opt_none_matches_shadcn_value_or_zero_and_stamps_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let model = cx.app.models_mut().insert(None::<f32>);
            let el = Progress::new_opt(model).into_element(cx);

            // shadcn/ui: `transform: translateX(-${100 - (value || 0)}%)`
            // When value is None, treat it as 0%.
            let tx = find_translate_x_fraction(&el).expect("translate_x_fraction");
            assert!(
                (tx + 1.0).abs() <= 1e-6,
                "expected translate fraction -1.0, got {tx}"
            );

            let semantics = el
                .semantics_decoration
                .as_ref()
                .expect("semantics decoration");
            assert_eq!(semantics.role, Some(SemanticsRole::ProgressBar));
            assert_eq!(
                semantics.orientation,
                Some(SemanticsOrientation::Horizontal),
                "expected progress bar to be horizontal by default"
            );
            assert_eq!(semantics.min_numeric_value, Some(0.0));
            assert_eq!(semantics.max_numeric_value, Some(100.0));
            assert_eq!(
                semantics.numeric_value, None,
                "expected None value to omit numeric_value"
            );
            assert_eq!(semantics.value.as_deref(), None);
        });
    }

    #[test]
    fn progress_from_value_stamps_numeric_value_and_default_value_text() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let el = Progress::from_value(42.0).into_element(cx);

            let semantics = el
                .semantics_decoration
                .as_ref()
                .expect("semantics decoration");
            assert_eq!(semantics.role, Some(SemanticsRole::ProgressBar));
            assert_eq!(semantics.numeric_value, Some(42.0));
            assert_eq!(semantics.value.as_deref(), Some("42%"));
        });
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

            let el = crate::direction::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                Progress::new(model.clone())
                    .mirror_in_rtl(true)
                    .into_element(cx)
            });
            let tx = find_translate_x_fraction(&el).expect("translate_x_fraction (rtl)");
            assert!((tx - 0.75).abs() <= 1e-6);
        });
    }

    #[test]
    fn progress_translate_animates_on_value_change() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let model = app.models_mut().insert(0.0f32);

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(std::time::Duration::from_millis(16)));
        });
        for fid in [FrameId(1), FrameId(2)] {
            app.set_frame_id(fid);
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
        }

        fn render_progress(app: &mut App, window: AppWindowId, model: Model<f32>) -> AnyElement {
            fret_ui::elements::with_element_cx(app, window, bounds(), "p_anim", |cx| {
                Progress::new(model).into_element(cx)
            })
        }

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));
        app.with_global_mut(WindowFrameClockService::default, |svc, app| {
            svc.record_frame(window, app.frame_id());
        });
        let el0 = render_progress(&mut app, window, model.clone());
        let tx0 = find_translate_x_fraction(&el0).expect("translate_x_fraction");
        assert!((tx0 + 1.0).abs() <= 1e-6, "expected tx=-1.0, got {tx0}");

        let _ = app.models_mut().update(&model, |v| *v = 100.0);

        // First render after retarget: tween has not advanced yet (advances at most once per
        // frame). In the headless authoring harness, `with_element_cx` constructs a full frame
        // context, so the tween will advance on the first post-retarget frame; assert we see
        // intermediate motion instead of snapping.
        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));
        app.with_global_mut(WindowFrameClockService::default, |svc, app| {
            svc.record_frame(window, app.frame_id());
        });
        let el1 = render_progress(&mut app, window, model.clone());
        let tx1 = find_translate_x_fraction(&el1).expect("translate_x_fraction after retarget");
        assert!(
            tx1 > tx0 + 1e-6 && tx1 < -1e-3,
            "expected tx to advance without snapping; start={tx0} now={tx1}"
        );

        let mut seen_mid = false;
        let mut tx_last = tx1;
        for i in 0..40u64 {
            app.set_tick_id(TickId(3 + i));
            app.set_frame_id(FrameId(3 + i));
            app.with_global_mut(WindowFrameClockService::default, |svc, app| {
                svc.record_frame(window, app.frame_id());
            });
            let el = render_progress(&mut app, window, model.clone());
            let tx = find_translate_x_fraction(&el).expect("translate_x_fraction during tween");

            if tx > -0.99 && tx < -0.01 {
                seen_mid = true;
            }

            // Should progress towards 0.0 (less negative) without oscillation.
            assert!(
                tx + 0.05 >= tx_last,
                "expected monotonic tx; last={tx_last} now={tx}"
            );
            tx_last = tx;

            if (tx - 0.0).abs() <= 1e-3 {
                break;
            }
        }

        assert!(
            seen_mid,
            "expected to observe intermediate translate values"
        );
        assert!(
            (tx_last - 0.0).abs() <= 1e-3,
            "expected to settle at tx=0, got {tx_last}"
        );
    }
}
