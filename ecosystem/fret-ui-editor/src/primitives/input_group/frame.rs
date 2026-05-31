use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::visuals::{
    EditorFrameSemanticState, EditorFrameState, EditorFrameVisuals, EditorWidgetVisuals,
};

pub(crate) fn editor_input_group_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    density: EditorDensity,
    chrome: ResolvedEditorFrameChrome,
    state: EditorFrameState,
    contents: impl FnOnce(&mut ElementContext<'_, H>, EditorFrameVisuals) -> Vec<AnyElement>,
) -> AnyElement {
    editor_input_group_frame_with_overrides(
        cx,
        layout,
        density,
        chrome,
        state,
        EditorInputGroupFrameOverrides::none(),
        contents,
    )
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct EditorInputGroupFrameOverrides {
    pub(crate) bg: Option<Color>,
    pub(crate) border: Option<Color>,
    pub(crate) semantic: Option<EditorFrameSemanticState>,
}

impl EditorInputGroupFrameOverrides {
    pub(crate) fn none() -> Self {
        Self {
            bg: None,
            border: None,
            semantic: None,
        }
    }
}

pub(crate) fn editor_input_group_frame_with_overrides<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut layout: LayoutStyle,
    density: EditorDensity,
    chrome: ResolvedEditorFrameChrome,
    state: EditorFrameState,
    overrides: EditorInputGroupFrameOverrides,
    contents: impl FnOnce(&mut ElementContext<'_, H>, EditorFrameVisuals) -> Vec<AnyElement>,
) -> AnyElement {
    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(Length::Px(density.row_height));
    }

    let mut state = state;
    if let Some(semantic) = overrides.semantic {
        state.semantic = semantic;
    }

    let theme = Theme::global(&*cx.app);
    let mut visuals = EditorWidgetVisuals::new(theme).frame_visuals(chrome, state);
    if let Some(bg) = overrides.bg {
        visuals.bg = bg;
    }
    if let Some(border) = overrides.border {
        visuals.border = border;
    }

    cx.container(
        ContainerProps {
            layout,
            padding: Edges::all(Px(0.0)).into(),
            background: Some(visuals.bg),
            border: Edges::all(chrome.border_width),
            border_color: Some(visuals.border),
            corner_radii: Corners::all(chrome.radius),
            ..Default::default()
        },
        move |cx| contents(cx, visuals),
    )
}
