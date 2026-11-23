//! Typed developer product, game pass, and badge multipart mutations.

use super::client::{AsyncClient, Attachment, Error, Operation, Request, Server};
use serde_json::Value;
use std::{fmt::Display, path::Path};

fn text(request: Request, name: &'static str, value: Option<impl Display>) -> Request {
    match value {
        Some(value) => request.multipart_text(name, value.to_string()),
        None => request,
    }
}
fn file(
    request: Request,
    name: &'static str,
    path: Option<&Path>,
    content_type: &str,
) -> Result<Request, Error> {
    match path {
        Some(path) => Ok(request.multipart_file(name, Attachment::from_path(path, content_type)?)),
        None => Ok(request),
    }
}

#[derive(Debug, Clone)]
pub struct CreateDeveloperProductInput {
    pub name: String,
    pub description: Option<String>,
    pub is_for_sale: Option<bool>,
    pub price: Option<i64>,
    pub is_regional_pricing_enabled: Option<bool>,
    pub is_managed_pricing_enabled: Option<bool>,
}
#[derive(Debug, Clone, Default)]
pub struct UpdateDeveloperProductInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_for_sale: Option<bool>,
    pub price: Option<i64>,
    pub is_regional_pricing_enabled: Option<bool>,
    pub store_page_enabled: Option<bool>,
    pub is_managed_pricing_enabled: Option<bool>,
}
#[derive(Debug, Clone)]
pub struct CreateGamePassInput {
    pub name: String,
    pub description: Option<String>,
    pub is_for_sale: Option<bool>,
    pub price: Option<i64>,
    pub is_regional_pricing_enabled: Option<bool>,
    pub is_managed_pricing_enabled: Option<bool>,
    pub is_trial_enabled: Option<bool>,
}
#[derive(Debug, Clone, Default)]
pub struct UpdateGamePassInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_for_sale: Option<bool>,
    pub price: Option<i64>,
    pub is_regional_pricing_enabled: Option<bool>,
    pub is_managed_pricing_enabled: Option<bool>,
    pub is_trial_enabled: Option<bool>,
}
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum BadgePaymentSource {
    User = 1,
    Group = 2,
}
impl Display for BadgePaymentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}
#[derive(Debug, Clone, Default)]
pub struct CreateBadgeInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub payment_source_type: Option<BadgePaymentSource>,
    pub expected_cost: Option<i64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct EconomyUploadClient {
    transport: AsyncClient,
}
impl EconomyUploadClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }

    pub async fn create_developer_product(
        &self,
        universe_id: u64,
        input: CreateDeveloperProductInput,
        image: Option<&Path>,
        content_type: &str,
    ) -> Result<Value, Error> {
        let mut request = Request::new(Operation::post(
            Server::Apis,
            "/developer-products/v2/universes/{universeId}/developer-products",
        ))
        .path_param("universeId", universe_id)
        .multipart_text("name", input.name);
        request = text(request, "description", input.description);
        request = text(request, "isForSale", input.is_for_sale);
        request = text(request, "price", input.price);
        request = file(request, "imageFile", image, content_type)?;
        request = text(
            request,
            "isRegionalPricingEnabled",
            input.is_regional_pricing_enabled,
        );
        request = text(
            request,
            "isManagedPricingEnabled",
            input.is_managed_pricing_enabled,
        );
        self.transport.execute(request).await
    }

    pub async fn update_developer_product(
        &self,
        universe_id: u64,
        product_id: u64,
        input: UpdateDeveloperProductInput,
        image: Option<&Path>,
        content_type: &str,
    ) -> Result<Value, Error> {
        let mut request = Request::new(Operation::patch(
            Server::Apis,
            "/developer-products/v2/universes/{universeId}/developer-products/{productId}",
        ))
        .path_param("universeId", universe_id)
        .path_param("productId", product_id);
        request = text(request, "name", input.name);
        request = text(request, "description", input.description);
        request = text(request, "isForSale", input.is_for_sale);
        request = text(request, "price", input.price);
        request = file(request, "imageFile", image, content_type)?;
        request = text(
            request,
            "isRegionalPricingEnabled",
            input.is_regional_pricing_enabled,
        );
        request = text(request, "storePageEnabled", input.store_page_enabled);
        request = text(
            request,
            "isManagedPricingEnabled",
            input.is_managed_pricing_enabled,
        );
        self.transport.execute(request).await
    }

    pub async fn create_game_pass(
        &self,
        universe_id: u64,
        input: CreateGamePassInput,
        image: Option<&Path>,
        content_type: &str,
    ) -> Result<Value, Error> {
        let mut request = Request::new(Operation::post(
            Server::Apis,
            "/game-passes/v1/universes/{universeId}/game-passes",
        ))
        .path_param("universeId", universe_id)
        .multipart_text("name", input.name);
        request = text(request, "description", input.description);
        request = file(request, "imageFile", image, content_type)?;
        request = text(request, "isForSale", input.is_for_sale);
        request = text(request, "price", input.price);
        request = text(
            request,
            "isRegionalPricingEnabled",
            input.is_regional_pricing_enabled,
        );
        request = text(
            request,
            "isManagedPricingEnabled",
            input.is_managed_pricing_enabled,
        );
        request = text(request, "isTrialEnabled", input.is_trial_enabled);
        self.transport.execute(request).await
    }

    pub async fn update_game_pass(
        &self,
        universe_id: u64,
        game_pass_id: u64,
        input: UpdateGamePassInput,
        image: Option<&Path>,
        file_alias: Option<&Path>,
        content_type: &str,
    ) -> Result<Value, Error> {
        let mut request = Request::new(Operation::patch(
            Server::Apis,
            "/game-passes/v1/universes/{universeId}/game-passes/{gamePassId}",
        ))
        .path_param("universeId", universe_id)
        .path_param("gamePassId", game_pass_id);
        request = text(request, "name", input.name);
        request = text(request, "description", input.description);
        request = file(request, "file", file_alias, content_type)?;
        request = file(request, "imageFile", image, content_type)?;
        request = text(request, "isForSale", input.is_for_sale);
        request = text(request, "price", input.price);
        request = text(
            request,
            "isRegionalPricingEnabled",
            input.is_regional_pricing_enabled,
        );
        request = text(
            request,
            "isManagedPricingEnabled",
            input.is_managed_pricing_enabled,
        );
        request = text(request, "isTrialEnabled", input.is_trial_enabled);
        self.transport.execute(request).await
    }

    pub async fn create_badge(
        &self,
        universe_id: u64,
        input: CreateBadgeInput,
        image: Option<&Path>,
        content_type: &str,
    ) -> Result<Value, Error> {
        let mut request = Request::new(Operation::post(
            Server::Apis,
            "/legacy-badges/v1/universes/{universeId}/badges",
        ))
        .path_param("universeId", universe_id);
        request = text(request, "name", input.name);
        request = text(request, "description", input.description);
        request = text(request, "paymentSourceType", input.payment_source_type);
        request = file(request, "files", image, content_type)?;
        request = text(request, "expectedCost", input.expected_cost);
        request = text(request, "isActive", input.is_active);
        self.transport.execute(request).await
    }

    pub async fn update_badge_icon(
        &self,
        badge_id: u64,
        image: &Path,
        content_type: &str,
    ) -> Result<Value, Error> {
        let request = Request::new(Operation::post(
            Server::Apis,
            "/legacy-publish/v1/badges/{badgeId}/icon",
        ))
        .path_param("badgeId", badge_id)
        .multipart_file("Files", Attachment::from_path(image, content_type)?);
        self.transport.execute(request).await
    }
}
