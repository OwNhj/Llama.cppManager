use llama_gui::views::model_view::ModelView;

#[test]
fn test_model_view_creation() {
    let view = ModelView::new();
    assert!(view.selected_path.is_none());
}

#[test]
fn test_model_view_selected_path_none_initially() {
    let view = ModelView::new();
    assert_eq!(view.selected_path, None);
}

#[test]
fn test_progress_clamping() {
    // Test that progress values are clamped to [0.0, 1.0]
    assert_eq!((-0.5_f32).clamp(0.0, 1.0), 0.0);
    assert_eq!((0.0_f32).clamp(0.0, 1.0), 0.0);
    assert_eq!((0.5_f32).clamp(0.0, 1.0), 0.5);
    assert_eq!((1.0_f32).clamp(0.0, 1.0), 1.0);
    assert_eq!((1.5_f32).clamp(0.0, 1.0), 1.0);
    assert!((f32::NAN).clamp(0.0, 1.0).is_nan());
    assert_eq!((f32::INFINITY).clamp(0.0, 1.0), 1.0);
    assert_eq!((f32::NEG_INFINITY).clamp(0.0, 1.0), 0.0);
}
