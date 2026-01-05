use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Space};

use crate::{CommandItem, CommandList, CommandPalette, Popover, PopoverContent};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone)]
pub struct ComboboxItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl ComboboxItem {
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

#[derive(Default)]
struct ComboboxState {
    query: Option<Model<String>>,
    was_open: bool,
}

#[derive(Clone)]
pub struct Combobox {
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: Vec<ComboboxItem>,
    width: Option<Px>,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    search_enabled: bool,
}

impl Combobox {
    pub fn new(model: Model<Option<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            model,
            open,
            query: None,
            items: Vec::new(),
            width: None,
            placeholder: Arc::from("Select..."),
            search_placeholder: Arc::from("Search..."),
            empty_text: Arc::from("No results."),
            disabled: false,
            a11y_label: None,
            search_enabled: true,
        }
    }

    /// When set, applies a fixed width to both the trigger and the popover content (shadcn demo
    /// uses `w-[200px]`).
    pub fn width(mut self, width: Px) -> Self {
        self.width = Some(width);
        self
    }

    pub fn query_model(mut self, query: Model<String>) -> Self {
        self.query = Some(query);
        self
    }

    pub fn item(mut self, item: ComboboxItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ComboboxItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
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

    pub fn search_enabled(mut self, enabled: bool) -> Self {
        self.search_enabled = enabled;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        combobox(
            cx,
            self.model,
            self.open,
            self.query,
            &self.items,
            self.width,
            self.placeholder,
            self.search_placeholder,
            self.empty_text,
            self.disabled,
            self.a11y_label,
            self.search_enabled,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn combobox<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Option<Model<String>>,
    items: &[ComboboxItem],
    width: Option<Px>,
    placeholder: Arc<str>,
    search_placeholder: Arc<str>,
    empty_text: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    search_enabled: bool,
) -> AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).clone();
        let selected = cx.watch_model(&model).cloned().unwrap_or_default();
        let is_open = cx.watch_model(&open).copied().unwrap_or(false);

        let query_model = if let Some(q) = query {
            cx.with_state(ComboboxState::default, |st| st.query = Some(q.clone()));
            q
        } else {
            let existing = cx.with_state(ComboboxState::default, |st| st.query.clone());
            if let Some(m) = existing {
                m
            } else {
                let m = cx.app.models_mut().insert(String::new());
                cx.with_state(ComboboxState::default, |st| st.query = Some(m.clone()));
                m
            }
        };

        let was_open = cx.with_state(ComboboxState::default, |st| {
            let prev = st.was_open;
            st.was_open = is_open;
            prev
        });
        if was_open && !is_open {
            let _ = cx.app.models_mut().update(&query_model, |v| v.clear());
        }

        let resolved = resolve_input_chrome(
            &theme,
            fret_ui_kit::Size::default(),
            &ChromeRefinement::default(),
            InputTokenKeys::none(),
        );

        let radius = resolved.radius;
        let ring = decl_style::focus_ring(&theme, radius);

        let resolved_label = selected
            .as_ref()
            .and_then(|v| items.iter().find(|it| it.value.as_ref() == v.as_ref()))
            .map(|it| it.label.clone())
            .unwrap_or(placeholder.clone());

        let text_style = TextStyle {
            font: FontId::default(),
            size: resolved.text_px,
            weight: FontWeight::NORMAL,
            line_height: theme
                .metric_by_key("font.line_height")
                .or(Some(theme.metrics.font_line_height)),
            letter_spacing_em: None,
        };

        let mut trigger_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .min_h(MetricRef::Px(resolved.min_height))
                .merge(if let Some(w) = width {
                    LayoutRefinement::default().w_px(MetricRef::Px(w))
                } else {
                    LayoutRefinement::default().w_full()
                }),
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
        let items: Vec<ComboboxItem> = items.to_vec();
        let open_for_trigger = open.clone();
        let trigger_gap = MetricRef::space(Space::N2).resolve(&theme);
        let has_selection = selected.is_some();
        let a11y_label_for_trigger = a11y_label.clone();

        Popover::new(open.clone())
            .auto_focus(true)
            .into_element_with_anchor(
                cx,
                move |cx| {
                    cx.pressable_with_id_props(|cx, st, _trigger_id| {
                        let border_color = if st.hovered || st.pressed {
                            alpha_mul(border_focus, 0.85)
                        } else {
                            border
                        };

                        cx.pressable_toggle_bool(&open_for_trigger);

                        let props = PressableProps {
                            layout: trigger_layout,
                            enabled,
                            focusable: true,
                            focus_ring: Some(ring),
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::ComboBox),
                                label: a11y_label_for_trigger
                                    .clone()
                                    .or_else(|| Some(resolved_label.clone())),
                                expanded: Some(is_open),
                                ..Default::default()
                            },
                            ..Default::default()
                        };

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
                            move |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle::default(),
                                        direction: fret_core::Axis::Horizontal,
                                        gap: trigger_gap,
                                        padding: Edges::all(Px(0.0)),
                                        justify: MainAlign::SpaceBetween,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    move |cx| {
                                        vec![
                                            cx.text_props(TextProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Fill;
                                                    layout
                                                },
                                                text: resolved_label.clone(),
                                                style: Some(text_style.clone()),
                                                wrap: TextWrap::None,
                                                overflow: TextOverflow::Ellipsis,
                                                color: Some(if has_selection {
                                                    fg
                                                } else {
                                                    fg_muted
                                                }),
                                            }),
                                            decl_icon::icon_with(
                                                cx,
                                                ids::ui::CHEVRON_DOWN,
                                                Some(Px(16.0)),
                                                None,
                                            ),
                                        ]
                                    },
                                )]
                            },
                        )];

                        (props, children)
                    })
                },
                move |cx, anchor| {
                    let max_h = theme
                        .metric_by_key("component.combobox.max_list_height")
                        .or_else(|| theme.metric_by_key("component.select.max_list_height"))
                        .unwrap_or(Px(280.0));
                    let desired_w = width.unwrap_or_else(|| Px(anchor.size.width.0.max(180.0)));

                    let transparent = Color::TRANSPARENT;
                    let list = if search_enabled {
                        let mut command_items: Vec<CommandItem> = Vec::with_capacity(items.len());
                        for item in items.iter().cloned() {
                            let item_disabled = disabled || item.disabled;
                            let is_selected = selected
                                .as_ref()
                                .is_some_and(|v| v.as_ref() == item.value.as_ref());

                            let model_for_select = model.clone();
                            let open_for_select = open.clone();
                            let query_for_select = query_model.clone();
                            let value_for_select = item.value.clone();
                            let on_select: fret_ui::action::OnActivate =
                                Arc::new(move |host, action_cx, _reason| {
                                    let _ = host.models_mut().update(&model_for_select, |v| {
                                        if v.as_ref().is_some_and(|cur| {
                                            cur.as_ref() == value_for_select.as_ref()
                                        }) {
                                            *v = None;
                                        } else {
                                            *v = Some(value_for_select.clone());
                                        }
                                    });
                                    let _ =
                                        host.models_mut().update(&open_for_select, |v| *v = false);
                                    let _ =
                                        host.models_mut().update(&query_for_select, |v| v.clear());
                                    host.request_redraw(action_cx.window);
                                });

                            command_items.push(
                                CommandItem::new(item.label.clone())
                                    .value(item.value.clone())
                                    .disabled(item_disabled)
                                    .checkmark(is_selected)
                                    .on_select_action(on_select),
                            );
                        }

                        CommandPalette::new(query_model.clone(), command_items)
                            .a11y_label("Combobox list")
                            .input_role(SemanticsRole::ComboBox)
                            .input_expanded(true)
                            .placeholder(search_placeholder.clone())
                            .disabled(disabled)
                            .empty_text(empty_text)
                            .refine_style(ChromeRefinement {
                                radius: Some(MetricRef::Px(Px(0.0))),
                                border_width: Some(MetricRef::Px(Px(0.0))),
                                background: Some(ColorRef::Color(transparent)),
                                border_color: Some(ColorRef::Color(transparent)),
                                ..Default::default()
                            })
                            .refine_scroll_layout(
                                LayoutRefinement::default().max_h(MetricRef::Px(max_h)),
                            )
                            .into_element(cx)
                    } else {
                        let fg = theme
                            .color_by_key("foreground")
                            .unwrap_or(theme.colors.text_primary);
                        let fg_disabled = theme.colors.text_disabled;
                        let item_text_style = crate::command::item_text_style(&theme);

                        let mut command_items: Vec<CommandItem> = Vec::with_capacity(items.len());
                        for item in items.iter().cloned() {
                            let item_disabled = disabled || item.disabled;
                            let is_selected = selected
                                .as_ref()
                                .is_some_and(|v| v.as_ref() == item.value.as_ref());

                            let model_for_select = model.clone();
                            let open_for_select = open.clone();
                            let query_for_select = query_model.clone();
                            let value_for_select = item.value.clone();
                            let on_select: fret_ui::action::OnActivate =
                                Arc::new(move |host, action_cx, _reason| {
                                    let _ = host.models_mut().update(&model_for_select, |v| {
                                        if v.as_ref().is_some_and(|cur| {
                                            cur.as_ref() == value_for_select.as_ref()
                                        }) {
                                            *v = None;
                                        } else {
                                            *v = Some(value_for_select.clone());
                                        }
                                    });
                                    let _ =
                                        host.models_mut().update(&open_for_select, |v| *v = false);
                                    let _ =
                                        host.models_mut().update(&query_for_select, |v| v.clear());
                                    host.request_redraw(action_cx.window);
                                });

                            let label_text = item.label.clone();
                            let icon = decl_icon::icon_with(
                                cx,
                                ids::ui::CHECK,
                                Some(Px(16.0)),
                                Some(ColorRef::Color(if item_disabled {
                                    fg_disabled
                                } else {
                                    fg
                                })),
                            );
                            let icon = cx
                                .opacity(if is_selected { 1.0 } else { 0.0 }, move |_cx| {
                                    vec![icon]
                                });

                            let text = cx.text_props(TextProps {
                                layout: LayoutStyle::default(),
                                text: label_text.clone(),
                                style: Some(item_text_style.clone()),
                                color: Some(if item_disabled { fg_disabled } else { fg }),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Ellipsis,
                            });

                            command_items.push(
                                CommandItem::new(label_text)
                                    .value(item.value.clone())
                                    .disabled(item_disabled)
                                    .on_select_action(on_select)
                                    .children(vec![text, icon]),
                            );
                        }

                        CommandList::new(command_items)
                            .disabled(disabled)
                            .empty_text(empty_text)
                            .refine_scroll_layout(
                                LayoutRefinement::default().max_h(MetricRef::Px(max_h)),
                            )
                            .into_element(cx)
                    };

                    PopoverContent::new(vec![list])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_px(MetricRef::Px(desired_w))
                                .min_w_0(),
                        )
                        .into_element(cx)
                },
            )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Point, Px, Rect, SemanticsRole, Size, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, TextStyle, UiServices,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
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
        items: Vec<ComboboxItem>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "combobox",
            |cx| vec![Combobox::new(model, open).items(items).into_element(cx)],
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn combobox_search_input_exposes_combobox_role_active_descendant_and_value() {
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
            ComboboxItem::new("alpha", "Alpha"),
            ComboboxItem::new("beta", "Beta"),
            ComboboxItem::new("gamma", "Gamma"),
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

        // Second frame: open the popover.
        //
        // `active_descendant` depends on stable element<->node mapping, so we render one extra
        // frame before asserting it (see cmdk tests).
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

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.value.is_some())
            .expect("combobox search input node");
        assert!(
            input.flags.expanded,
            "combobox search input should report expanded=true while open"
        );
        assert_eq!(input.value.as_deref(), Some(""));

        let active = input
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");
        assert_eq!(active_node.role, SemanticsRole::ListBoxOption);

        let input_id = input.id;
        ui.set_focus(Some(input_id));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::TextInput("a".to_string()),
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|n| n.id == input_id)
            .expect("combobox search input node after typing");
        assert_eq!(input.role, SemanticsRole::ComboBox);
        assert_eq!(input.value.as_deref(), Some("a"));
    }
}
