use serde::{Deserialize, Serialize};

use crate::rbx::{error::Error, types::GroupId, util::QueryString};

use super::http_err::handle_http_err;

impl std::fmt::Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct GetGroupParams {
    pub api_key: String,
    pub group_id: GroupId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetGroupResponse {
    pub path: String,
    pub create_time: String,
    pub update_time: String,
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub owner: Option<String>,
    pub member_count: u64,
    pub public_entry_allowed: bool,
    pub locked: bool,
    pub verified: bool,
}

pub struct GetGroupShoutParams {
    pub api_key: String,
    pub group_id: GroupId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetGroupShoutResponse {
    pub path: String,
    pub create_time: String,
    pub update_time: String,
    pub content: String,
    pub poster: Option<String>,
}

pub struct ListGroupRolesParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub max_page_size: Option<u32>,
    pub page_token: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupRolePermission {
    pub view_wall_posts: bool,
    pub create_wall_posts: bool,
    pub delete_wall_posts: bool,
    pub view_group_shout: bool,
    pub create_group_shout: bool,
    pub change_rank: bool,
    pub accept_requests: bool,
    pub exile_members: bool,
    pub manage_relationships: bool,
    pub view_audit_log: bool,
    pub spend_group_funds: bool,
    pub advertise_group: bool,
    pub create_avatar_items: bool,
    pub manage_avatar_items: bool,
    pub manage_group_universes: bool,
    pub view_universe_analytics: bool,
    pub create_api_keys: bool,
    pub manage_api_keys: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupRole {
    pub path: String,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub rank: u32,
    pub member_count: Option<u64>,
    pub permissions: Option<GroupRolePermission>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListGroupRolesResponse {
    pub group_roles: Vec<GroupRole>,
    pub next_page_token: Option<String>,
}

pub struct ListGroupMembershipsParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub max_page_size: Option<u32>,
    pub page_token: Option<String>,
    pub filter: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembership {
    pub path: String,
    pub create_time: String,
    pub update_time: String,
    pub user: String,
    pub role: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListGroupMembershipsResponse {
    pub group_memberships: Vec<GroupMembership>,
    pub next_page_token: Option<String>,
}

pub async fn get_group(params: &GetGroupParams) -> Result<GetGroupResponse, Error> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}",
        groupId = &params.group_id,
    );

    let res = client
        .get(url)
        .header("x-api-key", &params.api_key)
        .send()
        .await?;

    let status = res.status();

    if !status.is_success() {
        let code = status.as_u16();
        return handle_http_err(code);
    }

    let body = res.json::<GetGroupResponse>().await?;
    Ok(body)
}

pub async fn get_group_shout(params: &GetGroupShoutParams) -> Result<GetGroupShoutResponse, Error> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/shout",
        groupId = &params.group_id,
    );

    let res = client
        .get(url)
        .header("x-api-key", &params.api_key)
        .send()
        .await?;

    let status = res.status();

    if !status.is_success() {
        let code = status.as_u16();
        return handle_http_err(code);
    }

    let body = res.json::<GetGroupShoutResponse>().await?;
    Ok(body)
}

pub async fn list_group_roles(
    params: &ListGroupRolesParams,
) -> Result<ListGroupRolesResponse, Error> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/roles",
        groupId = &params.group_id,
    );

    let mut query: QueryString = vec![];
    if let Some(max_page_size) = &params.max_page_size {
        query.push(("maxPageSize", max_page_size.to_string()))
    }
    if let Some(page_token) = &params.page_token {
        query.push(("pageToken", page_token.clone()));
    }

    let res = client
        .get(url)
        .header("x-api-key", &params.api_key)
        .query(&query)
        .send()
        .await?;

    let status = res.status();

    if !status.is_success() {
        let code = status.as_u16();
        return handle_http_err(code);
    }

    let body = res.json::<ListGroupRolesResponse>().await?;
    Ok(body)
}

pub async fn list_group_memberships(
    params: &ListGroupMembershipsParams,
) -> Result<ListGroupMembershipsResponse, Error> {
    let client = reqwest::Client::new();

    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/memberships",
        groupId = &params.group_id,
    );

    let mut query: QueryString = vec![];
    if let Some(max_page_size) = &params.max_page_size {
        query.push(("maxPageSize", max_page_size.to_string()))
    }
    if let Some(page_token) = &params.page_token {
        query.push(("pageToken", page_token.clone()));
    }
    if let Some(filter) = &params.filter {
        query.push(("filter", filter.clone()));
    }

    let res = client
        .get(url)
        .header("x-api-key", &params.api_key)
        .query(&query)
        .send()
        .await?;

    let status = res.status();

    if !status.is_success() {
        let code = status.as_u16();
        return handle_http_err(code);
    }

    let body = res.json::<ListGroupMembershipsResponse>().await?;
    Ok(body)
}

pub struct GetGroupRoleParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub role_id: String,
}

pub struct ListGroupForumCategoriesParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub max_page_size: Option<u32>,
    pub page_token: Option<String>,
}

pub struct ListGroupForumPostsParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub forum_category_id: String,
    pub max_page_size: Option<u32>,
    pub page_token: Option<String>,
}

pub struct ListGroupForumCommentsParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub forum_category_id: String,
    pub post_id: String,
    pub max_page_size: Option<u32>,
    pub page_token: Option<String>,
}

pub struct ListGroupJoinRequestsParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub max_page_size: Option<u32>,
    pub page_token: Option<String>,
    pub filter: Option<String>,
}

pub struct MutateGroupJoinRequestParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub join_request_id: String,
}

pub struct MutateGroupMembershipRoleParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub membership_id: String,
    pub role: String,
}

pub struct UpdateGroupMembershipParams {
    pub api_key: String,
    pub group_id: GroupId,
    pub membership_id: String,
    pub body: serde_json::Value,
}

async fn send_json<T: serde::de::DeserializeOwned>(
    request: reqwest::RequestBuilder,
) -> Result<T, Error> {
    let res = request.send().await?;
    let status = res.status();
    if !status.is_success() {
        let code = status.as_u16();
        return handle_http_err(code);
    }
    let body = res.json::<T>().await?;
    Ok(body)
}

pub async fn get_group_role(params: &GetGroupRoleParams) -> Result<GroupRole, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/roles/{roleId}",
        groupId = &params.group_id,
        roleId = &params.role_id,
    );
    send_json(client.get(url).header("x-api-key", &params.api_key)).await
}

pub async fn list_group_forum_categories(
    params: &ListGroupForumCategoriesParams,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/forum-categories",
        groupId = &params.group_id,
    );
    let mut query: QueryString = vec![];
    if let Some(max_page_size) = &params.max_page_size {
        query.push(("maxPageSize", max_page_size.to_string()))
    }
    if let Some(page_token) = &params.page_token {
        query.push(("pageToken", page_token.clone()));
    }
    send_json(
        client
            .get(url)
            .header("x-api-key", &params.api_key)
            .query(&query),
    )
    .await
}

pub async fn list_group_forum_posts(
    params: &ListGroupForumPostsParams,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/forum-categories/{forumCategoryId}/posts",
        groupId = &params.group_id,
        forumCategoryId = &params.forum_category_id,
    );
    let mut query: QueryString = vec![];
    if let Some(max_page_size) = &params.max_page_size {
        query.push(("maxPageSize", max_page_size.to_string()))
    }
    if let Some(page_token) = &params.page_token {
        query.push(("pageToken", page_token.clone()));
    }
    send_json(
        client
            .get(url)
            .header("x-api-key", &params.api_key)
            .query(&query),
    )
    .await
}

pub async fn list_group_forum_comments(
    params: &ListGroupForumCommentsParams,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/forum-categories/{forumCategoryId}/posts/{postId}/comments",
        groupId = &params.group_id,
        forumCategoryId = &params.forum_category_id,
        postId = &params.post_id,
    );
    let mut query: QueryString = vec![];
    if let Some(max_page_size) = &params.max_page_size {
        query.push(("maxPageSize", max_page_size.to_string()))
    }
    if let Some(page_token) = &params.page_token {
        query.push(("pageToken", page_token.clone()));
    }
    send_json(
        client
            .get(url)
            .header("x-api-key", &params.api_key)
            .query(&query),
    )
    .await
}

pub async fn list_group_join_requests(
    params: &ListGroupJoinRequestsParams,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/join-requests",
        groupId = &params.group_id,
    );
    let mut query: QueryString = vec![];
    if let Some(max_page_size) = &params.max_page_size {
        query.push(("maxPageSize", max_page_size.to_string()))
    }
    if let Some(page_token) = &params.page_token {
        query.push(("pageToken", page_token.clone()));
    }
    if let Some(filter) = &params.filter {
        query.push(("filter", filter.clone()));
    }
    send_json(
        client
            .get(url)
            .header("x-api-key", &params.api_key)
            .query(&query),
    )
    .await
}

pub async fn accept_group_join_request(
    params: &MutateGroupJoinRequestParams,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/join-requests/{joinRequestId}:accept",
        groupId = &params.group_id,
        joinRequestId = &params.join_request_id,
    );
    send_json(
        client
            .post(url)
            .header("x-api-key", &params.api_key)
            .json(&serde_json::json!({})),
    )
    .await
}

pub async fn decline_group_join_request(
    params: &MutateGroupJoinRequestParams,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/join-requests/{joinRequestId}:decline",
        groupId = &params.group_id,
        joinRequestId = &params.join_request_id,
    );
    send_json(
        client
            .post(url)
            .header("x-api-key", &params.api_key)
            .json(&serde_json::json!({})),
    )
    .await
}

pub async fn update_group_membership(
    params: &UpdateGroupMembershipParams,
) -> Result<GroupMembership, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/memberships/{membershipId}",
        groupId = &params.group_id,
        membershipId = &params.membership_id,
    );
    send_json(
        client
            .patch(url)
            .header("x-api-key", &params.api_key)
            .json(&params.body),
    )
    .await
}

pub async fn assign_group_membership_role(
    params: &MutateGroupMembershipRoleParams,
) -> Result<GroupMembership, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/memberships/{membershipId}:assignRole",
        groupId = &params.group_id,
        membershipId = &params.membership_id,
    );
    send_json(
        client
            .post(url)
            .header("x-api-key", &params.api_key)
            .json(&serde_json::json!({ "role": params.role })),
    )
    .await
}

pub async fn unassign_group_membership_role(
    params: &MutateGroupMembershipRoleParams,
) -> Result<GroupMembership, Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://apis.roblox.com/cloud/v2/groups/{groupId}/memberships/{membershipId}:unassignRole",
        groupId = &params.group_id,
        membershipId = &params.membership_id,
    );
    send_json(
        client
            .post(url)
            .header("x-api-key", &params.api_key)
            .json(&serde_json::json!({ "role": params.role })),
    )
    .await
}
