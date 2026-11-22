use std::net::SocketAddr;
use std::time::Duration;

use reqwest::Url;

/// Hard ceilings, so no policy can make a call retry or poll without end.
pub const MAX_RETRIES_LIMIT: u32 = 10;
pub const MAX_POLL_ATTEMPTS_LIMIT: u32 = 1_000;
/// Longest `timeout` (one HTTP attempt).
pub const MAX_TIMEOUT_LIMIT: Duration = Duration::from_secs(600);
/// Longest `max_retry_delay` (one wait between retries).
pub const MAX_RETRY_DELAY_LIMIT: Duration = Duration::from_secs(300);
/// Longest `poll_delay` (one wait between operation status checks).
pub const MAX_POLL_DELAY_LIMIT: Duration = Duration::from_secs(300);
/// Longest `max_total_time` (one whole `execute` call).
pub const MAX_TOTAL_TIME_LIMIT: Duration = Duration::from_secs(3600);

/// The Roblox hosts that serve API-key operations. Each operation names its
/// server (the `server` field of `openapi/coverage.json`); callers never
/// supply a host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Server {
    /// `https://apis.roblox.com`, the default server of the specification.
    Apis,
    /// `https://itemconfiguration.roblox.com`
    ItemConfiguration,
    /// `https://thumbnails.roblox.com`
    Thumbnails,
}

impl Server {
    pub const ALL: [Server; 3] = [Self::Apis, Self::ItemConfiguration, Self::Thumbnails];

    /// The production origin: scheme and host, no path.
    pub const fn origin(self) -> &'static str {
        match self {
            Self::Apis => "https://apis.roblox.com",
            Self::ItemConfiguration => "https://itemconfiguration.roblox.com",
            Self::Thumbnails => "https://thumbnails.roblox.com",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Apis => 0,
            Self::ItemConfiguration => 1,
            Self::Thumbnails => 2,
        }
    }
}

/// Where requests go and how long the client keeps trying.
///
/// There are two ways to make one. [`ClientPolicy::default`] talks HTTPS to
/// the Roblox servers. [`ClientPolicy::loopback`] talks plain HTTP to a
/// loopback address and exists for tests. No policy can send a credential to
/// any other host.
#[derive(Debug, Clone)]
pub struct ClientPolicy {
    /// `Some` only for a loopback test policy, indexed by [`Server::index`].
    loopback: Option<[SocketAddr; 3]>,
    pub(crate) timeout: Duration,
    pub(crate) max_retries: u32,
    pub(crate) retry_delay: Duration,
    pub(crate) max_retry_delay: Duration,
    pub(crate) max_poll_attempts: u32,
    pub(crate) poll_delay: Duration,
    pub(crate) max_total_time: Duration,
    pub(crate) max_response_bytes: u64,
}

impl Default for ClientPolicy {
    fn default() -> Self {
        Self {
            loopback: None,
            timeout: Duration::from_secs(60),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            max_retry_delay: Duration::from_secs(30),
            max_poll_attempts: 30,
            poll_delay: Duration::from_secs(2),
            max_total_time: Duration::from_secs(600),
            max_response_bytes: 64 * 1024 * 1024,
        }
    }
}

impl ClientPolicy {
    /// A test policy: every server is reached over plain HTTP at `addr`.
    /// Fails unless `addr` is a loopback address.
    pub fn loopback(addr: SocketAddr) -> Result<Self, String> {
        require_loopback(addr)?;
        Ok(Self {
            loopback: Some([addr; 3]),
            ..Self::default()
        })
    }

    /// Sends one server to a different loopback address. Only valid on a
    /// policy made by [`ClientPolicy::loopback`].
    pub fn with_loopback_server(
        mut self,
        server: Server,
        addr: SocketAddr,
    ) -> Result<Self, String> {
        require_loopback(addr)?;
        match self.loopback.as_mut() {
            Some(addrs) => addrs[server.index()] = addr,
            None => return Err("only a loopback policy can redirect a server".into()),
        }
        Ok(self)
    }

    /// Time allowed for one HTTP attempt, from connecting to the last byte of
    /// the response body. At most [`MAX_TIMEOUT_LIMIT`].
    ///
    /// The attempt is abandoned at this deadline whether the server is
    /// silent or keeps sending data slowly.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Retries after the first attempt, at most [`MAX_RETRIES_LIMIT`]. Only
    /// operations marked safe or idempotent are ever retried.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// First wait between retries; it doubles each time. A `Retry-After` or
    /// `x-ratelimit-reset` response header takes precedence.
    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Longest single wait between retries, whatever the server asks for.
    /// At most [`MAX_RETRY_DELAY_LIMIT`].
    pub fn max_retry_delay(mut self, delay: Duration) -> Self {
        self.max_retry_delay = delay;
        self
    }

    /// Status checks made for one long-running operation, at most
    /// [`MAX_POLL_ATTEMPTS_LIMIT`].
    pub fn max_poll_attempts(mut self, attempts: u32) -> Self {
        self.max_poll_attempts = attempts;
        self
    }

    /// Wait before each status check of a long-running operation. At most
    /// [`MAX_POLL_DELAY_LIMIT`].
    pub fn poll_delay(mut self, delay: Duration) -> Self {
        self.poll_delay = delay;
        self
    }

    /// Time allowed for one whole [`execute`](super::Client::execute) call:
    /// every attempt, retry wait and operation status check together.
    /// Default 10 minutes, at most [`MAX_TOTAL_TIME_LIMIT`].
    ///
    /// No attempt starts and no wait begins once it would pass this deadline,
    /// and an attempt already running is cut to the time left.
    pub fn max_total_time(mut self, total: Duration) -> Self {
        self.max_total_time = total;
        self
    }

    /// Largest response body the client will read.
    pub fn max_response_bytes(mut self, bytes: u64) -> Self {
        self.max_response_bytes = bytes;
        self
    }

    /// `true` for the HTTPS Roblox policy, `false` for a loopback test policy.
    pub fn is_production(&self) -> bool {
        self.loopback.is_none()
    }

    /// The origin requests for `server` are sent to.
    pub fn origin(&self, server: Server) -> Url {
        let text = match &self.loopback {
            Some(addrs) => format!("http://{}", addrs[server.index()]),
            None => server.origin().to_string(),
        };
        Url::parse(&text).expect("origins are fixed, valid URLs")
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let durations = [
            ("timeout", self.timeout, MAX_TIMEOUT_LIMIT, false),
            ("retry_delay", self.retry_delay, MAX_RETRY_DELAY_LIMIT, true),
            (
                "max_retry_delay",
                self.max_retry_delay,
                MAX_RETRY_DELAY_LIMIT,
                true,
            ),
            ("poll_delay", self.poll_delay, MAX_POLL_DELAY_LIMIT, true),
            (
                "max_total_time",
                self.max_total_time,
                MAX_TOTAL_TIME_LIMIT,
                false,
            ),
        ];
        for (name, value, limit, zero_allowed) in durations {
            if value.is_zero() && !zero_allowed {
                return Err(format!("{name} must be greater than zero"));
            }
            if value > limit {
                return Err(format!(
                    "{name} must be at most {} seconds",
                    limit.as_secs()
                ));
            }
        }
        if self.max_retries > MAX_RETRIES_LIMIT {
            return Err(format!("max_retries must be at most {MAX_RETRIES_LIMIT}"));
        }
        if self.max_poll_attempts > MAX_POLL_ATTEMPTS_LIMIT {
            return Err(format!(
                "max_poll_attempts must be at most {MAX_POLL_ATTEMPTS_LIMIT}"
            ));
        }
        if self.max_response_bytes == 0 {
            return Err("max_response_bytes must be greater than zero".into());
        }
        Ok(())
    }
}

fn require_loopback(addr: SocketAddr) -> Result<(), String> {
    if addr.ip().is_loopback() {
        Ok(())
    } else {
        Err(format!("{addr} is not a loopback address"))
    }
}
