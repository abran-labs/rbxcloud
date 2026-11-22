mod support;
use rbxcloud::rbx::{
    client::{AsyncClient, ClientPolicy},
    credentials::{Credentials, SecretString},
    generated::{creator_models::*, creators::*},
};
use std::time::Duration;
use support::{Reply, TestServer};

async fn client(server: &TestServer) -> CreatorCloudClient {
    let policy = ClientPolicy::loopback(server.addr())
        .unwrap()
        .timeout(Duration::from_secs(2))
        .retry_delay(Duration::from_millis(1));
    CreatorCloudClient::new(
        AsyncClient::new(
            Credentials::api_key(SecretString::new("creator-contract-test-key")),
            policy,
        )
        .await
        .unwrap(),
    )
}

#[tokio::test]
async fn creator_methods_use_fixed_paths_and_typed_bodies() {
    let server = TestServer::start((0..14).map(|_| Reply::json(200, "{}")).collect());
    let client = client(&server).await;
    client
        .list_group_forum_categories(ListGroupForumCategoriesParams {
            group_id: "7".into(),
            max_page_size: Some(5),
            page_token: None,
            filter: None,
        })
        .await
        .unwrap();
    client
        .list_group_forum_posts(ListGroupForumPostsParams {
            group_id: "7".into(),
            forum_category_id: "c".into(),
            max_page_size: None,
            page_token: None,
            filter: None,
            view: Some(ListGroupForumPostsView::Full),
        })
        .await
        .unwrap();
    client
        .list_group_forum_comments(ListGroupForumCommentsParams {
            group_id: "7".into(),
            forum_category_id: "c".into(),
            post_id: "p".into(),
            max_page_size: None,
            page_token: None,
            filter: None,
        })
        .await
        .unwrap();
    client
        .list_group_join_requests(ListGroupJoinRequestsParams {
            group_id: "7".into(),
            max_page_size: None,
            page_token: None,
            filter: None,
        })
        .await
        .unwrap();
    client
        .accept_group_join_request(AcceptGroupJoinRequestParams {
            group_id: "7".into(),
            join_request_id: "j".into(),
            body: AcceptGroupJoinRequestRequest {},
        })
        .await
        .unwrap();
    client
        .decline_group_join_request(DeclineGroupJoinRequestParams {
            group_id: "7".into(),
            join_request_id: "j".into(),
            body: DeclineGroupJoinRequestRequest {},
        })
        .await
        .unwrap();
    client
        .update_group_membership(UpdateGroupMembershipParams {
            group_id: "7".into(),
            membership_id: "m".into(),
            body: UpdateGroupMembershipRequest {
                role: Some("groups/7/roles/3".into()),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .assign_role_group_membership(AssignRoleGroupMembershipParams {
            group_id: "7".into(),
            membership_id: "m".into(),
            body: AssignRoleGroupMembershipRequest {
                role: "groups/7/roles/3".into(),
            },
        })
        .await
        .unwrap();
    client
        .unassign_role_group_membership(UnassignRoleGroupMembershipParams {
            group_id: "7".into(),
            membership_id: "m".into(),
            body: UnassignRoleGroupMembershipRequest {
                role: "groups/7/roles/3".into(),
            },
        })
        .await
        .unwrap();
    client
        .get_group_role(GetGroupRoleParams {
            group_id: "7".into(),
            role_id: "3".into(),
        })
        .await
        .unwrap();
    client
        .get(GetParams {
            universe_id: "u".into(),
            subscription_product_id: "p".into(),
            subscription_id: "s".into(),
            view: None,
        })
        .await
        .unwrap();
    client
        .get_thumbnail_operation(GetThumbnailOperationParams {
            user_id: "5".into(),
            operation_id: "o".into(),
        })
        .await
        .unwrap();
    client
        .list_permissions(ListPermissionsParams { ids: vec![1, 2, 3] })
        .await
        .unwrap();
    client
        .get_policies(GetPoliciesParams {
            body: GetPoliciesRequest {
                group_ids: Some(vec![7, 8]),
            },
        })
        .await
        .unwrap();
    let requests = server.requests();
    assert_eq!(requests.len(), 14);
    assert_eq!(
        requests[0].target,
        "/cloud/v2/groups/7/forum-categories?maxPageSize=5"
    );
    assert!(requests[1]
        .target
        .ends_with("/forum-categories/c/posts?view=FULL"));
    assert!(requests[4].target.ends_with("/join-requests/j:accept"));
    assert_eq!(requests[6].body_text(), r#"{"role":"groups/7/roles/3"}"#);
    assert_eq!(
        requests[10].target,
        "/cloud/v2/universes/u/subscription-products/p/subscriptions/s"
    );
    assert_eq!(requests[11].target, "/cloud/v2/users/5/operations/o");
    assert_eq!(
        requests[12].target,
        "/legacy-develop/v1/universes/multiget/permissions?ids=1&ids=2&ids=3"
    );
    assert_eq!(requests[13].body_text(), r#"{"groupIds":[7,8]}"#);
}

#[test]
fn creator_cli_has_explicit_nested_commands_without_generic_request_flags() {
    for args in [
        vec!["creator-tools", "group", "get-group-role", "--help"],
        vec!["creator-tools", "subscription", "get", "--help"],
        vec!["creator-tools", "universe", "list-permissions", "--help"],
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rbxcloud"))
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        for flag in ["--url ", "--method ", "--body ", "--api-key "] {
            assert!(!help.contains(flag));
        }
    }
}
