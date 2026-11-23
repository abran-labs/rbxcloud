//! A scripted HTTP/1.1 server on a loopback `TcpListener`.
//!
//! It answers each connection with the next scripted reply and records what the
//! client sent, so transport tests never touch the network.
#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: String,
    /// Request target exactly as sent: path plus raw query string.
    pub target: String,
    /// Header names are lower-cased.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Recorded {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn body_text(&self) -> String {
        String::from_utf8(self.body.clone()).expect("request body is UTF-8")
    }
}

#[derive(Debug, Clone)]
enum Mode {
    Respond,
    /// Read the request, then close the connection without answering.
    Close,
    /// Read the request, wait, then close without answering.
    Hang(Duration),
    /// Send the head at once, then the body one byte per interval.
    Drip(Duration),
}

#[derive(Debug, Clone)]
pub struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    mode: Mode,
}

impl Reply {
    pub fn binary(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            body,
            ..Self::empty(status)
        }
    }

    pub fn json(status: u16, body: &str) -> Self {
        Self::text(status, body).header("Content-Type", "application/json")
    }

    pub fn text(status: u16, body: &str) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: body.as_bytes().to_vec(),
            mode: Mode::Respond,
        }
    }

    pub fn empty(status: u16) -> Self {
        Self::text(status, "")
    }

    pub fn close() -> Self {
        Self {
            mode: Mode::Close,
            ..Self::empty(0)
        }
    }

    pub fn hang(delay: Duration) -> Self {
        Self {
            mode: Mode::Hang(delay),
            ..Self::empty(0)
        }
    }

    /// A reply whose body arrives one byte per `interval`.
    pub fn drip(status: u16, body: &str, interval: Duration) -> Self {
        Self {
            mode: Mode::Drip(interval),
            ..Self::json(status, body)
        }
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }
}

pub struct TestServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<Recorded>>>,
    connections: Arc<AtomicUsize>,
}

impl TestServer {
    /// Starts a server that answers with `replies` in order. Once the script is
    /// used up it answers 500, so an unexpected extra request fails the test.
    pub fn start(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
        let addr = listener.local_addr().expect("listener address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let connections = Arc::new(AtomicUsize::new(0));
        let (seen, count) = (requests.clone(), connections.clone());
        thread::spawn(move || {
            let mut script = replies.into_iter();
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                count.fetch_add(1, Ordering::SeqCst);
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let Some(request) = read_request(&mut stream) else {
                    continue;
                };
                seen.lock().unwrap().push(request);
                let reply = script.next().unwrap_or_else(|| {
                    Reply::json(500, r#"{"message":"test server script exhausted"}"#)
                });
                match reply.mode {
                    Mode::Respond => write_reply(&mut stream, &reply),
                    Mode::Close => {}
                    Mode::Hang(delay) => thread::sleep(delay),
                    // On its own thread, so later connections are still served.
                    Mode::Drip(interval) => {
                        thread::spawn(move || {
                            write_head(&mut stream, &reply);
                            for byte in &reply.body {
                                thread::sleep(interval);
                                if stream.write_all(&[*byte]).is_err() || stream.flush().is_err() {
                                    break;
                                }
                            }
                        });
                    }
                }
            }
        });
        Self {
            addr,
            requests,
            connections,
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.requests.lock().unwrap().clone()
    }

    /// Number of TCP connections accepted, including ones that sent nothing.
    pub fn connections(&self) -> usize {
        self.connections.load(Ordering::SeqCst)
    }
}

fn write_reply(stream: &mut TcpStream, reply: &Reply) {
    write_head(stream, reply);
    let _ = stream.write_all(&reply.body);
    let _ = stream.flush();
}

fn write_head(stream: &mut TcpStream, reply: &Reply) {
    let mut head = format!("HTTP/1.1 {} Scripted\r\n", reply.status);
    for (name, value) in &reply.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        reply.body.len()
    ));
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.flush();
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn read_request(stream: &mut TcpStream) -> Option<Recorded> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    let head_end = loop {
        if let Some(at) = find(&buf, b"\r\n\r\n") {
            break at + 4;
        }
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..read]);
    };

    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut request_line = lines.next()?.split(' ');
    let method = request_line.next()?.to_string();
    let target = request_line.next()?.to_string();
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(n, v)| (n.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    let header = |name: &str| {
        headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    };

    let mut rest = buf[head_end..].to_vec();
    let body = if let Some(length) = header("content-length") {
        let length: usize = length.parse().ok()?;
        while rest.len() < length {
            let read = stream.read(&mut chunk).ok()?;
            if read == 0 {
                return None;
            }
            rest.extend_from_slice(&chunk[..read]);
        }
        rest.truncate(length);
        rest
    } else if header("transfer-encoding").is_some_and(|v| v.eq_ignore_ascii_case("chunked")) {
        while !rest.ends_with(b"0\r\n\r\n") {
            let read = stream.read(&mut chunk).ok()?;
            if read == 0 {
                return None;
            }
            rest.extend_from_slice(&chunk[..read]);
        }
        decode_chunked(&rest)?
    } else {
        Vec::new()
    };

    Some(Recorded {
        method,
        target,
        headers,
        body,
    })
}

fn decode_chunked(mut data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let line_end = find(data, b"\r\n")?;
        let size_text = std::str::from_utf8(&data[..line_end]).ok()?;
        let size = usize::from_str_radix(size_text.split(';').next()?.trim(), 16).ok()?;
        data = &data[line_end + 2..];
        if size == 0 {
            return Some(out);
        }
        out.extend_from_slice(data.get(..size)?);
        data = data.get(size + 2..)?;
    }
}

#[derive(Debug)]
pub struct PartSeen {
    pub name: String,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

/// Splits a `multipart/form-data` body into its parts.
pub fn parse_multipart(content_type: &str, body: &[u8]) -> Vec<PartSeen> {
    let boundary = content_type
        .split("boundary=")
        .nth(1)
        .expect("multipart content type has a boundary")
        .trim_matches('"');
    let delimiter = format!("\r\n--{boundary}").into_bytes();
    let mut framed = b"\r\n".to_vec();
    framed.extend_from_slice(body);

    let mut pieces = Vec::new();
    let mut rest = &framed[..];
    while let Some(at) = find(rest, &delimiter) {
        pieces.push(&rest[..at]);
        rest = &rest[at + delimiter.len()..];
    }
    assert!(
        rest.starts_with(b"--"),
        "multipart body has a closing delimiter"
    );

    let attribute = |disposition: &str, key: &str| {
        disposition
            .split(';')
            .map(str::trim)
            .find_map(|item| item.strip_prefix(&format!("{key}=\"")))
            .map(|v| v.trim_end_matches('"').to_string())
    };

    pieces
        .into_iter()
        .skip(1)
        .map(|piece| {
            let piece = piece
                .strip_prefix(b"\r\n")
                .expect("part starts on a new line");
            let head_end = find(piece, b"\r\n\r\n").expect("part has headers");
            let head = String::from_utf8_lossy(&piece[..head_end]).into_owned();
            let field = |name: &str| {
                head.split("\r\n")
                    .filter_map(|line| line.split_once(':'))
                    .find(|(n, _)| n.eq_ignore_ascii_case(name))
                    .map(|(_, v)| v.trim().to_string())
            };
            let disposition = field("content-disposition").expect("part has a disposition");
            PartSeen {
                name: attribute(&disposition, "name").expect("part has a name"),
                file_name: attribute(&disposition, "filename"),
                content_type: field("content-type"),
                data: piece[head_end + 4..].to_vec(),
            }
        })
        .collect()
}
