use std::{
    collections::{HashMap, HashSet},
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
    str::Utf8Error,
    sync::Arc,
};

use rustls::{pki_types::InvalidDnsNameError, ClientConfig, ClientConnection, Stream};
use thiserror::Error;

use crate::url::{Scheme, Url, UrlError};

pub struct Client {
    config: Arc<ClientConfig>,
    static_hosts: HashSet<&'static str>,
    open_http_streams: HashMap<String, TcpStream>,
    open_https_streams: HashMap<String, (ClientConnection, TcpStream)>,
}

const MAX_REDIRECTS: u16 = 128;

impl Client {
    pub fn new() -> Client {
        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        );

        Client {
            config,
            static_hosts: HashSet::new(),
            open_http_streams: HashMap::new(),
            open_https_streams: HashMap::new(),
        }
    }

    pub fn request(&mut self, url: &Url) -> Result<Response, RequestError> {
        self.request_internal(url, MAX_REDIRECTS)
    }

    fn request_internal(
        &mut self,
        url: &Url,
        remaining_redirects: u16,
    ) -> Result<Response, RequestError> {
        eprintln!("Making a GET request to {url}");
        let host = url.host();

        match url.scheme() {
            Scheme::Http => {
                if !self.open_http_streams.contains_key(host) {
                    let stream = TcpStream::connect((host, url.port().unwrap_or(80)))?;
                    self.open_http_streams.insert(host.to_string(), stream);
                }

                let mut stream = self.open_http_streams.get_mut(host).unwrap();
                send_get(url, &mut stream)?;

                let resp = recv_response(BufReader::new(stream))?;
                self.handle_redirect(url, resp, remaining_redirects)
            }
            Scheme::Https => {
                if !self.open_https_streams.contains_key(host) {
                    let static_host = self.static_host(host);
                    let tls = rustls::ClientConnection::new(
                        self.config.clone(),
                        static_host.try_into()?,
                    )?;
                    let tcp = TcpStream::connect((host, url.port().unwrap_or(443)))?;
                    self.open_https_streams.insert(host.to_string(), (tls, tcp));
                }
                let (tls, stream) = self.open_https_streams.get_mut(host).unwrap();
                let mut stream = rustls::Stream::new(tls, stream);
                send_get(url, &mut stream)?;
                let resp = recv_response(BufReader::new(stream))?;
                self.handle_redirect(url, resp, remaining_redirects)
            }
            Scheme::File => {
                return Ok(Response {
                    status_code: 200,
                    headers: HashMap::new(),
                    body: Body::Text(std::fs::read_to_string(url.path())?),
                });
            }
        }
    }

    fn handle_redirect(
        &mut self,
        url: &Url,
        mut response: Response,
        remaining_redirects: u16,
    ) -> Result<Response, RequestError> {
        if response.status_code >= 300 && response.status_code < 400 {
            if remaining_redirects == 0 {
                return Err(RequestError::MaximumRedirects);
            }

            if let Some(location) = response.headers.remove("Location") {
                let redirect = if location.trim().starts_with("/") {
                    url.with_path(location.trim().to_string())
                } else {
                    Url::new(location.clone())
                        .map_err(|e| RequestError::BadRedirectUrl(location, e))?
                };
                eprintln!("Redirecting from {url} to {redirect}");
                return self.request_internal(&redirect, remaining_redirects - 1);
            } else {
                Err(RequestError::NoRedirectFound)
            }
        } else {
            Ok(response)
        }
    }

    /**
     * Pull the static str from cache or create and cache a new one
     */
    fn static_host(&mut self, host: &str) -> &'static str {
        match self.static_hosts.get(host) {
            Some(static_host) => *static_host,
            None => {
                let static_host = host.to_string().leak() as &'static str;
                self.static_hosts.insert(static_host);
                static_host
            }
        }
    }
}

fn send_get(url: &Url, stream: &mut dyn Write) -> Result<(), RequestError> {
    write!(stream, "GET {} HTTP/1.1\r\n", url.path())?;
    header_line(stream, "host", url.host())?;
    //header_line(stream, "Connection", "close")?;
    header_line(stream, "User-Agent", "shelves")?;
    write!(stream, "\r\n")?;
    Ok(())
}

fn header_line(stream: &mut dyn Write, key: &str, value: &str) -> Result<(), RequestError> {
    write!(stream, "{}: {}\r\n", key, value)?;
    Ok(())
}

fn recv_response<T: Read>(mut lines: BufReader<T>) -> Result<Response, RequestError> {
    let mut statusline = String::new();
    lines.read_line(&mut statusline)?;

    let (_http_version, statusline) = statusline.split_once(" ").ok_or(RequestError::BadHTTP)?;
    let (status_code, _explanation) = statusline.split_once(" ").ok_or(RequestError::BadHTTP)?;
    let status_code: u16 = status_code
        .parse()
        .map_err(|_| RequestError::InvalidStatusCode(status_code.to_string()))?;

    let mut headers = HashMap::new();
    loop {
        let mut header_line = String::new();
        lines.read_line(&mut header_line)?;
        if header_line.trim().is_empty() {
            break;
        }
        let (header, value) = header_line
            .trim()
            .split_once(": ")
            .ok_or(RequestError::BadHTTP)?;
        headers.insert(header.to_string(), value.to_string());
    }
    assert!(!headers.contains_key("transfer-encoding"));
    assert!(!headers.contains_key("content-encoding"));

    let content_length: u32 = headers
        .get("Content-Length")
        .ok_or(RequestError::MissingContentLength)?
        .parse()
        .map_err(|_| RequestError::BadHTTP)?;
    let mut body = vec![0; content_length as usize];
    let existing_buffer = lines.buffer();
    let buffer_len = existing_buffer.len();
    body[0..buffer_len].copy_from_slice(existing_buffer);

    let mut inner = lines.into_inner();
    inner.read_exact(&mut body[buffer_len..])?;

    Ok(Response {
        status_code,
        headers,
        body: Body::Bytes(body),
    })
}

pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Body,
}

pub enum Body {
    Bytes(Vec<u8>),
    Text(String),
}

impl Body {
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        match self {
            Body::Bytes(bytes) => std::str::from_utf8(bytes),
            Body::Text(text) => Ok(text),
        }
    }
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
    #[error("tls error: {0}")]
    TlsError(#[from] rustls::Error),
    #[error("dns error: {0}")]
    InvalidDns(#[from] InvalidDnsNameError),
    #[error("invalid status code: {0}")]
    InvalidStatusCode(String),
    #[error("malformed HTTP")]
    BadHTTP,
    #[error("bad redirect URL {0}: {1}")]
    BadRedirectUrl(String, UrlError),
    #[error("maximum redirects exceeded")]
    MaximumRedirects,
    #[error("server returned redirect but no redirect location")]
    NoRedirectFound,
    #[error("missing Content-Length")]
    MissingContentLength,
}
