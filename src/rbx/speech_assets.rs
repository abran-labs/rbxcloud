//! Typed generative speech asset requests.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::Serialize;
use serde_json::Value;
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechStyleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSpeechInput {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_style: Option<SpeechStyleInput>,
}
#[derive(Debug, Clone)]
pub struct SpeechAssetClient {
    transport: AsyncClient,
}
impl SpeechAssetClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn generate(
        &self,
        universe_id: u64,
        input: GenerateSpeechInput,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/cloud/v2/universes/{universe_id}:generateSpeechAsset",
        ))
        .path_param("universe_id", universe_id)
        .json(&input)?;
        self.transport.execute(request).await
    }
}
