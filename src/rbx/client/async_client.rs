//! The supported way to use the transport from async code.
use std::cell::Cell;
use std::sync::Arc;

use serde_json::Value;

use super::{BinaryResponse, Client, ClientPolicy, Error, Request};
use crate::rbx::credentials::Credentials;

thread_local! {
    /// Set while this thread is running transport work for [`AsyncClient`].
    static ON_BLOCKING_POOL: Cell<bool> = const { Cell::new(false) };
}

/// Marks the current thread as allowed to block, until dropped.
struct BlockingAllowed(bool);

impl BlockingAllowed {
    fn enter() -> Self {
        Self(ON_BLOCKING_POOL.with(|flag| flag.replace(true)))
    }
}

impl Drop for BlockingAllowed {
    fn drop(&mut self) {
        ON_BLOCKING_POOL.with(|flag| flag.set(self.0));
    }
}

/// Fails when called on a thread that belongs to a tokio runtime, unless the
/// call came through [`AsyncClient`].
///
/// On such a thread the blocking HTTP client panics in debug builds and, in
/// release builds, silently stalls the runtime for the whole call. tokio has
/// no way to tell an async worker from one of its blocking-pool threads, so a
/// caller's own `spawn_blocking` is refused too; [`AsyncClient::run`] covers
/// that case.
pub(super) fn refuse_on_async_thread() -> Result<(), Error> {
    let in_runtime = tokio::runtime::Handle::try_current().is_ok();
    if in_runtime && !ON_BLOCKING_POOL.with(Cell::get) {
        return Err(Error::Config(
            "the blocking client was called on an async runtime thread; use AsyncClient".into(),
        ));
    }
    Ok(())
}

/// Async front of the blocking [`Client`].
///
/// Every call runs on tokio's blocking thread pool, so no async worker is
/// held while the transport sends, waits between retries or polls an
/// operation. CLI commands and other async code use this type and never call
/// [`Client`] directly. Cloning is cheap and shares one connection pool.
///
/// Dropping the returned future does not stop a call that has started; it
/// ends on its own within [`ClientPolicy::max_total_time`].
#[derive(Debug, Clone)]
pub struct AsyncClient {
    inner: Arc<Client>,
}

impl AsyncClient {
    /// Builds the client. Must be awaited inside a tokio runtime.
    pub async fn new(credentials: Credentials, policy: ClientPolicy) -> Result<Self, Error> {
        let built = tokio::task::spawn_blocking(move || {
            let _allowed = BlockingAllowed::enter();
            Client::new(credentials, policy)
        })
        .await;
        match built {
            Ok(client) => Ok(Self {
                inner: Arc::new(client?),
            }),
            Err(_) => Err(Error::TaskFailed {
                operation: "client setup".into(),
            }),
        }
    }

    pub fn policy(&self) -> &ClientPolicy {
        self.inner.policy()
    }

    /// Async form of [`Client::execute`].
    pub async fn execute(&self, request: Request) -> Result<Value, Error> {
        let label = request.operation.label();
        self.spawn(label, move |client| client.execute(request))
            .await
    }

    pub async fn execute_binary(&self, request: Request) -> Result<BinaryResponse, Error> {
        self.spawn("binary asset download".into(), move |client| {
            client.execute_binary(request)
        })
        .await
    }

    /// Runs `work` with the blocking client on the blocking thread pool. Use
    /// it for a sequence of calls, such as following every page of a list.
    pub async fn run<T, F>(&self, work: F) -> Result<T, Error>
    where
        F: FnOnce(&Client) -> Result<T, Error> + Send + 'static,
        T: Send + 'static,
    {
        self.spawn("transport task".into(), work).await
    }

    async fn spawn<T, F>(&self, label: String, work: F) -> Result<T, Error>
    where
        F: FnOnce(&Client) -> Result<T, Error> + Send + 'static,
        T: Send + 'static,
    {
        let client = self.inner.clone();
        let joined = tokio::task::spawn_blocking(move || {
            let _allowed = BlockingAllowed::enter();
            let result = work(&client);
            // Dropped here so that the last reference, which joins the HTTP
            // client's thread, is never released on an async worker.
            drop(client);
            result
        })
        .await;
        joined.unwrap_or(Err(Error::TaskFailed { operation: label }))
    }
}
