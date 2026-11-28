//! A small HTTP/1.1 client and server over `std::net`, enough for the runtime interface
//! (`ui/runtime.openapi.yaml`): JSON requests and answers, a stream of server-sent events, and
//! the requests of the fixture source. Plain `http://` only; a runtime behind TLS is reached
//! through a local address.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// An `http://host[:port]/path` address, split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    pub host: String,
    pub port: u16,
    /// The path and query, starting with `/`.
    pub path: String,
}

impl Url {
    pub fn parse(text: &str) -> Result<Url, String> {
        let rest = text.strip_prefix("http://").ok_or_else(|| {
            format!("{text}: only an http:// address is supported (put a local address before a TLS one)")
        })?;
        let (authority, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) => (
                h,
                p.parse::<u16>()
                    .map_err(|_| format!("{text}: the port is not a number"))?,
            ),
            None => (authority, 80),
        };
        if host.is_empty() {
            return Err(format!("{text}: no host"));
        }
        Ok(Url {
            host: host.to_string(),
            port,
            path: path.to_string(),
        })
    }
}

/// A complete answer.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    /// Header names in lower case.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn header(&self, name: &str) -> Option<&str> {
        header(&self.headers, name)
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn connect(url: &Url, timeout: Duration) -> io::Result<TcpStream> {
    let addr = (url.host.as_str(), url.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no address"))?;
    let stream = TcpStream::connect_timeout(&addr, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    Ok(stream)
}

fn send_head(
    stream: &mut TcpStream,
    method: &str,
    url: &Url,
    headers: &[(&str, &str)],
    body: Option<&[u8]>,
) -> io::Result<()> {
    let mut head = format!(
        "{method} {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n",
        url.path, url.host, url.port
    );
    for (k, v) in headers {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    if let Some(b) = body {
        head.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    if let Some(b) = body {
        stream.write_all(b)?;
    }
    stream.flush()
}

/// Reads a status line and the headers.
fn read_head<R: BufRead>(r: &mut R) -> io::Result<(u16, Vec<(String, String)>)> {
    let mut line = String::new();
    r.read_line(&mut line)?;
    let status = line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("not an HTTP answer: {:?}", line.trim_end()),
            )
        })?;
    let mut headers = Vec::new();
    loop {
        line.clear();
        if r.read_line(&mut line)? == 0 {
            break;
        }
        let l = line.trim_end();
        if l.is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':') {
            headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }
    Ok((status, headers))
}

/// The body of a chunked answer, read as it comes.
struct Chunked<R> {
    inner: R,
    left: usize,
    done: bool,
}

impl<R: BufRead> Read for Chunked<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.done || buf.is_empty() {
            return Ok(0);
        }
        if self.left == 0 {
            let mut line = String::new();
            if self.inner.read_line(&mut line)? == 0 {
                self.done = true;
                return Ok(0);
            }
            let size = line.trim().split(';').next().unwrap_or("");
            let size = usize::from_str_radix(size, 16).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "a chunk size is not hexadecimal",
                )
            })?;
            if size == 0 {
                self.done = true;
                loop {
                    line.clear();
                    if self.inner.read_line(&mut line)? == 0 || line.trim().is_empty() {
                        break;
                    }
                }
                return Ok(0);
            }
            self.left = size;
        }
        let want = buf.len().min(self.left);
        let n = self.inner.read(&mut buf[..want])?;
        if n == 0 {
            self.done = true;
            return Ok(0);
        }
        self.left -= n;
        if self.left == 0 {
            let mut crlf = String::new();
            self.inner.read_line(&mut crlf)?;
        }
        Ok(n)
    }
}

/// The body reader of an answer, by its headers.
fn body_reader<'a, R: BufRead + 'a>(r: R, headers: &[(String, String)]) -> Box<dyn Read + 'a> {
    if header(headers, "transfer-encoding").is_some_and(|v| v.eq_ignore_ascii_case("chunked")) {
        Box::new(Chunked {
            inner: r,
            left: 0,
            done: false,
        })
    } else if let Some(n) = header(headers, "content-length").and_then(|v| v.parse::<u64>().ok()) {
        Box::new(r.take(n))
    } else {
        Box::new(r)
    }
}

/// One request, answered completely.
pub fn request(
    method: &str,
    url: &str,
    headers: &[(&str, &str)],
    body: Option<&[u8]>,
    timeout: Duration,
) -> io::Result<Response> {
    let url = Url::parse(url).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let mut stream = connect(&url, timeout)?;
    send_head(&mut stream, method, &url, headers, body)?;
    let mut reader = BufReader::new(stream);
    let (status, headers) = read_head(&mut reader)?;
    let mut body = Vec::new();
    body_reader(reader, &headers).read_to_end(&mut body)?;
    Ok(Response {
        status,
        headers,
        body,
    })
}

/// One server-sent event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: Option<String>,
    pub event: String,
    pub data: String,
}

/// An open stream of server-sent events. Dropping it closes the connection.
pub struct EventStream {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    /// The body of an answer that is not a stream (a refusal).
    pub body: Vec<u8>,
    events: Receiver<Event>,
    stream: Option<TcpStream>,
}

impl EventStream {
    /// The next event, or none when nothing came within `wait` or the stream ended.
    pub fn next(&self, wait: Duration) -> Option<Event> {
        self.events.recv_timeout(wait).ok()
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        if let Some(s) = &self.stream {
            let _ = s.shutdown(std::net::Shutdown::Both);
        }
    }
}

/// Parses the event lines of `r` into `send` until the stream ends or `send` fails.
fn parse_events<R: BufRead>(mut r: R, send: impl Fn(Event) -> bool) {
    let (mut id, mut event, mut data): (Option<String>, String, Vec<String>) =
        (None, String::new(), Vec::new());
    let mut line = String::new();
    loop {
        line.clear();
        match r.read_line(&mut line) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let l = line.trim_end_matches(['\r', '\n']);
        if l.is_empty() {
            if !data.is_empty() {
                let e = Event {
                    id: id.take(),
                    event: if event.is_empty() {
                        "message".into()
                    } else {
                        std::mem::take(&mut event)
                    },
                    data: data.join("\n"),
                };
                data.clear();
                if !send(e) {
                    return;
                }
            } else {
                event.clear();
                id = None;
            }
            continue;
        }
        if l.starts_with(':') {
            continue;
        }
        let (field, value) = match l.split_once(':') {
            Some((f, v)) => (f, v.strip_prefix(' ').unwrap_or(v)),
            None => (l, ""),
        };
        match field {
            "event" => event = value.to_string(),
            "data" => data.push(value.to_string()),
            "id" => id = Some(value.to_string()),
            _ => {}
        }
    }
}

/// Opens a stream of server-sent events: `GET url` with `Accept: text/event-stream`.
pub fn subscribe(
    url: &str,
    headers: &[(&str, &str)],
    timeout: Duration,
) -> io::Result<EventStream> {
    let url = Url::parse(url).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let mut stream = connect(&url, timeout)?;
    let mut all: Vec<(&str, &str)> = vec![("Accept", "text/event-stream")];
    all.extend_from_slice(headers);
    send_head(&mut stream, "GET", &url, &all, None)?;
    let closer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let (status, headers) = read_head(&mut reader)?;
    let (tx, rx) = mpsc::channel();
    if status != 200 {
        let mut body = Vec::new();
        let _ = body_reader(reader, &headers).read_to_end(&mut body);
        return Ok(EventStream {
            status,
            headers,
            body,
            events: rx,
            stream: Some(closer),
        });
    }
    // The reader blocks; idle time is measured on the channel, never on the socket.
    reader.get_ref().set_read_timeout(None)?;
    let hs = headers.clone();
    std::thread::spawn(move || {
        let body = BufReader::new(body_reader(reader, &hs));
        parse_events(body, |e| tx.send(e).is_ok());
    });
    Ok(EventStream {
        status,
        headers,
        body: Vec::new(),
        events: rx,
        stream: Some(closer),
    })
}

/// A request the fixture source received.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    /// The path without the query, percent-decoded per segment.
    pub path: String,
    pub query: Vec<(String, String)>,
    /// Header names in lower case.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        header(&self.headers, name)
    }
}

/// `%xx` and `+` decoded.
pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(hex) = std::str::from_utf8(&b[i + 1..i + 3])
            && let Ok(x) = u8::from_str_radix(hex, 16)
        {
            out.push(x);
            i += 3;
            continue;
        }
        out.push(if b[i] == b'+' { b' ' } else { b[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `%xx` for every byte outside the unreserved set, as `encodeURIComponent` does.
pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Reads one request from a connection.
pub fn read_request<R: BufRead>(r: &mut R) -> io::Result<Request> {
    let mut line = String::new();
    r.read_line(&mut line)?;
    let mut words = line.split_whitespace();
    let (Some(method), Some(target)) = (words.next(), words.next()) else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a request"));
    };
    let (method, target) = (method.to_string(), target.to_string());
    let (raw_path, raw_query) = target
        .as_str()
        .split_once('?')
        .unwrap_or((target.as_str(), ""));
    let path = raw_path
        .split('/')
        .map(percent_decode)
        .collect::<Vec<_>>()
        .join("/");
    let query = raw_query
        .split('&')
        .filter(|kv| !kv.is_empty())
        .map(|kv| {
            let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
            (percent_decode(k), percent_decode(v))
        })
        .collect();
    let mut headers = Vec::new();
    loop {
        line.clear();
        if r.read_line(&mut line)? == 0 {
            break;
        }
        let l = line.trim_end();
        if l.is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':') {
            headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }
    let n = header(&headers, "content-length")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = vec![0; n.min(1 << 20)];
    r.read_exact(&mut body)?;
    Ok(Request {
        method,
        path,
        query,
        headers,
        body,
    })
}

pub fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        422 => "Unprocessable Content",
        501 => "Not Implemented",
        _ => "Status",
    }
}

/// Writes a complete JSON answer, `text` as given.
pub fn write_json(
    stream: &mut impl Write,
    status: u16,
    extra: &[(&str, &str)],
    text: &str,
) -> io::Result<()> {
    let mut head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        reason(status),
        text.len()
    );
    for (k, v) in extra {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(text.as_bytes())?;
    stream.flush()
}

/// One server-sent event as text.
pub fn event_text(event: &str, id: Option<&str>, data: &str) -> String {
    let mut out = format!("event: {event}\n");
    if let Some(id) = id {
        out.push_str(&format!("id: {id}\n"));
    }
    for line in data.split('\n') {
        out.push_str("data: ");
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
    out
}
