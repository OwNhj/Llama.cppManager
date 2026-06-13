use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct HfClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HfModel {
    pub id: String,
    #[serde(default)]
    pub model_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub downloads: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HfSearchResponse {
    #[serde(default)]
    pub items: Vec<HfModel>,
}

impl HfClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// 搜索模型（同步版本）
    pub fn search(&self, query: &str) -> anyhow::Result<Vec<HfModel>> {
        let url = reqwest::Url::parse_with_params(
            &format!("{}/api/models", self.base_url),
            &[("search", query), ("limit", "20"), ("sort", "downloads")],
        )?;
        
        let resp = self.client.get(url).send()?;
        let response: HfSearchResponse = resp.json()?;
        
        Ok(response.items)
    }

    /// 获取模型信息
    pub fn get_model(&self, model_id: &str) -> anyhow::Result<HfModel> {
        let url = format!("{}/api/models/{}", self.base_url, model_id);
        let resp = self.client.get(&url).send()?;
        let model: HfModel = resp.json()?;
        Ok(model)
    }

    /// 下载模型文件
    pub fn download_model(
        &self,
        model_id: &str,
        filename: &str,
        dest: &std::path::Path,
        progress_callback: Option<Box<dyn Fn(u64, u64) + Send>>,
    ) -> anyhow::Result<()> {
        let url = format!("{}/{}/resolve/main/{}", self.base_url, model_id, filename);
        
        let mut resp = self.client.get(&url).send()?;
        
        let total_size = resp.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        
        // 创建目标文件
        let mut file = std::fs::File::create(dest)?;
        
        // 分块下载
        use std::io::{Read, Write};
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = resp.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            file.write_all(&buffer[..bytes_read])?;
            downloaded += bytes_read as u64;
            
            if let Some(ref callback) = progress_callback {
                callback(downloaded, total_size);
            }
        }
        
        Ok(())
    }
}
