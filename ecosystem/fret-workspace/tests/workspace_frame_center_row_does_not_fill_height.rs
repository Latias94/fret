use fret_app::App;
use fret_core::{AppWindowId, Axis, Point, Px, Rect, Size};
use fret_ui::element::{ElementKind, Length};
use fret_ui::elements;
use fret_workspace::WorkspaceFrame;

#[test]
fn workspace_frame_center_row_does_not_fill_height() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
        let top = cx.text("top");
        let center = cx.text("center");
        let bottom = cx.text("bottom");

        let root = WorkspaceFrame::new(center)
            .top(top)
            .bottom(bottom)
            .into_element(cx);

        let vertical = root
            .children
            .iter()
            .find_map(|child| match &child.kind {
                ElementKind::Flex(props) if props.direction == Axis::Vertical => Some(child),
                _ => None,
            })
            .expect("expected WorkspaceFrame root container to include a vertical flex");

        assert_eq!(
            vertical.children.len(),
            3,
            "expected [top, center_row, bottom] children"
        );

        let center_row = &vertical.children[1];
        let ElementKind::Flex(props) = &center_row.kind else {
            panic!("expected center row to be a flex element");
        };

        assert_eq!(props.direction, Axis::Horizontal);
        assert_eq!(
            props.layout.size.height,
            Length::Auto,
            "expected center row height to be auto (flex-grow should allocate remaining space)"
        );
        assert!(
            props.layout.flex.grow > 0.0,
            "expected center row to flex-grow"
        );
    });
}
