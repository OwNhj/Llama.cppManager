use llama_core::quantize::{LayerConfig, QuantConfig, QuantType};

#[test]
fn test_quant_type_count() {
    let types = QuantType::all();
    assert_eq!(types.len(), 23);
}

#[test]
fn test_quant_type_categories() {
    let originals = QuantType::originals();
    assert_eq!(originals.len(), 3);

    let quantized = QuantType::quantized();
    assert!(quantized.len() > 15);
}

#[test]
fn test_quant_meta() {
    let meta = QuantType::Q4_K_M.meta();
    assert_eq!(meta.name, "Q4_K_M");
    assert_eq!(meta.category, "4-bit");
    assert!(meta.quality >= 3.0);
    assert!(meta.quality <= 5.0);
}

#[test]
fn test_quant_config_default() {
    let config = QuantConfig::default();
    assert_eq!(config.global_quant, QuantType::Q5_K_M);
    assert_eq!(config.layers.len(), 0);
}

#[test]
fn test_layer_config() {
    let config = LayerConfig {
        tensor: "blk.0.attn_q.weight".into(),
        quant_type: QuantType::Q4_K_M,
    };
    assert_eq!(config.tensor, "blk.0.attn_q.weight");
    assert_eq!(config.quant_type, QuantType::Q4_K_M);
}

#[test]
fn test_all_variants_meta_name_matches_variant() {
    for quant in QuantType::all() {
        let meta = quant.meta();
        assert!(
            !meta.name.is_empty(),
            "meta().name should be non-empty for {:?}",
            quant
        );
        assert_eq!(
            meta.name,
            format!("{:?}", quant),
            "meta().name should match Debug name for {:?}",
            quant
        );
    }
}
