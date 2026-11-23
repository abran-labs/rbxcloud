//! Typed analytics metric, dimension, and benchmark queries.
use super::client::{AsyncClient, Error, Operation, Request, Server};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum Granularity {
    OneMinute,
    HalfHour,
    OneHour,
    OneDay,
    OneWeek,
    OneMonth,
    None,
}
impl std::fmt::Display for Granularity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum FilterOperation {
    In,
    NotIn,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Match,
}
impl std::fmt::Display for FilterOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    pub dimension: String,
    pub operation: FilterOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsInput {
    pub metric: String,
    pub granularity: Granularity,
    pub start_time: String,
    pub end_time: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Vec<QueryFilter>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionsInput {
    pub metric: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<Vec<String>>,
    pub start_time: String,
    pub end_time: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Vec<QueryFilter>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub granularity: Option<Granularity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, Serialize, Deserialize)]
pub enum BenchmarkType {
    #[serde(rename = "BENCHMARK_TYPE_INVALID")]
    Invalid,
    #[serde(rename = "BENCHMARK_TYPE_GENRE")]
    Genre,
    #[serde(rename = "BENCHMARK_TYPE_SIMILARITY")]
    Similarity,
    #[serde(rename = "BENCHMARK_TYPE_TOP_EXPERIENCES")]
    TopExperiences,
}
impl std::fmt::Display for BenchmarkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => "BENCHMARK_TYPE_INVALID",
            Self::Genre => "BENCHMARK_TYPE_GENRE",
            Self::Similarity => "BENCHMARK_TYPE_SIMILARITY",
            Self::TopExperiences => "BENCHMARK_TYPE_TOP_EXPERIENCES",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakdownFilter {
    pub dimension: Option<String>,
    pub value: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkInput {
    pub metric: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Vec<BreakdownFilter>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benchmark_type: Option<BenchmarkType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percentiles: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benchmark_variant_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AnalyticsQueryClient {
    transport: AsyncClient,
}
impl AnalyticsQueryClient {
    pub fn new(transport: AsyncClient) -> Self {
        Self { transport }
    }
    async fn query(
        &self,
        template: &'static str,
        universe_id: u64,
        body: &impl Serialize,
    ) -> Result<serde_json::Value, Error> {
        let request = Request::new(Operation::post(Server::Apis, template))
            .path_param("universeId", universe_id)
            .json(body)?;
        self.transport.execute(request).await
    }
    pub async fn metrics(
        &self,
        universe_id: u64,
        input: MetricsInput,
    ) -> Result<serde_json::Value, Error> {
        self.query(
            "/analytics-query-api/v1/universes/{universeId}/metrics",
            universe_id,
            &input,
        )
        .await
    }
    pub async fn dimensions(
        &self,
        universe_id: u64,
        input: DimensionsInput,
    ) -> Result<serde_json::Value, Error> {
        self.query(
            "/analytics-query-api/v1/universes/{universeId}/dimension-values",
            universe_id,
            &input,
        )
        .await
    }
    pub async fn benchmarks(
        &self,
        universe_id: u64,
        input: BenchmarkInput,
    ) -> Result<serde_json::Value, Error> {
        self.query(
            "/analytics-benchmark/v1/universes/{universeId}/benchmarks",
            universe_id,
            &input,
        )
        .await
    }
}
