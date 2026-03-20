//! AI Elements-aligned `Checkpoint` surfaces.

use std::any::Any;
use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_runtime::ActionId;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, Space};
use fret_ui_kit::{WidgetStateProperty, WidgetStates};
use fret_ui_shadcn::facade::{
    Button, ButtonSize, ButtonVariant, Separator, Tooltip, TooltipAlign, TooltipContent,
    TooltipSide,
};
use fret_ui_shadcn::raw::button::ButtonStyle;

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>;

/// Checkpoint row aligned with AI Elements `Checkpoint`.
#[derive(Debug)]
pub struct Checkpoint {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Checkpoint {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .overflow_hidden(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Self {
            children,
            test_id,
            layout,
            chrome,
        } = self;
        Self {
            children: Vec::new(),
            test_id,
            layout,
            chrome,
        }
        .into_element_with_children(cx, move |_cx| children)
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = theme.color_token("muted-foreground");
        let separator = Separator::new()
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            .into_element(cx);

        let row = ui::h_row(move |cx| {
            let mut out = children(cx);
            out.push(separator);
            out
        })
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N0p5)
        .items(Items::Center)
        .into_element(cx)
        .inherit_foreground(muted_fg);
        let row = cx.container(
            decl_style::container_props(&theme, self.chrome, self.layout),
            move |_cx| vec![row],
        );

        let Some(test_id) = self.test_id else {
            return row;
        };
        row.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Default icon aligned with AI Elements `CheckpointIcon` (Bookmark).
#[derive(Debug)]
pub struct CheckpointIcon {
    children: Vec<AnyElement>,
    icon: fret_icons::IconId,
    size: Px,
    color: Option<ColorRef>,
    layout: LayoutRefinement,
}

impl Default for CheckpointIcon {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            icon: fret_icons::IconId::new_static("lucide.bookmark"),
            size: Px(16.0),
            color: None,
            layout: LayoutRefinement::default().flex_shrink_0(),
        }
    }
}

impl CheckpointIcon {
    pub fn children(mut self, child: AnyElement) -> Self {
        self.children = vec![child];
        self
    }

    pub fn children_many(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn icon_id(mut self, icon: fret_icons::IconId) -> Self {
        self.icon = icon;
        self
    }

    pub fn size(mut self, size: Px) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let children = if self.children.is_empty() {
            let fg = theme.color_token("muted-foreground");
            let color = self.color.unwrap_or(ColorRef::Color(fg));
            vec![decl_icon::icon_with(
                cx,
                self.icon,
                Some(self.size),
                Some(color),
            )]
        } else {
            self.children
        };
        let layout = decl_style::layout_style(&theme, self.layout);
        cx.container(
            fret_ui::element::ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| children,
        )
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.children_many(children(cx)).into_element(cx)
    }
}

/// Trigger button aligned with AI Elements `CheckpointTrigger`.
pub struct CheckpointTrigger {
    children: Vec<AnyElement>,
    a11y_label: Arc<str>,
    tooltip: Option<Arc<str>>,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
    variant: ButtonVariant,
    size: ButtonSize,
    muted_foreground: bool,
    test_id: Option<Arc<str>>,
    tooltip_panel_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for CheckpointTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckpointTrigger")
            .field("children_len", &self.children.len())
            .field("a11y_label", &self.a11y_label)
            .field("has_tooltip", &self.tooltip.is_some())
            .field("action", &self.action)
            .field("action_payload", &self.action_payload.is_some())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("test_id", &self.test_id.as_deref())
            .field(
                "tooltip_panel_test_id",
                &self.tooltip_panel_test_id.as_deref(),
            )
            .finish()
    }
}

impl CheckpointTrigger {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            a11y_label: Arc::<str>::from("Restore checkpoint"),
            tooltip: None,
            action: None,
            action_payload: None,
            on_activate: None,
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Sm,
            muted_foreground: true,
            test_id: None,
            tooltip_panel_test_id: None,
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = label.into();
        self
    }

    pub fn children_many(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<Arc<str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Bind a stable action ID to this checkpoint trigger (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized checkpoint-trigger actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`CheckpointTrigger::action_payload`], but computes the payload lazily.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// When true (default), uses `text-muted-foreground` for the idle state while preserving
    /// shadcn's hover/active foreground overrides (aligns AI Elements `Checkpoint` root text color).
    pub fn muted_foreground(mut self, enabled: bool) -> Self {
        self.muted_foreground = enabled;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn tooltip_panel_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.tooltip_panel_test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let button = {
            let a11y_label = self.a11y_label;
            let children = self.children;
            let variant = self.variant;
            let size = self.size;
            let muted_foreground = self.muted_foreground;
            let test_id = self.test_id.clone();
            let mut b = Button::new(a11y_label)
                .children(children)
                .variant(variant)
                .size(size);
            if muted_foreground && variant == ButtonVariant::Ghost {
                let muted = ColorRef::Token {
                    key: "muted-foreground",
                    fallback: ColorFallback::ThemeTextMuted,
                };
                b = b.style(
                    ButtonStyle::default().foreground(
                        WidgetStateProperty::new(Some(muted))
                            .when(WidgetStates::HOVERED, None)
                            .when(WidgetStates::ACTIVE, None),
                    ),
                );
            }
            if let Some(action) = self.action {
                b = b.action(action);
            }
            if let Some(payload) = self.action_payload {
                b = b.action_payload_factory(payload);
            }
            if let Some(on_activate) = self.on_activate {
                b = b.on_activate(on_activate);
            }
            if let Some(test_id) = test_id {
                b = b.test_id(test_id);
            }
            b.into_element(cx)
        };

        let Some(tooltip) = self.tooltip else {
            return button;
        };

        let panel_test_id = self.tooltip_panel_test_id.or_else(|| {
            self.test_id
                .as_ref()
                .map(|id| Arc::<str>::from(format!("{id}-tooltip-panel")))
        });

        let content = TooltipContent::build(cx, |_cx| [TooltipContent::text::<H, _>(tooltip)]);
        let mut tooltip = Tooltip::new(cx, button, content)
            .align(TooltipAlign::Start)
            .side(TooltipSide::Bottom);
        if let Some(panel_test_id) = panel_test_id {
            tooltip = tooltip.panel_test_id(panel_test_id);
        }
        tooltip.into_element(cx)
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.children_many(children(cx)).into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use crate::{Conversation, ConversationContent, Message, MessageContent, MessageRole};
    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, FrameId, Modifiers, MouseButtons, PathCommand, PathConstraints, PathId,
        PathMetrics, PathService, PathStyle, Point, PointerEvent, PointerId, PointerType, Px, Rect,
        Size, SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_ui::element::{AnyElement, ElementKind};
    use fret_ui::tree::UiTree;
    use fret_ui_kit::OverlayController;

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
                    size: fret_core::Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
                    baseline: fret_core::Px(8.0),
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

    fn contains_foreground_scope(el: &AnyElement) -> bool {
        matches!(el.kind, ElementKind::ForegroundScope(_))
            || el.children.iter().any(contains_foreground_scope)
    }

    fn find_first_inherited_foreground_node(el: &AnyElement) -> Option<&AnyElement> {
        if el.inherited_foreground.is_some() {
            return Some(el);
        }
        el.children
            .iter()
            .find_map(find_first_inherited_foreground_node)
    }

    fn count_text_nodes(el: &AnyElement) -> usize {
        let self_count = usize::from(matches!(el.kind, ElementKind::Text(_)));
        self_count + el.children.iter().map(count_text_nodes).sum::<usize>()
    }

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        request_semantics: bool,
        root: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
    ) {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "checkpoint-trigger-tooltip-hover",
            root,
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        if request_semantics {
            ui.request_semantics_snapshot();
        }
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn find_by_test_id<'a>(
        snap: &'a fret_core::SemanticsSnapshot,
        id: &str,
    ) -> &'a fret_core::SemanticsNode {
        snap.nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(id))
            .unwrap_or_else(|| panic!("missing semantics node with test_id={id:?}"))
    }

    fn has_test_id(snap: &fret_core::SemanticsSnapshot, id: &str) -> bool {
        snap.nodes
            .iter()
            .any(|node| node.test_id.as_deref() == Some(id))
    }

    fn pointer_move_at(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        position: Point,
    ) {
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(PointerEvent::Move {
                pointer_id: PointerId(0),
                position,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn build_checkpoint_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
        vec![
            CheckpointTrigger::new([cx.text("Restore checkpoint")])
                .tooltip("Restore workspace and chat to this point")
                .test_id("tooltip-trigger")
                .tooltip_panel_test_id("tooltip-panel")
                .into_element(cx),
        ]
    }

    fn build_scrolled_checkpoint_tooltip(
        cx: &mut ElementContext<'_, App>,
        scroll_handle: fret_ui::scroll::ScrollHandle,
    ) -> Vec<AnyElement> {
        let mut layout = fret_ui::element::LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Px(Px(320.0));
        layout.size.height = fret_ui::element::Length::Px(Px(180.0));

        vec![cx.scroll(
            fret_ui::element::ScrollProps {
                layout,
                axis: fret_ui::element::ScrollAxis::Y,
                scroll_handle: Some(scroll_handle),
                intrinsic_measure_mode: fret_ui::element::ScrollIntrinsicMeasureMode::Content,
                windowed_paint: false,
                probe_unbounded: true,
            },
            |cx| {
                let mut content_children: Vec<AnyElement> = Vec::new();
                for _ in 0..24 {
                    content_children.push(cx.text("filler"));
                }
                content_children.push(
                    CheckpointTrigger::new([cx.text("Restore checkpoint")])
                        .tooltip("Restore workspace and chat to this point")
                        .test_id("tooltip-trigger")
                        .tooltip_panel_test_id("tooltip-panel")
                        .into_element(cx),
                );
                for _ in 0..24 {
                    content_children.push(cx.text("filler"));
                }

                vec![cx.column(
                    fret_ui::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    move |_cx| content_children,
                )]
            },
        )]
    }

    fn build_conversation_checkpoint_tooltip(
        cx: &mut ElementContext<'_, App>,
        scroll_handle: fret_ui::scroll::ScrollHandle,
    ) -> Vec<AnyElement> {
        let conversation = Conversation::new([])
            .scroll_handle(scroll_handle)
            .stick_to_bottom(false)
            .show_scroll_to_bottom_button(false)
            .content_revision(1)
            .into_element_with_children(cx, move |cx| {
                let mut content_children: Vec<AnyElement> = Vec::new();

                for idx in 0..18 {
                    content_children.push(
                        Message::new(
                            MessageRole::Assistant,
                            [MessageContent::new(
                                MessageRole::Assistant,
                                [cx.text(format!("Message {idx}"))],
                            )
                            .into_element(cx)],
                        )
                        .into_element(cx),
                    );

                    if idx == 12 {
                        content_children.push(
                            Checkpoint::new([
                                CheckpointIcon::default().into_element(cx),
                                CheckpointTrigger::new([cx.text("Restore checkpoint")])
                                    .tooltip("Restore workspace and chat to this point")
                                    .test_id("tooltip-trigger")
                                    .tooltip_panel_test_id("tooltip-panel")
                                    .into_element(cx),
                            ])
                            .test_id("checkpoint-row")
                            .into_element(cx),
                        );
                    }
                }

                vec![ConversationContent::new(content_children).into_element(cx)]
            });

        let mut layout = fret_ui::element::LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Px(Px(360.0));
        layout.size.height = fret_ui::element::Length::Px(Px(180.0));

        vec![cx.container(
            fret_ui::element::ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| vec![conversation],
        )]
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(200.0)),
        )
    }

    fn ticks_100() -> u64 {
        fret_ui_kit::declarative::transition::ticks_60hz_for_duration(Duration::from_millis(100))
    }

    #[test]
    fn checkpoint_keeps_group_role_when_stamping_test_id() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Checkpoint::new([cx.text("Restore point")])
                    .test_id("ui-ai-checkpoint-row")
                    .into_element(cx)
            });

        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Group)
        );
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref()),
            Some("ui-ai-checkpoint-row")
        );
    }

    #[test]
    fn checkpoint_icon_accepts_custom_child_content() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                CheckpointIcon::default()
                    .children(cx.text("Custom icon"))
                    .into_element(cx)
            });

        assert!(matches!(element.kind, ElementKind::Container(_)));
        assert_eq!(element.children.len(), 1);
        assert!(matches!(element.children[0].kind, ElementKind::Text(_)));
    }

    #[test]
    fn checkpoint_icon_accepts_multiple_custom_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                CheckpointIcon::default()
                    .children_many([cx.text("A"), cx.text("B")])
                    .into_element(cx)
            });

        assert!(matches!(element.kind, ElementKind::Container(_)));
        assert_eq!(element.children.len(), 2);
        assert!(matches!(element.children[0].kind, ElementKind::Text(_)));
        assert!(matches!(element.children[1].kind, ElementKind::Text(_)));
    }

    #[test]
    fn checkpoint_root_inherits_muted_foreground_for_custom_children_without_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let expected_fg = theme.color_token("muted-foreground");

            let element = Checkpoint::new([
                CheckpointIcon::default()
                    .children(cx.text("Custom icon"))
                    .into_element(cx),
                CheckpointTrigger::new([cx.text("Restore checkpoint")]).into_element(cx),
            ])
            .into_element(cx);

            let inherited = find_first_inherited_foreground_node(&element)
                .expect("expected checkpoint subtree to carry inherited foreground");

            assert_eq!(inherited.inherited_foreground, Some(expected_fg));
            assert!(
                !contains_foreground_scope(&element),
                "expected checkpoint root to attach inherited foreground without inserting a ForegroundScope"
            );
        });
    }

    #[test]
    fn checkpoint_into_element_with_children_preserves_defaults() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                let theme = Theme::global(&*cx.app).clone();
                let expected_fg = theme.color_token("muted-foreground");

                let element = Checkpoint::new(Vec::<AnyElement>::new())
                    .test_id("ui-ai-checkpoint-row")
                    .into_element_with_children(cx, |cx| {
                        vec![
                            CheckpointIcon::default().into_element_with_children(cx, |cx| {
                                vec![cx.text("⟲"), cx.text("•")]
                            }),
                            CheckpointTrigger::new([cx.text("Restore checkpoint")])
                                .into_element(cx),
                        ]
                    });

                assert_eq!(
                    element.semantics_decoration.as_ref().and_then(|d| d.role),
                    Some(SemanticsRole::Group)
                );
                assert_eq!(
                    element
                        .semantics_decoration
                        .as_ref()
                        .and_then(|d| d.test_id.as_deref()),
                    Some("ui-ai-checkpoint-row")
                );
                let inherited = find_first_inherited_foreground_node(&element)
                    .expect("expected checkpoint subtree to carry inherited foreground");
                assert_eq!(inherited.inherited_foreground, Some(expected_fg));
                element
            });

        assert!(matches!(element.kind, ElementKind::Container(_)));
    }

    #[test]
    fn checkpoint_trigger_into_element_with_children_keeps_custom_content() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                CheckpointTrigger::new(Vec::<AnyElement>::new())
                    .into_element_with_children(cx, |cx| {
                        vec![cx.text("Restore"), cx.text("checkpoint")]
                    })
            });

        assert!(
            count_text_nodes(&element) >= 2,
            "expected checkpoint trigger subtree to preserve multiple custom child nodes"
        );
    }

    #[test]
    fn checkpoint_trigger_tooltip_hover_opens() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(480.0)),
        );

        let mut app = App::new();
        fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
            &mut app,
            fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
            fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
        );

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            build_checkpoint_tooltip,
        );

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let trigger = find_by_test_id(&snap, "tooltip-trigger");

        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            Point::new(
                Px(trigger.bounds.origin.x.0 + 5.0),
                Px(trigger.bounds.origin.y.0 + 5.0),
            ),
        );

        let open_settle_frames = ticks_100() + 2;
        for tick in 0..open_settle_frames {
            let request_semantics = tick + 1 == open_settle_frames;
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                request_semantics,
                build_checkpoint_tooltip,
            );
        }

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert!(
            has_test_id(&snap, "tooltip-panel"),
            "expected checkpoint trigger tooltip panel to be present after hover"
        );
    }

    #[test]
    fn checkpoint_trigger_tooltip_hover_opens_inside_scrolled_viewport() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(480.0)),
        );

        let mut app = App::new();
        fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
            &mut app,
            fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
            fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
        );

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;
        let scroll_handle = fret_ui::scroll::ScrollHandle::default();

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |cx| build_scrolled_checkpoint_tooltip(cx, scroll_handle.clone()),
        );

        scroll_handle.set_offset(Point::new(Px(0.0), Px(200.0)));

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |cx| build_scrolled_checkpoint_tooltip(cx, scroll_handle.clone()),
        );

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let trigger = find_by_test_id(&snap, "tooltip-trigger");

        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            Point::new(
                Px(trigger.bounds.origin.x.0 + 5.0),
                Px(trigger.bounds.origin.y.0 + 5.0),
            ),
        );

        let open_settle_frames = ticks_100() + 2;
        for tick in 0..open_settle_frames {
            let request_semantics = tick + 1 == open_settle_frames;
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                request_semantics,
                |cx| build_scrolled_checkpoint_tooltip(cx, scroll_handle.clone()),
            );
        }

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert!(
            has_test_id(&snap, "tooltip-panel"),
            "expected checkpoint trigger tooltip panel to open inside a scrolled viewport"
        );
    }

    #[test]
    fn checkpoint_trigger_tooltip_hover_opens_inside_conversation_scroll_area() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(480.0)),
        );

        let mut app = App::new();
        fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
            &mut app,
            fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
            fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
        );

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;
        let scroll_handle = fret_ui::scroll::ScrollHandle::default();

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |cx| build_conversation_checkpoint_tooltip(cx, scroll_handle.clone()),
        );

        scroll_handle.set_offset(Point::new(Px(0.0), Px(420.0)));

        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |cx| build_conversation_checkpoint_tooltip(cx, scroll_handle.clone()),
        );

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let trigger = find_by_test_id(&snap, "tooltip-trigger");

        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            Point::new(
                Px(trigger.bounds.origin.x.0 + 5.0),
                Px(trigger.bounds.origin.y.0 + 5.0),
            ),
        );

        let open_settle_frames = ticks_100() + 2;
        for tick in 0..open_settle_frames {
            let request_semantics = tick + 1 == open_settle_frames;
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                request_semantics,
                |cx| build_conversation_checkpoint_tooltip(cx, scroll_handle.clone()),
            );
        }

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert!(
            has_test_id(&snap, "tooltip-panel"),
            "expected checkpoint trigger tooltip panel to open inside Conversation's ScrollArea"
        );
    }
}
