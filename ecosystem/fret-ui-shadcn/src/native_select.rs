use std::sync::{Arc, Mutex};

use fret_core::{Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::OnCloseAutoFocus;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::popover as radix_popover;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, ShadowPreset,
    Size as ComponentSize, Space, WidgetState, WidgetStateProperty, WidgetStates, ui,
};

use crate::test_id::test_id_slug;
use crate::{
    CommandEntry, CommandGroup, CommandItem, CommandList, CommandSeparator, Popover, PopoverContent,
};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NativeSelectSize {
    Sm,
    #[default]
    Default,
}

#[derive(Debug, Clone)]
pub struct NativeSelectOption {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl NativeSelectOption {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Convenience for the common shadcn NativeSelect pattern where the "placeholder" is the first
    /// option with an empty value. Selecting it clears the model to `None`.
    pub fn placeholder(label: impl Into<Arc<str>>) -> Self {
        Self::new("", label)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone)]
pub struct NativeSelectOptGroup {
    pub label: Arc<str>,
    pub disabled: bool,
    pub options: Vec<NativeSelectOption>,
}

impl NativeSelectOptGroup {
    pub fn new(
        label: impl Into<Arc<str>>,
        options: impl IntoIterator<Item = NativeSelectOption>,
    ) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            options: options.into_iter().collect(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Default)]
struct NativeSelectState {
    focus_restore_target: Option<Arc<Mutex<Option<GlobalElementId>>>>,
}

#[derive(Clone)]
pub struct NativeSelect {
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    placeholder: Arc<str>,
    options: Vec<NativeSelectOption>,
    optgroups: Vec<NativeSelectOptGroup>,
    test_id_prefix: Option<Arc<str>>,
    trigger_test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    size: NativeSelectSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for NativeSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSelect")
            .field("placeholder", &self.placeholder.as_ref())
            .field("options_len", &self.options.len())
            .field("optgroups_len", &self.optgroups.len())
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("aria_invalid", &self.aria_invalid)
            .field("disabled", &self.disabled)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl NativeSelect {
    pub fn new(model: Model<Option<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            model,
            open,
            placeholder: Arc::from("Select..."),
            options: Vec::new(),
            optgroups: Vec::new(),
            test_id_prefix: None,
            trigger_test_id: None,
            a11y_label: None,
            aria_invalid: false,
            disabled: false,
            size: NativeSelectSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Creates a NativeSelect with controlled/uncontrolled `value` + `open` models.
    ///
    /// This mirrors the Radix-style controlled/uncontrolled pattern used across this crate.
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        value: Option<Model<Option<Arc<str>>>>,
        default_value: Option<Arc<str>>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_popover::PopoverRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        let value = controllable_state::use_controllable_model(cx, value, || default_value).model();
        Self::new(value, open)
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn option(mut self, option: NativeSelectOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = NativeSelectOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn optgroup(mut self, optgroup: NativeSelectOptGroup) -> Self {
        self.optgroups.push(optgroup);
        self
    }

    pub fn optgroups(mut self, optgroups: impl IntoIterator<Item = NativeSelectOptGroup>) -> Self {
        self.optgroups.extend(optgroups);
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn trigger_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.trigger_test_id = Some(id.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: NativeSelectSize) -> Self {
        self.size = size;
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        native_select(
            cx,
            self.model,
            self.open,
            self.placeholder,
            &self.options,
            &self.optgroups,
            self.test_id_prefix,
            self.trigger_test_id,
            self.a11y_label,
            self.aria_invalid,
            self.disabled,
            self.size,
            self.chrome,
            self.layout,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn native_select<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    placeholder: Arc<str>,
    options: &[NativeSelectOption],
    optgroups: &[NativeSelectOptGroup],
    test_id_prefix: Option<Arc<str>>,
    trigger_test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    size: NativeSelectSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).snapshot();

        let focus_restore_target = {
            let existing = cx.with_state(NativeSelectState::default, |st| {
                st.focus_restore_target.clone()
            });
            if let Some(target) = existing {
                target
            } else {
                let target = Arc::new(Mutex::new(None));
                cx.with_state(NativeSelectState::default, |st| {
                    st.focus_restore_target = Some(target.clone());
                });
                target
            }
        };

        let is_open: bool = cx.watch_model(&open).layout().copied().unwrap_or(false);
        let selected: Option<Arc<str>> = cx.watch_model(&model).cloned().unwrap_or_default();

        let resolved = resolve_input_chrome(
            Theme::global(&*cx.app),
            ComponentSize::default(),
            &chrome,
            InputTokenKeys::none(),
        );

        // shadcn/ui v4 SelectTrigger:
        // - size=default => `h-9` (36px) + `py-2` (8px) + `items-center`
        // - size=sm => `h-8` (32px) + `py-1.5`-ish (we keep a slightly tighter 4px here)
        //
        // In Fret we must preserve the `items-center` outcome: the label/icon should be centered
        // even when the content box is smaller than the label's fixed line box.
        let (h, py) = match size {
            NativeSelectSize::Sm => (Px(32.0), Px(4.0)),
            NativeSelectSize::Default => (Px(36.0), Px(8.0)),
        };

        let mut text_style = typography::control_text_style_scaled(
            Theme::global(&*cx.app),
            FontId::ui(),
            resolved.text_px,
        );
        text_style.weight = FontWeight::NORMAL;

        let muted_fg = theme.color_token("muted-foreground");

        let (resolved_label, has_selection): (Arc<str>, bool) = selected
            .as_ref()
            .and_then(|v: &Arc<str>| {
                options
                    .iter()
                    .chain(optgroups.iter().flat_map(|g| g.options.iter()))
                    .find(|opt| opt.value.as_ref() == v.as_ref())
            })
            .map(|opt| (opt.label.clone(), true))
            .unwrap_or((placeholder.clone(), false));

        let mut border_color = resolved.border_color;
        let mut focus_ring = decl_style::focus_ring(&theme, resolved.radius);
        if aria_invalid {
            border_color = theme.color_token("destructive");
            focus_ring.color =
                crate::theme_variants::invalid_control_ring_color(&theme, border_color);
        }

        let layout = decl_style::layout_style(&theme, layout.relative().min_w_0());
        let mut pressable_layout = layout;
        pressable_layout.size = SizeStyle {
            height: Length::Px(h),
            ..pressable_layout.size
        };

        let icon_size = Px(16.0);
        let icon_color = alpha_mul(theme.color_token("muted-foreground"), 0.5);

        let has_entries = !options.is_empty() || !optgroups.is_empty();

        let open_for_trigger = open.clone();
        let a11y_label_for_trigger = a11y_label
            .clone()
            .or_else(|| Some(Arc::from("Native select")));
        let trigger_test_id_for_trigger = trigger_test_id.clone();
        let theme_for_trigger = theme.clone();
        let focus_restore_target_for_trigger = focus_restore_target.clone();
        let test_id_prefix_for_trigger = test_id_prefix.clone();
        let test_id_prefix_for_content = test_id_prefix.clone();

        let trigger = control_chrome_pressable_with_id_props(cx, move |cx, st, trigger_id| {
            *focus_restore_target_for_trigger
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = Some(trigger_id);

            if has_entries && !disabled {
                cx.pressable_toggle_bool(&open_for_trigger);
            }

            let mut states = WidgetStates::from_pressable(cx, st, !disabled);
            states.set(WidgetState::Open, is_open);
            let bg = WidgetStateProperty::new(ColorRef::Token {
                key: "component.input.bg",
                fallback: ColorFallback::Color(Color::TRANSPARENT),
            })
            .when(
                WidgetStates::HOVERED,
                ColorRef::Token {
                    key: "component.input.bg_hover",
                    fallback: ColorFallback::Color(Color::TRANSPARENT),
                },
            )
            .when(
                WidgetStates::ACTIVE,
                ColorRef::Token {
                    key: "component.input.bg_hover",
                    fallback: ColorFallback::Color(Color::TRANSPARENT),
                },
            )
            .resolve(states)
            .clone()
            .resolve(&theme_for_trigger);

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: !disabled,
                focusable: !disabled,
                focus_ring: Some(focus_ring),
                a11y: PressableA11y {
                    role: Some(SemanticsRole::ComboBox),
                    label: a11y_label_for_trigger.clone(),
                    test_id: trigger_test_id_for_trigger.clone(),
                    expanded: Some(is_open),
                    ..Default::default()
                },
                ..Default::default()
            };

            // IMPORTANT: shadcn's SelectTrigger is `flex items-center justify-between gap-2`.
            // Make sure we mirror the `items-center` outcome here (don't rely on padding-only
            // centering), otherwise the label reads bottom-heavy in fixed-height triggers.
            let chrome_props = ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                padding: Edges::all(Px(0.0)).into(),
                background: None,
                shadow: Some(decl_style::shadow_xs(&theme_for_trigger, resolved.radius)),
                border: Edges::all(Px(0.0)),
                border_color: None,
                corner_radii: Corners::all(resolved.radius),
                ..Default::default()
            };

            let label = resolved_label.clone();
            let label_is_placeholder = !has_selection;
            let text_style = text_style.clone();
            (
                pressable_props,
                chrome_props,
                move |cx: &mut ElementContext<'_, H>| {
                    let surface = {
                        let surface_props = ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            padding: Edges {
                                left: resolved.padding.left,
                                right: resolved.padding.right,
                                top: py,
                                bottom: py,
                            }
                            .into(),
                            background: Some(bg),
                            shadow: None,
                            border: Edges::all(resolved.border_width),
                            border_color: Some(border_color),
                            corner_radii: Corners::all(resolved.radius),
                            ..Default::default()
                        };
                        cx.container(surface_props, move |cx| {
                            let fg = if label_is_placeholder {
                                ColorRef::Color(muted_fg)
                            } else {
                                ColorRef::Color(resolved.text_color)
                            };
                            let mut content = ui::text(cx, label)
                                .text_size_px(text_style.size)
                                .fixed_line_box_px(
                                    text_style.line_height.unwrap_or(text_style.size),
                                )
                                .line_box_in_bounds()
                                .font_normal()
                                .nowrap()
                                .text_color(fg)
                                .truncate();

                            content = content.overflow(fret_core::TextOverflow::Clip);

                            let mut icon = decl_icon::icon_with(
                                cx,
                                fret_icons::ids::ui::CHEVRON_DOWN,
                                Some(icon_size),
                                Some(ColorRef::Color(icon_color)),
                            );
                            if let Some(prefix) = test_id_prefix_for_trigger.as_deref() {
                                icon = icon.test_id(format!("{prefix}-icon"));
                            }

                            vec![stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full().h_full().min_w_0())
                                    .justify_between()
                                    .items_center()
                                    .gap_x(Space::N2),
                                |cx| vec![content.flex_1().min_w_0().into_element(cx), icon],
                            )]
                        })
                    };

                    let disabled = disabled;
                    let out: Vec<AnyElement> = if disabled {
                        vec![cx.opacity(0.5, move |_cx| vec![surface])]
                    } else {
                        vec![surface]
                    };
                    out
                },
            )
        });

        let close_auto_focus: OnCloseAutoFocus = Arc::new({
            let focus_restore_target = focus_restore_target.clone();
            move |host, _action_cx, req| {
                req.prevent_default();
                let target = *focus_restore_target
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                if let Some(target) = target {
                    host.request_focus(target);
                }
            }
        });

        let popover = Popover::new(open.clone())
            .auto_focus(true)
            .on_close_auto_focus(Some(close_auto_focus));

        popover.into_element_with_anchor(
            cx,
            move |_cx| trigger,
            move |cx, anchor| {
                let desired_w = Px(anchor.size.width.0.max(180.0));
                let max_list_h = theme
                    .metric_by_key("component.native_select.max_list_height")
                    .or_else(|| theme.metric_by_key("component.select.max_list_height"))
                    .unwrap_or(Px(280.0));

                let selected: Option<Arc<str>> =
                    cx.watch_model(&model).cloned().unwrap_or_default();

                let fg = theme
                    .color_by_key("foreground")
                    .unwrap_or_else(|| theme.color_token("foreground"));
                let fg_disabled = alpha_mul(fg, 0.5);
                let item_text_style = crate::command::item_text_style(&theme);

                let open_for_select = open.clone();
                let model_for_select = model.clone();

                let mut make_item =
                    |option: NativeSelectOption, group_disabled: bool| -> CommandItem {
                        let item_disabled = disabled || group_disabled || option.disabled;
                        let is_selected = selected
                            .as_ref()
                            .is_some_and(|v: &Arc<str>| v.as_ref() == option.value.as_ref());

                        let value_for_select = option.value.clone();
                        let on_select: fret_ui::action::OnActivate = Arc::new({
                            let model_for_select = model_for_select.clone();
                            let open_for_select = open_for_select.clone();
                            move |host, action_cx, _reason| {
                                let next_value = if value_for_select.as_ref().is_empty() {
                                    None
                                } else {
                                    Some(value_for_select.clone())
                                };
                                let _ = host.models_mut().update(&model_for_select, |v| {
                                    *v = next_value;
                                });
                                let _ = host.models_mut().update(&open_for_select, |v| *v = false);
                                host.request_redraw(action_cx.window);
                            }
                        });

                        let label_text = option.label.clone();
                        let label_style = item_text_style.clone();
                        let icon = decl_icon::icon_with(
                            cx,
                            fret_icons::ids::ui::CHECK,
                            Some(Px(16.0)),
                            Some(ColorRef::Color(if item_disabled {
                                fg_disabled
                            } else {
                                fg
                            })),
                        );
                        let icon =
                            cx.opacity(if is_selected { 1.0 } else { 0.0 }, move |_cx| vec![icon]);

                        let text = {
                            let mut label = ui::label(cx, label_text.clone())
                                .text_size_px(label_style.size)
                                .font_weight(label_style.weight)
                                .text_color(ColorRef::Color(if item_disabled {
                                    fg_disabled
                                } else {
                                    fg
                                }))
                                .truncate();
                            if let Some(line_height) = label_style.line_height {
                                label = label.line_height_px(line_height).line_height_policy(
                                    fret_core::TextLineHeightPolicy::FixedFromStyle,
                                );
                            }
                            if let Some(letter_spacing_em) = label_style.letter_spacing_em {
                                label = label.letter_spacing_em(letter_spacing_em);
                            }
                            label.into_element(cx)
                        };

                        let mut item = CommandItem::new(label_text)
                            .value(option.value.clone())
                            .disabled(item_disabled)
                            .on_select_action(on_select)
                            .children(vec![text, icon]);

                        if let Some(prefix) = test_id_prefix_for_content.as_deref() {
                            item = item.test_id(format!(
                                "{prefix}-item-{}",
                                test_id_slug(option.value.as_ref())
                            ));
                        }

                        item
                    };

                let mut entries: Vec<CommandEntry> =
                    Vec::with_capacity(options.len() + optgroups.len());

                for option in options.iter().cloned() {
                    entries.push(CommandEntry::Item(make_item(option, false)));
                }

                let non_empty_groups: Vec<NativeSelectOptGroup> = optgroups
                    .iter()
                    .cloned()
                    .filter(|g| !g.options.is_empty())
                    .collect();
                if !options.is_empty() && !non_empty_groups.is_empty() {
                    entries.push(CommandEntry::Separator(CommandSeparator::new()));
                }

                for group in non_empty_groups {
                    let group_disabled = group.disabled;
                    let group_items: Vec<CommandItem> = group
                        .options
                        .into_iter()
                        .map(|opt| make_item(opt, group_disabled))
                        .collect();
                    entries.push(CommandEntry::Group(
                        CommandGroup::new(group_items).heading(group.label),
                    ));
                }

                let popover_surface = ChromeRefinement::default()
                    .rounded(Radius::Md)
                    .border_width(Px(1.0))
                    .border_color(ColorRef::Token {
                        key: "border",
                        fallback: ColorFallback::ThemePanelBorder,
                    })
                    .bg(ColorRef::Token {
                        key: "popover.background",
                        fallback: ColorFallback::ThemePanelBackground,
                    })
                    .shadow(ShadowPreset::Md)
                    .p(Space::N1);

                let mut list = CommandList::new_entries(entries)
                    .disabled(disabled)
                    .empty_text("No options.")
                    .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h))
                    .into_element(cx);

                if let Some(prefix) = test_id_prefix_for_content.as_deref() {
                    list = list.test_id(format!("{prefix}-listbox"));
                }

                PopoverContent::new(vec![list])
                    .refine_style(popover_surface)
                    .refine_layout(LayoutRefinement::default().w_px(desired_w).min_w_0())
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
        AppWindowId, FrameId, MaterialId, MaterialRegistrationError, MaterialService, PathCommand,
        PathConstraints, PathId, PathMetrics, PathService, PathStyle, Point, Px, Rect,
        Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics,
        TextService,
    };
    use fret_ui::element::{ElementKind, PressableProps};
    use fret_ui::tree::UiTree;
    use fret_ui_kit::OverlayController;

    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: CoreSize::new(Px(10.0), Px(10.0)),
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

    impl MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<MaterialId, MaterialRegistrationError> {
            Ok(MaterialId::default())
        }

        fn unregister_material(&mut self, _id: MaterialId) -> bool {
            true
        }
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(320.0), Px(240.0)),
        )
    }

    fn find_pressable_by_test_id<'a>(
        el: &'a AnyElement,
        test_id: &str,
    ) -> Option<&'a PressableProps> {
        match &el.kind {
            ElementKind::Pressable(props) => {
                if props.a11y.test_id.as_deref() == Some(test_id) {
                    return Some(props);
                }
            }
            _ => {}
        }

        for child in &el.children {
            if let Some(found) = find_pressable_by_test_id(child, test_id) {
                return Some(found);
            }
        }

        None
    }

    #[test]
    fn native_select_trigger_stamps_combobox_role_and_expanded_state() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                NativeSelect::new(value.clone(), open.clone())
                    .option(NativeSelectOption::new("a", "A"))
                    .a11y_label("Choose")
                    .trigger_test_id("native-select-trigger")
                    .into_element(cx)
            });

        let trigger = find_pressable_by_test_id(&element, "native-select-trigger")
            .expect("trigger pressable");
        assert_eq!(trigger.a11y.role, Some(SemanticsRole::ComboBox));
        assert_eq!(trigger.a11y.label.as_deref(), Some("Choose"));
        assert_eq!(trigger.a11y.expanded, Some(false));

        let _ = app.models_mut().update(&open, |v| *v = true);
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                NativeSelect::new(value.clone(), open.clone())
                    .option(NativeSelectOption::new("a", "A"))
                    .a11y_label("Choose")
                    .trigger_test_id("native-select-trigger")
                    .into_element(cx)
            });

        let trigger = find_pressable_by_test_id(&element, "native-select-trigger")
            .expect("trigger pressable");
        assert_eq!(trigger.a11y.expanded, Some(true));
    }

    #[test]
    fn native_select_test_id_prefix_stamps_listbox_items_and_icon_when_open() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let value = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = bounds();
        let mut services = FakeServices;

        // Frame 0: closed render to establish stable trigger bounds.
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "native-select-test-id-prefix",
            |cx| {
                vec![
                    NativeSelect::new(value.clone(), open.clone())
                        .test_id_prefix("ns")
                        .option(NativeSelectOption::placeholder("Pick one"))
                        .option(NativeSelectOption::new("x1", "X1"))
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Frame 1: open render should request and surface overlay children.
        let _ = app.models_mut().update(&open, |v| *v = true);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "native-select-test-id-prefix",
            |cx| {
                vec![
                    NativeSelect::new(value, open)
                        .test_id_prefix("ns")
                        .option(NativeSelectOption::placeholder("Pick one"))
                        .option(NativeSelectOption::new("x1", "X1"))
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
        let ids: Vec<&str> = snapshot
            .nodes
            .iter()
            .filter_map(|n| n.test_id.as_deref())
            .collect();

        assert!(
            ids.iter().copied().any(|id| id == "ns-listbox"),
            "expected `ns-listbox` test id, got {ids:?}"
        );
        assert!(
            ids.iter().copied().any(|id| id == "ns-item-x1"),
            "expected `ns-item-x1` test id, got {ids:?}"
        );
        assert!(
            ids.iter().copied().any(|id| id == "ns-icon"),
            "expected `ns-icon` test id, got {ids:?}"
        );
    }
}
