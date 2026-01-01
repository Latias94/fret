use std::sync::Arc;

use fret_components_icons::ids;
use fret_components_ui::declarative::action_hooks::ActionHooksExt;
use fret_components_ui::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_components_ui::declarative::icon as decl_icon;
use fret_components_ui::declarative::model_watch::ModelWatchExt as _;
use fret_components_ui::declarative::scroll as decl_scroll;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::roving_focus;
use fret_components_ui::overlay;
use fret_components_ui::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_components_ui::{
    ChromeRefinement, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    OverlayRequest, Space,
};
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PositionStyle, PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps,
    SemanticsProps, SizeStyle, TextProps,
};
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
use fret_ui::{ElementCx, Theme, UiHost};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone)]
pub struct SelectItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl SelectItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone)]
pub struct Select {
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    items: Vec<SelectItem>,
    placeholder: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
}

impl Select {
    pub fn new(model: Model<Option<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            model,
            open,
            items: Vec::new(),
            placeholder: Arc::from("Select…"),
            disabled: false,
            a11y_label: None,
        }
    }

    pub fn item(mut self, item: SelectItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        select(
            cx,
            self.model,
            self.open,
            &self.items,
            self.placeholder,
            self.disabled,
            self.a11y_label,
        )
    }
}

pub fn select<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    items: &[SelectItem],
    placeholder: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
) -> AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).clone();
        let selected = cx.watch_model(&model).cloned().unwrap_or_default();
        let is_open = cx.watch_model(&open).copied().unwrap_or(false);

        let resolved = resolve_input_chrome(
            &theme,
            fret_components_ui::Size::default(),
            &ChromeRefinement::default(),
            InputTokenKeys::none(),
        );

        let radius = resolved.radius;
        let ring = decl_style::focus_ring(&theme, radius);

        let label = selected
            .as_ref()
            .and_then(|v| items.iter().find(|it| it.value.as_ref() == v.as_ref()))
            .map(|it| it.label.clone())
            .unwrap_or(placeholder);

        let text_style = TextStyle {
            font: FontId::default(),
            size: resolved.text_px,
            weight: FontWeight::NORMAL,
            line_height: theme.metric_by_key("font.line_height").or(Some(theme.metrics.font_line_height)),
            letter_spacing_em: None,
        };

        let mut trigger_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default().w_full().min_h(MetricRef::Px(resolved.min_height)),
        );
        trigger_layout.size.height = Length::Auto;
        trigger_layout.size.min_height = Some(resolved.min_height);

        let bg = resolved.background;
        let border = resolved.border_color;
        let border_focus = resolved.border_color_focused;
        let fg = resolved.text_color;
        let fg_muted = theme
            .color_by_key("muted-foreground")
            .unwrap_or(theme.colors.text_muted);

        let enabled = !disabled;

        cx.pressable_with_id_props(|cx, st, trigger_id| {
            let border_color = if st.hovered || st.pressed {
                alpha_mul(border_focus, 0.85)
            } else {
                border
            };

            cx.pressable_toggle_bool(&open);

            let props = PressableProps {
                layout: trigger_layout,
                enabled,
                focusable: true,
                focus_ring: Some(ring),
                a11y: PressableA11y {
                    role: Some(SemanticsRole::ComboBox),
                    label: a11y_label.clone(),
                    expanded: Some(is_open),
                    ..Default::default()
                },
                ..Default::default()
            };

            let overlay_root_name = OverlayController::popover_root_name(trigger_id);

            if is_open
                && enabled
                && let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id)
            {
                    let outer = cx.bounds;

                    let max_h = theme
                        .metric_by_key("component.select.max_list_height")
                        .unwrap_or(Px(240.0));
                    let item_h = theme
                        .metric_by_key("component.select.item_height")
                        .unwrap_or(Px(32.0));
                    let desired_h = Px((item_h.0 * items.len() as f32).min(max_h.0).max(item_h.0));
                    let desired_w = Px(anchor.size.width.0.max(180.0));
                    let desired = fret_core::Size::new(desired_w, desired_h);

                    let side_offset = theme
                        .metric_by_key("component.select.popover_offset")
                        .unwrap_or(Px(6.0));

                    let placed = anchored_panel_bounds_sized(
                        outer,
                        anchor,
                        desired,
                        side_offset,
                        Side::Bottom,
                        Align::Start,
                    );

                    let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                        let selected = cx.watch_model(&model).cloned().unwrap_or_default();

                        let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
                        let labels: Vec<Arc<str>> = items.iter().map(|i| i.label.clone()).collect();
                        let disabled: Vec<bool> = items.iter().map(|i| i.disabled || !enabled).collect();

                        let active = roving_focus::active_index_from_str_keys(
                            &values,
                            selected.as_deref(),
                            &disabled,
                        );

                        let values_arc: Arc<[Arc<str>]> = Arc::from(values.into_boxed_slice());
                        let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                        let roving = RovingFocusProps {
                            enabled: true,
                            wrap: true,
                            disabled: Arc::from(disabled.clone().into_boxed_slice()),
                            ..Default::default()
                        };

                        let popover_layout = LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(placed.origin.x),
                                top: Some(placed.origin.y),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(placed.size.width),
                                height: Length::Px(placed.size.height),
                                ..Default::default()
                            },
                            overflow: Overflow::Clip,
                            ..Default::default()
                        };

                        let shadow = decl_style::shadow_sm(&theme, radius);

                        vec![cx.container(
                            ContainerProps {
                                layout: popover_layout,
                                padding: Edges::all(Px(0.0)),
                                background: Some(theme.colors.panel_background),
                                shadow: Some(shadow),
                                border: Edges::all(resolved.border_width),
                                border_color: Some(border),
                                corner_radii: Corners::all(radius),
                            },
                            |cx| {
                                vec![decl_scroll::overflow_scrollbar(
                                    cx,
                                    LayoutRefinement::default().w_full().h_full(),
                                    |cx| {
                                        vec![cx.semantics(
                                            SemanticsProps {
                                                layout: LayoutStyle::default(),
                                                role: SemanticsRole::List,
                                                ..Default::default()
                                            },
                                            |cx| {
                                                vec![cx.roving_flex(
                                                    RovingFlexProps {
                                                        flex: FlexProps {
                                                            layout: LayoutStyle::default(),
                                                            direction: fret_core::Axis::Vertical,
                                                            gap: Px(0.0),
                                                            padding: Edges::all(Px(4.0)),
                                                            justify: MainAlign::Start,
                                                            align: CrossAlign::Stretch,
                                                            wrap: false,
                                                },
                                                roving,
                                            },
                                            |cx| {
                                                cx.roving_nav_apg();
                                                cx.roving_select_option_arc_str(
                                                    &model,
                                                    values_arc.clone(),
                                                );
                                                    cx.roving_typeahead_prefix_arc_str(
                                                        labels_arc.clone(),
                                                        30,
                                                    );

                                                    let mut out = Vec::with_capacity(items.len());
                                                    let item_count = items.len();
                                                    for (idx, item) in items.iter().cloned().enumerate() {
                                                        let item_disabled = disabled.get(idx).copied().unwrap_or(true);
                                                        let tab_stop = active.is_some_and(|a| a == idx);
                                                        let is_selected = selected
                                                                .as_ref()
                                                                .is_some_and(|v| v.as_ref() == item.value.as_ref());

                                                            let item_ring = decl_style::focus_ring(&theme, theme.metrics.radius_sm);

                                                            let model = model.clone();
                                                            let open = open.clone();
                                                            let text_style = text_style.clone();

                                                            out.push(cx.pressable_with_id(
                                                                PressableProps {
                                                                    layout: {
                                                                        let mut layout = LayoutStyle::default();
                                                                        layout.size.width = Length::Fill;
                                                                        layout.size.min_height = Some(item_h);
                                                                        layout
                                                                    },
                                                                    enabled: !item_disabled,
                                                                    focusable: tab_stop,
                                                                    focus_ring: Some(item_ring),
                                                                    a11y: PressableA11y {
                                                                        role: Some(SemanticsRole::ListItem),
                                                                        label: Some(item.label.clone()),
                                                                         selected: is_selected,
                                                                         ..Default::default()
                                                                    }
                                                                    .with_collection_position(idx, item_count),
                                                                     ..Default::default()
                                                                 },
                                                                 move |cx, st, id| {
                                                                    let _ = id;

                                                                    cx.pressable_set_option_arc_str(
                                                                        &model,
                                                                        item.value.clone(),
                                                                    );
                                                                    cx.pressable_set_bool(&open, false);

                                                                    let theme = Theme::global(&*cx.app).clone();
                                                                    let mut bg = Color::TRANSPARENT;
                                                                    if is_selected {
                                                                        bg = alpha_mul(theme.colors.selection_background, 0.35);
                                                                    }
                                                                    if st.hovered || st.pressed {
                                                                        bg = alpha_mul(theme.colors.selection_background, 0.45);
                                                                    }

                                                                    let fg = if item_disabled { alpha_mul(fg_muted, 0.8) } else { fg };

                                                                    vec![cx.container(
                                                                        ContainerProps {
                                                                            layout: {
                                                                                let mut layout = LayoutStyle::default();
                                                                                layout.size.width = Length::Fill;
                                                                                layout.size.height = Length::Fill;
                                                                                layout
                                                                            },
                                                                            padding: Edges::all(Px(8.0)),
                                                                            background: Some(bg),
                                                                            shadow: None,
                                                                            border: Edges::all(Px(0.0)),
                                                                            border_color: None,
                                                                            corner_radii: Corners::all(theme.metrics.radius_sm),
                                                                        },
                                                                        |cx| {
                                                                            vec![cx.text_props(TextProps {
                                                                                layout: LayoutStyle::default(),
                                                                                text: item.label.clone(),
                                                                                style: Some(text_style.clone()),
                                                                                wrap: TextWrap::None,
                                                                                overflow: TextOverflow::Ellipsis,
                                                                                color: Some(fg),
                                                                            })]
                                                                        },
                                                                    )]
                                                                },
                                                            ));
                                                        }
                                                        out
                                                    },
                                                )]
                                            },
                                        )]
                                    },
                                )]
                            },
                        )]
                    });

                    let mut request = OverlayRequest::dismissible_popover(
                        trigger_id,
                        trigger_id,
                        open,
                        OverlayPresence::instant(true),
                        overlay_children,
                    );
                    request.root_name = Some(overlay_root_name);
                    OverlayController::request(cx, request);
            }

            let children = vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle::default(),
                    padding: resolved.padding,
                    background: Some(bg),
                    shadow: None,
                    border: Edges::all(resolved.border_width),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(radius),
                },
                |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: MetricRef::space(Space::N2).resolve(&theme),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::SpaceBetween,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        |cx| {
                            vec![
                                cx.text_props(TextProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout
                                    },
                                    text: label,
                                    style: Some(text_style.clone()),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Ellipsis,
                                    color: Some(if selected.is_some() { fg } else { fg_muted }),
                                }),
                                decl_icon::icon_with(cx, ids::ui::CHEVRON_DOWN, Some(Px(16.0)), None),
                            ]
                        },
                    )]
                },
            )];

            (props, children)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics};
    use fret_core::{PathService, PathStyle, Point, Px, Rect, SemanticsRole, Size};
    use fret_core::{SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_core::{TextStyle, UiServices};
    use fret_runtime::FrameId;
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<SelectItem>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_components_ui::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "select", |cx| {
                vec![Select::new(model, open).items(items).into_element(cx)]
            });
        ui.set_root(root);
        fret_components_ui::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn select_popover_items_have_collection_position_metadata() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the popover and verify item metadata.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListItem && n.label.as_deref() == Some("Beta"))
            .expect("Beta list item");
        assert_eq!(beta.pos_in_set, Some(2));
        assert_eq!(beta.set_size, Some(3));
    }
}
