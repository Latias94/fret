use super::prelude::*;

#[derive(Default)]
pub(crate) struct ElementFrame {
    pub(super) windows: HashMap<AppWindowId, WindowFrame>,
}

pub(crate) struct WindowFrame {
    pub(super) frame_id: FrameId,
    pub(crate) instances: HashMap<NodeId, ElementRecord>,
}

impl Default for WindowFrame {
    fn default() -> Self {
        Self {
            frame_id: FrameId(0),
            instances: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ElementInstance {
    Container(ContainerProps),
    Semantics(crate::element::SemanticsProps),
    FocusScope(FocusScopeProps),
    Opacity(crate::element::OpacityProps),
    VisualTransform(VisualTransformProps),
    Pressable(PressableProps),
    PointerRegion(PointerRegionProps),
    DismissibleLayer(DismissibleLayerProps),
    RovingFlex(crate::element::RovingFlexProps),
    Stack(StackProps),
    Spacer(SpacerProps),
    Text(TextProps),
    TextInput(crate::element::TextInputProps),
    TextArea(crate::element::TextAreaProps),
    ResizablePanelGroup(crate::element::ResizablePanelGroupProps),
    VirtualList(crate::element::VirtualListProps),
    Flex(FlexProps),
    Grid(crate::element::GridProps),
    Image(crate::element::ImageProps),
    SvgIcon(crate::element::SvgIconProps),
    Spinner(SpinnerProps),
    HoverRegion(HoverRegionProps),
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

pub(super) fn layout_style_for_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    node: NodeId,
) -> LayoutStyle {
    element_record_for_node(app, window, node)
        .map(|r| match r.instance {
            ElementInstance::Container(p) => p.layout,
            ElementInstance::Semantics(p) => p.layout,
            ElementInstance::FocusScope(p) => p.layout,
            ElementInstance::Opacity(p) => p.layout,
            ElementInstance::VisualTransform(p) => p.layout,
            ElementInstance::Pressable(p) => p.layout,
            ElementInstance::PointerRegion(p) => p.layout,
            ElementInstance::DismissibleLayer(p) => p.layout,
            ElementInstance::RovingFlex(p) => p.flex.layout,
            ElementInstance::Stack(p) => p.layout,
            ElementInstance::Spacer(p) => p.layout,
            ElementInstance::Text(p) => p.layout,
            ElementInstance::TextInput(p) => p.layout,
            ElementInstance::TextArea(p) => p.layout,
            ElementInstance::ResizablePanelGroup(p) => p.layout,
            ElementInstance::VirtualList(p) => p.layout,
            ElementInstance::Flex(p) => p.layout,
            ElementInstance::Grid(p) => p.layout,
            ElementInstance::Image(p) => p.layout,
            ElementInstance::SvgIcon(p) => p.layout,
            ElementInstance::Spinner(p) => p.layout,
            ElementInstance::HoverRegion(p) => p.layout,
            ElementInstance::Scroll(p) => p.layout,
            ElementInstance::Scrollbar(p) => p.layout,
        })
        .unwrap_or_default()
}
