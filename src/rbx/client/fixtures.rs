//! Synthetic operations for `tests/transport.rs`.
//!
//! Compiled only with the `transport-fixtures` feature. They exist so the
//! transport can be tested from an integration test while the request
//! constructors stay private to the crate. Every path starts with
//! `/transport-fixtures/`, which no Roblox server serves, and the client
//! refuses to send them anywhere but a loopback test server.
use std::path::Path;

use serde_json::Value;

use super::operation::{finished_operation, Operation, Poll};
use super::request::Attachment;
use super::{ClientPolicy, Error, Request, Server};

const PREFIX: &str = "/transport-fixtures/";

const GET_ITEM: Operation = Operation::get(Server::Apis, "/transport-fixtures/items/{item_id}");
const LIST_ITEMS: Operation = Operation::get(Server::Apis, "/transport-fixtures/items");
const CREATE_ITEM: Operation = Operation::post(Server::Apis, "/transport-fixtures/items");
const UPDATE_ITEM: Operation =
    Operation::patch(Server::Apis, "/transport-fixtures/items/{item_id}");
const REPLACE_ITEM: Operation =
    Operation::put(Server::Apis, "/transport-fixtures/items/{item_id}").idempotent();
const DELETE_ITEM: Operation =
    Operation::delete(Server::Apis, "/transport-fixtures/items/{item_id}");
const UPLOAD: Operation = Operation::post(Server::Apis, "/transport-fixtures/uploads");
const START_JOB: Operation = Operation::post(Server::Apis, "/transport-fixtures/jobs");
const GET_JOB: Operation = Operation::get(Server::Apis, "/transport-fixtures/jobs/{job_id}");
const GET_THUMBNAIL: Operation = Operation::get(
    Server::Thumbnails,
    "/transport-fixtures/thumbnails/{item_id}",
);
const GET_COLLECTIBLE: Operation = Operation::get(
    Server::ItemConfiguration,
    "/transport-fixtures/collectibles/{item_id}",
);

pub(super) fn refuse_in_production(request: &Request, policy: &ClientPolicy) -> Result<(), Error> {
    if policy.is_production() && request.operation.path.starts_with(PREFIX) {
        return Err(Error::InvalidRequest {
            operation: request.operation.label(),
            reason: "test fixtures are only sent to a loopback test server".into(),
        });
    }
    Ok(())
}

pub fn get_item(item_id: &str) -> Request {
    Request::new(GET_ITEM).path_param("item_id", item_id)
}

pub fn list_items(page_size: Option<u32>, page_token: Option<&str>, tags: &[&str]) -> Request {
    let mut request = Request::new(LIST_ITEMS)
        .query_opt("maxPageSize", page_size)
        .query_opt("pageToken", page_token);
    for tag in tags {
        request = request.query("tag", tag);
    }
    request
}

pub fn create_item(body: &Value) -> Request {
    Request::new(CREATE_ITEM)
        .json(body)
        .expect("a JSON value always serializes")
}

pub fn update_item(item_id: &str, update_mask: &str, body: &Value) -> Request {
    Request::new(UPDATE_ITEM)
        .path_param("item_id", item_id)
        .query("updateMask", update_mask)
        .header("Roblox-Place-Id", 99)
        .json(body)
        .expect("a JSON value always serializes")
}

pub fn replace_item(item_id: &str, body: &Value) -> Request {
    Request::new(REPLACE_ITEM)
        .path_param("item_id", item_id)
        .json(body)
        .expect("a JSON value always serializes")
}

pub fn delete_item(item_id: &str) -> Request {
    Request::new(DELETE_ITEM).path_param("item_id", item_id)
}

/// A typed request that wrongly sets a header owned by the transport.
pub fn get_item_overriding_api_key(item_id: &str, value: &str) -> Request {
    get_item(item_id).header("X-Api-Key", value)
}

pub fn upload_bytes(
    metadata: &Value,
    file_name: &str,
    content_type: &str,
    data: Vec<u8>,
) -> Result<Request, Error> {
    Ok(upload(
        metadata,
        Attachment::from_bytes(file_name, content_type, data)?,
    ))
}

pub fn upload_file(metadata: &Value, path: &Path, content_type: &str) -> Result<Request, Error> {
    Ok(upload(metadata, Attachment::from_path(path, content_type)?))
}

fn upload(metadata: &Value, file: Attachment) -> Request {
    Request::new(UPLOAD)
        .multipart_text("request", metadata.to_string())
        .multipart_file("fileContent", file)
}

/// Starts a job and waits for it. The job is checked at
/// `/transport-fixtures/jobs/{operationId}`.
pub fn start_job(body: &Value) -> Request {
    Request::new(START_JOB)
        .json(body)
        .expect("a JSON value always serializes")
        .poll_with(job_state)
}

fn job_state(job: &Value) -> Result<Poll, String> {
    if let Some(finished) = finished_operation(job) {
        return Ok(finished);
    }
    let id = job
        .get("operationId")
        .and_then(Value::as_str)
        .ok_or("the job has no operationId")?;
    Ok(Poll::Pending(Box::new(
        Request::new(GET_JOB).path_param("job_id", id),
    )))
}

pub fn get_thumbnail(item_id: u64) -> Request {
    Request::new(GET_THUMBNAIL).path_param("item_id", item_id)
}

pub fn get_collectible(item_id: u64) -> Request {
    Request::new(GET_COLLECTIBLE).path_param("item_id", item_id)
}
