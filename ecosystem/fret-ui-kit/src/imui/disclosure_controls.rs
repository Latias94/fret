use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, KeyCode, MouseButton, Px, SemanticsRole};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    Overflow, PressableA11y, PressableProps, SizeStyle, SpacerProps, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use super::{
    CollapsingHeaderOptions, DisclosureResponse, ImUiFacade, TreeNodeOptions, UiWriterImUiFacadeExt,
};
use crate::declarative::ModelWatchExt;
use crate::primitives::collapsible as radix_collapsible;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisclosureKind {
    CollapsingHeader,
    TreeNode,
}

#[derive(Debug, Clone)]
struct DisclosureSpec {
    kind: DisclosureKind,
    label: Arc<str>,
    enabled: bool,
    open: Option<fret_runtime::Model<bool>>,
    default_open: bool,
    activate_shortcut: Option<fret_runtime::KeyChord>,
    shortcut_repeat: bool,
    selected: bool,
    leaf: bool,
    level: u32,
    pos_in_set: Option<u32>,
    set_size: Option<u32>,
    root_test_id: Option<Arc<str>>,
    header_test_id: Option<Arc<str>>,
    content_test_id: Option<Arc<str>>,
}

impl DisclosureSpec {
    fn collapsing_header(label: Arc<str>, options: CollapsingHeaderOptions) -> Self {
        Self {
            kind: DisclosureKind::CollapsingHeader,
            label,
            enabled: options.enabled,
            open: options.open,
            default_open: options.default_open,
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
            selected: false,
            leaf: false,
            level: 1,
            pos_in_set: None,
            set_size: None,
            root_test_id: options.test_id,
            header_test_id: options.header_test_id,
            content_test_id: options.content_test_id,
        }
    }

    fn tree_node(label: Arc<str>, options: TreeNodeOptions) -> Self {
        let level = options.level.max(1);
        Self {
            kind: DisclosureKind::TreeNode,
            label,
            enabled: options.enabled,
            open: options.open,
            default_open: options.default_open,
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
            selected: options.selected,
            leaf: options.leaf,
            level,
            pos_in_set: options.pos_in_set,
            set_size: options.set_size,
            root_test_id: None,
            header_test_id: options.test_id,
            content_test_id: options.content_test_id,
        }
    }

    fn has_children(&self) -> bool {
        !self.leaf
    }
}

pub(super) fn collapsing_header_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: CollapsingHeaderOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    disclosure_with_options(ui, id, DisclosureSpec::collapsing_header(label, options), f)
}

pub(super) fn tree_node_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: TreeNodeOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    disclosure_with_options(ui, id, DisclosureSpec::tree_node(label, options), f)
}

fn disclosure_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    spec: DisclosureSpec,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let mut response = DisclosureResponse::default();

    let element = ui.with_cx_mut(|cx| {
        let scope_key = format!("fret-ui-kit.imui.disclosure.{id}");
        cx.named(scope_key.as_str(), |cx| {
            let trigger_response = &mut response.trigger;
            let root = radix_collapsible::CollapsibleRoot::new()
                .open(spec.open.clone())
                .default_open(spec.default_open);
            let open_model = root.use_open_model(cx).model();
            let open_now = if spec.has_children() {
                cx.watch_model(&open_model)
                    .layout()
                    .copied()
                    .unwrap_or(false)
            } else {
                false
            };
            let toggled = super::model_value_changed_for(cx, cx.root_id(), open_now);
            let enabled = spec.enabled && !super::imui_is_disabled(cx);
            let active_item_model = super::active_item_model_for_window(cx);
            let mut build = Some(f);
            let content_id = cx.named("content", |cx| cx.root_id());
            let spec_for_header = spec.clone();

            let mut root_children = Vec::new();
            let header = cx.named("header", |cx| {
                let spec = spec_for_header.clone();
                let spec_for_pressable = spec.clone();
                let mut props = PressableProps::default();
                props.enabled = enabled;
                props.focusable = enabled;
                props.layout = LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                props.a11y = disclosure_a11y(&spec, open_now);

                let mut header = cx.pressable_with_id(props, move |cx, state, trigger_id| {
                    let spec = spec_for_pressable.clone();
                    let context_anchor_model = super::context_menu_anchor_model_for(cx, trigger_id);
                    let context_anchor_model_for_report = context_anchor_model.clone();
                    cx.pressable_clear_on_pointer_down();
                    cx.pressable_clear_on_pointer_move();
                    cx.pressable_clear_on_pointer_up();
                    cx.key_clear_on_key_down_for(trigger_id);

                    let action_label = spec.label.clone();
                    let open_model_for_activate = open_model.clone();
                    let has_children = spec.has_children();
                    let activate_shortcut = spec.activate_shortcut;
                    let shortcut_repeat = spec.shortcut_repeat;
                    cx.pressable_on_activate(crate::on_activate(
                        move |host, action_cx, _reason| {
                            host.record_transient_event(action_cx, super::KEY_CLICKED);
                            if has_children {
                                let _ = host
                                    .models_mut()
                                    .update(&open_model_for_activate, |value| *value = !*value);
                            }
                            host.notify(action_cx);
                        },
                    ));

                    if enabled {
                        cx.key_on_key_down_for(
                            trigger_id,
                            Arc::new(move |host, acx, down| {
                                if let Some(shortcut) = activate_shortcut {
                                    let matches_shortcut =
                                        down.key == shortcut.key && down.modifiers == shortcut.mods;
                                    if matches_shortcut
                                        && (!down.repeat || shortcut_repeat)
                                        && !down.ime_composing
                                    {
                                        host.record_transient_event(acx, super::KEY_CLICKED);
                                        if has_children {
                                            let _ = host
                                                .models_mut()
                                                .update(&open_model, |value| *value = !*value);
                                        }
                                        host.notify(acx);
                                        return true;
                                    }
                                }

                                let is_menu_key = down.key == KeyCode::ContextMenu;
                                let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                                if !(is_menu_key || is_shift_f10) {
                                    return false;
                                }

                                host.record_transient_event(acx, super::KEY_CONTEXT_MENU_REQUESTED);
                                host.notify(acx);
                                true
                            }),
                        );
                    }

                    cx.pressable_on_pointer_down(Arc::new(|_host, _acx, _down| {
                        PressablePointerDownResult::Continue
                    }));
                    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                        if up.is_click && up.button == MouseButton::Right {
                            let _ = host.update_model(&context_anchor_model, |value| {
                                *value = Some(up.position)
                            });
                            host.record_transient_event(acx, super::KEY_SECONDARY_CLICKED);
                            host.record_transient_event(acx, super::KEY_CONTEXT_MENU_REQUESTED);
                            host.notify(acx);
                            return PressablePointerUpResult::SkipActivate;
                        }

                        if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
                            host.record_transient_event(acx, super::KEY_DOUBLE_CLICKED);
                            host.notify(acx);
                        }

                        PressablePointerUpResult::Continue
                    }));

                    trigger_response.core.hovered = state.hovered;
                    trigger_response.core.pressed = state.pressed;
                    trigger_response.core.focused = state.focused;
                    trigger_response.nav_highlighted = state.focused
                        && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
                    trigger_response.id = Some(trigger_id);
                    trigger_response.core.clicked =
                        cx.take_transient_for(trigger_id, super::KEY_CLICKED);
                    trigger_response.secondary_clicked =
                        cx.take_transient_for(trigger_id, super::KEY_SECONDARY_CLICKED);
                    trigger_response.double_clicked =
                        cx.take_transient_for(trigger_id, super::KEY_DOUBLE_CLICKED);
                    trigger_response.context_menu_requested =
                        cx.take_transient_for(trigger_id, super::KEY_CONTEXT_MENU_REQUESTED);
                    trigger_response.context_menu_anchor = cx
                        .read_model(
                            &context_anchor_model_for_report,
                            Invalidation::Paint,
                            |_app, value| *value,
                        )
                        .unwrap_or(None);
                    trigger_response.core.rect = cx.last_bounds_for_element(trigger_id);
                    let hover_delay = super::install_hover_query_hooks_for_pressable(
                        cx,
                        trigger_id,
                        state.hovered_raw,
                        None,
                    );
                    trigger_response.pointer_hovered_raw = state.hovered_raw;
                    trigger_response.pointer_hovered_raw_below_barrier =
                        state.hovered_raw_below_barrier;
                    trigger_response.hover_stationary_met = hover_delay.stationary_met;
                    trigger_response.hover_delay_short_met = hover_delay.delay_short_met;
                    trigger_response.hover_delay_normal_met = hover_delay.delay_normal_met;
                    trigger_response.hover_delay_short_shared_met =
                        hover_delay.shared_delay_short_met;
                    trigger_response.hover_delay_normal_shared_met =
                        hover_delay.shared_delay_normal_met;
                    trigger_response.hover_blocked_by_active_item =
                        super::hover_blocked_by_active_item_for(cx, trigger_id, &active_item_model);
                    super::sanitize_response_for_enabled(enabled, trigger_response);

                    vec![header_row(cx, &spec, action_label, open_now, state)]
                });

                if spec.has_children() {
                    header = radix_collapsible::apply_collapsible_trigger_controls_expanded(
                        header, content_id, open_now,
                    );
                }
                if let Some(test_id) = spec.header_test_id.as_ref() {
                    header = header.test_id(test_id.clone());
                }
                header
            });
            root_children.push(header);

            if spec.has_children() && open_now {
                let mut content = cx.named("content", |cx| {
                    let mut props = ContainerProps::default();
                    props.layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    };
                    props.padding = disclosure_content_padding(&spec).into();

                    cx.container(props, move |cx| {
                        vec![cx.column(
                            ColumnProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Auto,
                                        ..Default::default()
                                    },
                                    overflow: Overflow::Visible,
                                    ..Default::default()
                                },
                                gap: SpacingLength::Px(Px(0.0)),
                                ..Default::default()
                            },
                            move |cx| {
                                let mut out = Vec::new();
                                let mut body_ui = ImUiFacade {
                                    cx,
                                    out: &mut out,
                                    build_focus: None,
                                };
                                if let Some(build) = build.take() {
                                    build(&mut body_ui);
                                }
                                out
                            },
                        )]
                    })
                });
                if let Some(test_id) = spec.content_test_id.as_ref() {
                    content = content.test_id(test_id.clone());
                }
                root_children.push(content);
            }

            response.open = open_now;
            response.toggled = toggled;

            let mut root = cx.column(
                ColumnProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    },
                    gap: SpacingLength::Px(Px(0.0)),
                    ..Default::default()
                },
                move |_cx| root_children,
            );
            if let Some(test_id) = spec.root_test_id.as_ref() {
                root = root.test_id(test_id.clone());
            }
            root
        })
    });

    ui.add(element);
    response
}

fn disclosure_a11y(spec: &DisclosureSpec, open_now: bool) -> PressableA11y {
    match spec.kind {
        DisclosureKind::CollapsingHeader => PressableA11y {
            role: Some(SemanticsRole::Button),
            label: Some(spec.label.clone()),
            expanded: spec.has_children().then_some(open_now),
            ..Default::default()
        },
        DisclosureKind::TreeNode => PressableA11y {
            role: Some(SemanticsRole::TreeItem),
            label: Some(spec.label.clone()),
            level: Some(spec.level),
            selected: spec.selected,
            expanded: spec.has_children().then_some(open_now),
            pos_in_set: spec.pos_in_set,
            set_size: spec.set_size,
            ..Default::default()
        },
    }
}

fn disclosure_content_padding(spec: &DisclosureSpec) -> Edges {
    match spec.kind {
        DisclosureKind::CollapsingHeader => Edges {
            top: Px(4.0),
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        },
        DisclosureKind::TreeNode => Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        },
    }
}

fn header_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: &DisclosureSpec,
    label: Arc<str>,
    open_now: bool,
    state: fret_ui::element::PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let palette = resolve_disclosure_palette(theme, spec, state);
    let border = theme.color_token("border");
    let indicator: Option<Arc<str>> = if spec.leaf {
        None
    } else if open_now {
        Some(Arc::from("v"))
    } else {
        Some(Arc::from(">"))
    };
    let row_padding = match spec.kind {
        DisclosureKind::CollapsingHeader => Edges {
            top: Px(4.0),
            right: Px(6.0),
            bottom: Px(4.0),
            left: Px(6.0),
        },
        DisclosureKind::TreeNode => {
            let indent = Px(16.0 * (spec.level.saturating_sub(1) as f32));
            Edges {
                top: Px(2.0),
                right: Px(6.0),
                bottom: Px(2.0),
                left: Px(6.0 + indent.0),
            }
        }
    };
    let border_edges = match spec.kind {
        DisclosureKind::CollapsingHeader => Edges::all(Px(1.0)),
        DisclosureKind::TreeNode => Edges::all(Px(0.0)),
    };

    let mut row_props = ContainerProps::default();
    row_props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        },
        overflow: Overflow::Visible,
        ..Default::default()
    };
    row_props.padding = row_padding.into();
    row_props.background = palette.background;
    row_props.border = border_edges;
    row_props.border_color = (spec.kind == DisclosureKind::CollapsingHeader).then_some(border);
    row_props.corner_radii = Corners::all(super::control_chrome::CONTROL_RADIUS);

    cx.container(row_props, move |cx| {
        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: Axis::Horizontal,
                gap: SpacingLength::Px(Px(4.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
                ..Default::default()
            },
            move |cx| {
                let mut out = Vec::new();
                out.push(cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(Px(12.0)),
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        indicator
                            .as_ref()
                            .map(|indicator| {
                                let mut props = TextProps::new(indicator.clone());
                                props.color = Some(palette.foreground);
                                vec![cx.text_props(props)]
                            })
                            .unwrap_or_default()
                    },
                ));

                let mut text_props = TextProps::new(label);
                text_props.layout.size.width = Length::Fill;
                text_props.layout.flex.shrink = 1.0;
                text_props.color = Some(palette.foreground);
                out.push(cx.text_props(text_props));
                out.push(cx.spacer(SpacerProps::default()));
                out
            },
        )]
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DisclosurePalette {
    background: Option<Color>,
    foreground: Color,
}

fn resolve_disclosure_palette(
    theme: &Theme,
    spec: &DisclosureSpec,
    state: fret_ui::element::PressableState,
) -> DisclosurePalette {
    let selected_bg = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("selection.background"))
        .unwrap_or_else(|| theme.color_token("selection.background"));
    let hover_bg = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    let idle_bg = theme
        .color_by_key("card")
        .or_else(|| theme.color_by_key("popover"))
        .unwrap_or_else(|| theme.color_token("popover"));
    let foreground = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"));
    let hover_foreground = theme
        .color_by_key("accent-foreground")
        .or_else(|| theme.color_by_key("foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"));
    let interactive = state.pressed || state.hovered || state.focused;

    match spec.kind {
        DisclosureKind::CollapsingHeader => {
            if interactive {
                DisclosurePalette {
                    background: Some(if state.pressed { selected_bg } else { hover_bg }),
                    foreground: hover_foreground,
                }
            } else {
                DisclosurePalette {
                    background: Some(idle_bg),
                    foreground,
                }
            }
        }
        DisclosureKind::TreeNode => {
            if spec.selected {
                DisclosurePalette {
                    background: Some(selected_bg),
                    foreground: hover_foreground,
                }
            } else if interactive {
                DisclosurePalette {
                    background: Some(if state.pressed { selected_bg } else { hover_bg }),
                    foreground: hover_foreground,
                }
            } else {
                DisclosurePalette {
                    background: None,
                    foreground,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_authoring::UiWriter;
    use fret_ui::ThemeConfig;
    use fret_ui::element::{ElementKind, PressableProps, PressableState};

    struct TestWriter<'cx, 'a, H: UiHost> {
        cx: &'cx mut ElementContext<'a, H>,
        out: &'cx mut Vec<AnyElement>,
    }

    impl<'cx, 'a, H: UiHost> UiWriter<H> for TestWriter<'cx, 'a, H> {
        fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
            f(self.cx)
        }

        fn add(&mut self, element: AnyElement) {
            self.out.push(element);
        }
    }

    fn contains_text(root: &AnyElement, expected: &str) -> bool {
        match &root.kind {
            ElementKind::Text(props) if props.text.as_ref() == expected => true,
            _ => root
                .children
                .iter()
                .any(|child| contains_text(child, expected)),
        }
    }

    fn first_pressable(root: &AnyElement) -> Option<&PressableProps> {
        match &root.kind {
            ElementKind::Pressable(props) => Some(props),
            _ => root.children.iter().find_map(first_pressable),
        }
    }

    #[test]
    fn collapsing_header_default_open_mounts_body() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |cx| {
                let mut out = Vec::new();
                let mut ui = TestWriter { cx, out: &mut out };
                let response = collapsing_header_with_options(
                    &mut ui,
                    "header",
                    Arc::from("Section"),
                    CollapsingHeaderOptions {
                        default_open: true,
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Body");
                    },
                );

                assert!(response.open());
                assert_eq!(out.len(), 1);
                assert!(contains_text(&out[0], "Section"));
                assert!(contains_text(&out[0], "Body"));
            },
        );
    }

    #[test]
    fn tree_node_leaf_uses_tree_item_semantics() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |cx| {
                let mut out = Vec::new();
                let mut ui = TestWriter { cx, out: &mut out };
                let response = tree_node_with_options(
                    &mut ui,
                    "leaf",
                    Arc::from("Leaf"),
                    TreeNodeOptions {
                        leaf: true,
                        level: 3,
                        selected: true,
                        ..Default::default()
                    },
                    |_ui| {},
                );

                assert!(!response.open());
                let pressable = first_pressable(&out[0]).expect("expected pressable row");
                assert_eq!(pressable.a11y.role, Some(SemanticsRole::TreeItem));
                assert_eq!(pressable.a11y.level, Some(3));
                assert!(pressable.a11y.selected);
                assert_eq!(pressable.a11y.expanded, None);
            },
        );
    }

    #[test]
    fn tree_node_default_options_start_at_level_one() {
        let options = TreeNodeOptions::default();
        assert_eq!(options.level, 1);
        assert!(!options.selected);
        assert!(!options.leaf);
    }

    #[test]
    fn tree_node_hover_palette_prefers_accent_chrome_over_popover_fill() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors
                .insert("list.active.background".to_string(), "#224466".to_string());
            cfg.colors
                .insert("accent".to_string(), "#335577".to_string());
            cfg.colors
                .insert("accent-foreground".to_string(), "#fefefe".to_string());
            cfg.colors
                .insert("foreground".to_string(), "#f5f6f7".to_string());
            cfg.colors.insert("card".to_string(), "#101418".to_string());
            theme.apply_config_patch(&cfg);
        });

        let theme = Theme::global(&app);
        let spec = DisclosureSpec::tree_node(Arc::from("Scene"), TreeNodeOptions::default());

        let hovered = resolve_disclosure_palette(
            theme,
            &spec,
            PressableState {
                hovered: true,
                ..Default::default()
            },
        );
        assert_eq!(
            hovered.background,
            Some(Color::from_srgb_hex_rgb(0x33_55_77))
        );
        assert_eq!(hovered.foreground, Color::from_srgb_hex_rgb(0xfe_fe_fe));

        let idle = resolve_disclosure_palette(theme, &spec, PressableState::default());
        assert_eq!(idle.background, None);
        assert_eq!(idle.foreground, Color::from_srgb_hex_rgb(0xf5_f6_f7));
    }
}
