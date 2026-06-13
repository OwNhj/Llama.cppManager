use llama_gui::views::offload_view::OffloadView;

#[test]
fn test_offload_view_creation_default() {
    let view = OffloadView::new();
    assert_eq!(view.total_layers(), 32);
}

#[test]
fn test_offload_view_creation_with_total_layers() {
    let view = OffloadView::with_total_layers(48);
    assert_eq!(view.total_layers(), 48);
}

#[test]
fn test_set_total_layers() {
    let mut view = OffloadView::new();
    assert_eq!(view.total_layers(), 32);

    view.set_total_layers(64);
    assert_eq!(view.total_layers(), 64);

    view.set_total_layers(0);
    assert_eq!(view.total_layers(), 0);
}
