//! Typed avatar bundle creation context.

use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum BundleType {
    Unknown,
    Body,
    DynamicHead,
    Shoes,
    AvatarAnimations,
}
impl std::fmt::Display for BundleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyScaleInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_type: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proportion: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyColorSetInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_color3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torso_color3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_arm_color3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_arm_color3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_leg_color3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_leg_color3: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBundleContextInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_quantities: Option<BTreeMap<String, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_type: Option<BundleType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_scale: Option<BodyScaleInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_color_set: Option<BodyColorSetInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_emissive: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct AvatarBundleContextClient {
    transport: AsyncClient,
}
impl AvatarBundleContextClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    pub async fn create_context(&self, input: CreateBundleContextInput) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::ItemConfiguration,
            "/v1/bundles/create-context",
        ))
        .json(&input)?;
        self.transport.execute(request).await
    }
}
