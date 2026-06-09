use super::harness::CollectionDragScenario;
use std::sync::Arc;

#[test]
fn collection_drag_payload_preserves_selected_keys_across_order_flip() {
    let mut scenario = CollectionDragScenario::new();

    scenario.render_frame();
    assert!(scenario.selected_ids().is_empty());
    assert!(scenario.preview_ids().is_empty());
    assert!(scenario.delivered_ids().is_empty());

    scenario.click_asset("beta");
    scenario.advance_and_render();
    assert_eq!(scenario.selected_ids(), vec![Arc::<str>::from("beta")]);

    scenario.meta_click_asset("delta");
    scenario.advance_and_render();
    assert_eq!(
        scenario.selected_ids(),
        vec![Arc::<str>::from("beta"), Arc::<str>::from("delta")]
    );

    scenario.set_reverse_order(true);
    scenario.advance_and_render();
    assert_eq!(
        scenario.selected_ids(),
        vec![Arc::<str>::from("beta"), Arc::<str>::from("delta")]
    );

    scenario.start_drag_to_target("delta");
    scenario.advance_and_render();
    assert_eq!(
        scenario.preview_ids(),
        vec![Arc::<str>::from("beta"), Arc::<str>::from("delta")]
    );
    assert_eq!(
        scenario.preview_paths(),
        vec![
            Arc::<str>::from("textures/beta.ktx2"),
            Arc::<str>::from("textures/delta.ktx2")
        ]
    );
    assert!(scenario.delivered_ids().is_empty());

    scenario.drop_on_target();
    scenario.advance_and_render();
    assert_eq!(
        scenario.delivered_ids(),
        vec![Arc::<str>::from("beta"), Arc::<str>::from("delta")]
    );
    assert_eq!(
        scenario.delivered_paths(),
        vec![
            Arc::<str>::from("textures/beta.ktx2"),
            Arc::<str>::from("textures/delta.ktx2")
        ]
    );
}
