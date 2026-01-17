use super::prelude::*;

#[derive(Default)]
pub(crate) struct ElementFrame {
    pub(super) windows: HashMap<AppWindowId, WindowFrame>,
}

pub(crate) struct WindowFrame {
    pub(super) frame_id: FrameId,
    pub(crate) instances: HashMap<NodeId, ElementRecord>,
    pub(crate) children: HashMap<NodeId, Vec<NodeId>>,
}

impl Default for WindowFrame {
    fn default() -> Self {
        Self {
            frame_id: FrameId(0),
            instances: HashMap::new(),
            children: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ElementInstance {
    Container(ContainerProps),
    Semantics(crate::element::SemanticsProps),
    SemanticFlex(crate::element::SemanticFlexProps),
    FocusScope(FocusScopeProps),
    InteractivityGate(crate::element::InteractivityGateProps),
    Opacity(crate::element::OpacityProps),
    EffectLayer(crate::element::EffectLayerProps),
    ViewCache(crate::element::ViewCacheProps),
    VisualTransform(VisualTransformProps),
    RenderTransform(crate::element::RenderTransformProps),
    Anchored(crate::element::AnchoredProps),
    Pressable(PressableProps),
    PointerRegion(PointerRegionProps),
    InternalDragRegion(crate::element::InternalDragRegionProps),
    DismissibleLayer(DismissibleLayerProps),
    RovingFlex(crate::element::RovingFlexProps),
    Stack(StackProps),
    Spacer(SpacerProps),
    Text(TextProps),
    StyledText(crate::element::StyledTextProps),
    SelectableText(crate::element::SelectableTextProps),
    TextInput(crate::element::TextInputProps),
    TextArea(crate::element::TextAreaProps),
    ResizablePanelGroup(crate::element::ResizablePanelGroupProps),
    VirtualList(crate::element::VirtualListProps),
    Flex(FlexProps),
    Grid(crate::element::GridProps),
    Image(crate::element::ImageProps),
    Canvas(crate::element::CanvasProps),
    ViewportSurface(crate::element::ViewportSurfaceProps),
    SvgIcon(crate::element::SvgIconProps),
    Spinner(SpinnerProps),
    HoverRegion(HoverRegionProps),
    WheelRegion(crate::element::WheelRegionProps),
    Scroll(crate::element::ScrollProps),
    Scrollbar(crate::element::ScrollbarProps),
}

#[derive(Debug, Clone)]
pub(crate) struct ElementRecord {
    pub element: GlobalElementId,
    pub instance: ElementInstance,
}

#[derive(Clone)]
pub(crate) struct DismissibleLayerProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
}

impl std::fmt::Debug for DismissibleLayerProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = f.debug_struct("DismissibleLayerProps");
        out.field("layout", &self.layout)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl Default for DismissibleLayerProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            enabled: true,
        }
    }
}

pub(crate) fn element_record_for_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    node: NodeId,
) -> Option<ElementRecord> {
    app.with_global_mut(ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&node))
            .cloned()
    })
}

#[derive(Debug, Clone)]
pub(crate) struct ScrollHandleBinding {
    pub handle_key: usize,
    pub element: GlobalElementId,
    pub handle: crate::scroll::ScrollHandle,
}

#[derive(Default)]
pub(crate) struct ScrollHandleRegistry {
    pub(super) windows: HashMap<AppWindowId, WindowScrollHandleRegistry>,
}

pub(crate) struct WindowScrollHandleRegistry {
    pub(super) frame_id: FrameId,
    pub(super) by_handle: HashMap<usize, Vec<GlobalElementId>>,
    pub(super) handles: HashMap<usize, crate::scroll::ScrollHandle>,
    pub(super) last_revision: HashMap<usize, u64>,
}

impl Default for WindowScrollHandleRegistry {
    fn default() -> Self {
        Self {
            frame_id: FrameId(0),
            by_handle: HashMap::new(),
            handles: HashMap::new(),
            last_revision: HashMap::new(),
        }
    }
}

fn prepare_window_scroll_registry_for_frame(
    registry: &mut WindowScrollHandleRegistry,
    frame_id: FrameId,
) {
    if registry.frame_id != frame_id {
        registry.frame_id = frame_id;
        registry.by_handle.clear();
        registry.handles.clear();
    }
}

pub(crate) fn register_scroll_handle_bindings_batch<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    frame_id: FrameId,
    bindings: impl IntoIterator<Item = ScrollHandleBinding>,
) {
    app.with_global_mut(ScrollHandleRegistry::default, |registry, _app| {
        let window_registry = registry.windows.entry(window).or_default();
        prepare_window_scroll_registry_for_frame(window_registry, frame_id);

        for binding in bindings {
            let handle_key = binding.handle_key;
            let element = binding.element;
            let handle = binding.handle;
            window_registry
                .by_handle
                .entry(handle_key)
                .or_default()
                .push(element);
            window_registry.handles.entry(handle_key).or_insert(handle);
        }
    });
}

pub(crate) fn bound_elements_for_scroll_handle<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    handle_key: usize,
) -> Vec<GlobalElementId> {
    app.with_global_mut(ScrollHandleRegistry::default, |registry, _app| {
        registry
            .windows
            .get(&window)
            .and_then(|window_registry| window_registry.by_handle.get(&handle_key))
            .cloned()
            .unwrap_or_default()
    })
}

pub(crate) fn take_changed_scroll_handle_keys<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
) -> Vec<usize> {
    app.with_global_mut(ScrollHandleRegistry::default, |registry, _app| {
        let Some(window_registry) = registry.windows.get_mut(&window) else {
            return Vec::new();
        };

        let mut changed: Vec<usize> = Vec::new();
        for (&handle_key, handle) in window_registry.handles.iter() {
            let revision = handle.revision();
            let prev = window_registry.last_revision.get(&handle_key).copied();
            if prev != Some(revision) {
                changed.push(handle_key);
            }
            window_registry.last_revision.insert(handle_key, revision);
        }
        changed
    })
}

pub(crate) fn element_id_map_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
) -> HashMap<u64, NodeId> {
    app.with_global_mut(ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .map(|w| {
                let mut out = HashMap::with_capacity(w.instances.len());
                for (node, record) in w.instances.iter() {
                    out.insert(record.element.0, *node);
                }
                out
            })
            .unwrap_or_default()
    })
}

pub(crate) fn layout_style_for_instance(instance: &ElementInstance) -> LayoutStyle {
    match instance {
        ElementInstance::Container(p) => p.layout,
        ElementInstance::Semantics(p) => p.layout,
        ElementInstance::SemanticFlex(p) => p.flex.layout,
        ElementInstance::FocusScope(p) => p.layout,
        ElementInstance::InteractivityGate(p) => p.layout,
        ElementInstance::Opacity(p) => p.layout,
        ElementInstance::EffectLayer(p) => p.layout,
        ElementInstance::ViewCache(p) => p.layout,
        ElementInstance::VisualTransform(p) => p.layout,
        ElementInstance::RenderTransform(p) => p.layout,
        ElementInstance::Anchored(p) => p.layout,
        ElementInstance::Pressable(p) => p.layout,
        ElementInstance::PointerRegion(p) => p.layout,
        ElementInstance::InternalDragRegion(p) => p.layout,
        ElementInstance::DismissibleLayer(p) => p.layout,
        ElementInstance::RovingFlex(p) => p.flex.layout,
        ElementInstance::Stack(p) => p.layout,
        ElementInstance::Spacer(p) => p.layout,
        ElementInstance::Text(p) => p.layout,
        ElementInstance::StyledText(p) => p.layout,
        ElementInstance::SelectableText(p) => p.layout,
        ElementInstance::TextInput(p) => p.layout,
        ElementInstance::TextArea(p) => p.layout,
        ElementInstance::ResizablePanelGroup(p) => p.layout,
        ElementInstance::VirtualList(p) => p.layout,
        ElementInstance::Flex(p) => p.layout,
        ElementInstance::Grid(p) => p.layout,
        ElementInstance::Image(p) => p.layout,
        ElementInstance::Canvas(p) => p.layout,
        ElementInstance::ViewportSurface(p) => p.layout,
        ElementInstance::SvgIcon(p) => p.layout,
        ElementInstance::Spinner(p) => p.layout,
        ElementInstance::HoverRegion(p) => p.layout,
        ElementInstance::WheelRegion(p) => p.layout,
        ElementInstance::Scroll(p) => p.layout,
        ElementInstance::Scrollbar(p) => p.layout,
    }
}

pub(crate) fn layout_style_for_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    node: NodeId,
) -> LayoutStyle {
    element_record_for_node(app, window, node)
        .map(|r| layout_style_for_instance(&r.instance))
        .unwrap_or_default()
}
