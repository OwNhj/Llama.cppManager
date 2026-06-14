use llama_gui::views::offload_view::OffloadView;

#[test]
fn test_offload_view_creation_default() {
    let view = OffloadView::new();
    assert_eq!(view.total_layers(), 0);
}

#[test]
fn test_offload_view_set_model_info() {
    let mut view = OffloadView::new();
    assert_eq!(view.total_layers(), 0);
    
    view.set_model_info("test-model", 32);
    assert_eq!(view.total_layers(), 32);
}

#[test]
fn test_offload_view_clear_model_info() {
    let mut view = OffloadView::new();
    view.set_model_info("test-model", 64);
    assert_eq!(view.total_layers(), 64);
    
    view.clear_model_info();
    assert_eq!(view.total_layers(), 0);
}
