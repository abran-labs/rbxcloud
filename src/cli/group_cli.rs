use clap::{Args, Subcommand};
use rbxcloud::rbx::{types::GroupId, v2::Client};
use serde::Serialize;
use serde_json::Value;

fn json_output<T: Serialize>(value: &T, pretty: bool) -> anyhow::Result<String> {
    if pretty {
        Ok(serde_json::to_string_pretty(value)?)
    } else {
        Ok(serde_json::to_string(value)?)
    }
}

#[derive(Debug, Subcommand)]
pub enum GroupCommands {
    /// Get info about the group
    Get {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// Get the current shout and other metadata
    Shout {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Only return the shout message string
        #[clap(short, long, value_parser, default_value_t = false)]
        only_message: bool,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// List the forum categories of a group
    ForumCategories {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Max items returned per page
        #[clap(short, long, value_parser)]
        max_page_size: Option<u32>,

        /// Next page token
        #[clap(short, long, value_parser)]
        next_page_token: Option<String>,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// List forum posts for a forum category
    ForumPosts {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Forum category ID
        #[clap(short = 'c', long, value_parser)]
        forum_category_id: String,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Max items returned per page
        #[clap(short, long, value_parser)]
        max_page_size: Option<u32>,

        /// Next page token
        #[clap(short, long, value_parser)]
        next_page_token: Option<String>,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// List forum comments for a forum post
    ForumComments {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Forum category ID
        #[clap(short = 'c', long, value_parser)]
        forum_category_id: String,

        /// Post ID
        #[clap(short = 'o', long, value_parser)]
        post_id: String,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Max items returned per page
        #[clap(short, long, value_parser)]
        max_page_size: Option<u32>,

        /// Next page token
        #[clap(short, long, value_parser)]
        next_page_token: Option<String>,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// List pending join requests for a group
    JoinRequests {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Max items returned per page
        #[clap(short, long, value_parser)]
        max_page_size: Option<u32>,

        /// Filter, e.g. "user == 'users/156'"
        #[clap(short, long, value_parser)]
        filter: Option<String>,

        /// Next page token
        #[clap(short, long, value_parser)]
        next_page_token: Option<String>,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// Accept a group join request
    AcceptJoinRequest {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Join request ID. For user join requests, this is usually the user ID.
        #[clap(short = 'j', long, value_parser)]
        join_request_id: String,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// Decline a group join request
    DeclineJoinRequest {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Join request ID. For user join requests, this is usually the user ID.
        #[clap(short = 'j', long, value_parser)]
        join_request_id: String,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// List the roles of a group
    Roles {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Max items returned per page
        #[clap(short, long, value_parser)]
        max_page_size: Option<u32>,

        /// Next page token
        #[clap(short, long, value_parser)]
        next_page_token: Option<String>,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// Get a group role by role ID
    Role {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Role ID
        #[clap(short, long, value_parser)]
        role_id: String,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// List the memberships of a group
    Memberships {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Max items returned per page
        #[clap(short, long, value_parser)]
        max_page_size: Option<u32>,

        /// Filter, e.g. "user == 'users/156'"
        #[clap(short, long, value_parser)]
        filter: Option<String>,

        /// Next page token
        #[clap(short, long, value_parser)]
        next_page_token: Option<String>,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// Patch a group membership with a raw JSON body
    UpdateMembership {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Membership ID. Roblox docs allow a user ID for membership-specific operations.
        #[clap(short = 'i', long, value_parser)]
        membership_id: String,

        /// JSON request body
        #[clap(short = 'b', long, value_parser)]
        body: String,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// Assign a role to a group membership
    AssignRole {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Membership ID. Roblox docs allow a user ID for membership-specific operations.
        #[clap(short = 'i', long, value_parser)]
        membership_id: String,

        /// Role path, e.g. groups/123/roles/456
        #[clap(short, long, value_parser)]
        role: String,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },

    /// Unassign a role from a group membership
    UnassignRole {
        /// Group ID
        #[clap(short, long, value_parser)]
        group_id: u64,

        /// Membership ID. Roblox docs allow a user ID for membership-specific operations.
        #[clap(short = 'i', long, value_parser)]
        membership_id: String,

        /// Role path, e.g. groups/123/roles/456
        #[clap(short, long, value_parser)]
        role: String,

        /// Pretty-print the JSON response
        #[clap(short, long, value_parser, default_value_t = false)]
        pretty: bool,

        /// Roblox Open Cloud API Key
        #[clap(short, long, value_parser, env = "RBXCLOUD_API_KEY")]
        api_key: String,
    },
}

#[derive(Debug, Args)]
pub struct Group {
    #[clap(subcommand)]
    command: GroupCommands,
}

impl Group {
    pub async fn run(self) -> anyhow::Result<Option<String>> {
        match self.command {
            GroupCommands::Get {
                group_id,
                api_key,
                pretty,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group.get_info().await;
                match res {
                    Ok(group_info) => Ok(Some(json_output(&group_info, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::Shout {
                group_id,
                pretty,
                only_message,
                api_key,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group.get_shout().await;
                match res {
                    Ok(group_info) => {
                        if only_message {
                            return Ok(Some(group_info.content));
                        }
                        Ok(Some(json_output(&group_info, pretty)?))
                    }
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::ForumCategories {
                group_id,
                api_key,
                pretty,
                max_page_size,
                next_page_token,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group
                    .list_forum_categories(max_page_size, next_page_token)
                    .await;
                match res {
                    Ok(value) => Ok(Some(json_output(&value, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::ForumPosts {
                group_id,
                forum_category_id,
                api_key,
                pretty,
                max_page_size,
                next_page_token,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group
                    .list_forum_posts(forum_category_id, max_page_size, next_page_token)
                    .await;
                match res {
                    Ok(value) => Ok(Some(json_output(&value, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::ForumComments {
                group_id,
                forum_category_id,
                post_id,
                api_key,
                pretty,
                max_page_size,
                next_page_token,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group
                    .list_forum_comments(forum_category_id, post_id, max_page_size, next_page_token)
                    .await;
                match res {
                    Ok(value) => Ok(Some(json_output(&value, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::JoinRequests {
                group_id,
                api_key,
                pretty,
                max_page_size,
                filter,
                next_page_token,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group
                    .list_join_requests(max_page_size, filter, next_page_token)
                    .await;
                match res {
                    Ok(value) => Ok(Some(json_output(&value, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::AcceptJoinRequest {
                group_id,
                join_request_id,
                api_key,
                pretty,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group.accept_join_request(join_request_id).await;
                match res {
                    Ok(value) => Ok(Some(json_output(&value, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::DeclineJoinRequest {
                group_id,
                join_request_id,
                api_key,
                pretty,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group.decline_join_request(join_request_id).await;
                match res {
                    Ok(value) => Ok(Some(json_output(&value, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::Roles {
                group_id,
                api_key,
                pretty,
                max_page_size,
                next_page_token,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group.list_roles(max_page_size, next_page_token).await;
                match res {
                    Ok(group_info) => Ok(Some(json_output(&group_info, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::Role {
                group_id,
                role_id,
                api_key,
                pretty,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group.get_role(role_id).await;
                match res {
                    Ok(role) => Ok(Some(json_output(&role, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::Memberships {
                group_id,
                api_key,
                pretty,
                max_page_size,
                next_page_token,
                filter,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group
                    .list_memberships(max_page_size, filter, next_page_token)
                    .await;
                match res {
                    Ok(group_info) => Ok(Some(json_output(&group_info, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::UpdateMembership {
                group_id,
                membership_id,
                body,
                api_key,
                pretty,
            } => {
                let body: Value = serde_json::from_str(&body)?;
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group.update_membership(membership_id, body).await;
                match res {
                    Ok(membership) => Ok(Some(json_output(&membership, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::AssignRole {
                group_id,
                membership_id,
                role,
                api_key,
                pretty,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group.assign_role(membership_id, role).await;
                match res {
                    Ok(membership) => Ok(Some(json_output(&membership, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }

            GroupCommands::UnassignRole {
                group_id,
                membership_id,
                role,
                api_key,
                pretty,
            } => {
                let client = Client::new(&api_key);
                let group = client.group(GroupId(group_id));
                let res = group.unassign_role(membership_id, role).await;
                match res {
                    Ok(membership) => Ok(Some(json_output(&membership, pretty)?)),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }
        }
    }
}
