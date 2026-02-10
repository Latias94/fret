use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::OnHoverChange;
use fret_ui::element::{
    CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, PressableA11y, PressableProps,
    SemanticsProps, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::controllable_state::use_controllable_model;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

use crate::test_id::attach_test_id;

fn star_test_id(index_1_based: u8) -> Arc<str> {
    Arc::<str>::from(format!("shadcn-extras.rating.star-{index_1_based}"))
}

fn star_label(index_1_based: u8) -> Arc<str> {
    let plural = if index_1_based == 1 { "" } else { "s" };
    Arc::<str>::from(format!("{index_1_based} star{plural}"))
}

/// A keyboard-first, shadcn-styled star rating control inspired by Kibo's shadcn blocks.
///
/// Notes:
/// - Selection is stored in a `Model<u8>` (0 = unset, 1..=count = selected).
/// - Hover only affects chrome (preview); it does not change the stored selection.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/rating`
#[derive(Clone)]
pub struct Rating {
    model: Option<Model<u8>>,
    default_value: u8,
    count: u8,
    read_only: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rating")
            .field("model", &self.model.is_some())
            .field("default_value", &self.default_value)
            .field("count", &self.count)
            .field("read_only", &self.read_only)
            .field("test_id", &self.test_id)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Rating {
    pub fn new(model: Model<u8>) -> Self {
        Self {
            model: Some(model),
            default_value: 0,
            count: 5,
            read_only: false,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn uncontrolled(default_value: u8) -> Self {
        Self {
            model: None,
            default_value,
            count: 5,
            read_only: false,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn default_value(mut self, default_value: u8) -> Self {
        self.default_value = default_value;
        self
    }

    pub fn count(mut self, count: u8) -> Self {
        self.count = count.max(1);
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> fret_ui::element::AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let model = use_controllable_model(cx, self.model, || self.default_value).model();
            let selected = cx.watch_model(&model).layout().copied().unwrap_or(0);

            let hover = use_controllable_model(cx, None::<Model<Option<u8>>>, || None).model();
            let hover_value: Option<u8> = cx.watch_model(&hover).layout().copied().flatten();

            let count = self.count.max(1);
            let count_usize = usize::from(count);
            let selected_clamped = selected.min(count);
            let active_count = hover_value.unwrap_or(selected_clamped).min(count);

            let tab_stop = if selected_clamped > 0 {
                selected_clamped.saturating_sub(1)
            } else {
                0
            };

            let icon_size = theme
                .metric_by_key("component.rating.icon_px")
                .unwrap_or(Px(20.0));
            let gap = theme
                .metric_by_key("component.rating.gap")
                .unwrap_or_else(|| decl_style::space(&theme, Space::N0p5));

            let fg_active = theme.color_required("foreground");
            let fg_inactive = theme.color_required("muted-foreground");

            let root_layout = decl_style::layout_style(&theme, self.layout);
            let test_id = self
                .test_id
                .clone()
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.rating"));

            let read_only = self.read_only;
            let model_for_items = model.clone();
            let hover_for_items = hover.clone();

            let semantics = SemanticsProps {
                role: SemanticsRole::RadioGroup,
                label: Some(Arc::<str>::from("Rating")),
                disabled: read_only,
                ..Default::default()
            };

            let el = cx.semantics(semantics, move |cx| {
                let flex = FlexProps {
                    layout: root_layout,
                    direction: fret_core::Axis::Horizontal,
                    gap,
                    padding: fret_core::Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                };

                let mut disabled: Vec<bool> = Vec::with_capacity(count_usize);
                disabled.resize(count_usize, read_only);
                let disabled: Arc<[bool]> = Arc::from(disabled.into_boxed_slice());

                vec![cx.roving_flex(
                    fret_ui::element::RovingFlexProps {
                        flex,
                        roving: fret_ui::element::RovingFocusProps {
                            enabled: !read_only,
                            wrap: true,
                            disabled,
                        },
                    },
                    move |cx| {
                        cx.roving_nav_apg();

                        if !read_only {
                            let model = model_for_items.clone();
                            let hover = hover_for_items.clone();
                            cx.roving_on_active_change(Arc::new(move |host, action_cx, idx| {
                                let next = idx.saturating_add(1);
                                let Ok(next) = u8::try_from(next) else {
                                    return;
                                };
                                let _ = host.models_mut().update(&hover, |v| *v = None);
                                let current = host.models_mut().get_cloned(&model).unwrap_or(0);
                                if current != next {
                                    let _ = host.models_mut().update(&model, |v| *v = next);
                                    host.request_redraw(action_cx.window);
                                }
                            }));
                        }

                        let mut out = Vec::with_capacity(count_usize);
                        for idx in 0..count_usize {
                            let index_1_based =
                                u8::try_from(idx.saturating_add(1)).unwrap_or(u8::MAX);

                            let hover = hover_for_items.clone();
                            let hover_target = index_1_based;
                            let hover_hook: OnHoverChange = Arc::new(move |host, action_cx, on| {
                                let current = host.models_mut().get_cloned(&hover).unwrap_or(None);
                                if on {
                                    let next = Some(hover_target);
                                    if current != next {
                                        let _ = host.models_mut().update(&hover, |v| *v = next);
                                        host.request_redraw(action_cx.window);
                                    }
                                } else if current == Some(hover_target) {
                                    let _ = host.models_mut().update(&hover, |v| *v = None);
                                    host.request_redraw(action_cx.window);
                                }
                            });

                            let model = model_for_items.clone();
                            let value = index_1_based;
                            let tab_stop_now = idx == usize::from(tab_stop);
                            let is_active = index_1_based <= active_count;
                            let is_checked = index_1_based == selected_clamped;

                            let star = cx.pressable_with_id_props(move |cx, st, _id| {
                                let theme = Theme::global(&*cx.app).clone();
                                if !read_only {
                                    cx.pressable_on_hover_change(hover_hook.clone());
                                    cx.pressable_set_model(&model, value);
                                }

                                let focusable_now = !read_only && (tab_stop_now || st.focused);
                                let ring_radius = Px(icon_size.0 * 0.5);

                                let props = PressableProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Auto,
                                            height: Length::Auto,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    enabled: !read_only,
                                    focusable: focusable_now,
                                    focus_ring: Some(decl_style::focus_ring(&theme, ring_radius)),
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::RadioButton),
                                        label: Some(star_label(index_1_based)),
                                        checked: Some(is_checked),
                                        test_id: Some(star_test_id(index_1_based)),
                                        ..Default::default()
                                    }
                                    .with_collection_position(idx, count_usize),
                                    ..Default::default()
                                };

                                let color = if is_active { fg_active } else { fg_inactive };
                                let icon = decl_icon::icon_with(
                                    cx,
                                    fret_icons::IconId::new_static("lucide.star"),
                                    Some(icon_size),
                                    Some(ColorRef::Color(color)),
                                );
                                let chrome = ChromeRefinement::default()
                                    .p(Space::N0p5)
                                    .rounded(Radius::Full);
                                let container = cx.container(
                                    decl_style::container_props(
                                        &theme,
                                        chrome,
                                        LayoutRefinement::default(),
                                    ),
                                    |_cx| vec![icon],
                                );

                                (props, vec![container])
                            });

                            out.push(star);
                        }
                        out
                    },
                )]
            });

            attach_test_id(el, test_id)
        })
    }
}
