use llama_core::params::ModelParams;

#[test]
fn test_default_params() {
    let params = ModelParams::default();
    assert_eq!(params.temperature, 0.8);
    assert_eq!(params.top_p, 0.9);
    assert_eq!(params.top_k, 40);
    assert_eq!(params.repeat_penalty, 1.1);
    assert_eq!(params.context_size, 2048);
    assert_eq!(params.batch_size, 512);
    assert_eq!(params.gpu_offload_layers, 32);
    assert!(params.flash_attention);
}

#[test]
fn test_params_validation() {
    let mut params = ModelParams::default();

    params.temperature = -0.1;
    assert!(params.validate().is_err());

    params.temperature = 2.1;
    assert!(params.validate().is_err());

    params.temperature = 1.0;
    assert!(params.validate().is_ok());
}

#[test]
fn test_validate_top_p() {
    let mut params = ModelParams::default();

    params.top_p = -0.1;
    assert!(params.validate().is_err());

    params.top_p = 1.1;
    assert!(params.validate().is_err());

    params.top_p = 0.0;
    assert!(params.validate().is_ok());

    params.top_p = 1.0;
    assert!(params.validate().is_ok());
}

#[test]
fn test_validate_top_k() {
    let mut params = ModelParams::default();

    params.top_k = 0;
    assert!(params.validate().is_err());

    params.top_k = 1;
    assert!(params.validate().is_ok());

    params.top_k = 100;
    assert!(params.validate().is_ok());
}

#[test]
fn test_validate_repeat_penalty() {
    let mut params = ModelParams::default();

    params.repeat_penalty = 0.5;
    assert!(params.validate().is_err());

    params.repeat_penalty = 2.5;
    assert!(params.validate().is_err());

    params.repeat_penalty = 1.0;
    assert!(params.validate().is_ok());

    params.repeat_penalty = 2.0;
    assert!(params.validate().is_ok());
}

#[test]
fn test_validate_context_size() {
    let mut params = ModelParams::default();

    params.context_size = 64;
    assert!(params.validate().is_err());

    params.context_size = 128;
    assert!(params.validate().is_ok());
}

#[test]
fn test_validate_batch_size() {
    let mut params = ModelParams::default();

    params.batch_size = 0;
    assert!(params.validate().is_err());

    params.batch_size = 1;
    assert!(params.validate().is_ok());
}

#[test]
fn test_validate_gpu_offload_layers() {
    let mut params = ModelParams::default();

    params.gpu_offload_layers = 0;
    assert!(params.validate().is_ok());

    params.gpu_offload_layers = 128;
    assert!(params.validate().is_ok());

    params.gpu_offload_layers = 129;
    assert!(params.validate().is_err());
}

#[test]
fn test_params_meta() {
    let meta = ModelParams::meta();
    assert_eq!(meta.len(), 11);

    let temp_meta = meta.iter().find(|m| m.name == "temperature").unwrap();
    assert_eq!(temp_meta.min, Some(0.0));
    assert_eq!(temp_meta.max, Some(2.0));
    assert_eq!(temp_meta.step, Some(0.05));
    assert_eq!(temp_meta.default, 0.8);
}

#[test]
fn test_params_serialize() {
    let params = ModelParams::default();
    let json = serde_json::to_string(&params).unwrap();
    let deserialized: ModelParams = serde_json::from_str(&json).unwrap();
    assert_eq!(params, deserialized);
}
