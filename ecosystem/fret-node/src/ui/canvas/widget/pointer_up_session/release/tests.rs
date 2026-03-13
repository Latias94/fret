use super::*;

#[test]
fn take_active_release_clears_pending_companion() {
    let mut active = Some(1_u32);
    let mut pending = Some(2_u32);

    let taken = take_active_release(&mut active, &mut pending);

    assert_eq!(taken, Some(1));
    assert_eq!(active, None);
    assert_eq!(pending, None);
}
