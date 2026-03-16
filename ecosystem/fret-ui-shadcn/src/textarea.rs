use std::sync::Arc;

use fret_core::{Color, Corners, CursorIcon, Edges, FontId, MouseButton, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, PressableProps, SemanticsDecoration, SizeStyle,
    TextAreaProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, TextAreaStyle, Theme, UiHost, action};
use fret_ui_kit::declarative::motion::{
    drive_tween_color_for_element, drive_tween_f32_for_element,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlEntry, ControlId, control_registry_model,
};
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Size as ComponentSize, Space};

use crate::overlay_motion;
use crate::text_value_model::IntoTextValueModel;

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn tailwind_transition_ease_in_out(t: f32) -> f32 {
    // Tailwind default transition timing function: cubic-bezier(0.4, 0, 0.2, 1).
    // (Often described as `ease-in-out`-ish.)
    fret_ui_headless::easing::SHADCN_EASE.sample(t)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextareaResizeDrag {
    start: fret_core::Point,
    start_height: Px,
}

fn textarea_resize_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<Option<Px>>, Model<Option<TextareaResizeDrag>>) {
    (
        cx.local_model_keyed("height_override", || None::<Px>),
        cx.local_model_keyed("drag", || None::<TextareaResizeDrag>),
    )
}

#[derive(Clone)]
pub struct Textarea {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    labelled_by_element: Option<GlobalElementId>,
    control_id: Option<ControlId>,
    test_id: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    aria_invalid: bool,
    aria_required: bool,
    disabled: bool,
    min_height: Px,
    resizable: bool,
    stable_line_boxes: bool,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Textarea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Textarea")
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field(
                "placeholder",
                &self.placeholder.as_ref().map(|s| s.as_ref()),
            )
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
            .field("min_height", &self.min_height)
            .field("resizable", &self.resizable)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Textarea {
    pub fn new(model: impl IntoTextValueModel) -> Self {
        Self {
            model: model.into_text_value_model(),
            a11y_label: None,
            labelled_by_element: None,
            control_id: None,
            test_id: None,
            placeholder: None,
            aria_invalid: false,
            aria_required: false,
            disabled: false,
            min_height: Px(64.0),
            resizable: true,
            stable_line_boxes: true,
            size: ComponentSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Associates this control with a label element (ARIA `aria-labelledby`-like outcome).
    pub fn labelled_by_element(mut self, label: GlobalElementId) -> Self {
        self.labelled_by_element = Some(label);
        self
    }

    /// Associates this textarea with a logical form control id so related elements (e.g. labels,
    /// helper text) can forward activation and attach `labelled-by` / `described-by` semantics.
    pub fn control_id(mut self, id: impl Into<ControlId>) -> Self {
        self.control_id = Some(id.into());
        self
    }

    /// Sets a stable `test_id` on the underlying `TextArea` element.
    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    pub fn aria_required(mut self, aria_required: bool) -> Self {
        self.aria_required = aria_required;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn min_height(mut self, min_height: Px) -> Self {
        self.min_height = min_height;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// If true, uses a fixed line box + forced strut for stable multiline metrics (UI/form
    /// surfaces). If false, uses an expand-to-fit line box to avoid clipping (content surfaces).
    pub fn stable_line_boxes(mut self, stable: bool) -> Self {
        self.stable_line_boxes = stable;
        self
    }

    pub fn size(mut self, size: ComponentSize) -> Self {
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
        textarea_element(
            cx,
            self.model,
            self.a11y_label,
            self.labelled_by_element,
            self.control_id,
            self.test_id,
            self.placeholder,
            self.aria_invalid,
            self.aria_required,
            self.disabled,
            self.min_height,
            self.resizable,
            self.stable_line_boxes,
            self.size,
            self.chrome,
            self.layout,
        )
    }
}

/// Builder-preserving helper for the common textarea authoring path.
pub fn textarea(model: impl IntoTextValueModel) -> Textarea {
    Textarea::new(model)
}

fn textarea_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: impl IntoTextValueModel,
    a11y_label: Option<Arc<str>>,
    labelled_by_element: Option<GlobalElementId>,
    control_id: Option<ControlId>,
    test_id: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    aria_invalid: bool,
    aria_required: bool,
    disabled: bool,
    min_height: Px,
    resizable: bool,
    stable_line_boxes: bool,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> AnyElement {
    let model = model.into_text_value_model();
    let show_resize_handle = resizable && !disabled;

    let theme_live = Theme::global(&*cx.app);
    let theme = theme_live.snapshot();

    let resolved = resolve_input_chrome(theme_live, size, &chrome, InputTokenKeys::none());

    let text_style = if stable_line_boxes {
        typography::text_area_control_text_style_scaled(
            Theme::global(&*cx.app),
            FontId::ui(),
            resolved.text_px,
        )
    } else {
        typography::text_area_content_text_style_scaled(
            Theme::global(&*cx.app),
            FontId::ui(),
            resolved.text_px,
        )
    };

    let mut chrome = TextAreaStyle::default();
    chrome.padding_x = crate::rtl::padding_x_from_physical_edges_max(
        resolved.padding.left,
        resolved.padding.right,
    );
    // shadcn/ui `Textarea` uses `py-2` (while `Input` uses `py-1`), so prefer a textarea-specific
    // padding intent here rather than reusing input padding wholesale.
    chrome.padding_y = fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme);
    chrome.background = resolved.background;
    chrome.border = Edges::all(resolved.border_width);
    chrome.border_color = resolved.border_color;
    chrome.border_color_focused = resolved.border_color_focused;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.text_color = resolved.text_color;
    chrome.placeholder_color = theme
        .color_by_key("muted-foreground")
        .unwrap_or(chrome.placeholder_color);
    chrome.selection_color = resolved.selection_color;
    chrome.caret_color = resolved.text_color;
    chrome.preedit_bg_color = alpha_mul(resolved.selection_color, 0.22);
    chrome.preedit_underline_color = resolved.selection_color;
    chrome.focus_ring = Some(decl_style::focus_ring(&theme, resolved.radius));

    if aria_invalid {
        let border_color = theme.color_token("destructive");
        chrome.border_color = border_color;
        chrome.border_color_focused = border_color;
        if let Some(mut ring) = chrome.focus_ring.take() {
            ring.color = crate::theme_variants::invalid_control_ring_color(&theme, border_color);
            chrome.focus_ring = Some(ring);
        }
    }

    let has_a11y_label = a11y_label.is_some();
    let mut props = TextAreaProps::new(model);
    props.test_id = test_id.clone();
    props.enabled = !disabled;
    props.focusable = !disabled;
    props.a11y_label = a11y_label;
    props.placeholder = placeholder;
    props.a11y_required = aria_required;
    props.a11y_invalid = aria_invalid.then_some(fret_core::SemanticsInvalid::True);
    props.chrome = chrome;
    props.text_style = text_style;
    props.min_height = min_height;
    props.layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Auto,
        min_width: Some(Length::Px(Px(0.0))),
        min_height: Some(Length::Px(min_height)),
        ..Default::default()
    };

    let mut root_layout = decl_style::layout_style(
        &theme,
        LayoutRefinement::default()
            .relative()
            .w_full()
            .min_w_0()
            .merge(layout),
    );
    root_layout.overflow = fret_ui::element::Overflow::Visible;
    props.layout = root_layout;

    let root_shadow = decl_style::shadow_xs(&theme, resolved.radius);

    let outer_layout = props.layout;
    let size_style = props.layout.size;
    let mut inner_layout =
        decl_style::layout_style(&theme, LayoutRefinement::default().w_full().min_w_0());
    inner_layout.size = size_style;
    props.layout = inner_layout;

    let root = cx.container(
        ContainerProps {
            layout: outer_layout,
            padding: Edges::all(Px(0.0)).into(),
            background: None,
            shadow: Some(root_shadow),
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(resolved.radius),
            ..Default::default()
        },
        move |cx| {
            let control_id = control_id.clone();
            let control_registry = control_id.as_ref().map(|_| control_registry_model(cx));
            let base_props = props.clone();

            let labelled_by_element = if labelled_by_element.is_some() {
                labelled_by_element
            } else if has_a11y_label {
                None
            } else if let (Some(control_id), Some(control_registry)) =
                (control_id.as_ref(), control_registry.as_ref())
            {
                cx.app
                    .models()
                    .read(control_registry, |reg| {
                        reg.label_for(cx.window, control_id).map(|l| l.element)
                    })
                    .ok()
                    .flatten()
            } else {
                None
            };

            let described_by_element = if let (Some(control_id), Some(control_registry)) =
                (control_id.as_ref(), control_registry.as_ref())
            {
                cx.app
                    .models()
                    .read(control_registry, |reg| {
                        reg.described_by_for(cx.window, control_id)
                    })
                    .ok()
                    .flatten()
            } else {
                None
            };

            if !show_resize_handle {
                let control_id_for_register = control_id.clone();
                let control_registry_for_register = control_registry.clone();
                let textarea = cx.text_area_with_id_props(move |cx, id| {
                    if let (Some(control_id), Some(control_registry)) = (
                        control_id_for_register.clone(),
                        control_registry_for_register.clone(),
                    ) {
                        let entry = ControlEntry {
                            element: id,
                            enabled: !disabled,
                            action: ControlAction::FocusOnly,
                        };
                        let _ = cx.app.models_mut().update(&control_registry, |reg| {
                            reg.register_control(cx.window, cx.frame_id, control_id, entry);
                        });
                    }
                    let mut props = base_props.clone();

                    // shadcn/ui v4 textarea uses `transition-[color,box-shadow]` with Tailwind
                    // defaults, so border + ring should ease instead of snapping.
                    let duration = overlay_motion::shadcn_motion_duration_150(cx);
                    let focus_visible = cx.is_focused_element(id)
                        && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));

                    let target_border = if focus_visible {
                        props.chrome.border_color_focused
                    } else {
                        props.chrome.border_color
                    };
                    let border_motion = drive_tween_color_for_element(
                        cx,
                        id,
                        "textarea.chrome.border",
                        target_border,
                        duration,
                        tailwind_transition_ease_in_out,
                    );
                    props.chrome.border_color = border_motion.value;
                    props.chrome.border_color_focused = border_motion.value;

                    let ring_alpha = drive_tween_f32_for_element(
                        cx,
                        id,
                        "textarea.chrome.ring.alpha",
                        if focus_visible { 1.0 } else { 0.0 },
                        duration,
                        tailwind_transition_ease_in_out,
                    );
                    if let Some(mut ring) = props.chrome.focus_ring.take() {
                        ring.color = alpha_mul(ring.color, ring_alpha.value);
                        if let Some(offset_color) = ring.offset_color {
                            ring.offset_color = Some(alpha_mul(offset_color, ring_alpha.value));
                        }
                        props.chrome.focus_ring = Some(ring);
                    }
                    props.focus_ring_always_paint = ring_alpha.animating;

                    props
                });

                let textarea = if labelled_by_element.is_some() || described_by_element.is_some() {
                    let mut decoration = SemanticsDecoration::default();
                    if let Some(label) = labelled_by_element {
                        decoration = decoration.labelled_by_element(label.0);
                    }
                    if let Some(desc) = described_by_element {
                        decoration = decoration.described_by_element(desc.0);
                    }
                    textarea.attach_semantics(decoration)
                } else {
                    textarea
                };
                return vec![textarea];
            }

            let (height_override, drag) = textarea_resize_models(cx);
            let override_px = cx
                .app
                .models_mut()
                .read(&height_override, |v| *v)
                .ok()
                .flatten();

            let theme = Theme::global(&*cx.app).snapshot();
            let resize_handle_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .absolute()
                    .right(Space::N1)
                    .bottom(Space::N1)
                    .w_px(Px(14.0))
                    .h_px(Px(14.0)),
            );
            let grip_color = theme
                .color_by_key("muted-foreground")
                .unwrap_or_else(|| theme.color_token("foreground"));
            let grip_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default().relative().size_full(),
            );

            let mut base_props = base_props;
            if let Some(px) = override_px {
                base_props.layout.size.height = Length::Px(px);
            }

            let control_id_for_register = control_id.clone();
            let control_registry_for_register = control_registry.clone();
            let textarea = cx.text_area_with_id_props(move |cx, id| {
                if let (Some(control_id), Some(control_registry)) = (
                    control_id_for_register.clone(),
                    control_registry_for_register.clone(),
                ) {
                    let entry = ControlEntry {
                        element: id,
                        enabled: !disabled,
                        action: ControlAction::FocusOnly,
                    };
                    let _ = cx.app.models_mut().update(&control_registry, |reg| {
                        reg.register_control(cx.window, cx.frame_id, control_id, entry);
                    });
                }
                let mut props = base_props.clone();

                let duration = overlay_motion::shadcn_motion_duration_150(cx);
                let focus_visible = cx.is_focused_element(id)
                    && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));

                let target_border = if focus_visible {
                    props.chrome.border_color_focused
                } else {
                    props.chrome.border_color
                };
                let border_motion = drive_tween_color_for_element(
                    cx,
                    id,
                    "textarea.chrome.border",
                    target_border,
                    duration,
                    tailwind_transition_ease_in_out,
                );
                props.chrome.border_color = border_motion.value;
                props.chrome.border_color_focused = border_motion.value;

                let ring_alpha = drive_tween_f32_for_element(
                    cx,
                    id,
                    "textarea.chrome.ring.alpha",
                    if focus_visible { 1.0 } else { 0.0 },
                    duration,
                    tailwind_transition_ease_in_out,
                );
                if let Some(mut ring) = props.chrome.focus_ring.take() {
                    ring.color = alpha_mul(ring.color, ring_alpha.value);
                    if let Some(offset_color) = ring.offset_color {
                        ring.offset_color = Some(alpha_mul(offset_color, ring_alpha.value));
                    }
                    props.chrome.focus_ring = Some(ring);
                }
                props.focus_ring_always_paint = ring_alpha.animating;

                props
            });

            let textarea = if labelled_by_element.is_some() || described_by_element.is_some() {
                let mut decoration = SemanticsDecoration::default();
                if let Some(label) = labelled_by_element {
                    decoration = decoration.labelled_by_element(label.0);
                }
                if let Some(desc) = described_by_element {
                    decoration = decoration.described_by_element(desc.0);
                }
                textarea.attach_semantics(decoration)
            } else {
                textarea
            };

            let resize_handle = cx.pressable_with_id_props(move |cx, _st, id| {
                let height_override_down = height_override.clone();
                let drag_down = drag.clone();
                cx.pressable_on_pointer_down_for(
                    id,
                    Arc::new(move |host, _action_cx, down| {
                        if down.button != MouseButton::Left {
                            return action::PressablePointerDownResult::SkipDefault;
                        }

                        host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                        host.capture_pointer();
                        host.set_cursor_icon(CursorIcon::NwseResize);

                        let start = down.position_window.unwrap_or(down.position);
                        let start_height = host
                            .models_mut()
                            .read(&height_override_down, |v| *v)
                            .ok()
                            .flatten()
                            .unwrap_or(min_height);

                        let _ = host.models_mut().update(&drag_down, |v| {
                            *v = Some(TextareaResizeDrag {
                                start,
                                start_height,
                            });
                        });

                        action::PressablePointerDownResult::SkipDefaultAndStopPropagation
                    }),
                );

                let height_override_move = height_override.clone();
                let drag_move = drag.clone();
                cx.pressable_on_pointer_move_for(
                    id,
                    Arc::new(move |host, action_cx, mv| {
                        host.set_cursor_icon(CursorIcon::NwseResize);

                        let Some(drag) = host.models_mut().read(&drag_move, |v| *v).ok().flatten()
                        else {
                            return false;
                        };

                        let current = mv.position_window.unwrap_or(mv.position);
                        let delta = current.y.0 - drag.start.y.0;
                        let next_height = Px((drag.start_height.0 + delta).max(min_height.0));

                        let _ = host
                            .models_mut()
                            .update(&height_override_move, |v| *v = Some(next_height));

                        host.request_redraw(action_cx.window);
                        true
                    }),
                );

                let drag_up = drag.clone();
                cx.pressable_on_pointer_up_for(
                    id,
                    Arc::new(move |host, _action_cx, _up| {
                        host.release_pointer_capture();
                        let _ = host.models_mut().update(&drag_up, |v| *v = None);
                        action::PressablePointerUpResult::SkipActivate
                    }),
                );

                let mut pressable = PressableProps::default();
                pressable.layout = resize_handle_layout;
                let dot_color = alpha_mul(grip_color, 0.65);
                let dot_size = Px(2.0);
                let dot_radius = Px(1.0);

                let dot = |cx: &mut ElementContext<'_, H>, right: Px, bottom: Px| {
                    cx.container(
                        ContainerProps {
                            layout: decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default()
                                    .absolute()
                                    .right_px(right)
                                    .bottom_px(bottom)
                                    .w_px(dot_size)
                                    .h_px(dot_size),
                            ),
                            padding: Edges::all(Px(0.0)).into(),
                            background: Some(dot_color),
                            shadow: None,
                            border: Edges::all(Px(0.0)),
                            border_color: None,
                            corner_radii: Corners::all(dot_radius),
                            ..Default::default()
                        },
                        move |_cx| [],
                    )
                };

                let grip = cx.container(
                    ContainerProps {
                        layout: grip_layout,
                        padding: Edges::all(Px(0.0)).into(),
                        background: None,
                        shadow: None,
                        border: Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: Corners::all(Px(0.0)),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            dot(cx, Px(2.0), Px(2.0)),
                            dot(cx, Px(5.0), Px(5.0)),
                            dot(cx, Px(8.0), Px(8.0)),
                        ]
                    },
                );

                (pressable, vec![grip])
            });

            vec![textarea, resize_handle]
        },
    );

    if disabled {
        cx.opacity(0.5, move |_cx| vec![root])
    } else {
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size as CoreSize};
    use fret_core::{
        Modifiers, PathCommand, Size as UiSize, SvgId, SvgService, TextBlobId, TextConstraints,
        TextMetrics, TextService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_runtime::FrameId;
    use fret_ui::element::{ElementKind, Length};
    use fret_ui::elements;
    use fret_ui::{UiTree, focus_visible};
    use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: UiSize::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
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

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn textarea_wraps_in_shadow_container_like_shadcn() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(320.0), Px(180.0)),
        );

        let model = app.models_mut().insert(String::new());
        let el =
            elements::with_element_cx(&mut app, window, bounds, "textarea-shadow-wrapper", |cx| {
                Textarea::new(model.clone())
                    .a11y_label("Textarea")
                    .resizable(false)
                    .into_element(cx)
            });

        let ElementKind::Container(root) = &el.kind else {
            panic!(
                "expected Textarea root to be a shadow container, got {:?}",
                el.kind
            );
        };
        assert!(
            root.shadow.is_some(),
            "expected Textarea to have shadow-xs wrapper"
        );
        assert_eq!(root.layout.size.width, Length::Fill);

        let child = el.children.first().expect("shadow wrapper child");
        let ElementKind::TextArea(props) = &child.kind else {
            panic!(
                "expected shadow wrapper child to be TextArea, got {:?}",
                child.kind
            );
        };
        assert_eq!(props.layout.size.width, Length::Fill);
    }

    #[test]
    fn textarea_focus_ring_tweens_in_and_out_like_a_transition() {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::Duration;

        use fret_core::{Event, KeyCode, Rect, Size};

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(String::new());

        let ring_alpha_out: Rc<Cell<Option<f32>>> = Rc::new(Cell::new(None));
        let always_paint_out: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            model: Model<String>,
            ring_alpha_out: Rc<Cell<Option<f32>>>,
            always_paint_out: Rc<Cell<Option<bool>>>,
        ) -> fret_core::NodeId {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "textarea-focus-ring-tween",
                move |cx| {
                    let el = Textarea::new(model)
                        .a11y_label("Textarea")
                        .resizable(false)
                        .into_element(cx);

                    let ElementKind::Container(_) = &el.kind else {
                        panic!("expected Textarea to wrap in a Container");
                    };
                    let child = el.children.first().expect("textarea inner child");
                    let ElementKind::TextArea(props) = &child.kind else {
                        panic!("expected Textarea inner to be TextArea");
                    };

                    let a = props
                        .chrome
                        .focus_ring
                        .map(|ring| ring.color.a)
                        .unwrap_or(0.0);
                    ring_alpha_out.set(Some(a));
                    always_paint_out.set(Some(props.focus_ring_always_paint));

                    vec![el]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            root
        }

        // Frame 1: baseline render (no focus-visible), ring alpha should be 0.
        app.set_frame_id(FrameId(1));
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a0 = ring_alpha_out.get().expect("a0");
        assert!(
            a0.abs() <= 1e-6,
            "expected ring alpha to start at 0, got {a0}"
        );

        // Focus the textarea and mark focus-visible via a navigation key.
        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable textarea");
        ui.set_focus(Some(focusable));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(
            focus_visible::is_focus_visible(&mut app, Some(window)),
            "sanity: focus-visible should be enabled after navigation key"
        );

        // Frame 2: ring should be in-between (not snapped).
        app.set_frame_id(FrameId(2));
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a1 = ring_alpha_out.get().expect("a1");
        assert!(
            a1 > 0.0,
            "expected ring alpha to start animating in, got {a1}"
        );

        // Advance frames until the default 150ms transition settles.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_focused = ring_alpha_out.get().expect("a_focused");
        assert!(
            a_focused > a1 + 1e-4,
            "expected ring alpha to increase over time, got a1={a1} a_focused={a_focused}"
        );

        // Blur and ensure ring animates out while still being painted.
        ui.set_focus(None);
        app.set_frame_id(FrameId(1000));
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a_blur = ring_alpha_out.get().expect("a_blur");
        let always_paint = always_paint_out.get().expect("always_paint");
        assert!(
            a_blur > 0.0 && a_blur < a_focused,
            "expected ring alpha to be intermediate after blur, got a_blur={a_blur} a_focused={a_focused}"
        );
        assert!(
            always_paint,
            "expected focus ring to request painting while animating out"
        );

        for i in 0..settle {
            app.set_frame_id(FrameId(1001 + i));
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_final = ring_alpha_out.get().expect("a_final");
        let always_paint_final = always_paint_out.get().expect("always_paint_final");
        assert!(
            a_final.abs() <= 1e-4,
            "expected ring alpha to settle at 0, got {a_final}"
        );
        assert!(
            !always_paint_final,
            "expected focus ring to stop requesting painting after settling"
        );
    }

    #[test]
    fn textarea_can_reference_a_label_element_for_a11y_association() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(320.0), Px(180.0)),
        );

        let model = app.models_mut().insert(String::new());
        let root = elements::with_element_cx(&mut app, window, bounds, "labelled-textarea", |cx| {
            let label = crate::label::Label::new("Notes").into_element(cx);
            let label_id = label.id;

            let textarea = Textarea::new(model.clone())
                .a11y_label("Textarea")
                .labelled_by_element(label_id)
                .resizable(false)
                .into_element(cx);

            cx.column(fret_ui::element::ColumnProps::default(), |_cx| {
                vec![label, textarea]
            })
        });

        let textarea = root.children.get(1).expect("textarea child");
        let ElementKind::Container(_) = &textarea.kind else {
            panic!("expected Textarea to wrap in a Container");
        };

        let text_area = textarea.children.first().expect("text area");
        let ElementKind::TextArea(_) = &text_area.kind else {
            panic!("expected Textarea inner node to be a TextArea");
        };
        let decoration = text_area
            .semantics_decoration
            .as_ref()
            .expect("expected labelled_by decoration on TextArea");
        assert_eq!(decoration.labelled_by_element, Some(root.children[0].id.0));
    }
}
