use clap::{Args, Subcommand};
use rbxcloud::rbx::{
    analytics_queries::*,
    client::{AsyncClient, ClientPolicy},
    credentials::Credentials,
};

#[derive(Debug, Args)]
pub struct AnalyticsQuery {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    QueryMetrics(MetricArgs),
    QueryDimensionValues(DimensionArgs),
    QueryBenchmarks(BenchmarkArgs),
}

#[derive(Debug, Args)]
struct FilterArgs {
    #[arg(long = "filter-dimension")]
    filter_dimensions: Vec<String>,
    #[arg(long = "filter-operation")]
    operations: Vec<FilterOperation>,
    #[arg(long = "filter-values", value_delimiter = ',')]
    values: Vec<String>,
}
impl FilterArgs {
    fn build(&self) -> anyhow::Result<Option<Vec<QueryFilter>>> {
        if self.filter_dimensions.is_empty() && self.operations.is_empty() && self.values.is_empty() {
            return Ok(None);
        }
        if self.filter_dimensions.len() != self.operations.len()
            || self.filter_dimensions.len() != self.values.len()
        {
            anyhow::bail!("repeat --filter-dimension, --filter-operation, and --filter-values equally for each filter");
        }
        Ok(Some(
            self.filter_dimensions
                .iter()
                .zip(&self.operations)
                .zip(&self.values)
                .map(|((dimension, operation), values)| QueryFilter {
                    dimension: dimension.clone(),
                    operation: *operation,
                    values: Some(values.split(',').map(str::to_string).collect()),
                })
                .collect(),
        ))
    }
}
#[derive(Debug, Args)]
struct MetricArgs {
    universe_id: u64,
    #[arg(long)]
    metric: String,
    #[arg(long)]
    granularity: Granularity,
    #[arg(long)]
    start_time: String,
    #[arg(long)]
    end_time: String,
    #[arg(long)]
    breakdown: Option<Vec<String>>,
    #[arg(long)]
    limit: Option<u32>,
    #[command(flatten)]
    filters: FilterArgs,
}
#[derive(Debug, Args)]
struct DimensionArgs {
    universe_id: u64,
    #[arg(long)]
    metric: String,
    #[arg(long)]
    dimensions: Option<Vec<String>>,
    #[arg(long)]
    start_time: String,
    #[arg(long)]
    end_time: String,
    #[arg(long)]
    granularity: Option<Granularity>,
    #[arg(long)]
    limit: Option<u32>,
    #[command(flatten)]
    filters: FilterArgs,
}
#[derive(Debug, Args)]
struct BenchmarkArgs {
    universe_id: u64,
    #[arg(long)]
    metric: Option<String>,
    #[arg(long)]
    start_time: Option<String>,
    #[arg(long)]
    end_time: Option<String>,
    #[arg(long)]
    benchmark_type: Option<BenchmarkType>,
    #[arg(long)]
    percentiles: Option<Vec<u8>>,
    #[arg(long)]
    benchmark_variant_id: Option<String>,
    #[arg(long = "breakdown-filter", value_parser = parse_breakdown)]
    filters: Vec<BreakdownFilter>,
}
fn parse_breakdown(value: &str) -> Result<BreakdownFilter, String> {
    let (dimension, value) = value.split_once('=').ok_or("expected DIMENSION=VALUE")?;
    Ok(BreakdownFilter {
        dimension: Some(dimension.into()),
        value: Some(value.into()),
    })
}
impl AnalyticsQuery {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        let client = AnalyticsQueryClient::new(
            AsyncClient::new(Credentials::from_process()?, ClientPolicy::default()).await?,
        );
        let value = match self.command {
            Command::QueryMetrics(args) => {
                client
                    .metrics(
                        args.universe_id,
                        MetricsInput {
                            metric: args.metric,
                            granularity: args.granularity,
                            start_time: args.start_time,
                            end_time: args.end_time,
                            breakdown: args.breakdown,
                            filter: args.filters.build()?,
                            limit: args.limit,
                        },
                    )
                    .await?
            }
            Command::QueryDimensionValues(args) => {
                client
                    .dimensions(
                        args.universe_id,
                        DimensionsInput {
                            metric: args.metric,
                            dimensions: args.dimensions,
                            start_time: args.start_time,
                            end_time: args.end_time,
                            filter: args.filters.build()?,
                            granularity: args.granularity,
                            limit: args.limit,
                        },
                    )
                    .await?
            }
            Command::QueryBenchmarks(args) => {
                client
                    .benchmarks(
                        args.universe_id,
                        BenchmarkInput {
                            metric: args.metric,
                            start_time: args.start_time,
                            end_time: args.end_time,
                            filter: (!args.filters.is_empty()).then_some(args.filters),
                            benchmark_type: args.benchmark_type,
                            percentiles: args.percentiles,
                            benchmark_variant_id: args.benchmark_variant_id,
                        },
                    )
                    .await?
            }
        };
        Ok(Some(serde_json::to_string_pretty(&value)?))
    }
}
