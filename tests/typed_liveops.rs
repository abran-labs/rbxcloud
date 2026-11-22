mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    generated::{liveops::*, liveops_models::ExperimentState},
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> LiveOpsCloudClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    LiveOpsCloudClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("liveops-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn liveops_queries_preserve_names_arrays_and_paths() {
    let server = TestServer::start((0..7).map(|_| Reply::json(200, "{}")).collect());
    let client = client(&server).await;
    client
        .get_config_repository_values(GetConfigRepositoryValuesParams {
            universe_id: "1".into(),
            repository: "game".into(),
        })
        .await
        .unwrap();
    client
        .list_revisions(ListRevisionsParams {
            universe_id: "1".into(),
            repository: "game".into(),
            start_time: None,
            end_time: None,
            max_page_size: Some(10),
            skip: Some(2),
            search_key: Some("x".into()),
            sort_order: None,
            sort_key: None,
        })
        .await
        .unwrap();
    client
        .list_experiments(ListExperimentsParams {
            universe_id: 1,
            max_page_size: None,
            skip: None,
            search_key: None,
            state_filters: Some(vec![
                ExperimentState::ExperimentStateDraft,
                ExperimentState::ExperimentStateRunning,
            ]),
            sort_order: None,
            sort_key: None,
            product_type_filter: None,
        })
        .await
        .unwrap();
    client
        .start_experiment(StartExperimentParams {
            universe_id: 1,
            experiment_id: "e".into(),
        })
        .await
        .unwrap();
    client
        .list_game_servers(ListGameServersParams {
            universe_id: 1,
            place_id: 2,
            version_number: "3".into(),
            max_page_size: Some(5),
            page_token: None,
            order_by: None,
            filter: None,
        })
        .await
        .unwrap();
    client
        .get_game_event(GetGameEventParams {
            event_id: "e".into(),
            fields: Some("id,name".into()),
        })
        .await
        .unwrap();
    client
        .get_game_events(GetGameEventsParams {
            universe_id: 1,
            page_size: Some(4),
            page_token: None,
            reverse: Some(true),
            starts_before: None,
            starts_after: None,
            ends_before: None,
            ends_after: None,
            visibility: None,
            fields: None,
        })
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(
        requests[0].target,
        "/creator-configs-public-api/v1/configs/universes/1/repositories/game"
    );
    assert_eq!(requests[1].target, "/creator-configs-public-api/v1/configs/universes/1/repositories/game/revisions?MaxPageSize=10&Skip=2&SearchKey=x");
    assert_eq!(requests[2].target, "/creator-configs-public-api/v1/experimentation/universes/1/experiments?stateFilters=EXPERIMENT_STATE_DRAFT&stateFilters=EXPERIMENT_STATE_RUNNING");
    assert_eq!(
        requests[3].target,
        "/creator-configs-public-api/v1/experimentation/universes/1/experiments/e:start"
    );
    assert_eq!(
        requests[4].target,
        "/server-management/v1/universes/1/places/2/versions/3/game-servers?MaxPageSize=5"
    );
    assert_eq!(
        requests[5].target,
        "/virtual-events/v3/game-events/e?fields=id%2Cname"
    );
    assert_eq!(
        requests[6].target,
        "/virtual-events/v3/universes/1/game-events?pageSize=4&reverse=true"
    );
}

#[test]
fn liveops_cli_exposes_all_seven_explicit_groups() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
        .args(["liveops-tools", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for group in [
        "configs",
        "developer-products",
        "experiments",
        "game-events",
        "game-passes",
        "matchmaking",
        "server-management",
    ] {
        assert!(help.contains(group));
    }
}
