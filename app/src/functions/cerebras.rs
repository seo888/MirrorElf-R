use reqwest::{header, Client};
use serde_json::{json, Value};

pub struct CerebrasClient {
    api_key: String,
    client: Client,
}

impl CerebrasClient {
    pub fn new(api_key: &str) -> Self {
        CerebrasClient {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn chat_completion(
        &self,
        model: &str,
        message: &str,
    ) -> Result<String, reqwest::Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );

        let request_body = json!({
            "model": model,
            "stream": false,
            "messages": vec![json!({"content": message, "role": "user"})],
            "temperature": 0,
            "max_tokens": -1,
            "seed": 0,
            "top_p": 1
        });

        let res = self
            .client
            .post("https://api.cerebras.ai/v1/chat/completions")
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        let response_json: Value = res.json().await?;
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default();
        Ok(content.to_string())
    }
}