use fret_app::App;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_workspace::WorkspaceTopBar;

pub(super) fn top_bar_view(cx: &mut ElementContext<'_, App>, left: Vec<AnyElement>) -> AnyElement {
    WorkspaceTopBar::new().left(left).into_element(cx)
}
