use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct HfClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HfModel {
    pub id: String,
    pub model_type: String,
    pub tags: Vec<String>,
    pub downloads: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HfSearchResult {
    pub models: Vec<HfModel>,
}

impl HfClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<HfModel>> {
        let url = reqwest::Url::parse_with_params(
            &format!("{}/api/models", self.base_url),
            &[("search", query), ("limit", "10")],
        )?;
        let resp: HfSearchResult = self.client.get(url).send().await?.json().await?;
        Ok(resp.models)
    }

    pub async fn download_model(
        &self,
        model_id: &str,
        dest: &std::path::Path,
    ) -> anyhow::Result<()> {
        let url = format!("{}/{}/resolve/main", self.base_url, model_id);
        let resp = self.client.get(&url).send().await?;
        let bytes = resp.bytes().await?;
        tokio::fs::write(dest, bytes).await?;
        Ok(())
    }
}
