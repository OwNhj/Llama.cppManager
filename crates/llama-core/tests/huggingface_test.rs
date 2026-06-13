use llama_core::huggingface::{HfClient, HfModel};

#[tokio::test]
async fn test_hf_client_creation() {
    let client = HfClient::new("https://huggingface.co".into());
    assert_eq!(client.base_url(), "https://huggingface.co");
}

#[test]
fn test_hf_model_struct() {
    let model = HfModel {
        id: "meta-llama/Llama-3.1-8B-Instruct".into(),
        model_type: "llama".into(),
        tags: vec!["text-generation".into()],
        downloads: 1200000,
    };
    assert_eq!(model.id, "meta-llama/Llama-3.1-8B-Instruct");
    assert_eq!(model.model_type, "llama");
    assert_eq!(model.tags.len(), 1);
    assert_eq!(model.tags[0], "text-generation");
    assert_eq!(model.downloads, 1200000);
}

#[test]
fn test_hf_model_with_multiple_tags() {
    let model = HfModel {
        id: "bert-base-uncased".into(),
        model_type: "bert".into(),
        tags: vec![
            "text-classification".into(),
            "fill-mask".into(),
            "pytorch".into(),
        ],
        downloads: 500000,
    };
    assert_eq!(model.tags.len(), 3);
    assert!(model.tags.contains(&"text-classification".to_string()));
    assert!(model.tags.contains(&"pytorch".to_string()));
}

#[tokio::test]
async fn test_hf_client_custom_url() {
    let custom_url = "https://my-huggingface-instance.example.com";
    let client = HfClient::new(custom_url.into());
    assert_eq!(client.base_url(), custom_url);
}
