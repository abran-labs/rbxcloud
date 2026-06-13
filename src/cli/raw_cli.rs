use clap::Args;
use reqwest::Method;

#[derive(Debug, Args)]
pub struct Raw {
    /// HTTP method
    #[clap(short, long, value_parser, default_value = "GET")]
    method: String,

    /// Open Cloud path (e.g. /cloud/v2/groups/123) or full URL
    #[clap(short = 'u', long, value_parser)]
    url: String,

    /// JSON request body
    #[clap(short, long, value_parser)]
    body: Option<String>,

    /// Query parameter as key=value. Can be repeated.
    #[clap(short, long, value_parser)]
    query: Vec<String>,

    /// Pretty-print JSON response
    #[clap(short, long, value_parser, default_value_t = false)]
    pretty: bool,

    /// Roblox Open Cloud API Key
    #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
    api_key: String,
}

impl Raw {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let method = self.method.parse::<Method>()?;
        let url = if self.url.starts_with("https://") || self.url.starts_with("http://") {
            self.url
        } else if self.url.starts_with('/') {
            format!("https://apis.roblox.com{}", self.url)
        } else {
            format!("https://apis.roblox.com/{}", self.url)
        };

        let client = reqwest::Client::new();
        let mut request = client
            .request(method, url)
            .header("x-api-key", self.api_key)
            .header("Content-Type", "application/json");

        let mut query: Vec<(String, String)> = Vec::new();
        for item in self.query {
            let Some((key, value)) = item.split_once('=') else {
                anyhow::bail!("query parameters must be key=value: {item}");
            };
            query.push((key.to_string(), value.to_string()));
        }
        if !query.is_empty() {
            request = request.query(&query);
        }

        if let Some(body) = self.body {
            let json_body: serde_json::Value = serde_json::from_str(&body)?;
            request = request.json(&json_body);
        }

        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            anyhow::bail!("HTTP {}: {}", status.as_u16(), text);
        }

        if self.pretty {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                return Ok(Some(serde_json::to_string_pretty(&json)?));
            }
        }

        Ok(Some(text))
    }
}
